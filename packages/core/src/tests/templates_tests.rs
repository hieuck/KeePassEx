//! Entry templates tests — uses the public built_in_templates() and TemplateManager APIs

use crate::templates::{built_in_templates, EntryTemplate, TemplateFieldType, TemplateManager};

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
fn test_all_built_in_have_non_empty_names() {
    for tmpl in built_in_templates() {
        assert!(
            !tmpl.name.is_empty(),
            "Template '{}' has empty name",
            tmpl.id
        );
    }
}

#[test]
fn test_all_built_in_have_at_least_one_field() {
    for tmpl in built_in_templates() {
        assert!(
            !tmpl.fields.is_empty(),
            "Template '{}' has no fields",
            tmpl.id
        );
    }
}

#[test]
fn test_template_manager_get_login() {
    let manager = TemplateManager::new();
    let login = manager.get("builtin.login").unwrap();
    assert_eq!(login.id, "builtin.login");
    let field_keys: Vec<&str> = login.fields.iter().map(|f| f.key.as_str()).collect();
    assert!(field_keys.contains(&"UserName"));
    assert!(field_keys.contains(&"Password"));
    assert!(field_keys.contains(&"URL"));
}

#[test]
fn test_template_manager_get_credit_card() {
    let manager = TemplateManager::new();
    let card = manager.get("builtin.credit_card").unwrap();
    let field_keys: Vec<&str> = card.fields.iter().map(|f| f.key.as_str()).collect();
    assert!(field_keys.contains(&"CardNumber"));
    assert!(field_keys.contains(&"CVV"));
}

#[test]
fn test_credit_card_sensitive_fields_are_protected() {
    let manager = TemplateManager::new();
    let card = manager.get("builtin.credit_card").unwrap();
    let card_number = card.fields.iter().find(|f| f.key == "CardNumber").unwrap();
    let cvv = card.fields.iter().find(|f| f.key == "CVV").unwrap();
    assert!(card_number.protected);
    assert!(cvv.protected);
}

#[test]
fn test_template_manager_get_nonexistent_returns_none() {
    let manager = TemplateManager::new();
    assert!(manager.get("builtin.nonexistent_xyz").is_none());
}

#[test]
fn test_ssh_key_template_has_key_type_field() {
    let manager = TemplateManager::new();
    let tmpl = manager.get("builtin.ssh_key").unwrap();
    let field_keys: Vec<&str> = tmpl.fields.iter().map(|f| f.key.as_str()).collect();
    assert!(field_keys.contains(&"KeyType"));
}

#[test]
fn test_crypto_wallet_template_has_seed_phrase() {
    let manager = TemplateManager::new();
    let tmpl = manager.get("builtin.crypto_wallet").unwrap();
    let seed = tmpl.fields.iter().find(|f| f.key == "SeedPhrase").unwrap();
    assert!(seed.protected, "Seed phrase should be protected");
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
fn test_cannot_add_duplicate_template_id() {
    let mut manager = TemplateManager::new();
    let custom = EntryTemplate {
        id: "custom.dup".to_string(),
        name: "Dup".to_string(),
        description: "".to_string(),
        icon_id: 0,
        fields: vec![],
        is_built_in: false,
        auto_type_sequence: None,
    };
    manager.add_custom(custom.clone()).unwrap();
    let result = manager.add_custom(custom);
    assert!(result.is_err());
}

#[test]
fn test_cannot_remove_built_in_template() {
    let mut manager = TemplateManager::new();
    let result = manager.remove_custom("builtin.login");
    assert!(result.is_err());
}

#[test]
fn test_login_template_has_auto_type_sequence() {
    let manager = TemplateManager::new();
    let login = manager.get("builtin.login").unwrap();
    assert!(login.auto_type_sequence.is_some());
    assert!(login
        .auto_type_sequence
        .as_ref()
        .unwrap()
        .contains("{USERNAME}"));
}

#[test]
fn test_template_field_types_are_correct() {
    let manager = TemplateManager::new();
    let login = manager.get("builtin.login").unwrap();
    let password_field = login.fields.iter().find(|f| f.key == "Password").unwrap();
    assert_eq!(password_field.field_type, TemplateFieldType::Password);
    let url_field = login.fields.iter().find(|f| f.key == "URL").unwrap();
    assert_eq!(url_field.field_type, TemplateFieldType::Url);
}
