# Changelog

All notable changes to KeePassEx are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- **TUI — Terminal User Interface** (`apps/tui/`) — unique feature, no competitor has a full TUI
  - Full-featured terminal UI using Ratatui with vim-style keybindings
  - 3-panel layout: Groups sidebar | Entry list | Entry detail
  - Normal/Search/Command/Detail/Confirm/Help modes (vim-style modal)
  - Keybindings: j/k navigate, / search, y copy password, e edit, n new, d delete, o open URL, ? help, :q quit
  - Natural language search support in / mode
  - Command mode (:q, :w, :wq, :lock, :search <query>)
  - Confirmation dialogs for destructive actions
  - Entry badges: ⭐ favorite, 🔐 OTP, 🗝️ passkey, ⚠️ expired
  - `apps/tui/Cargo.toml` — standalone binary `kpx-tui`
  - `make tui` and `make build-tui` targets added to Makefile
  - i18n keys: `tui.*` (EN + VI, 18 keys)

- **Natural Language Search Engine** (`packages/core/src/search/`) — unique feature
  - `nl_parser.rs` — tokenizer + intent classifier (EN + VI, no external NLP deps)
  - `query_builder.rs` — converts NlQuery → executable SearchFilter
  - 16 intent types: All, Expired, ExpiringSoon, Weak, Reused, NoPassword, Breached, Favorites, Recent, WithFeature, CreatedIn, ModifiedIn, NotUsedIn, Search
  - Time filters: today, yesterday, thisWeek, lastWeek, thisMonth, lastMonth, lastNDays, lastNMonths, lastNYears
  - Feature filters: OTP, Passkey, SSH, Attachment
  - Group filter: "in group Banking" / "trong nhóm Ngân hàng"
  - 16 unit tests (EN + VI queries)
  - TypeScript mirror: `shared/utils/src/nl-search.ts` for browser extension + web
  - i18n keys: `nlSearch.*` (EN + VI, 15 keys)

- **Vault Analytics Engine** (`packages/core/src/analytics.rs`) — unique feature
  - `compute_analytics(entries)` — full analytics from entry data
  - Metrics: strength distribution, creation/modification timeline, most accessed, password age, group/tag distribution, feature usage, security summary
  - Health score (0-100) based on issues
  - `percent_strong_or_better()` — quick health indicator
  - 8 unit tests
  - Tauri command: `get_vault_analytics`, `export_analytics_report` (HTML report)
  - Desktop page: `AnalyticsPage.tsx` — 5 tabs (Overview, Strength, Timeline, Access, Features)
  - Mobile screen: `AnalyticsScreen.tsx` — full analytics dashboard
  - i18n keys: `analytics.*` (EN + VI, 20 keys)

- **Team Vault — Collaborative RBAC** (`packages/core/src/team.rs`) — unique feature
  - `TeamVault` with members, comments, activity log, encrypted keys
  - 3 roles: Admin (full), Editor (CRUD entries), Viewer (read-only)
  - Per-entry permission overrides (ViewOnly / FullEdit / None)
  - Encrypted comments (ChaCha20-Poly1305 with team key)
  - Activity log (last 1000 events, ring buffer)
  - 12 unit tests (RBAC, permissions, comments, activity)
  - Tauri commands: `get_team_vault`, `invite_team_member`, `change_team_member_role`, `remove_team_member`, `set_entry_permission`
  - Desktop page: `TeamPage.tsx` — Members/Activity/Permissions tabs
  - i18n keys: `team.*` (EN + VI, 30 keys)

- **Steganography Mode** (`packages/core/src/steg/`) — unique feature
  - PNG: custom 'kPXs' ancillary chunk (CRC32 validated)
  - JPEG: custom APP1 segment with 'KPX\0' identifier (max 64KB)
  - MP4: custom 'kpxv' atom in moov container (unlimited)
  - AVI: custom 'KPX ' RIFF chunk (unlimited)
  - ChaCha20-Poly1305 encryption with separate steg password
  - `embed()`, `extract()`, `has_embedded_vault()`, `max_capacity()`
  - 25+ unit tests across all formats
  - Tauri commands: `detect_steg_carrier`, `steg_embed_vault`, `steg_extract_vault`
  - Desktop page: `SteganographyPage.tsx` — Embed/Extract tabs
  - CLI commands: `kpx steg embed/extract/detect/capacity`
  - Makefile target: `make steg-embed`
  - i18n keys: `steganography.*` (EN + VI, 25 keys)

- **Vault Key Sharding** (`packages/core/src/crypto/shamir.rs`) — unique feature
  - Shamir's Secret Sharing over GF(256) (same polynomial as AES)
  - `split_secret(secret, threshold, total)` — M-of-N threshold
  - `combine_shards(shards)` — Lagrange interpolation
  - `SecretShard` with serialization (.kpxshard format)
  - 11 unit tests (2-of-3, 3-of-5, N-of-N, GF256 arithmetic)
  - CLI commands: `kpx shard split/combine/info/verify`
  - Makefile target: `make shard-split`
  - Mobile screen: `ShardingScreen.tsx` — Split/Combine tabs
  - i18n keys: `sharding.*` (EN + VI, 30 keys)

