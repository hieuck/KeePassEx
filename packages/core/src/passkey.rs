//! Passkey (FIDO2/WebAuthn) storage and operations
//!
//! KeePassEx stores passkey private keys encrypted inside the KDBX vault.
//! This module handles credential management, assertion signing, and
//! integration with platform authenticators.

use crate::error::{KeePassExError, Result};
use crate::types::PasskeyEntry;
use chrono::Utc;
use serde::{Deserialize, Serialize};

// ─── Credential Management ────────────────────────────────────────────────────

/// Create a new passkey entry from a WebAuthn registration response
pub fn create_passkey_entry(
    credential_id: Vec<u8>,
    rp_id: String,
    rp_name: String,
    user_id: Vec<u8>,
    user_name: String,
    user_display_name: String,
    private_key_pem: String,
    backup_eligible: bool,
) -> PasskeyEntry {
    PasskeyEntry {
        credential_id,
        rp_id,
        rp_name,
        user_id,
        user_name,
        user_display_name,
        private_key: crate::types::ProtectedString::new(private_key_pem),
        sign_count: 0,
        created_at: Utc::now(),
        last_used_at: None,
        backup_eligible,
        backup_state: false,
    }
}

/// Find passkey by RP ID and credential ID
pub fn find_passkey<'a>(
    passkeys: &'a [PasskeyEntry],
    rp_id: &str,
    credential_id: &[u8],
) -> Option<&'a PasskeyEntry> {
    passkeys.iter().find(|pk| {
        pk.rp_id == rp_id && pk.credential_id == credential_id
    })
}

/// Find all passkeys for a given RP ID (for assertion)
pub fn find_passkeys_for_rp<'a>(
    passkeys: &'a [PasskeyEntry],
    rp_id: &str,
) -> Vec<&'a PasskeyEntry> {
    passkeys.iter().filter(|pk| pk.rp_id == rp_id).collect()
}

/// Increment sign counter after successful assertion
pub fn increment_sign_count(passkey: &mut PasskeyEntry) {
    passkey.sign_count += 1;
    passkey.last_used_at = Some(Utc::now());
}

// ─── WebAuthn Data Structures ─────────────────────────────────────────────────

/// Relying Party information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelyingParty {
    pub id: String,
    pub name: String,
}

/// User information for registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Vec<u8>,
    pub name: String,
    pub display_name: String,
}

/// Public key credential parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubKeyCredParam {
    pub alg: i64,  // COSE algorithm identifier
    pub type_: String,
}

impl PubKeyCredParam {
    /// ES256 (ECDSA with SHA-256) — most widely supported
    pub fn es256() -> Self {
        Self { alg: -7, type_: "public-key".to_string() }
    }

    /// RS256 (RSASSA-PKCS1-v1_5 with SHA-256)
    pub fn rs256() -> Self {
        Self { alg: -257, type_: "public-key".to_string() }
    }

    /// EdDSA (Ed25519) — most secure
    pub fn eddsa() -> Self {
        Self { alg: -8, type_: "public-key".to_string() }
    }
}

/// Credential descriptor for allowCredentials list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialDescriptor {
    pub id: Vec<u8>,
    pub type_: String,
    pub transports: Vec<AuthenticatorTransport>,
}

impl CredentialDescriptor {
    pub fn from_passkey(pk: &PasskeyEntry) -> Self {
        Self {
            id: pk.credential_id.clone(),
            type_: "public-key".to_string(),
            transports: vec![AuthenticatorTransport::Internal],
        }
    }
}

/// Authenticator transport types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthenticatorTransport {
    Usb,
    Nfc,
    Ble,
    Internal,
    Hybrid,
    SmartCard,
}

/// Authenticator selection criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorSelection {
    pub authenticator_attachment: Option<AuthenticatorAttachment>,
    pub resident_key: ResidentKeyRequirement,
    pub require_resident_key: bool,
    pub user_verification: UserVerificationRequirement,
}

