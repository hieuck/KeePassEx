//! Entry Templates — predefined entry types for common use cases
//!
//! Templates define a set of fields that are pre-populated when creating
//! a new entry of that type. Built-in templates cover the most common
//! credential types; users can also create custom templates.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Types ────────────────────────────────────────────────────────────────────

/// Field data type for template fields
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateFieldType {
    Text,
    Password,
    Url,
    Email,
    Date,
    Number,
    Multiline,
}

/// A single field definition in a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateField {
    pub key: String,
    pub label: String,
    pub field_type: TemplateFieldType,
    pub protected: bool,
    pub required: bool,
    pub placeholder: Option<String>,
    pub default_value: Option<String>,
}

/// An entry template (built-in or user-defined)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_id: u32,
    pub fields: Vec<TemplateField>,
    pub is_built_in: bool,
    pub auto_type_sequence: Option<String>,
}

// ─── Built-in Templates ───────────────────────────────────────────────────────

/// Returns all built-in entry templates
pub fn built_in_templates() -> Vec<EntryTemplate> {
    vec![
        login_template(),
        credit_card_template(),
        bank_account_template(),
        identity_template(),
        secure_note_template(),
        software_license_template(),
        wireless_router_template(),
        passport_template(),
        driver_license_template(),
        ssh_key_template(),
        api_key_template(),
        crypto_wallet_template(),
    ]
}

fn login_template() -> EntryTemplate {
    EntryTemplate {
        id: "builtin.login".to_string(),
        name: "Login".to_string(),
        description: "Standard username/password login".to_string(),
        icon_id: 1,
        is_built_in: true,
        auto_type_sequence: Some("{USERNAME}{TAB}{PASSWORD}{ENTER}".to_string()),
        fields: vec![
            TemplateField {
                key: "UserName".to_string(),
                label: "Username".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: Some("username or email".to_string()),
                default_value: None,
            },
            TemplateField {
                key: "Password".to_string(),
                label: "Password".to_string(),
                field_type: TemplateFieldType::Password,
                protected: true,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "URL".to_string(),
                label: "URL".to_string(),
                field_type: TemplateFieldType::Url,
                protected: false,
                required: false,
                placeholder: Some("https://".to_string()),
                default_value: None,
            },
        ],
    }
}

fn credit_card_template() -> EntryTemplate {
    EntryTemplate {
        id: "builtin.credit_card".to_string(),
        name: "Credit Card".to_string(),
        description: "Credit or debit card details".to_string(),
        icon_id: 18,
        is_built_in: true,
        auto_type_sequence: None,
        fields: vec![
            TemplateField {
                key: "CardNumber".to_string(),
                label: "Card Number".to_string(),
                field_type: TemplateFieldType::Text,
                protected: true,
                required: true,
                placeholder: Some("1234 5678 9012 3456".to_string()),
                default_value: None,
            },
            TemplateField {
                key: "CardHolder".to_string(),
                label: "Cardholder Name".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "ExpiryDate".to_string(),
                label: "Expiry Date".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: Some("MM/YY".to_string()),
                default_value: None,
            },
            TemplateField {
                key: "CVV".to_string(),
                label: "CVV / CVC".to_string(),
                field_type: TemplateFieldType::Password,
                protected: true,
                required: false,
                placeholder: Some("3 or 4 digits".to_string()),
                default_value: None,
            },
            TemplateField {
                key: "PIN".to_string(),
                label: "PIN".to_string(),
                field_type: TemplateFieldType::Password,
                protected: true,
                required: false,
                placeholder: None,
                default_value: None,
            },
        ],
    }
}

