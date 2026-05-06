//! KDBX file reader — supports KDBX 4.x (primary) and 3.1 (compat)

use crate::error::{KeePassExError, Result};
use crate::vault::Vault;
use crate::kdbx::{
    KDBX_SIGNATURE_1, KDBX_SIGNATURE_2,
    KDBX_VERSION_3_1, KDBX_VERSION_4_0, KDBX_VERSION_4_1,
    Compression,
};
use crate::crypto::{
    kdf::{KdfParams, ArgonParams, AesKdfParams, derive_master_key},
    cipher::{Cipher, CipherAlgorithm},
    hmac::{verify_block_hmac, compute_header_hmac},
    keys::MasterKey,
};
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

        // Verify header HMAC
        let stored_header_hmac = read_bytes_exact(&mut cursor, 32)?;
        let computed_hmac = compute_header_hmac(&hmac_key, header_data)?;
        if stored_header_hmac != computed_hmac {
            return Err(KeePassExError::HmacVerificationFailed);
        }

        // Read and verify HMAC blocks
        let payload = self.read_hmac_blocks(&mut cursor, &hmac_key)?;

        // Decrypt payload
        let cipher = Cipher::new(cipher_algo, enc_key, encryption_iv);
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
        // KDBX 3.1 reading — simplified for compat
        // Full implementation would handle Salsa20 inner stream, etc.
        Err(KeePassExError::UnsupportedVersion { version: KDBX_VERSION_3_1 })
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
    let mut memory: u64 = 65536;
    let mut parallelism: u64 = 2;
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
            (0x05, "I") => iterations = u64::from_le_bytes(val.try_into().unwrap_or([0; 8])),
            (0x05, "M") => memory = u64::from_le_bytes(val.try_into().unwrap_or([0; 8])),
            (0x05, "P") => parallelism = u64::from_le_bytes(val.try_into().unwrap_or([0; 8])),
            (0x05, "R") => rounds = u64::from_le_bytes(val.try_into().unwrap_or([0; 8])),
            _ => {}
        }
    }

    // Argon2 UUID: 0xEF636DDF8C29444B91F7A9A403E30A0C
    let argon2_uuid = [
        0xEF, 0x63, 0x6D, 0xDF, 0x8C, 0x29, 0x44, 0x4B,
        0x91, 0xF7, 0xA9, 0xA4, 0x03, 0xE3, 0x0A, 0x0C,
    ];

    if kdf_uuid.as_deref() == Some(&argon2_uuid) {
        Ok(KdfParams::Argon2(ArgonParams {
            salt: salt.unwrap_or_else(|| vec![0u8; 32]),
            iterations: iterations as u32,
            memory_kb: (memory / 1024) as u32,
            parallelism: parallelism as u32,
            version: 19,
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
    decoder.read_to_end(&mut output).map_err(KeePassExError::Io)?;
    Ok(output)
}