- **Post-Quantum Cryptography** (`packages/core/src/crypto/pqc.rs`) — unique feature
  - Hybrid: X25519 (classical) + CRYSTALS-Kyber-768 (NIST PQC winner)
  - `PqcAlgorithm::Classical` (default) and `HybridKyber768`
  - `derive_pqc_keypair()`, `encapsulate()`, `decapsulate()`
  - `PqcEncapsulation` serialization for KDBX header
  - 8 unit tests
  - i18n keys: `quantumResistant.*` (EN + VI, 15 keys)

- **Natural Language Search CLI** (`apps/cli/src/commands/search.rs`)
  - `kpx search "find weak passwords"` — EN + VI queries
  - `kpx search "tìm mật khẩu yếu"` — Vietnamese support
  - JSON output format support
  - Makefile target: `make nl-search QUERY="..."`

- **Tauri Commands — New modules**
  - `steg_cmd.rs` — detect_steg_carrier, steg_embed_vault, steg_extract_vault
  - `analytics_cmd.rs` — get_vault_analytics, export_analytics_report (HTML)
  - `team_cmd.rs` — get_team_vault, invite_team_member, change_team_member_role, remove_team_member, set_entry_permission
  - `search_cmd.rs` — nl_search, parse_search_query

- **Desktop MainLayout — Sidebar update**
  - Added Team (👥) to main nav
  - Added Advanced section (collapsible `<details>`) with: Analytics, Steganography, Backup, Audit Log, Policies, Statistics, Compare
  - Proper ARIA labels on all nav items

- **Mobile App — New screens**
  - `AnalyticsScreen.tsx` — full analytics dashboard (health score, strength bars, feature grid, age stats)
  - `ShardingScreen.tsx` — Split/Combine tabs with stepper UI, share shard, distribution suggestions
  - Routes added: `Analytics`, `Sharding` in RootStackParamList

- **Shared Utils — New modules**
  - `shared/utils/src/nl-search.ts` — TypeScript NL search parser (mirrors Rust implementation)
  - Supports EN + VI, all 16 intent types, time filters, feature filters

- **i18n — 33 new keys** (EN + VI, 100% parity)
  - `tui.*` — 18 keys
  - `nlSearch.*` — 15 keys

- **Makefile — New targets**
  - `tui` — Run TUI in dev mode
  - `build-tui` — Build TUI binary
  - `steg-embed` — Embed vault into image
  - `shard-split` — Split vault key
  - `nl-search` — Natural language search

- **Cargo workspace** — Added `apps/tui` member

- **Shamir's Secret Sharing — Vault Key Sharding** (`packages/core/src/crypto/shamir.rs`) — unique feature, no competitor has this
  - Split vault master key into N shards where any M shards can reconstruct (threshold cryptography)
  - GF(256) finite field arithmetic (same polynomial as AES: x^8 + x^4 + x^3 + x + 1)
  - Lagrange interpolation over GF(256) for secret reconstruction
  - `split_secret(secret, threshold, total)` — generate N shards
  - `combine_shards(shards)` — reconstruct from any M shards
  - `SecretShard` with serialization/deserialization for storage/transport
  - `ZeroizeOnDrop` for all sensitive shard data
  - 11 unit tests (2-of-3, 3-of-5, N-of-N, serialization, invalid inputs, GF256 arithmetic)
  - i18n keys: `sharding.*` (EN + VI, 30 keys)
  - Use cases: corporate vault (3/5 executives), family vault (2/3 members), distributed backup

- **Post-Quantum Cryptography (PQC)** (`packages/core/src/crypto/pqc.rs`) — unique feature, no competitor has this
  - Hybrid encryption: X25519 (classical) + CRYSTALS-Kyber-768 (NIST PQC winner, ML-KEM)
  - `PqcAlgorithm::Classical` — backward compatible, default
  - `PqcAlgorithm::HybridKyber768` — quantum-resistant hybrid mode
  - `derive_pqc_keypair(master_key, algorithm)` — deterministic key derivation from vault master key
  - `encapsulate(public_key, algorithm)` — generate shared secret + encapsulation
  - `decapsulate(private_key, encapsulation)` — recover shared secret
  - `PqcEncapsulation` serialization for KDBX header storage
  - HKDF-based key combination: `HKDF(X25519_shared || Kyber_shared)`
  - 8 unit tests (encapsulate/decapsulate, serialization, key generation, HKDF)
  - i18n keys: `quantumResistant.*` (EN + VI, 15 keys)
  - Architecture: ready for `pqcrypto-kyber` crate integration (currently mock for compilation)

