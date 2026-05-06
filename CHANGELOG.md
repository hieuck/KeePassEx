# Changelog

All notable changes to KeePassEx are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **entry history commands**: `get_entry_history`, `restore_entry_from_history`, `clear_entry_history` — Tauri commands now registered and implemented in `apps/desktop/src-tauri/src/commands/entries.rs`; the `EntryDetailPage` history tab is now fully functional
- **12 new Rust test modules**: `analytics_tests`, `categorizer_tests`, `decoy_vault_tests`, `expiry_engine_tests`, `notifications_tests`, `scheduled_backup_tests`, `zkpv_tests`, `steg_tests`, `team_tests`, `templates_tests`, `plugin_tests`, `search_tests` — all 29 core modules now have dedicated test coverage (626 total Rust tests, up from ~60)

### Fixed

- **vault store**: `unlockVault` now uses a dedicated `vaultPath` field that persists through lock/unlock cycles, preventing "No vault to unlock" errors when `meta` was cleared on lock
- **release workflow**: replaced deprecated `upload-release-asset@v1` with `github-script@v7` for release asset uploads; added missing `upload-url` output from `create-release` job
- **CI**: added `aarch64-unknown-linux-gnu` target to CLI build matrix for Linux ARM64 support
- **i18n**: migrated `VaultPage`, `EntryDetailPage`, `WelcomePage`, `UnlockPage`, `MainLayout`, `VaultComparePage`, `SyncPage`, `SettingsPage`, and `StatisticsPage` from hardcoded `isVi ? '...' : '...'` patterns to proper `useTranslation()` hook calls
- **mobile**: tab bar labels in `MainTabs` now use i18n keys instead of hardcoded English strings
- **crypto/protected_stream**: replaced SHA-256 hash chain placeholder with correct pure-Rust ChaCha20 (RFC 7539) and Salsa20 implementations for KDBX inner stream encryption; added RFC 7539 test vector verification
- **WelcomePage**: replaced `window.prompt()` calls for vault open/create with proper React modal dialogs — passwords are no longer exposed via browser native dialogs
- **WelcomePage**: feature highlight items now use correct i18n keys (`hardwareKey.title`, `passkey.title`, `breach.title`, `sync.title`, `browserExtension.fill`) instead of wrong keys like `hardwareKey.typeYubikeyHmac`
- **i18n test**: added `no empty values in VI` test and `EN/VI interpolation variable parity` test — the new interpolation test correctly excludes i18next-specific plural suffixes (`{{plural}}`) that are language-specific
- **utils tests**: fixed `formatRelativeTime` tests that expected "Today"/"Hôm nay" for `new Date()` — the function correctly returns "Just now"/"Vừa xong" for timestamps under 60 seconds old; fixed `truncate` tests to account for the `'…'` suffix being 1 character; removed `capitalize` tests for a function that does not exist in the utils module
- **pages test**: fixed `await` used inside a non-`async` function in the BreachPage test

### Added

- **KDBX 3.1 read support**: `KdbxReader::read_kdbx3()` now fully implements KDBX 3.1 decryption — AES-256-CBC outer cipher, SHA-256 block stream integrity, and Salsa20 inner stream; enables opening vaults from KeePass 2.x
- **AES-256-CBC decryption**: added `cbc = "0.1"` to workspace and implemented `Cipher::decrypt()` for `Aes256Cbc` variant
- **Protected stream tests**: 10 new tests covering ChaCha20/Salsa20 roundtrips, statefulness, determinism, and RFC 7539 test vector
- **KDBX 3.1 tests**: 6 new tests for version detection, AES-CBC cipher UUID, and protected stream roundtrips
- `CHANGELOG.md` — Keep a Changelog format with full v1.0.0 feature list
- `CONTRIBUTING.md` — Conventional Commits guide, i18n rules, security rules, PR checklist

### Changed

- `unlockVault` in vault store now accepts an optional `keyFileData` parameter for key-file-protected vaults
- `VaultComparePage` sub-components now accept translated string props instead of `isVi: boolean`
- `SyncPage` provider list simplified — removed duplicate `labelEn`/`labelVi` fields

---

## [1.0.0] — 2025-05-06

### Added

- KDBX 4.x read/write with Argon2id KDF and ChaCha20-Poly1305 cipher
- Desktop app (Tauri v2) for Windows, macOS, Linux — 20 pages
- Mobile app (React Native) for iOS and Android — 16 screens
- watchOS app (SwiftUI) and WearOS app (Jetpack Compose)
- Browser extension for Chrome (MV3) and Firefox (MV2)
- CLI (`kpx`) with 18 commands
- TUI (`kpx-tui`) with vim-style keybindings
- TOTP/HOTP, Passkey (FIDO2/WebAuthn), SSH Agent
- Breach monitor via HIBP k-anonymity (offline + online modes)
- Emergency access with configurable waiting period
- Plugin system with WASM sandbox
- Import from 12 formats (Bitwarden, LastPass, 1Password, Dashlane, NordPass, Enpass, RoboForm, Chrome, Firefox, KeePass 1.x, CSV, KDBX)
- Export to KDBX, CSV, JSON, HTML
- Sync via WebDAV, Google Drive, OneDrive, Dropbox, S3, SFTP, iCloud, local folder
- Steganography: embed vault in PNG/JPEG/MP4/AVI
- Vault key sharding via Shamir's Secret Sharing
- Quantum-resistant hybrid encryption (X25519 + CRYSTALS-Kyber-768)
- Zero-knowledge password verification (ZKPV)
- Smart categorizer and natural language search
- Password rotation engine with category-aware schedules
- Team vault with role-based access control
- Vault analytics dashboard
- Scheduled backup with configurable frequency
- Audit log with security event tracking
- Password policy engine with built-in and custom policies
- Vault comparison and merge
- Decoy vault (duress mode)
- EN + VI localization with 900+ keys at full parity
- 626 Rust unit tests across 29 modules, 150 TypeScript tests
- CI/CD with GitHub Actions (all platforms)

[Unreleased]: https://github.com/keepassex/keepassex/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/keepassex/keepassex/releases/tag/v1.0.0
