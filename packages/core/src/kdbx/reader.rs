//! KDBX file reader — supports KDBX 4.x (primary) and 3.1 (compat)

use crate::crypto::{
    cipher::{Cipher, CipherAlgorithm},
    hmac::{compute_header_hmac, verify_block_hmac},
    kdf::{derive_master_key, AesKdfParams, ArgonParams, KdfParams},
    keys::MasterKey,
};
use crate::error::{KeePassExError, Result};
use crate::kdbx::{
    Compression, KDBX_SIGNATURE_1, KDBX_SIGNATURE_2, KDBX_VERSION_3_1, KDBX_VERSION_4_0,
    KDBX_VERSION_4_1,
};
use crate::vault::Vault;
use std::io::{Cursor, Read};

pub struct KdbxReader;

impl KdbxReader {
    pub fn new() -> Self {
        Self
    }

    pub fn read(&self, data: &[u8], composite_key: &[u8]) -> Result<Vault> {
        let mut cursor = Cursor::new(data);

        // Read and verify signature
        let sig1 = read_u32_le(&mut cursor)?;
        let sig2 = read_u32_le(&mut cursor)?;

        if sig1 != KDBX_SIGNATURE_1 || sig2 != KDBX_SIGNATURE_2 {
            return Err(KeePassExError::CorruptedVault {
                reason: "Invalid KDBX signature".into(),
            });
        }

        let version = read_u32_le(&mut cursor)?;

        match version {
            v if v >= KDBX_VERSION_4_0 => self.read_kdbx4(cursor, composite_key, data),
            KDBX_VERSION_3_1 => self.read_kdbx3(cursor, composite_key),
            _ => Err(KeePassExError::UnsupportedVersion { version }),
        }
    }

    fn read_kdbx4(
        &self,
        mut cursor: Cursor<&[u8]>,
        composite_key: &[u8],
        full_data: &[u8],
    ) -> Result<Vault> {
        // Parse outer header
        let header_start = cursor.position() as usize;
        let (kdf_params, cipher_algo, compression, master_seed, encryption_iv) =
            self.parse_outer_header(&mut cursor)?;
        let header_end = cursor.position() as usize;
        let header_data = &full_data[..header_end];

        // Derive master key
        let transformed_key = derive_master_key(composite_key, &kdf_params)?;
        let master_key = MasterKey::new(transformed_key);
        let (enc_key, hmac_key) = master_key.derive_keys(&master_seed);

        // KDBX 4.x has TWO checksums after the header:
        // 1. Header SHA256 (32 bytes) — integrity check
        // 2. Header HMAC (32 bytes) — authentication check
        let _header_sha256 = read_bytes_exact(&mut cursor, 32)?; // skip SHA256
        let stored_header_hmac = read_bytes_exact(&mut cursor, 32)?;
        let computed_hmac = compute_header_hmac(&hmac_key, header_data)?;
        if stored_header_hmac != computed_hmac {
            return Err(KeePassExError::HmacVerificationFailed);
        }

        // Read and verify HMAC blocks
        let payload = self.read_hmac_blocks(&mut cursor, &hmac_key)?;

        // Decrypt payload
        let cipher = Cipher::new(cipher_algo.clone(), enc_key, encryption_iv);
        tracing::debug!(
            "Decrypting with cipher: {:?}, payload_len: {}",
            cipher_algo,
            payload.len()
        );
        let decrypted = cipher.decrypt(&payload)?;

        // Decompress
        let xml_data = match compression {
            Compression::GZip => decompress_gzip(&decrypted)?,
            Compression::None => decrypted,
        };

        // Parse inner header + XML
        let mut inner_cursor = Cursor::new(xml_data.as_slice());
        let (inner_stream_key, binaries) = self.parse_inner_header(&mut inner_cursor)?;

        let xml_start = inner_cursor.position() as usize;
        let xml_bytes = &xml_data[xml_start..];

        // Parse XML into Vault
        let xml_parser = super::xml::XmlParser::new(inner_stream_key);
        xml_parser.parse(xml_bytes, binaries)
    }

