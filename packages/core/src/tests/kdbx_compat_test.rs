/// KDBX compatibility tests with KeePassXC-generated vaults
#[cfg(test)]
mod kdbx_compat_tests {
    use crate::crypto::hmac::compute_block_hmac;
    use crate::crypto::keys::{CompositeKey, MasterKey};
    use crate::kdbx::KdbxReader;

    fn find_test_vault() -> Option<Vec<u8>> {
        let paths = [
            "packages/core/src/tests/test_kpxc.kdbx",
            "src/tests/test_kpxc.kdbx",
        ];
        paths.iter().find_map(|p| std::fs::read(p).ok())
    }

    #[test]
    fn test_open_keepassxc_vault() {
        let data = match find_test_vault() {
            Some(d) => d,
            None => {
                println!("Skipping: test vault not found");
                return;
            }
        };

        println!("File size: {} bytes", data.len());
        // Print cipher UUID (bytes 14..30 in KDBX 4.x header)
        if data.len() > 30 {
            println!(
                "First 80 bytes: {}",
                hex::encode(&data[..80.min(data.len())])
            );
        }

        let mut key = CompositeKey::new();
        key.add_password("test123");
        let composite = key.build().unwrap();
        println!("Composite key: {}", hex::encode(&composite));

        let reader = KdbxReader::new();
        match reader.read(&data, &composite) {
            Ok(vault) => println!(
                "SUCCESS! '{}' entries={}",
                vault.meta.name,
                vault.entry_count()
            ),
            Err(e) => panic!("FAILED: {:?}", e),
        }
    }

    #[test]
    fn test_hmac_key_derivation_spec() {
        use sha2::{Digest, Sha256, Sha512};

        let seed = vec![0xAAu8; 32];
        let tkey = vec![0xBBu8; 32];
        let mk = MasterKey::new(tkey.clone());
        let (enc_key, hmac_key) = mk.derive_keys(&seed);

        let mut h = Sha256::new();
        h.update(&seed);
        h.update(&tkey);
        h.update(&[0x01u8]);
        assert_eq!(enc_key, h.finalize().to_vec());

        let mut h2 = Sha512::new();
        h2.update(&seed);
        h2.update(&tkey);
        h2.update(&[0x01u8]);
        assert_eq!(hmac_key, h2.finalize().to_vec());
        assert_eq!(hmac_key.len(), 64);
    }

    #[test]
    fn test_block_hmac_spec() {
        use hmac::{Hmac, Mac};
        use sha2::{Digest, Sha256, Sha512};
        type HmacSha256 = Hmac<Sha256>;

        let hmac_key = vec![0xCCu8; 64];
        let block_index: u64 = 0;
        let data = b"hello world";

        // Block key = SHA512(block_index_le64 || hmac_key) — full 64 bytes
        let mut h = Sha512::new();
        h.update(block_index.to_le_bytes());
        h.update(&hmac_key);
        let block_key = h.finalize(); // 64 bytes

        // HMAC-SHA256 with full 64-byte block key
        let mut mac = HmacSha256::new_from_slice(&block_key).unwrap();
        mac.update(&block_index.to_le_bytes());
        mac.update(&(data.len() as u32).to_le_bytes());
        mac.update(data);
        let expected: [u8; 32] = mac.finalize().into_bytes().into();

        let computed = compute_block_hmac(&hmac_key, block_index, data).unwrap();
        assert_eq!(computed, expected, "block HMAC must use full 64-byte key");
    }
}
