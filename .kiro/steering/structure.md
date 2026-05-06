# KeePassEx — Project Structure

## Top-Level Layout

```
keepassex/
├── apps/               # Deployable applications
├── packages/           # Shared internal libraries
├── shared/             # Shared TS types, constants, utils
├── docs/               # Architecture, API, build docs
├── Cargo.toml          # Rust workspace root
├── package.json        # JS workspace root (pnpm)
├── turbo.json          # Turborepo task config
├── Makefile            # Developer convenience commands
└── tsconfig.base.json  # Shared TS compiler options
```

## `apps/`

| Directory                 | Description                                                                       |
| ------------------------- | --------------------------------------------------------------------------------- |
| `apps/desktop/`           | Tauri v2 desktop app — React frontend in `src/`, Rust backend in `src-tauri/src/` |
| `apps/mobile/`            | React Native app (iOS + Android)                                                  |
| `apps/watch/`             | watchOS (SwiftUI) + WearOS (Jetpack Compose)                                      |
| `apps/browser-extension/` | Chrome MV3 + Firefox MV2 extension                                                |
| `apps/cli/`               | Rust CLI (`kpx`)                                                                  |
| `apps/tui/`               | Rust terminal UI                                                                  |

### Desktop app layout (`apps/desktop/src/`)

```
components/   # Reusable UI components
pages/        # Full-page views (one file per route)
store/        # Zustand stores (vault, settings, sync, breach)
styles/       # Global CSS
__tests__/    # Vitest tests for components, pages, stores
```

### Tauri backend (`apps/desktop/src-tauri/src/`)

```
commands/     # Tauri command handlers (one file per domain)
```

## `packages/`

| Directory        | Description                                                                 |
| ---------------- | --------------------------------------------------------------------------- |
| `packages/core/` | Rust crate — all crypto, KDBX engine, vault logic. Shared by ALL platforms. |
| `packages/ui/`   | Tamagui-based shared React/RN design system                                 |
| `packages/i18n/` | i18next translations — EN + VI (must stay in sync)                          |

### Core crate layout (`packages/core/src/`)

```
kdbx/           # KDBX 4.x format read/write
vault/          # Vault CRUD operations
crypto/         # Crypto primitives
import_export/  # Bitwarden, LastPass, 1Password, CSV parsers
sync/           # WebDAV + local sync
plugin/         # WASM plugin sandbox
search/         # Entry search
steg/           # Steganography
tests/          # Rust integration tests
```

Key top-level modules: `lib.rs`, `types.rs`, `error.rs`, `generator.rs`, `otp.rs`, `breach.rs`, `health.rs`, `passkey.rs`, `ssh.rs`, `emergency_access.rs`, `team.rs`, `field_references.rs`, `favicon.rs`

Key kdbx modules: `kdbx/reader.rs`, `kdbx/writer.rs`, `kdbx/pqc_header.rs`

Key vault modules: `vault/operations.rs`, `vault/pqc_migration.rs`, `vault/search.rs`

## `shared/`

| Directory           | Description                                           |
| ------------------- | ----------------------------------------------------- |
| `shared/types/`     | TypeScript type definitions shared across all JS apps |
| `shared/constants/` | App-wide constants                                    |
| `shared/utils/`     | Shared utility functions                              |

## i18n Files

```
packages/i18n/src/locales/en.ts   # English (primary)
packages/i18n/src/locales/vi.ts   # Vietnamese
packages/i18n/src/locales/zh.ts   # Chinese Simplified
packages/i18n/src/locales/ja.ts   # Japanese
packages/i18n/src/locales/ko.ts   # Korean
packages/i18n/src/locales/es.ts   # Spanish
packages/i18n/src/locales/fr.ts   # French
```

Key format: `section.subsection.key` (e.g. `entry.copyPassword`, `vault.open`)
All 7 files must stay in parity — new keys go in all files.

## Conventions

- **One concern per file** — pages, commands, and store slices each get their own file
- **Tests co-located** in `__tests__/` directories next to the source they cover
- **Rust commands** in `src-tauri/src/commands/` mirror the domain structure of `packages/core/src/`
- **New features** must add i18n keys to both `en.ts` and `vi.ts` before merging
- **Sensitive data** must use `ProtectedString` in Rust and never be logged
