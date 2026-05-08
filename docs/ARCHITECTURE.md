# KeePassEx Architecture

## Overview

KeePassEx is a monorepo with a **Rust core** shared across all platforms via FFI/JNI/native modules.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              User Interfaces                                    │
│  Desktop  │  Mobile  │  watchOS  │  WearOS  │  CLI  │  TUI  │  Browser Ext.   │
│  (Tauri)  │   (RN)   │ (SwiftUI) │ (Compose)│(Rust) │(Ratatui)│ (Chrome/FF)  │
│           │          │           │          │       │       │                  │
│  macOS    │          │           │          │       │       │                  │
│  Menu Bar │          │           │          │       │       │                  │
│ (SwiftUI) │          │           │          │       │       │                  │
│           │          │           │          │       │       │                  │
│  Windows  │          │           │          │       │       │                  │
│ CredProv  │          │           │          │       │       │                  │
│  (Rust)   │          │           │          │       │       │                  │
└─────┬─────┴────┬─────┴─────┬─────┴────┬─────┴───┬───┴───┬───┴────────┬────────┘
      │          │           │          │          │       │            │
      └──────────┴───────────┴──────────┴──────────┴───────┴────────────┘
                                        │
                           ┌────────────▼────────────┐
                           │     keepassex-core       │
                           │     (Rust library)       │
                           │                          │
                           │  • KDBX 4.x engine       │
                           │  • Argon2id KDF           │
                           │  • ChaCha20 cipher        │
                           │  • TOTP/HOTP              │
                           │  • Passkey (FIDO2)        │
                           │  • SSH Agent              │
                           │  • Password gen           │
                           │  • Health audit           │
                           │  • Import/Export (12)     │
                           │  • Breach monitor         │
                           │  • Sync engine (8 prov.)  │
                           │  • Emergency access       │
                           │  • Plugin system (WASM)   │
                           │  • Steganography          │
                           │  • Shamir sharding        │
                           │  • Post-quantum crypto    │
                           │  • Team vault (RBAC)      │
                           │  • Analytics engine       │
                           │  • Entry cache (LRU)      │
                           └──────────────────────────┘
                                        │
                           ┌────────────▼────────────┐
                           │   KeePassEx Server       │
                           │   (Rust + Axum + SQLite) │
                           │   Zero-knowledge sync    │
                           │   WebSocket real-time    │
                           └──────────────────────────┘
