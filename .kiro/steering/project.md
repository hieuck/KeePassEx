---
inclusion: always
---

# KeePassEx — Project Steering

## Architecture

- **Monorepo** với Turborepo + Cargo workspace
- `packages/core` — Rust: all crypto, KDBX engine, vault logic, cache (**32 modules**, shared by ALL platforms)
- `packages/ui` — React/Tamagui: shared design system (15 components)
- `packages/i18n` — i18next: **10 languages** (~1082 keys each, 100% parity)
- `apps/desktop` — Tauri v2 (Windows/macOS/Linux), 22 pages, **29 Tauri command files**
- `apps/mobile` — React Native (iOS/Android), 16 screens
- `apps/watch` — SwiftUI (watchOS) + Jetpack Compose (WearOS)
- `apps/browser-extension` — Chrome MV3 + Firefox MV2
- `apps/cli` — Rust CLI (`kpx`), 18 commands
- `apps/tui` — Rust TUI (ratatui, vim keybindings)
- `apps/server` — Self-hosted sync server (Axum + SQLite + JWT + WebSocket)
- `apps/macos-menubar` — SwiftUI menu bar app (⌘⇧K)
- `apps/windows-credprov` — Windows Credential Provider (Rust cdylib)
- `shared/types` — TypeScript types
- `shared/constants` — App-wide constants (`APP_VERSION = '0.1.0'`)
- `shared/utils` — Shared utilities

## Core Principles

1. **Security first** — Argon2id KDF, ChaCha20-Poly1305, zero-knowledge, PQC (Kyber-768)
2. **Native everywhere** — No Electron, no web wrappers on mobile
3. **KDBX 4.x compatible** — Interoperable with KeePass/KeePassXC
4. **Offline first** — Works without internet, sync is optional
5. **Privacy** — No telemetry, no cloud accounts required

## Coding Standards

- Rust: follow `clippy` lints, use `thiserror` for errors, `zeroize` for secrets
- TypeScript: strict mode, no `any`, prefer `unknown`
- React: functional components, hooks only, no class components
- All user-facing strings MUST use i18n keys (never hardcode text in any language)
- Accessibility: WCAG 2.1 AA minimum, proper ARIA labels

## Security Rules (MANDATORY)

- **Never log passwords, keys, or sensitive data**
- **Always use `ProtectedString` for password/key fields**
- **Always `zeroize` sensitive data on drop**
- **Clipboard auto-clear must default to 10 seconds**
- **No network calls with plaintext passwords**
- **Screen capture protection on mobile**
- **Entry cache must NEVER cache passwords** — only metadata

## i18n Rules

- All strings in `packages/i18n/src/locales/en.ts` (primary)
- **10 languages**: `en`, `vi`, `zh`, `ja`, `ko`, `es`, `fr`, `de`, `pt`, `ru` — all must stay in parity
- Key format: `section.subsection.key` (e.g., `entry.copyPassword`)
- New features MUST add keys to ALL 10 language files before merging
- `SupportedLocale` type in `packages/i18n/src/index.ts` must be updated when adding languages

## Key Files

- Core engine: `packages/core/src/lib.rs` (32 modules)
- Entry cache: `packages/core/src/cache/mod.rs`
- KDBX format: `packages/core/src/kdbx/`
- Vault operations: `packages/core/src/vault/`
- PQC crypto: `packages/core/src/crypto/pqc.rs`
- Shamir sharding: `packages/core/src/crypto/shamir.rs`
- Steganography: `packages/core/src/steg/`
- Field references: `packages/core/src/field_references.rs`
- Favicon fetch: `packages/core/src/favicon.rs`
- Sync providers: `packages/core/src/sync/providers.rs`
- AI suggestions: `packages/core/src/ai/mod.rs`
- Desktop app: `apps/desktop/src/App.tsx`
- Desktop stores: `apps/desktop/src/store/` (vault, settings, sync, breach, tabs)
- Sync commands: `apps/desktop/src-tauri/src/commands/sync_cmd.rs` (real sync wired)
- Mobile app: `apps/mobile/src/App.tsx`
- CLI: `apps/cli/src/main.rs`
- Browser extension: `apps/browser-extension/src/background.ts`
- Tauri commands: `apps/desktop/src-tauri/src/commands/` (29 files)
- Server: `apps/server/src/main.rs` (Axum, SQLite, JWT, WebSocket)
- macOS Menu Bar: `apps/macos-menubar/KeePassExMenuBar/`
- Windows Credential Provider: `apps/windows-credprov/src/`

## Build Commands

```bash
make install      # Install all dependencies
make desktop      # Run desktop dev
make mobile       # Start mobile metro
make cli ARGS="--help"  # Run CLI
make tui ARGS="--vault path"  # Run TUI
cargo run -p keepassex-server -- --port 8080  # Run server
make test         # Run all tests (650+ Rust + 156 TS)
make check        # Lint + typecheck + fmt-check
make help         # Show all commands
```

## Feature Status

**All features are fully implemented.** Do not add stubs or TODOs — implement completely or ask first.

Breakthrough features (no competitor has these):

- Post-quantum encryption (X25519 + Kyber-768)
- Vault key sharding (Shamir's Secret Sharing)
- Steganography (PNG/JPEG/MP4/AVI)
- ZKPV (zero-knowledge password verification)
- Self-hosted sync server (zero-knowledge)
- macOS Menu Bar app
- Windows Credential Provider
- Natural language search (EN/VI)
- Smart categorizer (16 categories)
- Team vault with RBAC
- Analytics dashboard
- Password rotation engine
- Context-aware password advisor
- AI password suggestions (on-device)
- Entry cache (LRU, no passwords)

## i18n Usage Pattern

All UI components MUST use `useTranslation()` from `react-i18next`. Never hardcode strings.

```tsx
// ✅ Correct
import { useTranslation } from 'react-i18next';
const { t } = useTranslation();
<button>{t('vault.open')}</button>;

// ❌ Wrong — hardcoded strings
<button>Open Vault</button>;
// ❌ Wrong — conditional language check
const isVi = settings.language === 'vi';
<button>{isVi ? 'Mở kho' : 'Open Vault'}</button>;
```

## Commit Convention

Follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat(scope):` — new feature → minor bump
- `fix(scope):` — bug fix → patch bump
- `feat(scope)!:` — breaking change → major bump
- `chore(scope):` — tooling/CI → no bump
- `docs(scope):` — docs only → no bump

See `CONTRIBUTING.md` for full details.
