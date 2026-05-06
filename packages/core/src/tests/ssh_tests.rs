//! SSH key management tests

use crate::ssh::{parse_ssh_public_key, SshAgent};
use crate::types::{SshKeyEntry, SshKeyType};

fn make_ssh_key(key_type: SshKeyType, fingerprint: &str) -> SshKeyEntry {
    SshKeyEntry {
        key_type,
        private_key: crate::types::ProtectedString::new("-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----"),
        public_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAITest test@example.com".to_string(),
        comment: "test@example.com".to_string(),
        fingerprint: fingerprint.to_string(),
        add_to_agent: true,
        agent_duration: None,
        confirm_before_use: false,
    }
}

#[test]
fn test_ssh_agent_add_key() {
    let mut agent = SshAgent::new();
    let key = make_ssh_key(SshKeyType::Ed25519, "SHA256:abc123");

    agent.add_key(key);
    assert_eq!(agent.list_keys().len(), 1);
}

#[test]
fn test_ssh_agent_remove_key() {
    let mut agent = SshAgent::new();
    let key = make_ssh_key(SshKeyType::Ed25519, "SHA256:abc123");

    agent.add_key(key);
    assert_eq!(agent.list_keys().len(), 1);

    agent.remove_key("SHA256:abc123");
    assert_eq!(agent.list_keys().len(), 0);
}

#[test]
fn test_ssh_agent_remove_all() {
    let mut agent = SshAgent::new();

    for i in 0..5 {
        let key = make_ssh_key(SshKeyType::Ed25519, &format!("SHA256:key{}", i));
        agent.add_key(key);
    }

    assert_eq!(agent.list_keys().len(), 5);
    agent.remove_all();
    assert_eq!(agent.list_keys().len(), 0);
}

#[test]
fn test_ssh_agent_deduplicates_by_fingerprint() {
    let mut agent = SshAgent::new();

    let key1 = make_ssh_key(SshKeyType::Ed25519, "SHA256:same");
    let key2 = make_ssh_key(SshKeyType::Ed25519, "SHA256:same"); // same fingerprint

    agent.add_key(key1);
    agent.add_key(key2); // should replace key1

    assert_eq!(agent.list_keys().len(), 1);
}

#[test]
fn test_ssh_agent_multiple_key_types() {
    let mut agent = SshAgent::new();

    agent.add_key(make_ssh_key(SshKeyType::Ed25519, "SHA256:ed25519"));
    agent.add_key(make_ssh_key(SshKeyType::Rsa4096, "SHA256:rsa4096"));
    agent.add_key(make_ssh_key(SshKeyType::EcdsaP256, "SHA256:ecdsa"));

    assert_eq!(agent.list_keys().len(), 3);
}

#[test]
fn test_ssh_agent_cleanup_expired() {
    let mut agent = SshAgent::new();

    // Key with 0-second duration (immediately expired)
    let mut key = make_ssh_key(SshKeyType::Ed25519, "SHA256:expired");
    key.agent_duration = Some(0);
    agent.add_key(key);

    // Key with no expiry
    let key2 = make_ssh_key(SshKeyType::Ed25519, "SHA256:permanent");
    agent.add_key(key2);

    agent.cleanup_expired();

    // The expired key should be removed, permanent key stays
    let keys = agent.list_keys();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].fingerprint, "SHA256:permanent");
}

#[test]
fn test_parse_ssh_public_key_ed25519() {
    let pubkey = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl user@example.com";
    let result = parse_ssh_public_key(pubkey);
    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.key_type, SshKeyType::Ed25519);
    assert_eq!(info.comment, "user@example.com");
    assert!(!info.fingerprint.is_empty());
}

#[test]
fn test_parse_ssh_public_key_invalid() {
    let result = parse_ssh_public_key("not a valid key");
    assert!(result.is_err());
}

#[test]
fn test_parse_ssh_public_key_rsa() {
    let pubkey = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7 user@example.com";
    let result = parse_ssh_public_key(pubkey);
    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.key_type, SshKeyType::Rsa4096);
}