```

## Package Structure

### `packages/core` — Rust Core Engine

All security-critical code lives here. Shared by ALL platforms.

```
src/
├── cache/                   # LRU entry/group cache (performance)
│   └── mod.rs               # EntryCache, GroupCache, VaultCache
├── crypto/                  # Cryptographic primitives
│   ├── cipher.rs            # ChaCha20-Poly1305, AES-256-GCM, AES-256-CBC, Twofish
│   ├── hmac.rs              # HMAC-SHA256 block authentication
│   ├── kdf.rs               # Argon2id, AES-KDF
│   ├── keys.rs              # Composite key, master key derivation
│   ├── pqc.rs               # X25519 + Kyber-768 hybrid (post-quantum)
│   ├── protected_stream.rs  # In-memory field encryption (ChaCha20/Salsa20)
│   └── shamir.rs            # Shamir's Secret Sharing (GF(256))
├── kdbx/                    # KDBX file format
│   ├── header.rs            # Header structures and constants
│   ├── mod.rs               # Module exports
│   ├── pqc_header.rs        # KDBX extension for PQC (field 0x80)
│   ├── reader.rs            # KDBX 4.x reader + KDBX 3.1 compat
│   ├── writer.rs            # KDBX 4.x writer
│   └── xml.rs               # XML payload parser/serializer
├── vault/                   # Vault operations
│   ├── mod.rs               # Vault struct, CRUD, search
│   ├── operations.rs        # open/save/lock/change_credentials
│   ├── pqc_migration.rs     # Migrate vault to/from PQC encryption
│   └── search.rs            # Full-text search
├── import_export/           # Import from 12 password managers
│   ├── bitwarden.rs         # Bitwarden JSON
│   ├── chrome.rs            # Chrome/Firefox CSV
│   ├── csv.rs               # Generic CSV + export
│   ├── dashlane.rs          # Dashlane JSON
│   ├── enpass.rs            # Enpass JSON
│   ├── keepass1.rs          # KeePass 1.x XML
│   ├── lastpass.rs          # LastPass CSV
│   ├── nordpass.rs          # NordPass CSV
│   ├── onepassword.rs       # 1Password 1PUX
│   └── roboform.rs          # RoboForm HTML
├── search/                  # Search engine
│   ├── mod.rs               # Search entry point
│   ├── nl_parser.rs         # Natural language query parser (EN/VI)
│   └── query_builder.rs     # Structured query builder
├── steg/                    # Steganography
│   ├── mod.rs               # Core embed/extract logic
│   ├── png.rs               # PNG LSB embedding (~777KB for 1080p)
│   ├── jpeg.rs              # JPEG EXIF/APP1 embedding (max 64KB)
│   └── video.rs             # MP4 'kpxv' atom / AVI 'KPX ' chunk
├── sync/                    # Multi-provider sync engine
│   ├── mod.rs               # SyncProvider trait, types
│   ├── merge.rs             # CRDT-inspired vault merge (last-write-wins)
│   └── providers.rs         # WebDAV, Local, Google Drive, OneDrive,
│                            # Dropbox, S3 (Sig V4), SFTP, iCloud
├── plugin/                  # WASM plugin sandbox
│   └── mod.rs               # Plugin manifest, registry, wasmtime host
├── tests/                   # Integration tests (626 tests, 29 modules)
│   └── ...
├── analytics.rs             # Vault analytics engine
├── audit_log.rs             # Audit log (24 event types, ring buffer)
├── breach.rs                # HIBP k-anonymity breach check
├── categorizer.rs           # Smart entry categorization (16 categories)
├── decoy_vault.rs           # Decoy vault generation
├── emergency_access.rs      # Trusted contact vault sharing (X25519 ECDH)
├── error.rs                 # KeePassExError (30+ variants)
├── expiry_engine.rs         # Password rotation urgency analysis
├── favicon.rs               # Privacy-safe favicon fetching
├── field_references.rs      # {REF:F@I:uuid} KeePass-compatible references
├── generator.rs             # Password/passphrase generator
├── hardware_key.rs          # YubiKey HMAC/OTP, FIDO2, Smart Card, OnlyKey
├── health.rs                # Vault health audit
├── lib.rs                   # Crate root (31 public modules)
├── notifications.rs         # Notification generator
├── otp.rs                   # TOTP/HOTP (RFC 6238/4226)
├── passkey.rs               # FIDO2/WebAuthn passkey storage + verification
├── password_advisor.rs      # Context-aware password strength advisor
├── password_policy.rs       # Password policy engine (14 rule types)
├── scheduled_backup.rs      # Scheduled backup with retention policy
├── ssh.rs                   # SSH key management + agent protocol
├── team.rs                  # Team vault (RBAC, comments, per-entry perms)
├── templates.rs             # Entry templates (12 built-in types)
├── types.rs                 # Domain types (Entry, Group, VaultMeta, etc.)
├── vault_compare.rs         # Vault diff + merge
└── zkpv.rs                  # Zero-knowledge password verification
```

**Test coverage: 650+ tests across 32 test modules**

### `packages/ui` — Shared Design System

React Native + Tamagui components used by both desktop and mobile.

**15 components:**

- `Button`, `Input`, `PasswordInput`, `PasswordField` — form elements
- `EntryListItem`, `GroupListItem`, `EntryHistoryViewer` — vault list items
- `StrengthMeter`, `HealthBadge` — security indicators
- `OtpDisplay`, `OtpSetupModal` — OTP management
- `SearchBar`, `IconPicker`, `TagList` — utility components
- `VaultLockScreen` — lock screen

**Themes:** Light, Dark, OLED (all WCAG 2.1 AA compliant)

### `packages/i18n` — Internationalization

**10 locale files, ~900 keys each, 100% parity:**

| Code | Language   | Status                 |
| ---- | ---------- | ---------------------- |
| `en` | English    | ✅ Primary (1082 keys) |
| `vi` | Tiếng Việt | ✅ Full parity         |
| `zh` | 简体中文   | ✅ Full parity         |
| `ja` | 日本語     | ✅ Full parity         |
| `ko` | 한국어     | ✅ Full parity         |
| `es` | Español    | ✅ Full parity         |
| `fr` | Français   | ✅ Full parity         |
| `de` | Deutsch    | ✅ Full parity         |
| `pt` | Português  | ✅ Full parity         |
| `ru` | Русский    | ✅ Full parity         |

Key format: `section.subsection.key` (e.g. `entry.copyPassword`, `vault.open`)

### `apps/desktop` — Tauri v2

**Frontend (React + TypeScript):**

- 22 pages: Vault, Entry Detail, Health, Breach, Generator, Import/Export, Sync, Emergency Access, Plugins, Settings, Security, Welcome, Unlock, Analytics, AuditLog, Backup, VaultCompare, PasswordPolicy, Steganography, Statistics, Team, HardwareKey
- 13 components: EntryRow, GroupTree, SearchBar, CommandPalette, IdleLockManager, OtpSetupDialog, CustomFieldEditor, AttachmentViewer, TagInput, BulkActionBar, PasswordStrengthBar, NaturalLanguageSearch, VaultTabBar
- 5 Zustand stores: vault, settings (with sync config), sync, breach, tabs

**Backend (Rust/Tauri):**

- 29 command modules covering all features
- SSH Agent server (Unix socket / Windows named pipe)
- Native messaging host (browser extension)
- System tray, global shortcuts (Ctrl+Alt+K), auto-type
- Tauri plugins: shell, dialog, fs, clipboard, global-shortcut, notification, autostart, updater

### `apps/mobile` — React Native

**iOS:**

- Swift native module (`KeePassExCore.swift`) — FFI bridge to Rust core
- AutoFill Extension (`CredentialProviderViewController.swift`)
- WidgetKit widget (small, medium, large, lock screen)
- Secure Enclave key storage (`SecureEnclaveKeystore.ts`)

**Android:**

- Kotlin native module (`KeePassExCoreModule.kt`)
- AutoFill Service (`KeePassExAutofillService.kt`)
- StrongBox Keymaster integration
- Quick Settings tile + home screen widget

**16 screens:** Vault, Entry Detail/Edit, Generator, Health, Breach, OTP, Sync, Import/Export, Settings, Plugins, Emergency Access, Unlock, Welcome, Analytics, Sharding, BreachDetail

### `apps/watch` — Native Watch Apps

**watchOS (SwiftUI):**

- Full vault browsing with Digital Crown navigation
- OTP countdown complications
- WatchConnectivity for phone sync
- Haptic feedback

**WearOS (Jetpack Compose):**

- Rotary input support
- Quick Settings tile
- Watch face complications
- Wearable Data Layer sync

### `apps/browser-extension` — Browser Extension

- **Chrome/Edge** (Manifest V3): service worker background
- **Firefox** (Manifest V2): persistent background
- Native messaging via `com.keepassex.app`
- Form detection, credential filling, OTP inline display
- Context menu, keyboard shortcut (Ctrl+Shift+F)
- Dark mode, i18n (EN/VI based on navigator.language)

### `apps/cli` — Rust CLI (`kpx`)

18 subcommands: list, get, add, edit, delete, generate, health, otp, export, import, sync, breach, stats, template (list/show), hardware-key (list/test/setup), compare, audit, server (status/login/register/history), steg (embed/extract), shard (split/combine)

### `apps/tui` — Rust TUI

Full-featured terminal UI using Ratatui with vim-style keybindings. Themes: dark, light, solarized, nord, gruvbox. Mouse support optional.

### `apps/server` — KeePassEx Server

Axum + SQLite + JWT. Zero-knowledge design (server never sees plaintext).

**Routes:**

- `POST /api/v1/auth/register|login|refresh|logout`
- `GET|PUT /api/v1/vault` — vault metadata + upload
- `GET /api/v1/vault/download` — download vault
- `GET /api/v1/vault/history[/:version]` — version history
- `GET /ws` — WebSocket real-time sync
- `GET /health`, `GET /api/v1/server/info`
- Admin: `GET /api/v1/admin/users`, `DELETE /api/v1/admin/users/:id`, `GET /api/v1/admin/stats`

**Deployment:** Single binary, Docker image, Kubernetes Helm chart

### `apps/macos-menubar` — macOS Menu Bar (SwiftUI)

320px popover with search, entry list, OTP countdown badges, hover actions (copy password/username/OTP). Status indicator (green/orange/red). Keyboard shortcut: ⌘⇧K.

### `apps/windows-credprov` — Windows Credential Provider

Rust `cdylib` implementing `ICredentialProvider` COM interface. Allows unlocking Windows login screen with KeePassEx master password. Uses ZKPV commitment for fast pre-check without full vault decryption.

---

## Data Flow

### Opening a Vault

```
User enters password
        │
        ▼