    fn read_kdbx3(&self, mut cursor: Cursor<&[u8]>, composite_key: &[u8]) -> Result<Vault> {
        // KDBX 3.1 format:
        // - Outer header (TLV, 2-byte length fields)
        // - SHA-256 hash of header (32 bytes)
        // - Encrypted payload (AES-256-CBC, no HMAC blocks)
        // - Payload: GZip-compressed XML (no inner header)
        // - Inner stream: Salsa20 for protected fields

        let (kdf_params, cipher_algo, compression, master_seed, encryption_iv, stream_start_bytes) =
            self.parse_outer_header_v3(&mut cursor)?;

        // Derive master key (AES-KDF for KDBX 3.1)
        let transformed_key = derive_master_key(composite_key, &kdf_params)?;
        let master_key = MasterKey::new(transformed_key);
        // KDBX 3.1: enc_key = SHA256(masterSeed || transformedKey) — no 0x01 suffix
        let enc_key = master_key.derive_key_v3(&master_seed);

        // KDBX 3.1: ciphertext starts immediately after EndOfHeader
        // (no 32-byte header hash prefix — that's stored inside the XML, not before ciphertext)
        let pos = cursor.position() as usize;
        let encrypted = cursor.into_inner()[pos..].to_vec();

        // Decrypt with AES-256-CBC
        let cipher = Cipher::new(cipher_algo, enc_key.clone(), encryption_iv.clone());
        let decrypted = cipher.decrypt(&encrypted)?;

        // Verify stream start bytes (first 32 bytes of decrypted payload)
        if decrypted.len() < 32 {
            return Err(KeePassExError::CorruptedVault {
                reason: "Decrypted payload too short".into(),
            });
        }
        if decrypted[..32] != stream_start_bytes[..] {
            return Err(KeePassExError::HmacVerificationFailed);
        }

        // Skip stream start bytes, then read KDBX 3.1 block stream
        let payload_data = &decrypted[32..];
        let xml_data = self.read_v3_block_stream(payload_data)?;

        // Decompress
        let xml_bytes = match compression {
            Compression::GZip => decompress_gzip(&xml_data)?,
            Compression::None => xml_data,
        };

        // KDBX 3.1 has no inner header — use Salsa20 inner stream
        // The inner stream key is derived from the stream start bytes
        let inner_stream_key = stream_start_bytes.to_vec();
        let xml_parser = super::xml::XmlParser::new(inner_stream_key);
        xml_parser.parse(&xml_bytes, Vec::new())
    }

    fn parse_outer_header_v3(
        &self,
        cursor: &mut Cursor<&[u8]>,
    ) -> Result<(
        KdfParams,
        CipherAlgorithm,
        Compression,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
    )> {
        let mut kdf_params = None;
        let mut cipher_algo = CipherAlgorithm::Aes256Cbc;
        let mut compression = Compression::GZip;
        let mut master_seed = Vec::new();
        let mut encryption_iv = Vec::new();
        let mut stream_start_bytes = Vec::new();
        // KDBX 3.1 also has TransformSeed and TransformRounds for AES-KDF
        let mut transform_seed = Vec::new();
        let mut transform_rounds: u64 = 6000;

        loop {
            let field_id = read_u8(cursor)?;
            // KDBX 3.1 uses 2-byte length fields
            let field_len = {
                let mut buf = [0u8; 2];
                cursor.read_exact(&mut buf).map_err(KeePassExError::Io)?;
                u16::from_le_bytes(buf) as usize
            };
            let field_data = read_bytes_exact(cursor, field_len)?;

            match field_id {
                0 => break, // EndOfHeader
                2 => {
                    // CipherId
                    if field_data.len() == 16 {
                        cipher_algo = parse_cipher_id(&field_data)?;
                    }
                }
                3 => {
                    // CompressionFlags
                    if field_data.len() >= 4 {
                        let id = u32::from_le_bytes(field_data[..4].try_into().unwrap());
                        compression = Compression::from_id(id).unwrap_or(Compression::GZip);
                    }
                }
                4 => master_seed = field_data,
                5 => transform_seed = field_data,
                6 => {
                    // TransformRounds
                    if field_data.len() == 8 {
                        transform_rounds = u64::from_le_bytes(field_data.try_into().unwrap());
                    }
                }
                7 => encryption_iv = field_data,
                9 => stream_start_bytes = field_data,
                _ => {}
            }
        }

        // Build AES-KDF params from KDBX 3.1 fields
        kdf_params = Some(KdfParams::AesKdf(AesKdfParams {
            seed: transform_seed,
            rounds: transform_rounds,
        }));

        let kdf = kdf_params.ok_or_else(|| KeePassExError::CorruptedVault {
            reason: "Missing KDF parameters in KDBX 3.1 header".into(),
        })?;

        Ok((
            kdf,
            cipher_algo,
            compression,
            master_seed,
            encryption_iv,
            stream_start_bytes,
        ))
    }