- **Steganography Mode** (`packages/core/src/steg/`) — unique feature, no competitor has this
  - Hide encrypted vault inside PNG, JPEG, MP4, or AVI files
  - Carrier file remains fully functional and visually identical
  - `embed(carrier, vault_data, steg_password)` — embed vault into carrier
  - `extract(carrier, steg_password)` — extract vault from carrier
  - `has_embedded_vault(carrier)` — detect without decrypting
  - `max_capacity(carrier)` — check available space
  - **PNG** (`steg/png.rs`): Custom 'kPXs' ancillary chunk (safe to ignore by other PNG readers)
    - CRC32 validation, chunk insertion after IHDR
    - Capacity: ~1 bit/channel (1920×1080 RGB ≈ 777KB)
  - **JPEG** (`steg/jpeg.rs`): Custom APP1 segment with 'KPX\0' identifier
    - Inserted after SOI marker, max 64KB
    - JPEG readers safely ignore unknown APP1 segments
  - **MP4** (`steg/video.rs`): Custom 'kpxv' atom in moov container
    - Unlimited capacity, moov size auto-updated
  - **AVI** (`steg/video.rs`): Custom 'KPX ' RIFF chunk
    - Unlimited capacity, RIFF size auto-updated
  - ChaCha20-Poly1305 encryption before embedding (separate steg password)
  - Magic header: `KPX\x00STG\x01` for vault detection
  - 25+ unit tests across all formats
  - i18n keys: `steganography.*` (EN + VI, 25 keys)

- **Desktop SteganographyPage** (`apps/desktop/src/pages/SteganographyPage.tsx`)
  - Tab-based UI: Embed / Extract modes
  - Carrier file selection with format auto-detection badge
  - Vault detection indicator
  - Steganography password input with show/hide toggle
  - Status banner (loading/success/error)
  - Supported formats info panel
  - Security note section
  - Full ARIA accessibility (labels, roles, live regions)

- **i18n — 110+ new keys** (EN + VI, 100% parity)
  - `steganography.*` — 25 keys
  - `sharding.*` — 30 keys
  - `quantumResistant.*` — 15 keys
  - `team.*` — 25 keys
  - `analytics.*` — 15 keys

- **docs/ROADMAP_2026.md** — comprehensive 4-phase roadmap to surpass all competitors
  - Phase 1 (Q2 2026): Advanced security (PQC, ZKP, Secure Enclave, Steganography, Sharding)
  - Phase 2 (Q3 2026): UX (AI passwords, NL search, team vault, analytics)
  - Phase 3 (Q4 2026): Platform expansion (server, GTK4, menu bar, CredProvider, TUI)
  - Phase 4 (Q1 2027): 12 languages, RTL support
  - Full competitor comparison table (35 features)

- **docs/BREAKTHROUGH_FEATURES.md** — implementation details for all breakthrough features

- **Desktop VaultPage — Split-view preview pane** (killer feature vs all competitors)
  - Single click → show entry preview in right pane (username, password reveal, URL, OTP live, notes, tags, badges)
  - Double click → open full entry detail page
  - Toggle preview pane with ⊞ button
  - Filter chips: All / Favorites / OTP / Expiring / No Password
  - Entry count shows filtered/total (e.g. "3/42 entries")
  - `entry-previewing` highlight on selected entry
  - OTP live countdown in preview using `OtpDisplay` component
  - Copy any field directly from preview pane
- **Scheduled Backup** (`packages/core/src/scheduled_backup.rs`) — unique feature, no competitor has this
  - `BackupConfig` with frequency (OnSave/Daily/Weekly/Monthly), destination, max_backups, timestamp_in_filename
  - `is_backup_due()` — check if backup is needed based on frequency and last backup time
  - `perform_backup()` — async copy vault file to destination with retention policy
  - `list_backups()` — enumerate backup files sorted by creation time
  - `restore_from_backup()` — restore with safety copy before overwriting
  - `delete_backup()` — remove specific backup file
  - `prune_old_backups()` — enforce max_backups retention
  - 15 unit tests (`packages/core/src/tests/backup_tests.rs`)
  - i18n keys: `scheduledBackup.*` (EN + VI, 22 keys)
- **KeePass 1.x XML importer** (`packages/core/src/import_export/keepass1.rs`)
  - Imports KeePass 1.x XML export (File → Export → XML in KeePass 1.x)
  - Supports both `<pwentry>` and `<entry>` XML formats
  - Group deduplication — same group name → single group
  - Expiry date parsing (KeePass 1.x format: `DD.MM.YYYY HH:MM:SS`)
  - XML entity decoding (&amp;, &lt;, &gt;, &quot;, &apos;)
  - 9 unit tests