CompositeKey::build()
  SHA256(password) + SHA256(keyfile) + hardware_key_response
        │
        ▼
Argon2id KDF (64 MB memory, 2 iterations, 1 parallelism)
  → transformed_key (32 bytes)
        │
        ▼
MasterKey::derive_keys(master_seed)
  → enc_key (32 bytes)
  → hmac_key (32 bytes)
        │
        ▼
Verify header HMAC (tamper detection)
        │
        ▼
Read HMAC blocks (verify each 1 MB block)
        │
        ▼
ChaCha20-Poly1305 decrypt
        │
        ▼
GZip decompress
        │
        ▼
Parse inner header (stream key, binaries)
        │
        ▼
Parse XML → Vault { entries, groups, meta }
        │
        ▼
Populate EntryCache (LRU, 500 entries, no passwords)
```

### Sync Flow

```
User triggers sync
        │
        ▼
Read vault + SyncConfig from AppState
        │
        ▼
Serialize vault → KDBX bytes (KdbxWriter)
        │
        ▼
Build SyncProvider (WebDAV/GDrive/OneDrive/Local/...)
        │
        ▼
Check remote metadata (etag/size)
        │
        ├── Remote unchanged → no-op
        │
        ├── Remote newer (KeepRemote) → download + overwrite local
        │
        ├── Local newer (KeepLocal) → upload + overwrite remote
        │
        └── Both changed (Merge) →
              Download remote → parse → merge_vaults() →
              Save merged locally → upload merged to remote →
              Update in-memory vault