    /// Read KDBX 3.1 block stream (SHA-256 integrity, not HMAC)
    fn read_v3_block_stream(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(data);
        let mut payload = Vec::new();

        loop {
            // Block ID (4 bytes, little-endian)
            let _block_id = {
                let mut buf = [0u8; 4];
                if cursor.read_exact(&mut buf).is_err() {
                    break;
                }
                u32::from_le_bytes(buf)
            };

            // Block hash (32 bytes SHA-256)
            let block_hash = read_bytes_exact(&mut cursor, 32)?;

            // Block size (4 bytes)
            let block_size = {
                let mut buf = [0u8; 4];
                cursor.read_exact(&mut buf).map_err(KeePassExError::Io)?;
                u32::from_le_bytes(buf) as usize
            };

            if block_size == 0 {
                // Terminal block — verify hash is all zeros
                break;
            }

            let block_data = read_bytes_exact(&mut cursor, block_size)?;

            // Verify SHA-256 hash
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&block_data);
            let computed: [u8; 32] = hasher.finalize().into();
            if computed != block_hash.as_slice() {
                return Err(KeePassExError::CorruptedVault {
                    reason: "KDBX 3.1 block hash mismatch".into(),
                });
            }

            payload.extend_from_slice(&block_data);
        }

        Ok(payload)
    }

    fn parse_outer_header(
        &self,
        cursor: &mut Cursor<&[u8]>,
    ) -> Result<(KdfParams, CipherAlgorithm, Compression, Vec<u8>, Vec<u8>)> {
        let mut kdf_params = None;
        let mut cipher_algo = CipherAlgorithm::ChaCha20Poly1305;
        let mut compression = Compression::GZip;
        let mut master_seed = Vec::new();
        let mut encryption_iv = Vec::new();

        loop {
            let field_id = read_u8(cursor)?;
            let field_len = read_u32_le(cursor)? as usize;
            let field_data = read_bytes_exact(cursor, field_len)?;

            match field_id {
                0 => break, // EndOfHeader
                2 => {
                    // CipherId
                    if field_data.len() == 16 {
                        cipher_algo = parse_cipher_id(&field_data)?;
                    }
                }
                3 => {
                    // CompressionFlags
                    if field_data.len() >= 4 {
                        let id = u32::from_le_bytes(field_data[..4].try_into().unwrap());
                        compression = Compression::from_id(id).unwrap_or(Compression::GZip);
                    }
                }
                4 => {
                    // MasterSeed
                    master_seed = field_data;
                }
                7 => {
                    // EncryptionIV
                    encryption_iv = field_data;
                }
                11 => {
                    // KdfParameters (VariantMap)
                    kdf_params = Some(parse_kdf_variant_map(&field_data)?);
                }
                _ => {} // Unknown fields ignored
            }
        }

        let kdf = kdf_params.ok_or_else(|| KeePassExError::CorruptedVault {
            reason: "Missing KDF parameters in header".into(),
        })?;

        Ok((kdf, cipher_algo, compression, master_seed, encryption_iv))
    }

    fn parse_inner_header(
        &self,
        cursor: &mut Cursor<&[u8]>,
    ) -> Result<(Vec<u8>, Vec<(String, Vec<u8>)>)> {
        let mut inner_stream_key = Vec::new();
        let mut binaries = Vec::new();

        loop {
            let field_id = read_u8(cursor)?;
            let field_len = read_u32_le(cursor)? as usize;
            let field_data = read_bytes_exact(cursor, field_len)?;

            match field_id {
                0 => break, // EndOfHeader
                2 => {
                    // InnerRandomStreamKey
                    inner_stream_key = field_data;
                }
                3 => {
                    // Binary attachment
                    if !field_data.is_empty() {
                        let _flags = field_data[0];
                        let data = field_data[1..].to_vec();
                        binaries.push((format!("binary_{}", binaries.len()), data));
                    }
                }
                _ => {}
            }
        }

        Ok((inner_stream_key, binaries))
    }

    fn read_hmac_blocks(&self, cursor: &mut Cursor<&[u8]>, hmac_key: &[u8]) -> Result<Vec<u8>> {
        let mut payload = Vec::new();
        let mut block_index = 0u64;

        loop {
            let stored_hmac = read_bytes_exact(cursor, 32)?;
            let block_size = read_u32_le(cursor)? as usize;
            let block_data = read_bytes_exact(cursor, block_size)?;

            // Verify block HMAC
            let mut expected = [0u8; 32];
            expected.copy_from_slice(&stored_hmac);
            verify_block_hmac(hmac_key, block_index, &block_data, &expected)?;

            if block_size == 0 {
                break; // End of blocks
            }

            payload.extend_from_slice(&block_data);
            block_index += 1;
        }

        Ok(payload)
    }
}