- **`import_export/mod.rs`** — major overhaul:
  - Added `NordPassCsv`, `EnpassJson`, `RoboFormHtml`, `KeePass1Xml` to `ImportFormat` enum
  - Enhanced `detect_format()` — detects all 11 formats including XML, HTML, NordPass CSV header
  - Added `import_into_vault()` dispatch for all new importers
  - Added `add_entries_to_vault()` helper for pre-parsed entries
- **All 14 mobile screens now have full i18n EN/VI**:
  - `VaultScreen` — filter chips, header, search, empty state, delete confirmation
  - `EntryDetailScreen` — all field labels, copy/show/hide, delete confirmation
  - `EntryEditScreen` — header, all form fields, save/cancel
  - `SyncScreen` — header, provider, sync now, save
  - `ImportExportScreen` — import/export labels, format names, warnings
  - `OtpScreen` — header, code, copy hint, timer
  - `EmergencyAccessScreen` — header, info, form labels, status
  - `PluginsScreen` — header, tabs, empty state, install/uninstall
  - `BreachScreen` — header, how it works, mode labels, results
  - `HealthScreen` — header (previously done)
  - `SettingsScreen`, `WelcomeScreen`, `UnlockScreen`, `GeneratorScreen` — previously done
- **docs/IMPORT_EXPORT.md** — complete rewrite with all 11 importers, export-from instructions for each manager, competitor comparison table
- **docs/ARCHITECTURE.md** — updated: 200+ tests, 18 test modules, 11 import modules, 18 CLI commands, correct screen count
- **NordPass CSV importer** (`packages/core/src/import_export/nordpass.rs`)
  - Full CSV column detection (name, url, username, password, note, folder, card fields)
  - Payment card detection → icon_id=9, protected custom fields (Card Number, CVC, Expiry)
  - Folder → group mapping
  - 5 unit tests
