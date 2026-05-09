/// KDBX compatibility tests
#[cfg(test)]
mod kdbx_compat_tests {
    use crate::crypto::hmac::compute_block_hmac;
    use crate::crypto::keys::{CompositeKey, MasterKey};
    use crate::kdbx::KdbxReader;

    /// Test with real KDBX 4.x vault (wrong password — just check structure)
    #[test]
    fn test_open_real_vault_structure() {
        let data = match std::fs::read("packages/core/src/tests/test_real.kdbx")
            .or_else(|_| std::fs::read("src/tests/test_real.kdbx"))
        {
            Ok(d) => d,
            Err(_) => {
                println!("Skipping: real vault not found");
                return;
            }
        };
        println!("Real vault: {} bytes", data.len());
        let ver = u32::from_le_bytes(data[8..12].try_into().unwrap());
        println!("KDBX version: 0x{:08X}", ver);

        let mut key = CompositeKey::new();
        key.add_password("wrong_password_to_test_structure");
        let composite = key.build().unwrap();

        let reader = KdbxReader::new();
        match reader.read(&data, &composite) {
            Ok(_) => println!("Opened (unexpected)"),
            Err(e) => println!("Error with wrong pw (expected): {:?}", e),
        }
    }

    /// Test with KeePassXC KDBX 3.1 vault (password: test123)
    #[test]
    fn test_open_keepassxc_vault() {
        let paths = [
            "packages/core/src/tests/test_kpxc.kdbx",
            "src/tests/test_kpxc.kdbx",
        ];
        let data = match paths.iter().find_map(|p| std::fs::read(p).ok()) {
            Some(d) => d,
            None => {
                println!("Skipping: test_kpxc.kdbx not found");
                return;
            }
        };

        let mut key = CompositeKey::new();
        key.add_password("test123");
        let composite = key.build().unwrap();

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

    /// Test with pykeepass-created KDBX 4.x vault (Argon2id, password: TestPassword123)
    #[test]
    fn test_open_kdbx4_argon2id_vault() {
        let paths = [
            "packages/core/src/tests/test_kpxc4_argon2id.kdbx",
            "src/tests/test_kpxc4_argon2id.kdbx",
        ];
        let data = match paths.iter().find_map(|p| std::fs::read(p).ok()) {
            Some(d) => d,
            None => {
                println!("Skipping: test_kpxc4_argon2id.kdbx not found");
                return;
            }
        };

        println!("KDBX4 vault: {} bytes", data.len());
        let ver = u32::from_le_bytes(data[8..12].try_into().unwrap());
        println!("Version: 0x{:08X}", ver);

        let mut key = CompositeKey::new();
        key.add_password("TestPassword123");
        let composite = key.build().unwrap();

        let reader = KdbxReader::new();
        match reader.read(&data, &composite) {
            Ok(vault) => {
                println!(
                    "SUCCESS! '{}' entries={}",
                    vault.meta.name,
                    vault.entry_count()
                );
                assert!(vault.entry_count() >= 1, "Should have at least 1 entry");
            }
            Err(e) => panic!("FAILED to open KDBX4 Argon2id vault: {:?}", e),
        }
    }

    /// Test with pykeepass KDBX 4.x vault (AES-256-CBC + Argon2d, password: Hello123)
    #[test]
    fn test_open_kdbx4_aes_argon2d_vault() {
        let paths = [
            "packages/core/src/tests/test_argon2d.kdbx",
            "src/tests/test_argon2d.kdbx",
        ];
        let data = match paths.iter().find_map(|p| std::fs::read(p).ok()) {
            Some(d) => d,
            None => {
                println!("Skipping: test_argon2d.kdbx not found");
                return;
            }
        };

        println!("AES+Argon2d vault: {} bytes", data.len());
        let ver = u32::from_le_bytes(data[8..12].try_into().unwrap());
        println!("Version: 0x{:08X}", ver);

        let mut key = CompositeKey::new();
        key.add_password("Hello123");
        let composite = key.build().unwrap();

        let reader = KdbxReader::new();
        match reader.read(&data, &composite) {
            Ok(vault) => {
                println!(
                    "SUCCESS! '{}' entries={}",
                    vault.meta.name,
                    vault.entry_count()
                );
            }
            Err(e) => panic!("FAILED to open AES+Argon2d vault: {:?}", e),
        }
    }

    /// Test wrong password on KDBX 4.x vault
    #[test]
    fn test_kdbx4_wrong_password_rejected() {
        let paths = [
            "packages/core/src/tests/test_kpxc4_argon2id.kdbx",
            "src/tests/test_kpxc4_argon2id.kdbx",
        ];
        let data = match paths.iter().find_map(|p| std::fs::read(p).ok()) {
            Some(d) => d,
            None => {
                println!("Skipping: test_kpxc4_argon2id.kdbx not found");
                return;
            }
        };

        let mut key = CompositeKey::new();
        key.add_password("WrongPassword");
        let composite = key.build().unwrap();

        let reader = KdbxReader::new();
        let result = reader.read(&data, &composite);
        assert!(result.is_err(), "Wrong password should be rejected");
        println!("Wrong password correctly rejected: {:?}", result.err());
    }

    #[test]
    fn test_hmac_key_derivation_spec() {
        use sha2::{Digest, Sha256, Sha512};

        let seed = vec![0xAAu8; 32];
        let tkey = vec![0xBBu8; 32];
        let mk = MasterKey::new(tkey.clone());
        let (enc_key, hmac_key) = mk.derive_keys(&seed);

        // enc_key = SHA256(masterSeed || transformedKey)  — NO 0x01 suffix
        // Per KeePassXC Kdbx4Reader.cpp:
        //   hash.addData(m_masterSeed); hash.addData(db->transformedDatabaseKey()); finalKey = hash.result();
        let mut h = Sha256::new();
        h.update(&seed);
        h.update(&tkey);
        assert_eq!(
            enc_key,
            h.finalize().to_vec(),
            "enc_key must NOT have 0x01 suffix"
        );

        // hmac_key = SHA512(masterSeed || transformedKey || 0x01)  — WITH 0x01 suffix
        // Per KeePassXC KeePass2.cpp: hmacKeyHash.addData(masterSeed); hmacKeyHash.addData(transformedMasterKey); hmacKeyHash.addData(QByteArray(1, '\x01'));
        let mut h2 = Sha512::new();
        h2.update(&seed);
        h2.update(&tkey);
        h2.update(&[0x01u8]);
        assert_eq!(
            hmac_key,
            h2.finalize().to_vec(),
            "hmac_key must have 0x01 suffix"
        );
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

        let mut mac = HmacSha256::new_from_slice(&block_key).unwrap();
        mac.update(&block_index.to_le_bytes());
        mac.update(&(data.len() as u32).to_le_bytes());
        mac.update(data);
        let expected: [u8; 32] = mac.finalize().into_bytes().into();

        let computed = compute_block_hmac(&hmac_key, block_index, data).unwrap();
        assert_eq!(computed, expected, "block HMAC must use full 64-byte key");
    }
}
