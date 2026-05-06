# KeePassEx — Tech Stack & Build System

## Languages

- **Rust** — core crypto/vault engine, CLI, TUI, Tauri backend
- **TypeScript** (strict mode) — desktop frontend, mobile, browser extension, shared packages
- **Swift** — watchOS app
- **Kotlin** — WearOS app

## Key Frameworks & Libraries

### Rust

| Crate                                   | Purpose                           |
| --------------------------------------- | --------------------------------- |
| `argon2`, `chacha20poly1305`, `aes-gcm` | Cryptography                      |
| `zeroize`                               | Zeroing secrets from memory       |
| `thiserror` / `anyhow`                  | Error handling                    |
| `tokio`                                 | Async runtime                     |
| `serde` / `serde_json`                  | Serialization                     |
| `quick-xml`                             | KDBX XML parsing                  |
| `reqwest` (rustls)                      | HTTP (HIBP breach checks)         |
| `tauri` v2                              | Desktop app shell                 |
| `axum`                                  | Self-hosted server HTTP framework |
| `sqlx` + SQLite                         | Server database                   |
| `jsonwebtoken`                          | Server JWT authentication         |
| `windows` crate                         | Windows Credential Provider COM   |

### TypeScript / React

| Package                  | Purpose                                 |
| ------------------------ | --------------------------------------- |
| React 18                 | UI (functional components + hooks only) |
| Tauri v2 JS API          | Desktop ↔ Rust bridge                   |
| React Native             | Mobile app                              |
| Tamagui                  | Shared design system (`packages/ui`)    |
| i18next / react-i18next  | Internationalization                    |
| Zustand                  | State management (desktop stores)       |
| Vitest + Testing Library | Unit/component tests                    |

## Build System

- **Turborepo** — JS/TS task orchestration across the monorepo
- **Cargo workspace** — Rust crate management
- **pnpm** (v9, workspaces) — JS package manager; Node ≥ 20 required

## Common Commands

```bash
# Setup
make install          # Install all JS deps + build Rust core

# Development
make desktop          # Desktop app dev mode (Tauri)
make mobile           # React Native metro bundler
make cli ARGS="..."   # Run CLI (e.g. ARGS="--help")
make tui ARGS="..."   # Run TUI
cargo run -p keepassex-server -- --port 8080  # Run server

# Building
make build            # Build everything (Rust release + JS)
make build-core       # Rust core only
make build-desktop    # Desktop production build
make build-cli        # CLI for current platform
make build-extension  # Chrome + Firefox extensions

# Testing
make test             # All tests (Rust + TypeScript)
make test-rust        # cargo test --all --all-features
make test-ts          # vitest run

# Code Quality
make check            # fmt-check + lint + typecheck
make lint             # clippy (errors) + eslint
make fmt              # cargo fmt + prettier
make typecheck        # tsc across all packages

# Security
make audit            # cargo audit + pnpm audit
```

## TypeScript Configuration

- Base config: `tsconfig.base.json` — strict, `noUnusedLocals`, `noUnusedParameters`, `isolatedModules`
- Module resolution: `bundler`
- Target: ES2021

## Linting & Formatting

- **ESLint**: `@typescript-eslint/no-explicit-any` is an error; `consistent-type-imports` enforced; `no-eval`/`no-new-func` are errors
- **Prettier**: single quotes, 2-space indent, 100 char print width, trailing commas (ES5), LF line endings
- **Clippy**: `-D warnings` — all warnings are errors in CI