```

### AutoFill Flow (Browser)

```
User visits login page
        │
        ▼
content.ts detects password field
        │
        ▼
background.ts (service worker)
        │
        ▼
Native messaging → Desktop app (JSON over stdio)
        │
        ▼
native_messaging.rs: search by URL domain
        │
        ▼
Return matching entries (no passwords in response)
        │
        ▼
User selects entry in popup
        │
        ▼
Native messaging: get credentials for selected entry
        │
        ▼
content.ts fills form fields (setNativeValue)
```

### Emergency Access Flow

```
Owner invites grantee
        │
        ▼
Grantee generates X25519 key pair
        │
        ▼
Grantee's public key stored in vault
        │
        ▼
Emergency: grantee requests access
        │
        ▼
Owner can deny during wait period (default 7 days)
        │
        ▼
After wait period: vault key encrypted with grantee's public key
  encrypt_for_grantee(vault_key, grantee_public_key)
  = X25519 ECDH + HKDF + ChaCha20-Poly1305
        │
        ▼
Grantee decrypts with their private key
```

### Breach Monitor Flow (k-anonymity)

```
Password: "correct-horse-battery-staple"
        │
        ▼
SHA-1 hash: "4DC3B5B5B5B5B5B5B5B5B5B5B5B5B5B5B5B5B5B5"
        │
        ▼