fn bank_account_template() -> EntryTemplate {
    EntryTemplate {
        id: "builtin.bank_account".to_string(),
        name: "Bank Account".to_string(),
        description: "Bank account and routing information".to_string(),
        icon_id: 37,
        is_built_in: true,
        auto_type_sequence: None,
        fields: vec![
            TemplateField {
                key: "BankName".to_string(),
                label: "Bank Name".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "AccountNumber".to_string(),
                label: "Account Number".to_string(),
                field_type: TemplateFieldType::Text,
                protected: true,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "RoutingNumber".to_string(),
                label: "Routing / SWIFT / IBAN".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "PIN".to_string(),
                label: "PIN".to_string(),
                field_type: TemplateFieldType::Password,
                protected: true,
                required: false,
                placeholder: None,
                default_value: None,
            },
        ],
    }
}

fn identity_template() -> EntryTemplate {
    EntryTemplate {
        id: "builtin.identity".to_string(),
        name: "Identity".to_string(),
        description: "Personal identity information".to_string(),
        icon_id: 2,
        is_built_in: true,
        auto_type_sequence: None,
        fields: vec![
            TemplateField {
                key: "FirstName".to_string(),
                label: "First Name".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "LastName".to_string(),
                label: "Last Name".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "Email".to_string(),
                label: "Email".to_string(),
                field_type: TemplateFieldType::Email,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "Phone".to_string(),
                label: "Phone".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "DateOfBirth".to_string(),
                label: "Date of Birth".to_string(),
                field_type: TemplateFieldType::Date,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
        ],
    }
}

fn secure_note_template() -> EntryTemplate {
    EntryTemplate {
        id: "builtin.secure_note".to_string(),
        name: "Secure Note".to_string(),
        description: "Encrypted text note".to_string(),
        icon_id: 22,
        is_built_in: true,
        auto_type_sequence: None,
        fields: vec![
            TemplateField {
                key: "Notes".to_string(),
                label: "Note".to_string(),
                field_type: TemplateFieldType::Multiline,
                protected: true,
                required: true,
                placeholder: Some("Enter your secure note here...".to_string()),
                default_value: None,
            },
        ],
    }
}

fn software_license_template() -> EntryTemplate {
    EntryTemplate {
        id: "builtin.software_license".to_string(),
        name: "Software License".to_string(),
        description: "Software license key and registration info".to_string(),
        icon_id: 38,
        is_built_in: true,
        auto_type_sequence: None,
        fields: vec![
            TemplateField {
                key: "LicenseKey".to_string(),
                label: "License Key".to_string(),
                field_type: TemplateFieldType::Password,
                protected: true,
                required: true,
                placeholder: Some("XXXX-XXXX-XXXX-XXXX".to_string()),
                default_value: None,
            },
            TemplateField {
                key: "RegisteredTo".to_string(),
                label: "Registered To".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "Version".to_string(),
                label: "Version".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "PurchaseDate".to_string(),
                label: "Purchase Date".to_string(),
                field_type: TemplateFieldType::Date,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
        ],
    }
}

fn wireless_router_template() -> EntryTemplate {
    EntryTemplate {
        id: "builtin.wireless_router".to_string(),
        name: "Wireless Router".to_string(),
        description: "Wi-Fi network credentials".to_string(),
        icon_id: 12,
        is_built_in: true,
        auto_type_sequence: None,
        fields: vec![
            TemplateField {
                key: "SSID".to_string(),
                label: "Network Name (SSID)".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: true,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "Password".to_string(),
                label: "Wi-Fi Password".to_string(),
                field_type: TemplateFieldType::Password,
                protected: true,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "RouterIP".to_string(),
                label: "Router IP".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: Some("192.168.1.1".to_string()),
                default_value: None,
            },
            TemplateField {
                key: "AdminPassword".to_string(),
                label: "Admin Password".to_string(),
                field_type: TemplateFieldType::Password,
                protected: true,
                required: false,
                placeholder: None,
                default_value: None,
            },
        ],
    }
}

fn passport_template() -> EntryTemplate {
    EntryTemplate {
        id: "builtin.passport".to_string(),
        name: "Passport".to_string(),
        description: "Passport and travel document details".to_string(),
        icon_id: 2,
        is_built_in: true,
        auto_type_sequence: None,
        fields: vec![
            TemplateField {
                key: "PassportNumber".to_string(),
                label: "Passport Number".to_string(),
                field_type: TemplateFieldType::Text,
                protected: true,
                required: true,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "IssuingCountry".to_string(),
                label: "Issuing Country".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "IssueDate".to_string(),
                label: "Issue Date".to_string(),
                field_type: TemplateFieldType::Date,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "ExpiryDate".to_string(),
                label: "Expiry Date".to_string(),
                field_type: TemplateFieldType::Date,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
        ],
    }
}

fn driver_license_template() -> EntryTemplate {
    EntryTemplate {
        id: "builtin.driver_license".to_string(),
        name: "Driver's License".to_string(),
        description: "Driver's license details".to_string(),
        icon_id: 2,
        is_built_in: true,
        auto_type_sequence: None,
        fields: vec![
            TemplateField {
                key: "LicenseNumber".to_string(),
                label: "License Number".to_string(),
                field_type: TemplateFieldType::Text,
                protected: true,
                required: true,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "State".to_string(),
                label: "State / Province".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "ExpiryDate".to_string(),
                label: "Expiry Date".to_string(),
                field_type: TemplateFieldType::Date,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
        ],
    }
}

fn ssh_key_template() -> EntryTemplate {
    EntryTemplate {
        id: "builtin.ssh_key".to_string(),
        name: "SSH Key".to_string(),
        description: "SSH key pair with passphrase".to_string(),
        icon_id: 30,
        is_built_in: true,
        auto_type_sequence: None,
        fields: vec![
            TemplateField {
                key: "KeyType".to_string(),
                label: "Key Type".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: Some("Ed25519".to_string()),
                default_value: Some("Ed25519".to_string()),
            },
            TemplateField {
                key: "Comment".to_string(),
                label: "Comment".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: Some("user@hostname".to_string()),
                default_value: None,
            },
        ],
    }
}

fn api_key_template() -> EntryTemplate {
    EntryTemplate {
        id: "builtin.api_key".to_string(),
        name: "API Key".to_string(),
        description: "API key or token for a service".to_string(),
        icon_id: 38,
        is_built_in: true,
        auto_type_sequence: None,
        fields: vec![
            TemplateField {
                key: "APIKey".to_string(),
                label: "API Key".to_string(),
                field_type: TemplateFieldType::Password,
                protected: true,
                required: true,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "APISecret".to_string(),
                label: "API Secret".to_string(),
                field_type: TemplateFieldType::Password,
                protected: true,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "Endpoint".to_string(),
                label: "API Endpoint".to_string(),
                field_type: TemplateFieldType::Url,
                protected: false,
                required: false,
                placeholder: Some("https://api.example.com".to_string()),
                default_value: None,
            },
        ],
    }
}

fn crypto_wallet_template() -> EntryTemplate {
    EntryTemplate {
        id: "builtin.crypto_wallet".to_string(),
        name: "Crypto Wallet".to_string(),
        description: "Cryptocurrency wallet seed and keys".to_string(),
        icon_id: 37,
        is_built_in: true,
        auto_type_sequence: None,
        fields: vec![
            TemplateField {
                key: "SeedPhrase".to_string(),
                label: "Seed Phrase (BIP-39)".to_string(),
                field_type: TemplateFieldType::Multiline,
                protected: true,
                required: false,
                placeholder: Some("word1 word2 word3 ... word24".to_string()),
                default_value: None,
            },
            TemplateField {
                key: "WalletAddress".to_string(),
                label: "Wallet Address".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "PrivateKey".to_string(),
                label: "Private Key".to_string(),
                field_type: TemplateFieldType::Password,
                protected: true,
                required: false,
                placeholder: None,
                default_value: None,
            },
            TemplateField {
                key: "Network".to_string(),
                label: "Network".to_string(),
                field_type: TemplateFieldType::Text,
                protected: false,
                required: false,
                placeholder: Some("Bitcoin, Ethereum, etc.".to_string()),
                default_value: None,
            },
        ],
    }
}

// ─── Template Manager ─────────────────────────────────────────────────────────

/// Manages built-in and user-defined templates
pub struct TemplateManager {
    templates: Vec<EntryTemplate>,
}

impl TemplateManager {
    pub fn new() -> Self {
        Self {
            templates: built_in_templates(),
        }
    }

    pub fn all(&self) -> &[EntryTemplate] {
        &self.templates
    }

    pub fn get(&self, id: &str) -> Option<&EntryTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    pub fn add_custom(&mut self, template: EntryTemplate) -> Result<()> {
        if self.templates.iter().any(|t| t.id == template.id) {
            return Err(crate::error::KeePassExError::Other(
                format!("Template with id '{}' already exists", template.id)
            ));
        }
        self.templates.push(template);
        Ok(())
    }

    pub fn remove_custom(&mut self, id: &str) -> Result<()> {
        let pos = self.templates.iter().position(|t| t.id == id && !t.is_built_in)
            .ok_or_else(|| crate::error::KeePassExError::Other(
                format!("Custom template '{}' not found", id)
            ))?;
        self.templates.remove(pos);
        Ok(())
    }

    pub fn built_in(&self) -> impl Iterator<Item = &EntryTemplate> {
        self.templates.iter().filter(|t| t.is_built_in)
    }

    pub fn custom(&self) -> impl Iterator<Item = &EntryTemplate> {
        self.templates.iter().filter(|t| !t.is_built_in)
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_built_in_templates_count() {
        let templates = built_in_templates();
        assert_eq!(templates.len(), 12);
    }

    #[test]
    fn test_all_built_in_have_unique_ids() {
        let templates = built_in_templates();
        let mut ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), templates.len(), "Duplicate template IDs found");
    }

    #[test]
    fn test_template_manager_get() {
        let manager = TemplateManager::new();
        assert!(manager.get("builtin.login").is_some());
        assert!(manager.get("builtin.credit_card").is_some());
        assert!(manager.get("nonexistent").is_none());
    }

    #[test]
    fn test_login_template_has_required_fields() {
        let manager = TemplateManager::new();
        let login = manager.get("builtin.login").unwrap();
        let field_keys: Vec<&str> = login.fields.iter().map(|f| f.key.as_str()).collect();
        assert!(field_keys.contains(&"UserName"));
        assert!(field_keys.contains(&"Password"));
        assert!(field_keys.contains(&"URL"));
    }

    #[test]
    fn test_credit_card_password_fields_are_protected() {
        let manager = TemplateManager::new();
        let card = manager.get("builtin.credit_card").unwrap();
        let card_number = card.fields.iter().find(|f| f.key == "CardNumber").unwrap();
        let cvv = card.fields.iter().find(|f| f.key == "CVV").unwrap();
        assert!(card_number.protected);
        assert!(cvv.protected);
    }

    #[test]
    fn test_add_and_remove_custom_template() {
        let mut manager = TemplateManager::new();
        let custom = EntryTemplate {
            id: "custom.test".to_string(),
            name: "Test Template".to_string(),
            description: "A test template".to_string(),
            icon_id: 0,
            fields: vec![],
            is_built_in: false,
            auto_type_sequence: None,
        };
        manager.add_custom(custom).unwrap();
        assert!(manager.get("custom.test").is_some());
        manager.remove_custom("custom.test").unwrap();
        assert!(manager.get("custom.test").is_none());
    }

    #[test]
    fn test_cannot_remove_built_in_template() {
        let mut manager = TemplateManager::new();
        let result = manager.remove_custom("builtin.login");
        assert!(result.is_err());
    }
}
