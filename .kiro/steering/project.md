---
inclusion: always
---

# KeePassEx — Project Steering

## Architecture

- **Monorepo** with Turborepo + Cargo workspace
- `packages/core` — Rust: all crypto, KDBX engine, vault logic (shared by ALL platforms)
- `packages/ui` — React/Tamagui: shared design system
- `packages/i18n` — i18next: EN + VI translations (400+ keys, full parity)
- `apps/desktop` — Tauri v2 (Windows/macOS/Linux)
- `apps/mobile` — React Native (iOS/Android)
- `apps/watch` — SwiftUI (watchOS) + Jetpack Compose (WearOS)
- `apps/browser-extension` — Chrome MV3 + Firefox MV2
- `apps/cli` — Rust CLI (`kpx`)
- `shared/types` — TypeScript types
- `shared/constants` — App-wide constants
- `shared/utils` — Shared utilities

## Core Principles

1. **Security first** — Argon2id KDF, ChaCha20-Poly1305 cipher, zero-knowledge
2. **Native everywhere** — No Electron, no web wrappers on mobile
3. **KDBX 4.x compatible** — Interoperable with KeePass/KeePassXC
4. **Offline first** — Works without internet, sync is optional
5. **Privacy** — No telemetry, no cloud accounts required

## Coding Standards

- Rust: follow `clippy` lints, use `thiserror` for errors, `zeroize` for secrets
- TypeScript: strict mode, no `any`, prefer `unknown`
- React: functional components, hooks only, no class components
- All user-facing strings MUST use i18n keys (never hardcode EN/VI text)
- Accessibility: WCAG 2.1 AA minimum, proper ARIA labels

## Security Rules (MANDATORY)

- **Never log passwords, keys, or sensitive data**
- **Always use `ProtectedString` for password/key fields**
- **Always `zeroize` sensitive data on drop**
- **Clipboard auto-clear must default to 10 seconds**
- **No network calls with plaintext passwords**
- **Screen capture protection on mobile**

## i18n Rules

- All strings in `packages/i18n/src/locales/en.ts` and `vi.ts` (primary)
- 7 languages: `en`, `vi`, `zh`, `ja`, `ko`, `es`, `fr` — all must stay in parity
- Key format: `section.subsection.key` (e.g., `entry.copyPassword`)
- New features MUST add keys to ALL 7 language files before merging
- `SupportedLocale` type in `packages/i18n/src/index.ts` must be updated when adding languages

## Key Files

- Core engine: `packages/core/src/lib.rs`
- KDBX format: `packages/core/src/kdbx/`
- Vault operations: `packages/core/src/vault/`
- Field references: `packages/core/src/field_references.rs`
- Favicon fetch: `packages/core/src/favicon.rs`
- Desktop app: `apps/desktop/src/App.tsx`
- Desktop stores: `apps/desktop/src/store/` (vault, settings, sync, breach, tabs)
- Mobile app: `apps/mobile/src/App.tsx`
- CLI: `apps/cli/src/main.rs`
- Browser extension: `apps/browser-extension/src/background.ts`
- Tauri commands: `apps/desktop/src-tauri/src/commands/` (one file per domain)

## Build Commands

```bash
make install      # Install all dependencies
make desktop      # Run desktop dev
make mobile       # Start mobile metro
make cli ARGS="--help"  # Run CLI
make test         # Run all tests
make check        # Lint + typecheck + fmt-check
make help         # Show all commands
```

## Feature Status

All features are fully implemented. Do not add stubs or TODOs — implement completely or ask first.

## i18n Usage Pattern

All UI components MUST use `useTranslation()` from `react-i18next`. Never use `isVi ? '...' : '...'` patterns.

```tsx
// ✅ Correct
import { useTranslation } from 'react-i18next';
const { t } = useTranslation();
<button>{t('vault.open')}</button>;

// ❌ Wrong — hardcoded strings
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