Send ONLY first 5 chars: "4DC3B" → HIBP API
        │
        ▼
HIBP returns all hashes starting with "4DC3B"
        │
        ▼
Match locally — password NEVER leaves device
```

### Post-Quantum Encryption Flow

```
Vault open with PQC enabled
        │
        ▼
derive_pqc_keypair(master_key, HybridKyber768)
  → (public_key, private_key)
        │
        ▼
encapsulate(public_key) → (shared_secret, encapsulation)
        │
        ▼
HKDF(X25519_shared || Kyber_shared) → combined_key
        │
        ▼
ChaCha20-Poly1305 encrypt with combined_key
        │
        ▼
Store encapsulation in KDBX header (field 0x80)
```

---

## Security Boundaries

```
┌─────────────────────────────────────────────────────┐
│  TRUSTED (in-process, Rust)                         │
│  • keepassex-core (all crypto)                      │
│  • Tauri backend commands                           │
│  • iOS Swift native module                          │
│  • Android Kotlin native module                     │
│  • Windows Credential Provider DLL                  │
└─────────────────────────────────────────────────────┘
         │ IPC (serialized JSON, no raw memory sharing)
┌─────────────────────────────────────────────────────┐
│  SEMI-TRUSTED (sandboxed)                           │
│  • Tauri frontend (WebView, CSP enforced)           │
│  • React Native JS thread                           │
│  • Browser extension popup                         │
│  • macOS Menu Bar (XPC to desktop app)             │
└─────────────────────────────────────────────────────┘
         │ Native messaging (JSON over stdio, localhost only)
┌─────────────────────────────────────────────────────┐
│  UNTRUSTED                                          │
│  • Browser content scripts                         │
│  • Web pages                                       │
│  • External sync providers                         │
│  • WASM plugins (sandboxed, limited API)           │
│  • KeePassEx Server (zero-knowledge — cannot decrypt)│
└─────────────────────────────────────────────────────┘
```

**Key principles:**

- Passwords are only decrypted in the trusted layer
- The browser extension only receives credentials for the specific entry the user explicitly selected
- The sync server never receives plaintext vault data — all encryption is client-side
- WASM plugins have no direct memory access to vault data; declared permissions enforced at runtime
- Sensitive strings use `ProtectedString` (zeroized on drop, Display shows `***`)

---

## Plugin Architecture

```
KeePassEx Core
      │
      ▼
Plugin Registry (loads plugin.json manifests)
      │
      ▼
Plugin Host (wasmtime sandbox)
      │
      ├── Host functions (kpx_log, kpx_generate_password, kpx_notify)
      │
      └── Plugin exports (kpx_plugin_init, kpx_generate, kpx_import)
```

Plugins are WASM modules with:

- No direct memory access to vault data
- Declared permissions (enforced at runtime)
- User approval required for sensitive permissions
- Resource limits (memory, CPU time)
- SHA-256 hash verification before loading

---

## Sync Architecture

```
Local Vault (KDBX)
      │
      ▼
SyncProvider trait
  ├── WebDavProvider (HTTP PUT/GET/PROPFIND)
  ├── LocalFolderProvider (file copy)
  ├── GoogleDriveProvider (REST API v3, OAuth2)
  ├── OneDriveProvider (Microsoft Graph API, OAuth2)
  ├── DropboxProvider (Dropbox API v2, OAuth2)
  ├── S3Provider (AWS Signature V4, S3-compatible)
  ├── SftpProvider (russh-sftp)
  ├── ICloudDriveProvider (platform bridge)
  └── KeePassExServerProvider (REST + WebSocket, JWT)
      │
      ▼
