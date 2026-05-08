//! KDBX 4.x file writer

use crate::crypto::{
    cipher::{Cipher, CipherAlgorithm},
    hmac::{compute_block_hmac, compute_header_hmac},
    kdf::{derive_master_key, ArgonParams, KdfParams},
    keys::MasterKey,
};
use crate::error::{KeePassExError, Result};
use crate::kdbx::{Compression, KDBX_SIGNATURE_1, KDBX_SIGNATURE_2, KDBX_VERSION_4_1};
use crate::vault::Vault;
use rand::RngCore;

pub struct KdbxWriter;

impl KdbxWriter {
    pub fn new() -> Self {
        Self
    }

    pub fn write(&self, vault: &Vault, composite_key: &[u8]) -> Result<Vec<u8>> {
        // Generate random values
        let mut master_seed = vec![0u8; 32];
        let mut encryption_iv = vec![0u8; 12]; // ChaCha20 nonce
        let mut inner_stream_key = vec![0u8; 64];
        rand::thread_rng().fill_bytes(&mut master_seed);
        rand::thread_rng().fill_bytes(&mut encryption_iv);
        rand::thread_rng().fill_bytes(&mut inner_stream_key);

        // KDF params
        let mut salt = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);
        let kdf_params = KdfParams::Argon2(ArgonParams {
            salt,
            iterations: 2,
            memory_kb: 65536,
            parallelism: 2,
            version: 19,
            secret_key: None,
            associated_data: None,
        });

        // Derive keys
        let transformed_key = derive_master_key(composite_key, &kdf_params)?;
        let master_key = MasterKey::new(transformed_key);
        let (enc_key, hmac_key) = master_key.derive_keys(&master_seed);

        // Build outer header fields
        let header_fields = self.build_outer_header(
            &kdf_params,
            &CipherAlgorithm::ChaCha20Poly1305,
            &Compression::GZip,
            &master_seed,
            &encryption_iv,
        )?;

        // The header HMAC covers: signature(8) + version(4) + header_fields
        // This matches what the reader does: full_data[..header_end]
        let mut header_data_for_hmac = Vec::new();
        header_data_for_hmac.extend_from_slice(&KDBX_SIGNATURE_1.to_le_bytes());
        header_data_for_hmac.extend_from_slice(&KDBX_SIGNATURE_2.to_le_bytes());
        header_data_for_hmac.extend_from_slice(&KDBX_VERSION_4_1.to_le_bytes());
        header_data_for_hmac.extend_from_slice(&header_fields);

        // Compute header SHA256 (integrity) and HMAC (authentication)
        // KDBX 4.x writes both after the header
        use sha2::{Digest, Sha256};
        let header_sha256: [u8; 32] = Sha256::digest(&header_data_for_hmac).into();
        let header_hmac = compute_header_hmac(&hmac_key, &header_data_for_hmac)?;

        // Build XML payload
        let xml_parser = super::xml::XmlSerializer::new(inner_stream_key.clone());
        let xml_bytes = xml_parser.serialize(vault)?;

        // Build inner header
        let inner_header = self.build_inner_header(&inner_stream_key, vault)?;
        let mut payload = inner_header;
        payload.extend_from_slice(&xml_bytes);

        // Compress
        let compressed = compress_gzip(&payload)?;

        // Encrypt
        let cipher = Cipher::new(CipherAlgorithm::ChaCha20Poly1305, enc_key, encryption_iv);
        let encrypted = cipher.encrypt(&compressed)?;

        // Build HMAC blocks
        let blocks = self.build_hmac_blocks(&encrypted, &hmac_key)?;

        // Assemble final file: sig + version + header_fields + header_sha256 + header_hmac + blocks
        let mut output = Vec::new();
        output.extend_from_slice(&KDBX_SIGNATURE_1.to_le_bytes());
        output.extend_from_slice(&KDBX_SIGNATURE_2.to_le_bytes());
        output.extend_from_slice(&KDBX_VERSION_4_1.to_le_bytes());
        output.extend_from_slice(&header_fields);
        output.extend_from_slice(&header_sha256); // SHA256 first
        output.extend_from_slice(&header_hmac); // HMAC second
        output.extend_from_slice(&blocks);

