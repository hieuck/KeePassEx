# KeePassEx Architecture

## Overview

KeePassEx is a monorepo with a **Rust core** shared across all platforms via FFI/JNI/native modules.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           User Interfaces                               │
│  Desktop  │  Mobile  │  watchOS  │  WearOS  │  CLI  │  Browser Ext.   │
│  (Tauri)  │   (RN)   │ (SwiftUI) │ (Compose)│(Rust) │  (Chrome/FF)    │
└─────┬─────┴────┬─────┴─────┬─────┴────┬─────┴───┬───┴────────┬────────┘
      │          │           │          │          │            │
      └──────────┴───────────┴──────────┴──────────┴────────────┘
                                    │
                         ┌──────────▼──────────┐
                         │   keepassex-core     │
                         │   (Rust library)     │
                         │                      │
                         │  • KDBX 4.x engine   │
                         │  • Argon2id KDF       │
                         │  • ChaCha20 cipher    │
                         │  • TOTP/HOTP          │
                         │  • Passkey (FIDO2)    │
                         │  • SSH Agent          │
                         │  • Password gen       │
                         │  • Health audit       │
                         │  • Import/Export      │
                         │  • Breach monitor     │
                         │  • Sync engine        │
                         │  • Emergency access   │
                         │  • Plugin system      │
                         └──────────────────────┘
```

## Package Structure

### `packages/core` — Rust Core Engine

All security-critical code lives here. Shared by ALL platforms.

```
src/
├── crypto/                  # Cryptographic primitives
│   ├── kdf.rs               # Argon2id, AES-KDF
│   ├── cipher.rs            # ChaCha20-Poly1305, AES-256-GCM
│   ├── hmac.rs              # HMAC-SHA256 block authentication
│   ├── keys.rs              # Composite key, master key derivation
│   └── protected_stream.rs  # In-memory field encryption
├── kdbx/                    # KDBX file format
│   ├── header.rs            # Header structures
│   ├── reader.rs            # KDBX 4.x reader (+ 3.1 compat)
│   ├── writer.rs            # KDBX 4.x writer
│   └── xml.rs               # XML payload parser/serializer (full impl)
├── vault/                   # Vault operations
│   ├── mod.rs               # Vault struct, CRUD, search
│   ├── operations.rs        # open/save/lock/change credentials
│   └── search.rs            # Full-text search
├── generator.rs             # Password/passphrase generator
├── health.rs                # Vault health audit
├── otp.rs                   # TOTP/HOTP (RFC 6238/4226)
├── passkey.rs               # FIDO2/WebAuthn passkey storage + verification
├── ssh.rs                   # SSH key management + agent protocol
├── breach.rs                # HIBP k-anonymity breach check
├── emergency_access.rs      # Trusted contact vault sharing (X25519 ECDH)
├── plugin/                  # WASM plugin system
│   └── mod.rs               # Plugin manifest, registry, host
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
├── sync/                    # Multi-provider sync engine
│   ├── merge.rs             # CRDT-inspired vault merge
│   └── providers.rs         # WebDAV, local folder
└── types.rs                 # Domain types (Entry, Group, VaultMeta, etc.)
```

**Test coverage: 626 tests across 29 test modules**

### `packages/ui` — Shared Design System

React Native + Tamagui components used by both desktop and mobile.

**Components:**

- `Button`, `Input`, `PasswordInput` — form elements
- `EntryListItem`, `GroupListItem` — vault list items
- `StrengthMeter`, `HealthBadge` — security indicators
- `OtpDisplay`, `OtpSetupModal` — OTP management
- `SearchBar`, `IconPicker`, `TagList` — utility components
- `VaultLockScreen` — lock screen

**Themes:** Light, Dark, OLED (all WCAG 2.1 AA compliant)

### `packages/i18n` — Internationalization

- **English** (en) — 900+ keys, default
- **Vietnamese** (vi) — full parity, verified by automated tests
- Extensible: add new locale by creating `locales/{code}.ts`

### `apps/desktop` — Tauri v2

**Frontend (React + TypeScript):**

- 20 pages: Vault, Entry Detail, Health, Breach, Generator, Import/Export, Sync, Emergency Access, Plugins, Settings, Welcome, Unlock, Analytics, AuditLog, Backup, VaultCompare, PasswordPolicy, Steganography, Statistics, Team, HardwareKey
- 12 components: EntryRow, GroupTree, SearchBar, CommandPalette, IdleLockManager, OtpSetupDialog, CustomFieldEditor, AttachmentViewer, TagInput, BulkActionBar, PasswordStrengthBar, NaturalLanguageSearch
- 4 stores: vault, settings, breach, sync

**Backend (Rust/Tauri):**

- 17 command modules: vault, entries, groups, generator, otp, health, clipboard, ssh, settings, import_export, breach, sync_cmd, attachments, hardware_key, backup, audit_log_cmd, policy_cmd, vault_compare_cmd, analytics_cmd, steg_cmd, team_cmd, search_cmd
- SSH Agent server (Unix socket)
- Native messaging host (browser extension)
- System tray, global shortcuts, auto-type

### `apps/mobile` — React Native

**iOS:**

- Swift native module (`KeePassExCore.swift`)
- AutoFill Extension (`CredentialProviderViewController.swift`)
- WidgetKit widget (small, medium, large, lock screen)
- App Delegate with screen capture protection

**Android:**

- Kotlin native module (`KeePassExCoreModule.kt`)
- AutoFill Service (`KeePassExAutofillService.kt`)
- Quick Settings tile
- Home screen widget

**Shared screens (16):** Vault, Entry Detail/Edit, Generator, Health, Breach, OTP, Sync, Import/Export, Settings, Plugins, Emergency Access, Unlock, Welcome, Analytics, Sharding, BreachDetail

**Shared components:** OtpCountdown, PasswordField, TagList, EmptyState, SectionHeader

**Stores:** vault, settings, theme, i18n

### `apps/watch` — Native Watch Apps

**watchOS (SwiftUI):**

- `ContentView.swift` — main UI
- `KeePassExWatchApp.swift` — app entry point
- `ComplicationController.swift` — watch face complications
- WatchConnectivity for phone communication

**WearOS (Jetpack Compose):**

- `MainActivity.kt` — main UI
- `KeePassExTileService.kt` — Quick Settings tile
- `KeePassExComplicationService.kt` — watch face complication
- `WearDataListenerService.kt` — phone communication

### `apps/browser-extension` — Browser Extension

- **Chrome/Edge** (Manifest V3): service worker background
- **Firefox** (Manifest V2): persistent background
- `background.ts` — native messaging, context menu, keyboard shortcuts
- `content.ts` — form detection, credential filling, fill picker UI
- `popup/Popup.tsx` — React popup with search, copy, OTP

### `apps/cli` — Rust CLI (`kpx`)

18 commands: list, get, add, edit, delete, generate, health, otp, export, import, sync, breach, stats, template, hardware-key, compare, audit, steg, shard

---

## Data Flow

### Opening a Vault

```
User enters password
        │
        ▼
