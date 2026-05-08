//! Cryptography tests

use crate::crypto::kdf::{derive_master_key, AesKdfParams, ArgonParams, KdfParams};
use crate::crypto::keys::{CompositeKey, KeyFile};

#[test]
fn test_composite_key_password_only() {
    let mut key = CompositeKey::new();
    key.add_password("test_password");
    let result = key.build();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 32);
}

#[test]
fn test_composite_key_empty_fails() {
    let key = CompositeKey::new();
    let result = key.build();
    assert!(result.is_err());
}

#[test]
fn test_composite_key_deterministic() {
    let mut key1 = CompositeKey::new();
    key1.add_password("same_password");
    let hash1 = key1.build().unwrap();

    let mut key2 = CompositeKey::new();
    key2.add_password("same_password");
    let hash2 = key2.build().unwrap();

    assert_eq!(hash1, hash2);
}

#[test]
fn test_composite_key_different_passwords() {
    let mut key1 = CompositeKey::new();
    key1.add_password("password1");
    let hash1 = key1.build().unwrap();

    let mut key2 = CompositeKey::new();
    key2.add_password("password2");
    let hash2 = key2.build().unwrap();

    assert_ne!(hash1, hash2);
}

#[test]
fn test_argon2_kdf() {
    let params = KdfParams::Argon2(ArgonParams {
        salt: vec![0u8; 32],
        iterations: 1,
        memory_kb: 1024, // 1 MB for fast test
        parallelism: 1,
        version: 19,
        secret_key: None,
        associated_data: None,
    });

    let composite_key = vec![0u8; 32];
    let result = derive_master_key(&composite_key, &params);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 32);
}

#[test]
fn test_argon2_kdf_deterministic() {
    let params = KdfParams::Argon2(ArgonParams {
        salt: vec![42u8; 32],
        iterations: 1,
        memory_kb: 1024,
        parallelism: 1,
        version: 19,
        secret_key: None,
        associated_data: None,
    });

    let composite_key = vec![1u8; 32];
    let result1 = derive_master_key(&composite_key, &params).unwrap();
    let result2 = derive_master_key(&composite_key, &params).unwrap();
    assert_eq!(result1, result2);
}

#[test]
fn test_key_file_binary() {
    let data = vec![0xABu8; 32];
    let kf = KeyFile::from_bytes(data.clone());
    let hash = kf.hash().unwrap();
    assert_eq!(hash, data);
}

#[test]
fn test_key_file_arbitrary() {
    let data = b"arbitrary key file content".to_vec();
    let kf = KeyFile::from_bytes(data);
    let hash = kf.hash().unwrap();
    assert_eq!(hash.len(), 32); // SHA-256
}

#[test]
fn test_master_key_derive_keys() {
    use crate::crypto::keys::MasterKey;

    let key = MasterKey::new(vec![0u8; 32]);
    let seed = vec![1u8; 32];
    let (enc_key, hmac_key) = key.derive_keys(&seed);

    // enc_key is SHA256 output = 32 bytes
    assert_eq!(enc_key.len(), 32);
    // hmac_key is SHA512 output = 64 bytes (KDBX 4.x spec)
    assert_eq!(hmac_key.len(), 64);
    // They must be different
    assert_ne!(enc_key, hmac_key[..32]);
}

#[test]
fn test_hmac_block_verification() {
    use crate::crypto::hmac::{compute_block_hmac, verify_block_hmac};

    let key = vec![0u8; 32];
    let data = b"test block data";
    let block_index = 0u64;

    let hmac = compute_block_hmac(&key, block_index, data).unwrap();
    let result = verify_block_hmac(&key, block_index, data, &hmac);
    assert!(result.is_ok());
}

#[test]
fn test_hmac_block_tamper_detection() {
    use crate::crypto::hmac::{compute_block_hmac, verify_block_hmac};

    let key = vec![0u8; 32];
    let data = b"test block data";
    let tampered = b"tampered data!!";
    let block_index = 0u64;

    let hmac = compute_block_hmac(&key, block_index, data).unwrap();
    let result = verify_block_hmac(&key, block_index, tampered, &hmac);
    assert!(result.is_err());
}

#[test]
fn test_chacha20_encrypt_decrypt() {
    use crate::crypto::cipher::{Cipher, CipherAlgorithm};

    let key = vec![0u8; 32];
    let iv = vec![0u8; 12];
    let cipher = Cipher::new(CipherAlgorithm::ChaCha20Poly1305, key, iv);

    let plaintext = b"Hello, KeePassEx!";
    let ciphertext = cipher.encrypt(plaintext).unwrap();
    assert_ne!(ciphertext, plaintext);

    let decrypted = cipher.decrypt(&ciphertext).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_aes_gcm_encrypt_decrypt() {
    use crate::crypto::cipher::{Cipher, CipherAlgorithm};

    let key = vec![0u8; 32];
    let iv = vec![0u8; 12];
    let cipher = Cipher::new(CipherAlgorithm::Aes256Gcm, key, iv);

    let plaintext = b"Test AES-GCM encryption";
    let ciphertext = cipher.encrypt(plaintext).unwrap();
    let decrypted = cipher.decrypt(&ciphertext).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_chacha20_tamper_detection() {
    use crate::crypto::cipher::{Cipher, CipherAlgorithm};

    let key = vec![0u8; 32];
    let iv = vec![0u8; 12];
    let cipher = Cipher::new(CipherAlgorithm::ChaCha20Poly1305, key, iv);

    let plaintext = b"Sensitive data";
    let mut ciphertext = cipher.encrypt(plaintext).unwrap();

    // Tamper with ciphertext
    ciphertext[0] ^= 0xFF;

    let result = cipher.decrypt(&ciphertext);
    assert!(result.is_err());
}
