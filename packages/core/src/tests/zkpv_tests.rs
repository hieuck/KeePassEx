//! ZKPV tests — uses the public ZkpvCommitment and PasswordHint APIs

use crate::zkpv::{PasswordHint, ZkpvCommitment};

#[test]
fn test_commitment_create_and_verify_correct() {
    let commitment = ZkpvCommitment::create("MyMasterPassword123!").unwrap();
    assert!(commitment.verify("MyMasterPassword123!").unwrap());
}

#[test]
fn test_commitment_reject_wrong_password() {
    let commitment = ZkpvCommitment::create("CorrectPassword").unwrap();
    assert!(!commitment.verify("WrongPassword").unwrap());
}

#[test]
fn test_commitment_serialization_roundtrip() {
    let original = ZkpvCommitment::create("TestPassword").unwrap();
    let bytes = original.to_bytes();
    let restored = ZkpvCommitment::from_bytes(&bytes).unwrap();
    assert_eq!(original.salt, restored.salt);
    assert_eq!(original.commitment, restored.commitment);
    assert_eq!(original.m_cost, restored.m_cost);
    assert_eq!(original.t_cost, restored.t_cost);
    assert_eq!(original.p_cost, restored.p_cost);
}

#[test]
fn test_commitment_bytes_length() {
    let c = ZkpvCommitment::create("test").unwrap();
    assert_eq!(c.to_bytes().len(), 76); // 32 + 32 + 4 + 4 + 4
}

#[test]
fn test_commitment_different_salts_for_same_password() {
    let c1 = ZkpvCommitment::create("SamePassword").unwrap();
    let c2 = ZkpvCommitment::create("SamePassword").unwrap();
    // Different salts → different commitments
    assert_ne!(c1.salt, c2.salt);
    assert_ne!(c1.commitment, c2.commitment);
    // But both verify correctly
    assert!(c1.verify("SamePassword").unwrap());
    assert!(c2.verify("SamePassword").unwrap());
}

#[test]
fn test_commitment_from_bytes_too_short_fails() {
    let short = vec![0u8; 10];
    assert!(ZkpvCommitment::from_bytes(&short).is_err());
}

#[test]
fn test_password_hint_encrypt_decrypt() {
    let hint = "First pet's name";
    let password = "MySecretPassword";
    let stored = PasswordHint::create(hint, password);
    let decrypted = stored.decrypt(password).unwrap();
    assert_eq!(decrypted, hint);
}

#[test]
fn test_password_hint_wrong_password_garbles() {
    let hint = "My hint text";
    let stored = PasswordHint::create(hint, "correct_password");
    // Wrong password produces garbage (not the original hint)
    let wrong = stored.decrypt("wrong_password");
    assert!(wrong.is_none() || wrong.unwrap() != hint);
}

#[test]
fn test_password_hint_empty_hint() {
    let stored = PasswordHint::create("", "password");
    let decrypted = stored.decrypt("password").unwrap();
    assert_eq!(decrypted, "");
}

#[test]
fn test_commitment_empty_password() {
    // Empty password should still work
    let commitment = ZkpvCommitment::create("").unwrap();
    assert!(commitment.verify("").unwrap());
    assert!(!commitment.verify("notempty").unwrap());
}
