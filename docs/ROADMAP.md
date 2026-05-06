# KeePassEx — Roadmap

## Version 1.0 (Released) ✅

### Core Engine (Rust)

- [x] KDBX 4.x read/write (Argon2id + ChaCha20-Poly1305)
- [x] KDBX 3.1 compatibility (read)
- [x] Composite key: password + key file + hardware key
- [x] HMAC-SHA256 block authentication (tamper detection)
- [x] Full vault CRUD with history and recycle bin
- [x] Full-text search with filters
- [x] TOTP/HOTP (RFC 6238/4226) with SHA-1/256/512
- [x] Password generator (random, passphrase, pronounceable)
- [x] Vault health audit (weak, reused, expired, old)
- [x] HIBP breach monitor (k-anonymity, offline mode)
- [x] Import: Bitwarden, LastPass, Chrome, Firefox, 1Password, CSV
- [x] Export: CSV, JSON
- [x] Emergency access (trusted contact with waiting period)
- [x] Plugin system (WASM sandbox architecture)
- [x] SSH key management + agent protocol
- [x] Passkey (FIDO2/WebAuthn) storage

### Desktop (Tauri v2)

- [x] Windows, macOS, Linux native
- [x] System tray with lock/unlock
- [x] Global shortcuts (Ctrl+Alt+K)
- [x] Command palette (Ctrl+K)
- [x] Auto-lock on idle / screen lock
- [x] Clipboard auto-clear (default 10s)
- [x] Browser extension native messaging
- [x] All pages: Vault, Health, Breach, Generator, Import/Export, Sync, Emergency Access, Plugins, Settings

### Mobile (React Native)

- [x] iOS 15+ and Android 8.0+
- [x] Biometric unlock (Face ID, Touch ID, Fingerprint)
- [x] iOS AutoFill Extension
- [x] Android AutoFill Service
- [x] iOS WidgetKit (small, medium, large, lock screen)
- [x] Android home screen widget
- [x] Screen capture protection

### Watch

- [x] watchOS (SwiftUI + WatchConnectivity + Complications)
- [x] WearOS (Jetpack Compose + Wearable Data Layer + Tiles)

### Browser Extension

- [x] Chrome/Edge (Manifest V3)
- [x] Firefox (Manifest V2)
- [x] Form detection and credential filling
- [x] Native messaging with desktop app

### CLI (`kpx`)

- [x] All CRUD commands (list, get, add, edit, delete)
- [x] Password generator
- [x] Health check
- [x] OTP with countdown (watch mode)
- [x] Import/Export
- [x] Sync (WebDAV + local)
- [x] Breach check (HIBP)
- [x] Stats

### i18n

- [x] English (900+ keys)
- [x] Vietnamese (full parity)

---

## Version 1.1 (In Progress) 🚧

### Security Enhancements ✅

- [x] **YubiKey HMAC-SHA1** — hardware key support (slot 1/2)
- [x] **FIDO2 hardware key** — WebAuthn hardware authenticator
- [x] **Smart Card / PIV** — enterprise hardware key
- [x] **OnlyKey** — hardware token support
- [x] **Decoy vault** — fake vault for duress scenarios
- [x] **Audit log** — track all vault access and modifications (24 event types)
- [x] **Password policies** — 4 built-in policies + custom rules engine

### Sync Improvements ✅

- [x] **Google Drive** — OAuth2 REST API
- [x] **OneDrive** — Microsoft Graph API
- [x] **Dropbox** — Dropbox API v2
- [x] **Amazon S3** — AWS Signature V4 (+ MinIO, Backblaze B2)
- [x] **SFTP** — SSH File Transfer Protocol
- [x] **iCloud Drive** — native iOS/macOS platform bridge

### Import Improvements ✅

- [x] **Dashlane JSON** — credentials, secure notes, payment cards, IDs
- [x] **NordPass CSV** — credentials, payment cards, folders
- [x] **Enpass JSON** — all item types, folders, OTP
- [x] **RoboForm HTML** — table-based HTML export

### Desktop Improvements ✅

- [x] **Entry preview pane** — split-view with live OTP, copy fields
- [x] **Filter chips** — All / Favorites / OTP / Expiring / No Password
- [x] **Entry history viewer** — timeline with diff highlighting
- [x] **Password field component** — strength meter, show/hide, generate
- [x] **OTP display component** — SVG countdown ring, copy
- [x] **Vault comparison** — UUID-based diff + merge with 4 strategies
- [x] **Scheduled backup** — OnSave/Daily/Weekly/Monthly + retention policy
- [x] **Password policy page** — enable/disable policies, test passwords
- [x] **Audit log page** — filter by type, search, export, clear
- [x] **Backup page** — configure, backup now, restore, history