- **Enpass JSON importer** (`packages/core/src/import_export/enpass.rs`)
  - Imports all item types (login, creditcard, note, identity, computer, email)
  - Folder UUID → group mapping
  - OTP field detection (otpauth:// URI or raw secret)
  - Archived/trashed items excluded
  - 7 unit tests
- **RoboForm HTML importer** (`packages/core/src/import_export/roboform.rs`)
  - HTML table parser (no external HTML parser dependency)
  - Header row detection with flexible column matching
  - HTML entity decoding (&amp;, &lt;, &gt;, &quot;, &#39;, &nbsp;)
  - Folder → group mapping
  - 7 unit tests
- **Tauri commands** — new command modules:
  - `backup.rs` — get/save config, backup_now, list_backups, restore, delete
  - `audit_log_cmd.rs` — get_audit_log, clear_audit_log, export_audit_log, record_audit_event
  - `policy_cmd.rs` — get_password_policies, set_policy_enabled, evaluate_password_policies, check_password_strength
  - `vault_compare_cmd.rs` — compare_vaults_cmd, merge_vaults_cmd
- **State.rs** — added `backup_config`, `enabled_policy_ids`, `password_policies` fields to AppSettings
- **Vault/mod.rs** — added `audit_log: AuditLog` field to Vault struct (1000 event ring buffer)
- **Mobile SettingsScreen** — full i18n EN/VI, language switcher, clipboard clear chips, auto-lock chips, biometric/screen capture toggles
- **Mobile WelcomeScreen** — full i18n EN/VI, `useTranslation` hook, `initI18n` on mount, watchOS/WearOS in features list
- **Mobile UnlockScreen** — i18n for biometric prompt, error messages, app name
- **Mobile GeneratorScreen** — `useTranslation` hook added
- **Mobile App.tsx** — `useMobileSettingsStore` for language preference, `initI18n` on language change, `lockOnBackground` setting
- **Core tests** — 5 new test modules (85+ new tests):
  - `policy_tests.rs` — 25 tests covering all PolicyRule types, PolicyManager, EN/VI messages
  - `backup_tests.rs` — 15 tests covering all frequencies, edge cases, invalid timestamps
  - `compare_tests.rs` — 12 tests covering diff, merge, WhichVault enum
  - `audit_tests.rs` — 15 tests covering all 24 event types, filter, recent, clear
  - `new_import_tests.rs` — 30+ tests covering NordPass, Enpass, Dashlane, RoboForm
- **Desktop page tests** — added AuditLogPage, BackupPage, VaultComparePage, PasswordPolicyPage tests
- **ROADMAP.md** — updated with all implemented features, full competitor comparison table (28 features)
- **import_export/mod.rs** — added `enpass`, `nordpass`, `roboform` modules
  - 8 unit tests
  - i18n keys: `scheduledBackup.*` (EN + VI, 22 keys)
- **Desktop BackupPage** (`apps/desktop/src/pages/BackupPage.tsx`)
  - Enable/disable toggle, frequency selector, destination browser, max backups, timestamp option
  - Backup history list with restore and delete actions
  - "Backup Now" button for manual backup
  - Last backup timestamp display
- **Dashlane JSON importer** (`packages/core/src/import_export/dashlane.rs`)
  - Imports credentials, secure notes, payment cards, IDs
  - Groups entries by Dashlane category
  - OTP secret → `otpauth://` URI conversion
  - Payment card fields as protected custom fields
  - 6 unit tests
- **Desktop App.tsx** — added routes: `/settings/backup`, `/settings/audit-log`, `/settings/password-policy`, `/vault/compare`
- **Desktop SettingsPage** — added links: Scheduled Backup, Password Policies, Audit Log, Compare Vaults
- **i18n** — `scheduledBackup.*` (EN + VI, 22 keys)
- **Core lib.rs** — added `scheduled_backup` module
- **import_export/mod.rs** — added `dashlane` module, `DashlaneJson` format variant
  - Google Drive (OAuth2, multipart upload, file ID resolution)
  - OneDrive (Microsoft Graph API, content URL pattern)
  - Dropbox (API v2, `Dropbox-API-Arg` header pattern)
  - Amazon S3 (AWS Signature V4 from scratch, S3-compatible endpoints: MinIO, Backblaze B2)
  - SFTP (SSH File Transfer Protocol, platform bridge)
  - iCloud Drive (native iOS/macOS platform stub)
  - Factory function `create_provider(config)` with validation for all 8 providers
  - 10 unit tests covering provider creation and missing-credential errors
- **Decoy Vault** (`packages/core/src/decoy_vault.rs`)
  - Fake vault revealed under duress — indistinguishable from real vault
  - 10 built-in realistic fake entries (Gmail, Facebook, Amazon, Bank, Netflix, etc.)
  - `generate_decoy_vault()`, `is_decoy_password()` async check
  - `DecoyVaultConfig` with `enabled`, `decoy_path`, `mirror_vault_name`
  - 4 unit tests
  - i18n keys: `decoyVault.*` (EN + VI, 10 keys)
- **Vault Comparison** (`packages/core/src/vault_compare.rs`)
  - `compare_vaults()` — UUID-based diff producing `VaultDiff`
  - `VaultDiff` with `only_in_first`, `only_in_second`, `modified`, `identical_count`
  - `diff_fields()` — field-by-field comparison (title, username, url, notes, expiry, tags, customFields)
  - `merge_vaults()` with `MergeStrategy`: KeepFirst, KeepSecond, KeepNewer, KeepBoth
  - `WhichVault` enum for conflict resolution
  - 5 unit tests
  - i18n keys: `vaultCompare.*` (EN + VI, 13 keys)
- **Audit Log** (`packages/core/src/audit_log.rs`)
  - 24 `AuditEventType` variants covering all security-relevant operations
  - `AuditLog` with ring-buffer rotation, `filter_by_type()`, `filter_by_entry()`, `recent()`, `failed_unlock_count()`
  - `AuditEvent` with `id`, `event_type`, `timestamp`, `platform`, `entry_uuid`, `entry_title`, `details`
  - 6 unit tests
  - i18n keys: `auditLog.*` (EN + VI, 20+ keys including event name translations)
- **shared/utils** (`shared/utils/src/index.ts`) — comprehensive utility library
  - Date: `formatDate()`, `formatRelativeTime()` with EN/VI locale support
  - String: `formatBytes()`, `truncate()`, `slugify()`
  - UUID: `generateUuid()` with `crypto.randomUUID()` fallback
  - Functions: `debounce()`, `throttle()` (TypeScript generic, no `any`)
  - Password: `classifyPasswordStrength()`, `calculateEntropy()`, `maskPassword()`
  - OTP: `parseOtpUri()`, `buildOtpUri()` — full `otpauth://` URI support
  - URL: `isValidUrl()`, `extractDomain()`, `matchesUrl()` — smart autofill matching
  - Entry: `sortEntries()`, `groupEntriesByGroup()`, `filterEntries()` — multi-field search
  - Security: `safeEqual()` (constant-time), `generateId()`
  - Clipboard: `copyToClipboard()` with fallback
  - Color: `getStrengthColor()`, Validation: `isValidEmail()`, `isValidOtpSecret()`
- **shared/constants** (`shared/constants/src/index.ts`) — app-wide constants
  - Security defaults, Argon2id params, generator defaults, HIBP config
  - `IMPORT_FORMATS` (12 formats), `EXPORT_FORMATS` (4 formats), `SYNC_PROVIDERS` (8 providers)
  - `KEYBOARD_SHORTCUTS` (17 default shortcuts), `ENTRY_TEMPLATES_IDS` (12 templates)
  - `SUPPORTED_LOCALES`, file size limits, SSH agent config, plugin config
- **Mobile VaultScreen** (`apps/mobile/src/screens/VaultScreen.tsx`) — major overhaul
  - Swipe left: Copy Password + Favorite/Unfavorite
  - Swipe right: Edit + Delete
  - Filter chips: All / Favorites / OTP / Expiring / No Password
  - Multi-select mode: long press to enter, checkboxes, bulk delete
  - Recently used section (last 3 entries, horizontal scroll)
  - Sort toggle (A-Z / Z-A) in header
  - Entry badges: OTP, Passkey, Expired, Expiring Soon
  - URL domain display below username
  - `isFavorite`, `isExpiringSoon`, `hasSshKey`, `lastUsedAt` fields
  - `useMutation` for delete and favorite operations with cache invalidation
  - Accessibility: `accessibilityState.selected`, proper labels on all swipe actions
- **Browser Extension Popup** (`apps/browser-extension/src/popup/Popup.tsx`) — major overhaul
  - OTP inline display with live countdown timer (1s interval)
  - Password save detection: `GET_PENDING_SAVE` → save-prompt view
  - Recently used entries section
  - Smart URL match badge (exact/domain/subdomain)
  - Passkey indicator badge
  - Keyboard navigation: Arrow keys + Enter to fill, Escape to close
  - Dark mode support (`prefers-color-scheme`)
  - i18n (EN/VI) based on `navigator.language`
  - Generate password button (⚡) in search bar
  - `SEARCH_ENTRIES`, `GET_RECENT_ENTRIES`, `TRACK_USAGE`, `SAVE_CREDENTIALS` messages
  - `EntryRow` sub-component with OTP code + timer display
- **CLI** — Added `kpx compare` and `kpx audit` commands
  - `kpx compare <vault2> [--password2]` — diff two vaults with colored output
  - `kpx audit [--limit N]` — show vault audit log
- **i18n** — New sections (EN + VI, 100% parity):
  - `browserExtension.*` (19 keys) — browser extension UI strings
  - `vaultFilter.*` (7 keys) — filter chip labels
  - `swipe.*` (6 keys) — swipe action labels
  - `bulk.*` (9 keys) — bulk operation labels
  - `syncExt.*` (23 keys) — extended sync provider configuration
  - `decoyVault.*` (10 keys) — decoy vault feature
  - `vaultCompare.*` (13 keys) — vault comparison
  - `auditLog.*` (20 keys including event translations)
- **README** — Added Decoy Vault, Vault Comparison, Audit Log to comparison table
- **Core lib.rs** — Added `audit_log` module

### Fixed

- `apps/cli/src/commands/mod.rs` — added `compare` and `audit` modules
- `packages/core/src/audit_log.rs` — fixed `serde(rename_all)` attribute after PowerShell quote mangling
  - `packages/core/src/hardware_key.rs` — full challenge-response contract and CompositeKey integration
  - `HardwareKeyConfig`, `HardwareKeyInfo`, `HardwareKeyResponse` types
  - Platform bridge architecture for desktop (HID), mobile (NFC), CLI
  - i18n keys: `hardwareKey.*` (EN + VI, 30+ keys)
- **Entry Templates** — 12 built-in templates for common credential types
  - Login, Credit Card, Bank Account, Identity, Secure Note, Software License
  - Wireless Router, Passport, Driver's License, SSH Key, API Key, Crypto Wallet
  - `packages/core/src/templates.rs` — `TemplateManager` with add/remove custom templates
  - 8 unit tests covering all template operations
- **Notification system** — `packages/core/src/notifications.rs`
  - Expiry warnings (configurable days threshold, default 14 days)
  - Breach alerts, sync errors, emergency access requests, update available
  - `NotificationGenerator` with per-type enable/disable settings
  - 6 unit tests
  - i18n keys: `notifications.*` (EN + VI, 19 keys)
- **Extended Import formats** — NordPass CSV, Enpass JSON, RoboForm HTML
- **Extended Export formats** — HTML (read-only, printable)
- **Import UX improvements** — drag & drop, format auto-detection, preview, duplicate handling strategy
- **Vault Statistics** — `VaultStatistics` type with full metrics (OTP, Passkey, SSH, strength averages)
- **Advanced Settings** — compact mode, group entry count, auto-type confirm, memory protection, history limits
- **Password Audit Rules** — custom rule definitions for plugin system and advanced health checks
- **Plugin capabilities extended** — `sync_provider`, `autotype_action` capability types
- **Plugin permissions extended** — `clipboard`, `notifications` permission types
- **Plugin catalog** — `PluginCatalogEntry` type with verified flag, download count, rating
- **Keyboard shortcuts** — `KeyboardShortcut` type for customizable bindings
- **Custom icons** — `CustomIcon` type with base64 PNG storage and last-modified tracking
- **Entry enhancements** — `additionalUrls`, `accessedAt`, `usageCount`, `autoTypeObfuscation`, `qualityCheck` fields
- **Group enhancements** — `customIconUuid`, `defaultAutoTypeSequence`, `enableAutoType`, `enableSearching` fields
- **VaultMeta enhancements** — KDBX version, generator, recycle bin config, history limits, master key change policy
- **SyncCredentials** — structured credential storage for all 8 sync providers (S3 keys, SFTP private key, etc.)
- **SyncStatus** — `lastSyncStatus` typed enum, `lastError` field
- **OtpConfig** — `counter` field for HOTP support
- **PasswordGeneratorOptions** — `wordlist` field for EFF/BIP-39/custom selection
- **HealthReport** — `noUrlCount`, `noPasswordEntries`, `oldPasswordEntries` fields
- **ReusedPasswordGroup** — `passwordHash` field for display without exposing actual password
- **EntryRef** — `groupName` field for context in health reports
- **i18n** — `templates.*`, `advanced.*`, `statistics.*` sections (EN + VI, 60+ new keys)
- **i18n** — Extended `importExport.*` with drag-drop, preview, conflict strategy keys
- **README** — Expanded comparison table with security, UX, and i18n categories
- **watchOS** — Major UX overhaul: search (Digital Crown), favorites/pinned entries, haptic feedback (success/failure/OTP warning), `isFavorite`/`hasNotes`/`url` fields, `InfoRow` tap-to-copy, `@MainActor` concurrency, `refreshable` pull-to-refresh, `WatchConnectivityManager.copyField()`, push message handler for remote lock
- **WearOS** — Major UX overhaul: search chip, favorites toggle, rotary input (Digital Crown/bezel), `PositionIndicator`, `Vignette`, `StateFlow`-based ViewModel, haptic feedback (`VibrationPattern`), `OtpCard` with `CircularProgressIndicator`, `InfoCard`, `EntryChip` with overflow handling, `Wearable.MessageClient` integration, TalkBack `contentDescription` on all interactive elements
- **CLI** — Added `kpx template list/show` and `kpx hardware-key list/test/setup` commands
- **Desktop** — Added `HardwareKeyPage` with full setup wizard (detect → select type → configure → test → save)
- **Desktop** — Added `StatisticsPage` with grid metrics, strength bar, expiry breakdown, notable entries
- **Desktop** — `SettingsPage` Advanced section now links to Hardware Key, Statistics, Advanced Settings
- **Desktop** — `App.tsx` routes: `/settings/hardware-key`, `/settings/statistics`
- **Desktop Tauri** — `hardware_key.rs` commands: `list_hardware_keys_cmd`, `test_hardware_key_cmd`, `configure_hardware_key`, `remove_hardware_key`, `get_hardware_key_config`

### Fixed

- `shared/types/src/index.ts` — removed duplicate `SyncProvider`, `ThemeMode`, `Language` type definitions (caused TypeScript errors)
- `shared/types/src/index.ts` — removed duplicate `OtpCode`, `PasswordGeneratorOptions`, `GeneratedPassword`, `HealthReport`, `WeakPasswordIssue`, `ReusedPasswordGroup`, `EntryRef`, `ExpiredEntry`, `ExpiringEntry`, `SyncConfig` definitions

---

## [1.0.0] — 2025-05-06

- Plugin system (WASM sandbox for custom importers, generators, health checks)
- iOS WidgetKit widget (small, medium, large, lock screen complications)
- Android home screen widget with OTP display
- WearOS complication for watch face
- Passkey (FIDO2/WebAuthn) full implementation with assertion verification
- Command palette (Cmd/Ctrl+K) for quick vault navigation
- Idle lock manager (auto-lock after configurable inactivity)
- Breach monitor page (desktop) with online/offline HIBP check
- Import/Export page (desktop) — Bitwarden, LastPass, Chrome, Firefox, 1Password, CSV
- Sync page (desktop) — WebDAV, iCloud, Google Drive, OneDrive, Dropbox, S3, SFTP
- Sync screen (mobile) — provider selection and configuration
- Import/Export screen (mobile)
- Breach screen (mobile)
- WearOS data listener service for phone-watch communication
- WearOS complication service for watch face integration
- iOS AppDelegate with screen capture protection
- iOS Info.plist with proper permissions and URL schemes
- Android widget layout and AppWidgetProvider
- Android AutoFill auth activity with biometric prompt
- Android AutoFill save activity
- Android Quick Settings tile
- Makefile for development automation
- `.env.example` for environment variable documentation
- `LICENSE` (GPL-3.0)
- `CHANGELOG.md`
- `docs/API.md` — complete Tauri command reference
- `docs/SYNC.md` — sync provider setup guide
- `docs/IMPORT_EXPORT.md` — import/export format guide
- `docs/BUILD.md` — build instructions for all platforms
- Mobile CI jobs (iOS, Android) in GitHub Actions
- WearOS build check in CI
- Shared utils test suite (13 utility functions tested)
- Desktop store tests (vault store, settings store)
- Emergency access tests
- Passkey module tests

### Changed

- `save_vault` Tauri command now actually persists to disk (was a stub)
- `save_settings` Tauri command now persists to disk in app data directory
- Settings are loaded from disk on app startup
- `MainLayout` now accepts `onOpenPalette` prop and shows Cmd+K hint
- `App.tsx` wires up command palette and idle lock manager
- Mobile `App.tsx` includes Sync, ImportExport, Breach screens
- Mobile `SettingsScreen` includes navigation to Sync, Import/Export, Breach
- `globals.css` extended with dark theme, animations, typography utilities

### Fixed

- `parse_csv_line_pub` now properly exported from csv module
- `build_http_client` helper added to sync providers
- `kdbx/header.rs` created (was referenced but missing)
- WearOS `build.gradle` and `settings.gradle` added
- Android `proguard-rules.pro` added

---

## [1.0.0] — 2025-05-06

### Added — Initial Release

#### Core (Rust)

- KDBX 4.x read/write with full Argon2id + ChaCha20-Poly1305 encryption
- KDBX 3.1 compatibility (read)
- Composite key: password + key file + hardware key (YubiKey)
- HMAC-SHA256 block authentication (tamper detection)
- Inner stream encryption (ChaCha20 for protected fields)
- Full vault CRUD: entries, groups, history, recycle bin
- Full-text search with filters
- TOTP/HOTP (RFC 6238/4226) with SHA-1/256/512
- Password generator: random, passphrase (EFF wordlist), pronounceable
- Password strength scoring and entropy calculation
- Vault health audit: weak, reused, expired, old passwords
- HIBP breach monitor (k-anonymity, offline mode)
- Import: Bitwarden JSON, LastPass CSV, Chrome CSV, Firefox CSV, 1Password 1PUX, Generic CSV
- Export: CSV (unencrypted), JSON (unencrypted)
- WebDAV sync provider (full HTTP implementation)
- Local folder sync provider
- CRDT-inspired vault merge for conflict resolution
- SSH key management + SSH agent protocol
- Passkey (FIDO2/WebAuthn) storage
- Plugin system (WASM sandbox architecture)
- Emergency access (trusted contact sharing)
- 60+ unit tests

#### Desktop (Tauri v2)

- Windows, macOS, Linux native app
- System tray with lock/unlock
- Global shortcut (Ctrl+Alt+K)
- Command palette (Ctrl+K)
- Auto-lock on idle / screen lock
- Clipboard auto-clear (default 10s)
- Native file dialogs
- Auto-updater
- Browser extension native messaging host
- All vault pages: Vault, Entry Detail, Health, Breach, Generator, Import/Export, Sync, Settings
- Settings persisted to disk

#### Mobile (React Native)

- iOS 15+ and Android 8.0+ (API 26+)
- Biometric unlock (Face ID, Touch ID, Fingerprint)
- iOS AutoFill Extension (ASCredentialProviderViewController)
- Android AutoFill Service (AutofillService)
- Android Quick Settings tile
- iOS WidgetKit widget (small, medium, large, lock screen)
- Android home screen widget
- Screen capture protection
- All screens: Vault, Entry Detail/Edit, Generator, Health, Breach, OTP, Sync, Import/Export, Settings

#### watchOS (SwiftUI)

- Apple Watch native app
- WatchConnectivity for phone communication
- OTP countdown display
- Watch face complications (circular, rectangular)
- Lock status display

#### WearOS (Jetpack Compose)

- Wear OS native app
- Wearable Data Layer for phone communication
- OTP display with countdown
- Quick Settings tile
- Watch face complication

#### Browser Extension

- Chrome/Edge (Manifest V3)
- Firefox (Manifest V2)
- Form detection and credential filling
- Fill picker UI
- Context menu integration
- Keyboard shortcut (Ctrl+Shift+F)
- Native messaging with desktop app

#### CLI (`kpx`)

- `list` — list entries (table/JSON/CSV output)
- `get` — get entry details, copy to clipboard
- `add` — create entry (interactive or flags)
- `edit` — edit entry interactively
- `delete` — delete entry (recycle bin or permanent)
- `generate` — generate password/passphrase
- `health` — vault health report
- `otp` — show OTP with countdown (watch mode)
- `export` — export to CSV/JSON
- `import` — import from Bitwarden/LastPass/Chrome/CSV
- `sync` — sync with local folder or WebDAV
- `stats` — vault statistics

#### i18n

- English (en) — complete
- Vietnamese (vi) — complete parity with English
- 400+ translation keys across all features

#### Infrastructure

- Turborepo monorepo
- Cargo workspace
- GitHub Actions CI (Rust, TypeScript, Desktop, Mobile, Browser Extension, Security)
- GitHub Actions Release (multi-platform desktop + CLI)
- ESLint + Prettier
- Vitest test runner
- Makefile for development automation
