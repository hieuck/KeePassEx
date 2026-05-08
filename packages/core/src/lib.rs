//! KeePassEx Core Engine
//!
//! Provides KDBX 4.x vault management, cryptography, and all business logic.
//! This crate is used by all platform apps (desktop, mobile, CLI, watch).
//! 32 modules, 650+ unit tests.

pub mod ai;
pub mod analytics;
pub mod audit_log;
pub mod breach;
pub mod cache;
pub mod categorizer;
pub mod crypto;
pub mod decoy_vault;
pub mod emergency_access;
pub mod error;
pub mod expiry_engine;
pub mod favicon;
pub mod field_references;
pub mod generator;
pub mod hardware_key;
pub mod health;
pub mod import_export;
pub mod kdbx;
pub mod notifications;
pub mod otp;
pub mod passkey;
pub mod password_advisor;
pub mod password_policy;
pub mod plugin;
pub mod scheduled_backup;
pub mod search;
pub mod ssh;
pub mod steg;
pub mod sync;
pub mod team;
pub mod templates;
pub mod types;
pub mod vault;
pub mod vault_compare;
pub mod zkpv;

#[cfg(test)]
mod tests;

pub use error::{KeePassExError, Result};
pub use types::*;
pub use vault::Vault;
