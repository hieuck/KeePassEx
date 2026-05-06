//! Passkey (FIDO2/WebAuthn) tests

use crate::passkey::*;

#[test]
fn test_create_passkey_entry() {
    let entry = create_passkey_entry(
        vec![0xDE, 0xAD, 0xBE, 0xEF],
        "example.com".to_string(),
        "Example".to_string(),
        vec![0x01, 0x02, 0x03],
        "user@example.com".to_string(),
        "Test User".to_string(),
        "-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC7\n-----END PRIVATE KEY-----".to_string(),
        true,
    );

    assert_eq!(entry.rp_id, "example.com");
    assert_eq!(entry.rp_name, "Example");
    assert_eq!(entry.user_name, "user@example.com");
    assert_eq!(entry.sign_count, 0);
    assert!(entry.backup_eligible);
    assert!(!entry.backup_state);
    assert!(entry.last_used_at.is_none());
}

#[test]
fn test_find_passkey_by_rp_and_credential() {
    let pk1 = create_passkey_entry(
        vec![1, 2, 3], "github.com".to_string(), "GitHub".to_string(),
        vec![10], "user1".to_string(), "User 1".to_string(), "key1".to_string(), false,
    );
    let pk2 = create_passkey_entry(
        vec![4, 5, 6], "github.com".to_string(), "GitHub".to_string(),
        vec![20], "user2".to_string(), "User 2".to_string(), "key2".to_string(), false,
    );
    let pk3 = create_passkey_entry(
        vec![7, 8, 9], "google.com".to_string(), "Google".to_string(),
        vec![30], "user3".to_string(), "User 3".to_string(), "key3".to_string(), false,
    );

    let passkeys = vec![pk1, pk2, pk3];

    // Find by exact match
    let found = find_passkey(&passkeys, "github.com", &[1, 2, 3]);
    assert!(found.is_some());
    assert_eq!(found.unwrap().user_name, "user1");

    // Wrong credential ID
    assert!(find_passkey(&passkeys, "github.com", &[99, 99]).is_none());

    // Wrong RP ID
    assert!(find_passkey(&passkeys, "evil.com", &[1, 2, 3]).is_none());
}

#[test]
fn test_find_passkeys_for_rp() {
    let pk1 = create_passkey_entry(
        vec![1], "github.com".to_string(), "GitHub".to_string(),
        vec![1], "user1".to_string(), "User 1".to_string(), "k1".to_string(), false,
    );
    let pk2 = create_passkey_entry(
        vec![2], "github.com".to_string(), "GitHub".to_string(),
        vec![2], "user2".to_string(), "User 2".to_string(), "k2".to_string(), false,
    );
    let pk3 = create_passkey_entry(
        vec![3], "google.com".to_string(), "Google".to_string(),
        vec![3], "user3".to_string(), "User 3".to_string(), "k3".to_string(), false,
    );

    let passkeys = vec![pk1, pk2, pk3];

    let github_keys = find_passkeys_for_rp(&passkeys, "github.com");
    assert_eq!(github_keys.len(), 2);

    let google_keys = find_passkeys_for_rp(&passkeys, "google.com");
    assert_eq!(google_keys.len(), 1);

    let empty = find_passkeys_for_rp(&passkeys, "unknown.com");
    assert!(empty.is_empty());
}

#[test]
fn test_increment_sign_count() {
    let mut pk = create_passkey_entry(
        vec![1], "example.com".to_string(), "Example".to_string(),
        vec![1], "user".to_string(), "User".to_string(), "key".to_string(), false,
    );

    assert_eq!(pk.sign_count, 0);
    assert!(pk.last_used_at.is_none());

    increment_sign_count(&mut pk);
    assert_eq!(pk.sign_count, 1);
    assert!(pk.last_used_at.is_some());

    increment_sign_count(&mut pk);
    assert_eq!(pk.sign_count, 2);
}

#[test]
fn test_registration_options_unique_challenges() {
    let rp = RelyingParty { id: "example.com".to_string(), name: "Example".to_string() };
    let user = UserInfo {
        id: vec![1, 2, 3],
        name: "user@example.com".to_string(),
        display_name: "User".to_string(),
    };

    let opts1 = RegistrationOptions::new(rp.clone(), user.clone(), vec![]);
    let opts2 = RegistrationOptions::new(rp.clone(), user.clone(), vec![]);

    // Challenges must be unique (random)
    assert_ne!(opts1.challenge, opts2.challenge);
    assert_eq!(opts1.challenge.len(), 32);
    assert_eq!(opts2.challenge.len(), 32);
}

#[test]
fn test_registration_options_preferred_algorithms() {
    let opts = RegistrationOptions::new(
        RelyingParty { id: "example.com".to_string(), name: "Example".to_string() },
        UserInfo { id: vec![1], name: "u".to_string(), display_name: "U".to_string() },
        vec![],
    );

    // Should prefer EdDSA (Ed25519) first
    assert_eq!(opts.pub_key_cred_params[0].alg, -8); // EdDSA
    assert_eq!(opts.pub_key_cred_params[1].alg, -7); // ES256
    assert_eq!(opts.pub_key_cred_params[2].alg, -257); // RS256
}

#[test]
fn test_assertion_options_unique_challenges() {
    let opts1 = AssertionOptions::new("example.com".to_string(), vec![]);
    let opts2 = AssertionOptions::new("example.com".to_string(), vec![]);

    assert_ne!(opts1.challenge, opts2.challenge);
    assert_eq!(opts1.challenge.len(), 32);
}

#[test]
fn test_credential_descriptor_from_passkey() {
    let pk = create_passkey_entry(
        vec![0xAB, 0xCD, 0xEF],
        "example.com".to_string(),
        "Example".to_string(),
        vec![1],
        "user".to_string(),
        "User".to_string(),
        "key".to_string(),
        false,
    );

    let desc = CredentialDescriptor::from_passkey(&pk);
    assert_eq!(desc.id, vec![0xAB, 0xCD, 0xEF]);
    assert_eq!(desc.type_, "public-key");
    assert!(desc.transports.contains(&AuthenticatorTransport::Internal));
}

#[test]
fn test_authenticator_selection_defaults() {
    let sel = AuthenticatorSelection::default();
    assert_eq!(sel.authenticator_attachment, Some(AuthenticatorAttachment::Platform));
    assert_eq!(sel.resident_key, ResidentKeyRequirement::Required);
    assert!(sel.require_resident_key);
    assert_eq!(sel.user_verification, UserVerificationRequirement::Required);
}