// ─── Helper functions ─────────────────────────────────────────────────────────

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut buf = [0u8; 1];
    cursor.read_exact(&mut buf).map_err(KeePassExError::Io)?;
    Ok(buf[0])
}

fn read_u32_le(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf).map_err(KeePassExError::Io)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_bytes_exact(cursor: &mut Cursor<&[u8]>, len: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    cursor.read_exact(&mut buf).map_err(KeePassExError::Io)?;
    Ok(buf)
}

fn parse_cipher_id(data: &[u8]) -> Result<CipherAlgorithm> {
    if data.len() != 16 {
        return Err(KeePassExError::CorruptedVault {
            reason: "Invalid cipher UUID length".into(),
        });
    }

    let chacha_uuid = CipherAlgorithm::ChaCha20Poly1305.uuid_bytes();
    let aes_gcm_uuid = CipherAlgorithm::Aes256Gcm.uuid_bytes();
    let aes_cbc_uuid = CipherAlgorithm::Aes256Cbc.uuid_bytes();
    let twofish_uuid = CipherAlgorithm::TwofishCbc.uuid_bytes();

    if data == chacha_uuid {
        Ok(CipherAlgorithm::ChaCha20Poly1305)
    } else if data == aes_gcm_uuid {
        Ok(CipherAlgorithm::Aes256Gcm)
    } else if data == aes_cbc_uuid {
        Ok(CipherAlgorithm::Aes256Cbc)
    } else if data == twofish_uuid {
        Ok(CipherAlgorithm::TwofishCbc)
    } else {
        Err(KeePassExError::CorruptedVault {
            reason: "Unknown cipher UUID".into(),
        })
    }
}