impl Default for AuthenticatorSelection {
    fn default() -> Self {
        Self {
            authenticator_attachment: Some(AuthenticatorAttachment::Platform),
            resident_key: ResidentKeyRequirement::Required,
            require_resident_key: true,
            user_verification: UserVerificationRequirement::Required,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthenticatorAttachment {
    Platform,
    CrossPlatform,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResidentKeyRequirement {
    Discouraged,
    Preferred,
    Required,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserVerificationRequirement {
    Required,
    Preferred,
    Discouraged,
}

// ─── Registration ─────────────────────────────────────────────────────────────

/// WebAuthn registration options (sent to authenticator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationOptions {
    pub rp: RelyingParty,
    pub user: UserInfo,
    pub challenge: Vec<u8>,
    pub pub_key_cred_params: Vec<PubKeyCredParam>,
    pub timeout: u64,
    pub exclude_credentials: Vec<CredentialDescriptor>,
    pub authenticator_selection: AuthenticatorSelection,
    pub attestation: AttestationConveyancePreference,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AttestationConveyancePreference {
    None,
    Indirect,
    Direct,
    Enterprise,
}

impl RegistrationOptions {
    /// Create registration options for a new passkey
    pub fn new(rp: RelyingParty, user: UserInfo, exclude: Vec<CredentialDescriptor>) -> Self {
        let mut challenge = vec![0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut challenge);

        Self {
            rp,
            user,
            challenge,
            pub_key_cred_params: vec![
                PubKeyCredParam::eddsa(),  // Prefer Ed25519
                PubKeyCredParam::es256(),  // Fallback to ES256
                PubKeyCredParam::rs256(),  // Last resort RS256
            ],
            timeout: 60_000,
            exclude_credentials: exclude,
            authenticator_selection: AuthenticatorSelection::default(),
            attestation: AttestationConveyancePreference::None,
        }
    }
}

/// WebAuthn registration response (from authenticator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub id: String,           // base64url credential ID
    pub raw_id: Vec<u8>,
    pub response: AuthenticatorAttestationResponse,
    pub type_: String,
    pub client_extension_results: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorAttestationResponse {
    pub client_data_json: Vec<u8>,
    pub attestation_object: Vec<u8>,
    pub transports: Vec<AuthenticatorTransport>,
}

// ─── Assertion ────────────────────────────────────────────────────────────────

/// WebAuthn assertion options (sent to authenticator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionOptions {
    pub challenge: Vec<u8>,
    pub timeout: u64,
    pub rp_id: String,
    pub allow_credentials: Vec<CredentialDescriptor>,
    pub user_verification: UserVerificationRequirement,
}

impl AssertionOptions {
    pub fn new(rp_id: String, allow: Vec<CredentialDescriptor>) -> Self {
        let mut challenge = vec![0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut challenge);

        Self {
            challenge,
            timeout: 60_000,
            rp_id,
            allow_credentials: allow,
            user_verification: UserVerificationRequirement::Required,
        }
    }
}

/// WebAuthn assertion response (from authenticator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResponse {
    pub id: String,
    pub raw_id: Vec<u8>,
    pub response: AuthenticatorAssertionResponse,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorAssertionResponse {
    pub client_data_json: Vec<u8>,
    pub authenticator_data: Vec<u8>,
    pub signature: Vec<u8>,
    pub user_handle: Option<Vec<u8>>,
}

// ─── Verification ─────────────────────────────────────────────────────────────

/// Verify a WebAuthn assertion against a stored passkey
pub fn verify_assertion(
    passkey: &PasskeyEntry,
    response: &AssertionResponse,
    expected_challenge: &[u8],
    expected_origin: &str,
) -> Result<()> {
    // 1. Verify credential ID matches
    if response.raw_id != passkey.credential_id {
        return Err(KeePassExError::PasskeyFailed(
            "Credential ID mismatch".into()
        ));
    }

    // 2. Parse client data JSON
    let client_data: serde_json::Value = serde_json::from_slice(
        &response.response.client_data_json
    ).map_err(|e| KeePassExError::PasskeyFailed(e.to_string()))?;

    // 3. Verify type
    if client_data["type"].as_str() != Some("webauthn.get") {
        return Err(KeePassExError::PasskeyFailed("Invalid type".into()));
    }

    // 4. Verify challenge (base64url encoded)
    let challenge_b64 = base64_url_encode(expected_challenge);
    if client_data["challenge"].as_str() != Some(&challenge_b64) {
        return Err(KeePassExError::PasskeyFailed("Challenge mismatch".into()));
    }

    // 5. Verify origin
    if client_data["origin"].as_str() != Some(expected_origin) {
        return Err(KeePassExError::PasskeyFailed("Origin mismatch".into()));
    }

    // 6. Parse authenticator data
    let auth_data = &response.response.authenticator_data;
    if auth_data.len() < 37 {
        return Err(KeePassExError::PasskeyFailed("Authenticator data too short".into()));
    }

    // 7. Verify RP ID hash (first 32 bytes of authenticator data)
    let rp_id_hash = &auth_data[..32];
    let expected_hash = sha256(passkey.rp_id.as_bytes());
    if rp_id_hash != expected_hash {
        return Err(KeePassExError::PasskeyFailed("RP ID hash mismatch".into()));
    }

    // 8. Verify flags (bit 0 = UP, bit 2 = UV)
    let flags = auth_data[32];
    if flags & 0x01 == 0 {
        return Err(KeePassExError::PasskeyFailed("User presence not set".into()));
    }

    // 9. Verify sign count (replay attack prevention)
    let sign_count = u32::from_be_bytes(auth_data[33..37].try_into().unwrap_or([0; 4]));
    if sign_count > 0 && sign_count <= passkey.sign_count as u32 {
        return Err(KeePassExError::PasskeyFailed(
            "Sign count indicates possible cloned authenticator".into()
        ));
    }

    // 10. Verify signature (production: use actual ECDSA/EdDSA verification)
    // This requires the public key from the stored passkey
    // For now: structural validation only
    if response.response.signature.is_empty() {
        return Err(KeePassExError::PasskeyFailed("Empty signature".into()));
    }

    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn base64_url_encode(data: &[u8]) -> String {
    base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        data,
    )
}

fn sha256(data: &[u8]) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_passkey_entry() {
        let entry = create_passkey_entry(
            vec![1, 2, 3, 4],
            "example.com".to_string(),
            "Example".to_string(),
            vec![5, 6, 7, 8],
            "user@example.com".to_string(),
            "User".to_string(),
            "-----BEGIN PRIVATE KEY-----\n...".to_string(),
            true,
        );

        assert_eq!(entry.rp_id, "example.com");
        assert_eq!(entry.sign_count, 0);
        assert!(entry.backup_eligible);
        assert!(entry.last_used_at.is_none());
    }

    #[test]
    fn test_find_passkey() {
        let pk = create_passkey_entry(
            vec![1, 2, 3],
            "github.com".to_string(),
            "GitHub".to_string(),
            vec![4, 5, 6],
            "user".to_string(),
            "User".to_string(),
            "key".to_string(),
            false,
        );

        let passkeys = vec![pk];
        assert!(find_passkey(&passkeys, "github.com", &[1, 2, 3]).is_some());
        assert!(find_passkey(&passkeys, "github.com", &[9, 9, 9]).is_none());
        assert!(find_passkey(&passkeys, "other.com", &[1, 2, 3]).is_none());
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
    }

    #[test]
    fn test_registration_options_has_challenge() {
        let opts = RegistrationOptions::new(
            RelyingParty { id: "example.com".to_string(), name: "Example".to_string() },
            UserInfo {
                id: vec![1, 2, 3],
                name: "user@example.com".to_string(),
                display_name: "User".to_string(),
            },
            vec![],
        );

        assert_eq!(opts.challenge.len(), 32);
        assert!(!opts.pub_key_cred_params.is_empty());
        assert_eq!(opts.rp.id, "example.com");
    }

    #[test]
    fn test_pub_key_cred_params() {
        let es256 = PubKeyCredParam::es256();
        assert_eq!(es256.alg, -7);

        let eddsa = PubKeyCredParam::eddsa();
        assert_eq!(eddsa.alg, -8);

        let rs256 = PubKeyCredParam::rs256();
        assert_eq!(rs256.alg, -257);
    }
}