CompositeKey::build()
  SHA256(password) + SHA256(keyfile) + hardware_key
        │
        ▼
Argon2id KDF (64 MB memory, 2 iterations)
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
Return matching entries (no passwords)
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

---

## Security Boundaries

```
┌─────────────────────────────────────────────────────┐
│  TRUSTED (in-process, Rust)                         │
│  • keepassex-core (all crypto)                      │
│  • Tauri backend commands                           │
│  • iOS Swift native module                          │
│  • Android Kotlin native module                     │
└─────────────────────────────────────────────────────┘
         │ IPC (serialized JSON, no raw memory sharing)
┌─────────────────────────────────────────────────────┐
│  SEMI-TRUSTED (sandboxed)                           │
│  • Tauri frontend (WebView, CSP enforced)           │
│  • React Native JS thread                           │
│  • Browser extension popup                         │
└─────────────────────────────────────────────────────┘
         │ Native messaging (JSON over stdio, localhost only)
┌─────────────────────────────────────────────────────┐
│  UNTRUSTED                                          │
│  • Browser content scripts                         │
│  • Web pages                                       │
│  • External sync providers                         │
│  • WASM plugins (sandboxed, limited API)           │
└─────────────────────────────────────────────────────┘
```

**Key principle**: Passwords are only decrypted in the trusted layer. The browser extension only receives credentials for the specific entry the user explicitly selected.

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

---

## Sync Architecture

```
Local Vault (KDBX)
      │
      ▼
SyncProvider trait
  ├── WebDavProvider (HTTP PUT/GET/PROPFIND)
  ├── LocalFolderProvider (file copy)
  └── [Future: iCloud, GDrive, OneDrive, Dropbox, S3]
      │
      ▼
Conflict Resolution (CRDT-inspired)
  └── merge_vaults(): last-write-wins per entry UUID
```

---

## Test Architecture

```
packages/core/src/tests/  (Rust, 626 tests across 29 modules)
├── vault_tests.rs         # CRUD, search, recycle bin, history
├── crypto_tests.rs        # KDF, cipher, HMAC, key derivation
├── otp_tests.rs           # RFC 4226/6238 vectors, URI parsing
├── generator_tests.rs     # Password generation, strength scoring
├── health_tests.rs        # Audit logic, weak/reused/expired
├── import_tests.rs        # Format importers (Bitwarden, Chrome, CSV)
├── new_import_tests.rs    # New importers (NordPass, Enpass, Dashlane, RoboForm, KeePass1)
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
└── mod.rs

packages/i18n/src/__tests__/  (TypeScript)
└── i18n.test.ts           # EN/VI parity, interpolation variable consistency, no empty values

packages/ui/src/__tests__/  (TypeScript)
├── StrengthMeter.test.tsx
└── components.test.tsx    # Button, Input, HealthBadge, OtpDisplay

shared/utils/src/__tests__/  (TypeScript)
└── utils.test.ts          # Utility functions (formatRelativeTime, truncate, etc.)

apps/desktop/src/__tests__/  (TypeScript)
├── store.test.ts          # Vault + settings stores
├── components.test.tsx    # CustomFieldEditor, SearchBar, EntryRow
└── pages.test.tsx         # WelcomePage (dialog-based), UnlockPage, GeneratorPage, etc.

apps/mobile/src/__tests__/  (TypeScript)
└── screens.test.tsx       # VaultScreen, HealthScreen, GeneratorScreen, etc.

apps/browser-extension/src/__tests__/  (TypeScript)
└── background.test.ts     # URL extraction, form detection, filling, HTML escaping
```