### Mobile Improvements ✅

- [x] **Swipe actions** — copy/favorite (left), edit/delete (right)
- [x] **Filter chips** — All / Favorites / OTP / Expiring / No Password
- [x] **Multi-select mode** — long press, checkboxes, bulk delete
- [x] **Recently used section** — last 3 entries, horizontal scroll
- [x] **i18n in all screens** — SettingsScreen, WelcomeScreen, UnlockScreen, GeneratorScreen
- [x] **Language switcher** — EN/VI toggle in settings
- [x] **Clipboard clear settings** — 10s/30s/60s/Never chips
- [x] **Auto-lock settings** — 1/5/15/Never chips

### Watch Improvements ✅

- [x] **watchOS search** — Digital Crown scroll, filter by title/username
- [x] **watchOS favorites** — star indicator, favorites-only filter
- [x] **watchOS haptic feedback** — success/failure/OTP warning
- [x] **WearOS rotary input** — Digital Crown/bezel scrolling
- [x] **WearOS favorites toggle** — filter chip
- [x] **WearOS haptic feedback** — VibrationPattern enum

### Browser Extension Improvements ✅

- [x] **OTP inline display** — live countdown timer (1s interval)
- [x] **Password save detection** — detect new credentials, offer to save
- [x] **Recently used entries** — last 3 filled entries
- [x] **Smart URL matching** — exact/domain/subdomain badges
- [x] **Keyboard navigation** — Arrow keys + Enter + Escape
- [x] **Dark mode** — prefers-color-scheme
- [x] **i18n EN/VI** — based on navigator.language
- [x] **Generate password** — ⚡ button in search bar

### CLI Improvements ✅

- [x] **kpx template list/show** — list and inspect entry templates
- [x] **kpx hardware-key list/test/setup** — hardware key management
- [x] **kpx compare** — diff two vaults with colored output
- [x] **kpx audit** — show vault audit log

### Core Improvements ✅

- [x] **Entry templates** — 12 built-in types (Login, Credit Card, Bank, Identity, etc.)
- [x] **Notification system** — expiry warnings, breach alerts, sync errors
- [x] **Vault comparison** — UUID-based diff + CRDT-inspired merge
- [x] **Audit log** — ring-buffer, filter by type/entry, failed unlock detection
- [x] **Password policy engine** — 4 built-in policies, custom rules, EN/VI messages
- [x] **Scheduled backup** — frequency-based, retention policy, restore
- [x] **Decoy vault** — 10 realistic fake entries, silent detection

### Testing ✅

- [x] **Policy tests** — 25 unit tests covering all rule types
- [x] **Backup tests** — 15 unit tests covering all frequency/edge cases
- [x] **Compare tests** — 12 unit tests covering diff and merge
- [x] **Audit tests** — 15 unit tests covering all event types
- [x] **New import tests** — NordPass, Enpass, Dashlane, RoboForm (30+ tests)
- [x] **Desktop page tests** — AuditLogPage, BackupPage, VaultComparePage, PasswordPolicyPage
- [x] **12 new Rust test modules** — analytics, categorizer, decoy_vault, expiry_engine, notifications, scheduled_backup, zkpv, steg, team, templates, plugin, search (all previously untested modules now covered)
- [x] **i18n interpolation parity test** — validates `{{variable}}` placeholders match between EN and VI
- [x] **WelcomePage dialog tests** — tests for the new modal-based vault open/create flow
- [x] **Total: 626 Rust tests + 150 TypeScript tests**

---

## Version 1.2 (Q3 2025)

### Security

- [ ] **Memory encryption** — encrypt vault in memory between operations
- [ ] **Secure memory allocator** — prevent sensitive data in swap
- [ ] **Key file rotation** — generate new key file, re-encrypt vault
- [ ] **Master password change UI** — desktop + mobile

### New Features

- [ ] **Vault sharing** — share specific entries/groups (E2EE)
- [ ] **Secure notes** — rich text with markdown support
- [ ] **File vault** — store arbitrary files encrypted in vault
- [ ] **Duplicate entry detection** — find and merge duplicates
- [ ] **Entry usage analytics** — most used, least used, never used

### Platform