Conflict Resolution (CRDT-inspired)
  └── merge_vaults(): last-write-wins per entry UUID
```

---

## Cache Architecture

```
VaultCache (per open vault)
  ├── EntryCache (LRU, 500 entries)
  │   └── CachedEntry (metadata only — NO passwords)
  ├── GroupCache (HashMap)
  │   └── CachedGroup (tree structure)
  └── SearchCache (HashMap, 50 queries)
      └── query → Vec<Uuid>

Invalidation:
  • Entry create/update/delete → invalidate_entry(uuid)
  • Vault save/reload → invalidate_all()
  • Search cache cleared on any mutation
```

---

## Test Architecture

```
packages/core/src/tests/  (Rust, 650+ tests across 32 modules)
├── vault_tests.rs         # CRUD, search, recycle bin, history
├── crypto_tests.rs        # KDF, cipher, HMAC, key derivation
├── otp_tests.rs           # RFC 4226/6238 vectors, URI parsing
├── generator_tests.rs     # Password generation, strength scoring
├── health_tests.rs        # Audit logic, weak/reused/expired
├── import_tests.rs        # Format importers (Bitwarden, Chrome, CSV)
├── new_import_tests.rs    # NordPass, Enpass, Dashlane, RoboForm, KeePass1
├── breach_tests.rs        # k-anonymity, SHA-1, offline check
├── passkey_tests.rs       # FIDO2 credentials
├── emergency_access_tests.rs  # Lifecycle, revoke, manager
├── ssh_tests.rs           # Agent, key parsing, deduplication
├── sync_tests.rs          # Merge, diff, conflict resolution
├── kdbx_tests.rs          # Format, XML, round-trip, KDBX 3.1
├── policy_tests.rs        # Password policy engine (25 tests)
├── backup_tests.rs        # Scheduled backup (15 tests)
├── compare_tests.rs       # Vault comparison (12 tests)
├── audit_tests.rs         # Audit log (15 tests)
├── analytics_tests.rs     # compute_analytics(), strength distribution (11 tests)
├── categorizer_tests.rs   # categorize_entry(), domain matching (17 tests)
├── decoy_vault_tests.rs   # generate_decoy_vault(), fake entry quality (8 tests)
├── expiry_engine_tests.rs # analyze_rotation(), urgency levels (15 tests)
├── notifications_tests.rs # NotificationGenerator, all types (12 tests)
├── scheduled_backup_tests.rs # is_backup_due(), all frequencies (11 tests)
├── zkpv_tests.rs          # ZkpvCommitment, PasswordHint (10 tests)
├── steg_tests.rs          # StegFormat::detect(), embed/extract (12 tests)
├── team_tests.rs          # TeamVault, RBAC, entry overrides (16 tests)
├── templates_tests.rs     # TemplateManager, 12 built-in templates (14 tests)
├── plugin_tests.rs        # PluginManifest, PluginRegistry (13 tests)
├── search_tests.rs        # vault.search(), SearchQuery (10 tests)
├── ai_tests.rs            # AI password suggestions, strategies, bilingual (24 tests)
├── password_advisor_tests.rs  # Context-aware advisor (12 tests)
└── mod.rs

packages/core/src/cache/mod.rs  (Rust, 6 unit tests)
  # EntryCache LRU, GroupCache, VaultCache, search cache

packages/i18n/src/__tests__/  (TypeScript)
└── i18n.test.ts           # EN/VI parity, interpolation variable consistency

packages/ui/src/__tests__/  (TypeScript)
├── StrengthMeter.test.tsx
└── components.test.tsx

shared/utils/src/__tests__/  (TypeScript)
└── utils.test.ts

apps/desktop/src/__tests__/  (TypeScript)
├── store.test.ts
├── components.test.tsx
└── pages.test.tsx

apps/mobile/src/__tests__/  (TypeScript)
└── screens.test.tsx

apps/browser-extension/src/__tests__/  (TypeScript)
└── background.test.ts
```
