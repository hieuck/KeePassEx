# KeePassEx — Plugin Development Guide

## Overview

KeePassEx plugins are **WebAssembly (WASM) modules** that run in a sandboxed environment. They can extend KeePassEx with:

- Custom password generators
- Custom import/export formats
- Custom health check rules
- Custom field validators
- Custom icon sets
- UI extensions (menu items)

---

## Plugin Manifest

Every plugin needs a `plugin.json` manifest:

```json
{
  "id": "com.example.my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "A custom KeePassEx plugin",
  "author": "Your Name",
  "license": "MIT",
  "capabilities": ["password_generator"],
  "permissions": ["file_system"],
  "entry_point": "my_plugin.wasm",
  "min_keepassex_version": "1.0.0"
}
```

### Capabilities

| Capability | Description |
|-----------|-------------|
| `password_generator` | Add custom password generation algorithms |
| `importer` | Add custom import formats |
| `exporter` | Add custom export formats |
| `health_check` | Add custom vault health rules |
| `field_validator` | Validate custom entry fields |
| `icon_set` | Add custom entry icons |
| `ui_extension` | Add menu items and UI elements |

### Permissions

| Permission | Access |
|-----------|--------|
| `read_entry_metadata` | Read entry titles, URLs (no passwords) |
| `read_passwords` | Read entry passwords (requires user approval) |
| `write_entries` | Create/modify entries |
| `network` | Make HTTP requests |
| `file_system` | Read/write files |

---

## Plugin API

Plugins communicate with KeePassEx via WASM imports/exports.

### Host Functions (available to plugins)

```rust
// Log a message
extern "C" fn kpx_log(level: u32, msg_ptr: *const u8, msg_len: u32);

// Generate a random password using core generator
extern "C" fn kpx_generate_password(
    length: u32,
    out_ptr: *mut u8,
    out_len: *mut u32,
) -> i32;

// Get vault name
extern "C" fn kpx_get_vault_name(out_ptr: *mut u8, out_len: *mut u32) -> i32;

// Show notification
extern "C" fn kpx_notify(
    title_ptr: *const u8, title_len: u32,
    msg_ptr: *const u8, msg_len: u32,
);
```

### Plugin Exports (required)

```rust
// Called when plugin is loaded
#[no_mangle]
pub extern "C" fn kpx_plugin_init() -> i32 { 0 }

// Called when plugin is unloaded
#[no_mangle]
pub extern "C" fn kpx_plugin_destroy() {}

// For password_generator capability:
#[no_mangle]
pub extern "C" fn kpx_generate(
    config_ptr: *const u8,
    config_len: u32,
    out_ptr: *mut u8,
    out_len: *mut u32,
) -> i32 { 0 }

// For importer capability:
#[no_mangle]
pub extern "C" fn kpx_import(
    data_ptr: *const u8,
    data_len: u32,
    out_ptr: *mut u8,
    out_len: *mut u32,
) -> i32 { 0 }
```

---

## Example: Diceware Generator Plugin

```rust
// src/lib.rs
use std::slice;

// EFF Large Wordlist (abbreviated)
const WORDS: &[&str] = &[
    "aardvark", "abacus", "abandon", "abbey", "abbot",
    // ... full 7776-word list
];

#[no_mangle]
pub extern "C" fn kpx_plugin_init() -> i32 {
    0 // Success
}

#[no_mangle]
pub extern "C" fn kpx_generate(
    config_ptr: *const u8,
    config_len: u32,
    out_ptr: *mut u8,
    out_len: *mut u32,
) -> i32 {
    // Parse config JSON
    let config_bytes = unsafe { slice::from_raw_parts(config_ptr, config_len as usize) };
    let config: serde_json::Value = serde_json::from_slice(config_bytes).unwrap_or_default();

    let word_count = config["word_count"].as_u64().unwrap_or(6) as usize;
    let separator = config["separator"].as_str().unwrap_or("-");

    // Generate passphrase
    let mut words = Vec::new();
    for _ in 0..word_count {
        // Use WASM-compatible random (no OS entropy needed)
        let idx = pseudo_random() % WORDS.len();
        words.push(WORDS[idx]);
    }

    let passphrase = words.join(separator);
    let bytes = passphrase.as_bytes();

    // Write output
    unsafe {
        let out = slice::from_raw_parts_mut(out_ptr, bytes.len());
        out.copy_from_slice(bytes);
        *out_len = bytes.len() as u32;
    }

    0 // Success
}

fn pseudo_random() -> usize {
    // Simple LCG for WASM (no OS entropy)
    static mut SEED: u64 = 12345;
    unsafe {
        SEED = SEED.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (SEED >> 33) as usize
    }
}
```

### Build

```toml
# Cargo.toml
[package]
name = "kpx-diceware"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde_json = "1.0"
```

```bash
# Build for WASM
cargo build --target wasm32-unknown-unknown --release

# Output: target/wasm32-unknown-unknown/release/kpx_diceware.wasm
```

### Package

```
my-plugin/
├── plugin.json
└── kpx_diceware.wasm
```

Zip and rename to `kpx-diceware-1.0.0.kpxplugin`.

---

## Example: Dashlane Importer Plugin

```rust
#[no_mangle]
pub extern "C" fn kpx_import(
    data_ptr: *const u8,
    data_len: u32,
    out_ptr: *mut u8,
    out_len: *mut u32,
) -> i32 {
    let data = unsafe { slice::from_raw_parts(data_ptr, data_len as usize) };
    let csv = std::str::from_utf8(data).unwrap_or("");

    // Parse Dashlane CSV format
    let mut entries = Vec::new();
    for (i, line) in csv.lines().enumerate() {
        if i == 0 { continue; } // Skip header

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 4 { continue; }

        entries.push(serde_json::json!({
            "title": fields[0],
            "url": fields[1],
            "username": fields[2],
            "password": fields[3],
        }));
    }

    let output = serde_json::json!({
        "entries": entries,
        "groups": [],
    });

    let bytes = output.to_string().into_bytes();
    unsafe {
        let out = slice::from_raw_parts_mut(out_ptr, bytes.len());
        out.copy_from_slice(&bytes);
        *out_len = bytes.len() as u32;
    }

    0
}
```

---

## Security Considerations

1. **Plugins run in a WASM sandbox** — no direct memory access to vault data
2. **Permissions are enforced** — plugins can only access what they declare
3. **No network by default** — must declare `network` permission
4. **User approval required** for `read_passwords` permission
5. **Plugins are signed** — future versions will require code signing

---

## Publishing Plugins

1. Build and test your plugin
2. Create a `plugin.json` manifest
3. Package as `.kpxplugin` (zip file)
4. Submit to the KeePassEx plugin registry (coming in v1.2)

---

## Plugin Registry (Coming in v1.2)

The plugin registry will be available at https://plugins.keepassex.app and will include:
- Browse and search plugins
- Automatic updates
- Security scanning
- Community ratings
