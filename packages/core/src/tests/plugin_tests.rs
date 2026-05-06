//! Plugin system tests — uses the public PluginManifest, PluginRegistry, PluginCapability APIs

use crate::plugin::{
    dashlane_importer_manifest, enpass_importer_manifest, InstalledPlugin, PluginCapability,
    PluginManifest, PluginPermission, PluginRegistry,
};
use std::path::PathBuf;

fn sample_manifest() -> PluginManifest {
    PluginManifest {
        id: "com.example.test-plugin".to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        author: "Test Author".to_string(),
        license: "MIT".to_string(),
        capabilities: vec![PluginCapability::PasswordGenerator],
        permissions: vec![],
        entry_point: "plugin.wasm".to_string(),
        min_keepassex_version: "1.0.0".to_string(),
    }
}

#[test]
fn test_plugin_manifest_serialization() {
    let manifest = sample_manifest();
    let json = serde_json::to_string(&manifest).unwrap();
    let deserialized: PluginManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, manifest.id);
    assert_eq!(deserialized.name, manifest.name);
    assert_eq!(deserialized.version, manifest.version);
    assert_eq!(deserialized.author, manifest.author);
    assert_eq!(deserialized.license, manifest.license);
}

#[test]
fn test_plugin_capability_variants_serialize() {
    let caps = vec![
        PluginCapability::PasswordGenerator,
        PluginCapability::Importer,
        PluginCapability::Exporter,
        PluginCapability::HealthCheck,
        PluginCapability::FieldValidator,
        PluginCapability::IconSet,
        PluginCapability::UiExtension,
    ];
    for cap in &caps {
        let json = serde_json::to_string(cap).unwrap();
        assert!(!json.is_empty());
        let deserialized: PluginCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, *cap);
    }
}

#[test]
fn test_plugin_permission_variants_serialize() {
    let perms = vec![
        PluginPermission::ReadEntryMetadata,
        PluginPermission::ReadPasswords,
        PluginPermission::WriteEntries,
        PluginPermission::Network,
        PluginPermission::FileSystem,
    ];
    for perm in &perms {
        let json = serde_json::to_string(perm).unwrap();
        assert!(!json.is_empty());
        let deserialized: PluginPermission = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, *perm);
    }
}

#[test]
fn test_plugin_registry_starts_empty() {
    let registry = PluginRegistry::new(PathBuf::from("/tmp/test_plugins_empty"));
    assert_eq!(registry.list().len(), 0);
}

#[test]
fn test_plugin_registry_get_nonexistent_returns_none() {
    let registry = PluginRegistry::new(PathBuf::from("/tmp/test_plugins_empty"));
    assert!(registry.get("com.example.nonexistent").is_none());
}

#[test]
fn test_installed_plugin_serialization() {
    let plugin = InstalledPlugin {
        manifest: sample_manifest(),
        enabled: true,
        install_path: "/tmp/plugins/test".to_string(),
        installed_at: chrono::Utc::now(),
        last_used: None,
    };
    let json = serde_json::to_string(&plugin).unwrap();
    let deserialized: InstalledPlugin = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.manifest.id, plugin.manifest.id);
    assert_eq!(deserialized.enabled, plugin.enabled);
    assert!(deserialized.last_used.is_none());
}

#[test]
fn test_plugin_manifest_has_required_fields() {
    let manifest = sample_manifest();
    assert!(!manifest.id.is_empty());
    assert!(!manifest.name.is_empty());
    assert!(!manifest.version.is_empty());
    assert!(!manifest.entry_point.is_empty());
    assert!(!manifest.min_keepassex_version.is_empty());
}

#[test]
fn test_plugin_wasm_magic_bytes() {
    // Valid WASM starts with \0asm
    let valid_wasm = b"\x00asm\x01\x00\x00\x00";
    assert_eq!(&valid_wasm[0..4], b"\x00asm");

    // Invalid data
    let invalid = b"not wasm data";
    assert_ne!(&invalid[0..4], b"\x00asm");
}

#[test]
fn test_dashlane_importer_manifest() {
    let manifest = dashlane_importer_manifest();
    assert_eq!(manifest.id, "com.keepassex.importer.dashlane");
    assert!(manifest.capabilities.contains(&PluginCapability::Importer));
    assert!(manifest.permissions.contains(&PluginPermission::FileSystem));
    assert!(!manifest.name.is_empty());
}

#[test]
fn test_enpass_importer_manifest() {
    let manifest = enpass_importer_manifest();
    assert_eq!(manifest.id, "com.keepassex.importer.enpass");
    assert!(manifest.capabilities.contains(&PluginCapability::Importer));
    assert!(manifest.permissions.contains(&PluginPermission::FileSystem));
}

#[test]
fn test_plugin_manifest_capabilities_not_empty() {
    let manifest = sample_manifest();
    assert!(!manifest.capabilities.is_empty());
    assert!(manifest
        .capabilities
        .contains(&PluginCapability::PasswordGenerator));
}

#[test]
fn test_plugin_registry_enable_disable_nonexistent_fails() {
    let mut registry = PluginRegistry::new(PathBuf::from("/tmp/test_plugins_empty"));
    assert!(registry.enable("nonexistent").is_err());
    assert!(registry.disable("nonexistent").is_err());
}