fn parse_kdf_variant_map(data: &[u8]) -> Result<KdfParams> {
    // VariantMap parsing (KDBX 4.x)
    // Format: version(2) + entries + end(0x00)
    let mut cursor = Cursor::new(data);
    let _version = {
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).map_err(KeePassExError::Io)?;
        u16::from_le_bytes(buf)
    };

    let mut kdf_uuid: Option<Vec<u8>> = None;
    let mut salt: Option<Vec<u8>> = None;
    let mut iterations: u64 = 2;
    let mut memory: u64 = 65536 * 1024; // bytes (KeePassXC stores bytes)
    let mut parallelism: u64 = 2;
    let mut argon2_version: u32 = 0x13;
    let mut rounds: u64 = 6000;

    loop {
        let type_id = read_u8(&mut cursor)?;
        if type_id == 0x00 {
            break;
        }

        let key_len = read_u32_le(&mut cursor)? as usize;
        let key_bytes = read_bytes_exact(&mut cursor, key_len)?;
        let key = String::from_utf8_lossy(&key_bytes).to_string();

        let val_len = read_u32_le(&mut cursor)? as usize;
        let val = read_bytes_exact(&mut cursor, val_len)?;

        match (type_id, key.as_str()) {
            (0x42, "$UUID") => kdf_uuid = Some(val),
            (0x42, "S") => salt = Some(val),
            // Argon2 params (UInt64 = 0x05)
            (0x05, "I") => iterations = u64::from_le_bytes(val.try_into().unwrap_or([0; 8])),
            (0x05, "M") => memory = u64::from_le_bytes(val.try_into().unwrap_or([0; 8])),
            (0x05, "P") => parallelism = u64::from_le_bytes(val.try_into().unwrap_or([0; 8])),
            // Argon2 version (UInt32 = 0x04)
            (0x04, "V") => {
                if val.len() == 4 {
                    argon2_version = u32::from_le_bytes(val.try_into().unwrap_or([0; 4]));
                }
            }
            // AES-KDF rounds (UInt64 = 0x05)
            (0x05, "R") => rounds = u64::from_le_bytes(val.try_into().unwrap_or([0; 8])),
            _ => {}
        }
    }

    // KeePassXC UUIDs (from KeePass2.cpp):
    // Argon2d  UUID: ef636ddf-8c29-444b-91f7-a9a403e30a0c
    // Argon2id UUID: 9e298b19-56db-4773-b23d-fc3ec6f0a1e6
    // AES-KDF  UUID: 7c02bb82-79a7-4ac0-927d-114a00648238
    let argon2d_uuid = [
        0xEF, 0x63, 0x6D, 0xDF, 0x8C, 0x29, 0x44, 0x4B, 0x91, 0xF7, 0xA9, 0xA4, 0x03, 0xE3, 0x0A,
        0x0C,
    ];
    let argon2id_uuid = [
        0x9E, 0x29, 0x8B, 0x19, 0x56, 0xDB, 0x47, 0x73, 0xB2, 0x3D, 0xFC, 0x3E, 0xC6, 0xF0, 0xA1,
        0xE6,
    ];

    let uuid_bytes = kdf_uuid.as_deref().unwrap_or(&[]);

    if uuid_bytes == argon2d_uuid || uuid_bytes == argon2id_uuid {
        let variant = if uuid_bytes == argon2id_uuid {
            crate::crypto::kdf::Argon2Variant::Argon2id
        } else {
            crate::crypto::kdf::Argon2Variant::Argon2d
        };

        // KeePassXC stores memory in bytes; argon2 crate expects KiB
        let memory_kb = (memory / 1024) as u32;

        Ok(KdfParams::Argon2(ArgonParams {
            variant,
            salt: salt.unwrap_or_else(|| vec![0u8; 32]),
            iterations: iterations as u32,
            memory_kb,
            parallelism: parallelism as u32,
            version: argon2_version,
            secret_key: None,
            associated_data: None,
        }))
    } else {
        // Assume AES-KDF
        Ok(KdfParams::AesKdf(AesKdfParams {
            seed: salt.unwrap_or_else(|| vec![0u8; 32]),
            rounds,
        }))
    }
}

fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    let mut decoder = GzDecoder::new(data);
    let mut output = Vec::new();
    decoder
        .read_to_end(&mut output)
        .map_err(KeePassExError::Io)?;
    Ok(output)
}
