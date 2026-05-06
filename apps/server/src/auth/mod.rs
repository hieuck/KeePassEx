//! Authentication — JWT + Argon2id password hashing
//!
//! # Zero-knowledge design
//! The server stores only Argon2id hashes of passwords — never plaintext.
//! The vault itself is encrypted client-side and the server cannot decrypt it.
//! This means even a compromised server cannot access vault contents.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use crate::config::ServerConfig;
use crate::error::{Result, ServerError};

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Email
    pub email: String,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiry (Unix timestamp)
    pub exp: i64,
    /// Token type: "access" or "refresh"
    pub token_type: String,
}

/// Hash a password with Argon2id
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| ServerError::Internal(format!("Password hashing failed: {}", e)))
}

/// Verify a password against an Argon2id hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| ServerError::Internal(format!("Invalid password hash: {}", e)))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generate a JWT access token
pub fn generate_access_token(user_id: &str, email: &str, config: &ServerConfig) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ServerConfig::TOKEN_EXPIRY_SECS as i64);

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
        token_type: "access".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| ServerError::Internal(format!("JWT generation failed: {}", e)))
}

/// Generate a JWT refresh token
pub fn generate_refresh_token(user_id: &str, email: &str, config: &ServerConfig) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ServerConfig::REFRESH_TOKEN_EXPIRY_SECS as i64);

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
        token_type: "refresh".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| ServerError::Internal(format!("Refresh token generation failed: {}", e)))
}

/// Validate a JWT token and return the claims
pub fn validate_token(token: &str, config: &ServerConfig) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| ServerError::Unauthorized)
}

/// Hash a refresh token for storage (we store the hash, not the token itself)
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Extract Bearer token from Authorization header
pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ")
}
