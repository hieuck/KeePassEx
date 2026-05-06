//! TOTP/HOTP implementation (RFC 6238 / RFC 4226)

use crate::error::{KeePassExError, Result};
use crate::types::{OtpConfig, OtpAlgorithm, OtpType};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate current TOTP code
pub fn generate_totp(config: &OtpConfig) -> Result<OtpCode> {
    let secret = decode_secret(config.secret.get())?;
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| KeePassExError::OtpFailed(e.to_string()))?
        .as_secs();

    let counter = time / config.period;
    let remaining = config.period - (time % config.period);

    let code = hotp_generate(&secret, counter, config.digits, &config.algorithm)?;

    Ok(OtpCode {
        code,
        remaining_seconds: remaining,
        period: config.period,
        issuer: config.issuer.clone(),
        account: config.account.clone(),
    })
}

/// Generate HOTP code for a specific counter value
pub fn generate_hotp(config: &OtpConfig, counter: u64) -> Result<String> {
    let secret = decode_secret(config.secret.get())?;
    hotp_generate(&secret, counter, config.digits, &config.algorithm)
}

/// Parse OTP URI (otpauth://totp/... or otpauth://hotp/...)
pub fn parse_otp_uri(uri: &str) -> Result<OtpConfig> {
    if !uri.starts_with("otpauth://") {
        return Err(KeePassExError::InvalidOtpSecret);
    }

    let uri = uri.trim_start_matches("otpauth://");
    let (otp_type_str, rest) = uri.split_once('/').ok_or(KeePassExError::InvalidOtpSecret)?;

    let otp_type = match otp_type_str {
        "totp" => OtpType::Totp,
        "hotp" => OtpType::Hotp,
        _ => return Err(KeePassExError::InvalidOtpSecret),
    };

    // Parse label and params
    let (label, params_str) = rest.split_once('?').unwrap_or((rest, ""));

    let label = urlencoding::decode(label)
        .map_err(|_| KeePassExError::InvalidOtpSecret)?
        .to_string();

    let (issuer_from_label, account) = if let Some((i, a)) = label.split_once(':') {
        (Some(i.trim().to_string()), a.trim().to_string())
    } else {
        (None, label)
    };

    let mut secret = String::new();
    let mut algorithm = OtpAlgorithm::Sha1;
    let mut digits = 6u8;
    let mut period = 30u64;
    let mut counter = 0u64;
    let mut issuer: Option<String> = issuer_from_label;

    for param in params_str.split('&') {
        if let Some((key, value)) = param.split_once('=') {
            let value = urlencoding::decode(value)
                .map_err(|_| KeePassExError::InvalidOtpSecret)?
                .to_string();
            match key {
                "secret" => secret = value,
                "algorithm" => {
                    algorithm = match value.to_uppercase().as_str() {
                        "SHA1" => OtpAlgorithm::Sha1,
                        "SHA256" => OtpAlgorithm::Sha256,
                        "SHA512" => OtpAlgorithm::Sha512,
                        _ => OtpAlgorithm::Sha1,
                    }
                }
                "digits" => digits = value.parse().unwrap_or(6),
                "period" => period = value.parse().unwrap_or(30),
                "counter" => counter = value.parse().unwrap_or(0),
                "issuer" => issuer = Some(value),
                _ => {}
            }
        }
    }

    if secret.is_empty() {
        return Err(KeePassExError::InvalidOtpSecret);
    }

    Ok(OtpConfig {
        secret: crate::types::ProtectedString::new(secret),
        algorithm,
        digits,
        period,
        counter,
        otp_type,
        issuer,
        account: Some(account),
    })
}

/// Generate OTP URI from config
pub fn to_otp_uri(config: &OtpConfig) -> String {
    let type_str = match config.otp_type {
        OtpType::Totp => "totp",
        OtpType::Hotp => "hotp",
    };

    let label = match (&config.issuer, &config.account) {
        (Some(issuer), Some(account)) => format!("{}:{}", issuer, account),
        (None, Some(account)) => account.clone(),
        _ => "KeePassEx".to_string(),
    };

    let algo_str = match config.algorithm {
        OtpAlgorithm::Sha1 => "SHA1",
        OtpAlgorithm::Sha256 => "SHA256",
        OtpAlgorithm::Sha512 => "SHA512",
    };

    let mut uri = format!(
        "otpauth://{}/{}?secret={}&algorithm={}&digits={}&period={}",
        type_str,
        urlencoding::encode(&label),
        config.secret.get(),
        algo_str,
        config.digits,
        config.period,
    );

    if let Some(ref issuer) = config.issuer {
        uri.push_str(&format!("&issuer={}", urlencoding::encode(issuer)));
    }

    uri
}

// ─── Internal ─────────────────────────────────────────────────────────────────

fn hotp_generate(secret: &[u8], counter: u64, digits: u8, algorithm: &OtpAlgorithm) -> Result<String> {
    let counter_bytes = counter.to_be_bytes();

    let hash = match algorithm {
        OtpAlgorithm::Sha1 => {
            type HmacSha1 = Hmac<Sha1>;
            let mut mac = HmacSha1::new_from_slice(secret)
                .map_err(|e| KeePassExError::OtpFailed(e.to_string()))?;
            mac.update(&counter_bytes);
            mac.finalize().into_bytes().to_vec()
        }
        OtpAlgorithm::Sha256 => {
            type HmacSha256 = Hmac<Sha256>;
            let mut mac = HmacSha256::new_from_slice(secret)
                .map_err(|e| KeePassExError::OtpFailed(e.to_string()))?;
            mac.update(&counter_bytes);
            mac.finalize().into_bytes().to_vec()
        }
        OtpAlgorithm::Sha512 => {
            type HmacSha512 = Hmac<Sha512>;
            let mut mac = HmacSha512::new_from_slice(secret)
                .map_err(|e| KeePassExError::OtpFailed(e.to_string()))?;
            mac.update(&counter_bytes);
            mac.finalize().into_bytes().to_vec()
        }
    };

    // Dynamic truncation
    let offset = (hash[hash.len() - 1] & 0x0F) as usize;
    let code = ((hash[offset] as u32 & 0x7F) << 24)
        | ((hash[offset + 1] as u32) << 16)
        | ((hash[offset + 2] as u32) << 8)
        | (hash[offset + 3] as u32);

    let modulus = 10u32.pow(digits as u32);
    let otp = code % modulus;

    Ok(format!("{:0>width$}", otp, width = digits as usize))
}

fn decode_secret(secret: &str) -> Result<Vec<u8>> {
    // Base32 decode (standard OTP secret format)
    let secret = secret.to_uppercase().replace(' ', "").replace('-', "");
    base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret)
        .ok_or(KeePassExError::InvalidOtpSecret)
}

/// Result of OTP generation
#[derive(Debug, Clone)]
pub struct OtpCode {
    pub code: String,
    pub remaining_seconds: u64,
    pub period: u64,
    pub issuer: Option<String>,
    pub account: Option<String>,
}

impl OtpCode {
    /// Progress 0.0..=1.0 for UI countdown
    pub fn progress(&self) -> f32 {
        self.remaining_seconds as f32 / self.period as f32
    }
}
