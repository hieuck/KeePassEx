---
inclusion: always
---

# KeePassEx — Project Structure

## Top-Level Layout

```
keepassex/
├── apps/               # Deployable applications (9 apps)
├── packages/           # Shared internal libraries
├── shared/             # Shared TS types, constants, utils
├── docs/               # Architecture, API, build docs (17 files)
├── scripts/            # Build/release scripts
├── test/               # Global test setup
├── Cargo.toml          # Rust workspace root (6 members)
├── package.json        # JS workspace root (pnpm)
├── turbo.json          # Turborepo task config
├── Makefile            # Developer convenience commands
├── README.md           # Project overview
├── CHANGELOG.md        # Keep a Changelog format
├── CONTRIBUTING.md     # Contribution guide
├── SECURITY.md         # Security policy
└── tsconfig.base.json  # Shared TS compiler options
```

## `apps/`

| Directory                 | Description                                                         |
| ------------------------- | ------------------------------------------------------------------- |
| `apps/desktop/`           | Tauri v2 — React frontend (`src/`), Rust backend (`src-tauri/src/`) |
| `apps/mobile/`            | React Native — iOS + Android                                        |
| `apps/watch/`             | watchOS (SwiftUI) + WearOS (Jetpack Compose)                        |
| `apps/browser-extension/` | Chrome MV3 + Firefox MV2                                            |
| `apps/cli/`               | Rust CLI (`kpx`), 18 commands                                       |
| `apps/tui/`               | Rust TUI (ratatui, vim keybindings)                                 |
| `apps/server/`            | Self-hosted sync server (Axum + SQLite + JWT + WebSocket)           |
| `apps/macos-menubar/`     | SwiftUI menu bar app (⌘⇧K)                                          |
| `apps/windows-credprov/`  | Windows Credential Provider (Rust cdylib)                           |

### Desktop app layout (`apps/desktop/src/`)

```
components/   # 13 reusable UI components
pages/        # 22 full-page views (one file per route)
store/        # 5 Zustand stores (vault, settings, sync, breach, tabs)
layouts/      # MainLayout
styles/       # Global CSS
__tests__/    # Vitest tests
App.tsx       # Root component with routing
main.tsx      # Entry point
```

### Tauri backend (`apps/desktop/src-tauri/src/`)

```
commands/     # 29 Tauri command files (one per domain)
autotype.rs   # Auto-type engine
native_messaging.rs  # Browser extension IPC
ssh_agent_server.rs  # SSH agent Unix socket
state.rs      # AppState (vault, settings, ssh_agent)
tray.rs       # System tray
lib.rs        # Tauri builder + plugin registration
```

### Mobile app layout (`apps/mobile/src/`)

```
screens/      # 16 screens
components/   # Shared components (OtpCountdown, PasswordField, TagList, etc.)
store/        # 4 Zustand stores (vault, settings, theme, i18n)
native/       # Native module bridges (SecureEnclaveKeystore.ts)
App.tsx       # Navigation root
```

### Server layout (`apps/server/src/`)

```
api/          # REST handlers (auth, vault, health, admin)
auth/         # JWT validation
db/           # SQLite operations
ws/           # WebSocket handler
config.rs     # Server configuration
error.rs      # Error types
main.rs       # Axum router + CLI
```

## `packages/`

| Directory        | Description                                                   |
| ---------------- | ------------------------------------------------------------- |
| `packages/core/` | Rust crate — 32 modules, 650+ tests. Shared by ALL platforms. |
| `packages/ui/`   | Tamagui-based shared React/RN design system (15 components)   |
| `packages/i18n/` | i18next — 10 languages, ~1082 keys each, 100% parity          |

### Core crate layout (`packages/core/src/`)

```
cache/          # LRU entry/group cache (no passwords cached)
crypto/         # cipher, hmac, kdf, keys, pqc, protected_stream, shamir
kdbx/           # KDBX 4.x + 3.1 read/write, pqc_header
vault/          # CRUD operations, pqc_migration, search
import_export/  # 12 importers (Bitwarden, LastPass, 1Password, ...)
sync/           # 8 providers + CRDT merge
search/         # Full-text + natural language parser
steg/           # PNG/JPEG/MP4/AVI steganography
plugin/         # WASM sandbox (wasmtime)
ai/             # AI password suggestion engine
tests/          # 32 test modules, 650+ tests

# Top-level modules (32 total):
ai (dir), analytics.rs, audit_log.rs, breach.rs, cache (dir), categorizer.rs,
crypto (dir), decoy_vault.rs, emergency_access.rs, error.rs,
expiry_engine.rs, favicon.rs, field_references.rs, generator.rs,
hardware_key.rs, health.rs, import_export (dir), kdbx (dir),
notifications.rs, otp.rs, passkey.rs, password_advisor.rs,
password_policy.rs, plugin (dir), scheduled_backup.rs, search (dir),
ssh.rs, steg (dir), sync (dir), team.rs, templates.rs, types.rs,
vault (dir), vault_compare.rs, zkpv.rs
```

## `shared/`

| Directory           | Description                                                    |
| ------------------- | -------------------------------------------------------------- |
| `shared/types/`     | TypeScript types shared across all JS apps (~400 lines)        |
| `shared/constants/` | App-wide constants (security defaults, icon IDs, shortcuts)    |
| `shared/utils/`     | Shared utilities (date, string, OTP, URL, password, clipboard) |

## i18n Files

```
packages/i18n/src/locales/en.ts   # English (primary, ~1082 keys)
packages/i18n/src/locales/vi.ts   # Tiếng Việt
packages/i18n/src/locales/zh.ts   # 简体中文
packages/i18n/src/locales/ja.ts   # 日本語
packages/i18n/src/locales/ko.ts   # 한국어
packages/i18n/src/locales/es.ts   # Español
packages/i18n/src/locales/fr.ts   # Français
packages/i18n/src/locales/de.ts   # Deutsch
packages/i18n/src/locales/pt.ts   # Português
packages/i18n/src/locales/ru.ts   # Русский
```

Key format: `section.subsection.key` (e.g. `entry.copyPassword`, `vault.open`)
**All 10 files must stay in parity** — new keys go in ALL 10 files.

## Conventions

- **One concern per file** — pages, commands, and store slices each get their own file
- **Tests co-located** in `__tests__/` directories next to the source they cover
- **Rust commands** in `src-tauri/src/commands/` mirror the domain structure of `packages/core/src/`
- **New features** must add i18n keys to ALL 10 language files before merging
- **Sensitive data** must use `ProtectedString` in Rust and never be logged
- **Entry cache** must NEVER cache passwords — only metadata (title, username, url, flags)
- **Sync config** persisted in `AppSettings.sync_config` — credentials in memory only