        Ok(output)
    }

    fn build_outer_header(
        &self,
        kdf_params: &KdfParams,
        cipher: &CipherAlgorithm,
        compression: &Compression,
        master_seed: &[u8],
        encryption_iv: &[u8],
    ) -> Result<Vec<u8>> {
        let mut header = Vec::new();

        // CipherId (field 2)
        write_header_field(&mut header, 2, &cipher.uuid_bytes());

        // CompressionFlags (field 3)
        write_header_field(&mut header, 3, &compression.id().to_le_bytes());

        // MasterSeed (field 4)
        write_header_field(&mut header, 4, master_seed);

        // EncryptionIV (field 7)
        write_header_field(&mut header, 7, encryption_iv);

        // KdfParameters (field 11)
        let kdf_map = build_kdf_variant_map(kdf_params)?;
        write_header_field(&mut header, 11, &kdf_map);

        // EndOfHeader (field 0)
        write_header_field(&mut header, 0, &[0x0D, 0x0A, 0x0D, 0x0A]);

        Ok(header)
    }

    fn build_inner_header(&self, stream_key: &[u8], _vault: &Vault) -> Result<Vec<u8>> {
        let mut header = Vec::new();

        // InnerRandomStreamId = ChaCha20 (3)
        header.push(1u8); // field id
        header.extend_from_slice(&4u32.to_le_bytes()); // length
        header.extend_from_slice(&3u32.to_le_bytes()); // ChaCha20

        // InnerRandomStreamKey
        header.push(2u8);
        header.extend_from_slice(&(stream_key.len() as u32).to_le_bytes());
        header.extend_from_slice(stream_key);

        // EndOfHeader
        header.push(0u8);
        header.extend_from_slice(&0u32.to_le_bytes());

        Ok(header)
    }

    fn build_hmac_blocks(&self, data: &[u8], hmac_key: &[u8]) -> Result<Vec<u8>> {
        let block_size = 1024 * 1024; // 1 MB blocks
        let mut output = Vec::new();
        let mut block_index = 0u64;

        for chunk in data.chunks(block_size) {
            let hmac = compute_block_hmac(hmac_key, block_index, chunk)?;
            output.extend_from_slice(&hmac);
            output.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
            output.extend_from_slice(chunk);
            block_index += 1;
        }

        // Terminal block (empty)
        let terminal_hmac = compute_block_hmac(hmac_key, block_index, &[])?;
        output.extend_from_slice(&terminal_hmac);
        output.extend_from_slice(&0u32.to_le_bytes());

        Ok(output)
    }
}

fn write_header_field(buf: &mut Vec<u8>, id: u8, data: &[u8]) {
    buf.push(id);
    buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
    buf.extend_from_slice(data);
}

fn build_kdf_variant_map(params: &KdfParams) -> Result<Vec<u8>> {
    let mut map = Vec::new();

    // Version
    map.extend_from_slice(&0x0100u16.to_le_bytes());

    match params {
        KdfParams::Argon2(p) => {
            // UUID
            let argon2_uuid = [
                0xEF, 0x63, 0x6D, 0xDF, 0x8C, 0x29, 0x44, 0x4B, 0x91, 0xF7, 0xA9, 0xA4, 0x03, 0xE3,
                0x0A, 0x0C,
            ];
            write_variant_entry(&mut map, 0x42, b"$UUID", &argon2_uuid);
            write_variant_entry(&mut map, 0x42, b"S", &p.salt);
            write_variant_entry(&mut map, 0x05, b"I", &(p.iterations as u64).to_le_bytes());
            write_variant_entry(
                &mut map,
                0x05,
                b"M",
                &((p.memory_kb as u64) * 1024).to_le_bytes(),
            );
            write_variant_entry(&mut map, 0x05, b"P", &(p.parallelism as u64).to_le_bytes());
            write_variant_entry(&mut map, 0x04, b"V", &(p.version as u32).to_le_bytes());
        }
        KdfParams::AesKdf(p) => {
            let aes_kdf_uuid = [
                0xC9, 0xD9, 0xF3, 0x9A, 0x62, 0x8A, 0x44, 0x60, 0xBF, 0x74, 0x0D, 0x08, 0xC1, 0x8A,
                0x4F, 0xEA,
            ];
            write_variant_entry(&mut map, 0x42, b"$UUID", &aes_kdf_uuid);
            write_variant_entry(&mut map, 0x42, b"S", &p.seed);
            write_variant_entry(&mut map, 0x05, b"R", &p.rounds.to_le_bytes());
        }
    }

    // End marker
    map.push(0x00);
    Ok(map)
}

fn write_variant_entry(buf: &mut Vec<u8>, type_id: u8, key: &[u8], value: &[u8]) {
    buf.push(type_id);
    buf.extend_from_slice(&(key.len() as u32).to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&(value.len() as u32).to_le_bytes());
    buf.extend_from_slice(value);
}

fn compress_gzip(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).map_err(KeePassExError::Io)?;
    encoder.finish().map_err(KeePassExError::Io)
}