- [ ] **Safari extension** — native Safari Web Extension
- [ ] **Firefox for Android** — mobile browser extension
- [ ] **Linux Wayland** — proper Wayland auto-type support
- [ ] **iPadOS split view** — optimized iPad layout

### CLI

- [ ] **Interactive mode** — `kpx shell` for interactive session
- [ ] **Config file** — `~/.config/kpx/config.toml`
- [ ] **Pipe support** — `echo "password" | kpx add --title GitHub`

---

## Version 2.0 (2026)

### Architecture

- [ ] **Rust mobile core** — native Rust + Swift/Kotlin UI (no React Native)
- [ ] **WASM plugin runtime** — full wasmtime integration
- [ ] **Distributed vault** — CRDTs for true multi-device sync

### AI Features

- [ ] **AI password audit** — ML-based strength analysis
- [ ] **Smart autofill** — AI-powered form field detection
- [ ] **Breach prediction** — predict likely breach targets

### Enterprise

- [ ] **LDAP/Active Directory** — enterprise authentication
- [ ] **SSO integration** — SAML/OIDC support
- [ ] **Compliance reports** — SOC2, HIPAA, PCI-DSS
- [ ] **Admin console** — web-based team management

---

## Feature Comparison vs Competitors

| Feature             | KeePass | KeePassXC | Keepassium | KeePass2Android | **KeePassEx** |
| ------------------- | ------- | --------- | ---------- | --------------- | ------------- |
| Native Windows      | ✅      | ✅        | ❌         | ❌              | ✅            |
| Native macOS        | ❌      | ✅        | ✅         | ❌              | ✅            |
| Native Linux        | ❌      | ✅        | ❌         | ❌              | ✅            |
| Native iOS          | ❌      | ❌        | ✅         | ❌              | ✅            |
| Native Android      | ❌      | ❌        | ❌         | ✅              | ✅            |
| Native watchOS      | ❌      | ❌        | ❌         | ❌              | ✅            |
| Native WearOS       | ❌      | ❌        | ❌         | ❌              | ✅            |
| YubiKey             | ✅      | ✅        | ❌         | ❌              | ✅            |
| Passkey (FIDO2)     | ❌      | ✅        | ❌         | ❌              | ✅            |
| SSH Agent           | ❌      | ✅        | ❌         | ❌              | ✅            |
| Breach Monitor      | ❌      | ✅        | ❌         | ❌              | ✅            |
| Emergency Access    | ❌      | ❌        | ❌         | ❌              | ✅            |
| Plugin System       | ✅      | ❌        | ❌         | ❌              | ✅            |
| Entry Templates     | ✅      | ❌        | ❌         | ❌              | ✅            |
| Audit Log           | ❌      | ❌        | ❌         | ❌              | ✅            |
| Decoy Vault         | ✅      | ❌        | ❌         | ❌              | ✅            |
| Vault Compare       | ❌      | ❌        | ❌         | ❌              | ✅            |
| Scheduled Backup    | ❌      | ❌        | ❌         | ❌              | ✅            |
| Password Policies   | ❌      | ❌        | ❌         | ❌              | ✅            |
| Browser Extension   | ❌      | ✅        | ❌         | ❌              | ✅            |
| iOS AutoFill        | ❌      | ❌        | ✅         | ❌              | ✅            |
| Android AutoFill    | ❌      | ❌        | ❌         | ✅              | ✅            |
| iOS Widget          | ❌      | ❌        | ❌         | ❌              | ✅            |
| Android Widget      | ❌      | ❌        | ❌         | ❌              | ✅            |
| Watch Complications | ❌      | ❌        | ❌         | ❌              | ✅            |
| Command Palette     | ❌      | ❌        | ❌         | ❌              | ✅            |
| CLI                 | ❌      | ✅        | ❌         | ❌              | ✅            |
| Vietnamese i18n     | ❌      | ❌        | ❌         | ❌              | ✅            |
| Sync (8 providers)  | ❌      | ✅        | ✅         | ✅              | ✅            |
| Import (12 formats) | ✅      | ✅        | ✅         | ✅              | ✅            |
| Open Source         | ✅      | ✅        | ❌         | ✅              | ✅            |

**KeePassEx is the only password manager with ALL of the above features.**

---

## Contributing

Have a feature request? Open an issue on GitHub with the `enhancement` label.

Priority is determined by:

1. Security impact
2. User demand (GitHub reactions)
3. Implementation complexity
4. Alignment with core principles (offline-first, zero-knowledge, native everywhere)
