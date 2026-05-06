# Contributing to KeePassEx

Cảm ơn bạn đã quan tâm đến KeePassEx! / Thank you for your interest in KeePassEx!

## 🌐 Language / Ngôn ngữ

Issues and PRs can be written in **English** or **Vietnamese (Tiếng Việt)**.

---

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.75+ (`rustup install stable`)
- **Node.js** 20+ 
- **pnpm** 9+ (`npm install -g pnpm`)
- **Tauri CLI** (`cargo install tauri-cli`)

### Setup

```bash
git clone https://github.com/keepassex/keepassex
cd keepassex
pnpm install
cargo build
```

### Running

```bash
# Desktop (dev)
pnpm desktop

# Mobile (dev)
pnpm mobile

# CLI
pnpm cli -- --help

# Run all tests
cargo test --all
pnpm test
```

---

## 📋 Development Guidelines

### Code Style

- **Rust**: Follow `rustfmt` and `clippy` — run `cargo fmt && cargo clippy`
- **TypeScript**: Follow ESLint config — run `pnpm lint`
- **Swift**: Follow Swift API Design Guidelines
- **Kotlin**: Follow Kotlin coding conventions

### Security Rules (MANDATORY)

1. **Never log passwords, keys, or sensitive data**
2. **Always use `ProtectedString` for password fields**
3. **Always `zeroize` sensitive data on drop**
4. **Clipboard auto-clear must default to 10 seconds**
5. **No network calls with plaintext passwords**

### i18n Rules

- All user-facing strings MUST use i18n keys
- Never hardcode English or Vietnamese text in components
- Add both `en.ts` and `vi.ts` entries for every new string
- Key format: `section.subsection.key`

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(vault): add emergency access feature
fix(otp): correct HOTP counter increment
docs(readme): update build instructions
test(crypto): add AES-GCM tamper detection test
security(breach): implement k-anonymity HIBP check
i18n(vi): add missing Vietnamese translations
```

---

## 🔒 Security Vulnerabilities

**Do NOT open public issues for security vulnerabilities.**

Email: security@keepassex.app

We follow responsible disclosure. We'll respond within 48 hours.

---

## 📝 Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make your changes
4. Run tests: `cargo test --all && pnpm test`
5. Run linters: `cargo clippy && pnpm lint`
6. Commit with conventional commit message
7. Push and open a PR against `develop`

### PR Checklist

- [ ] Tests added/updated
- [ ] i18n keys added for both EN and VI
- [ ] No hardcoded strings
- [ ] Security rules followed
- [ ] Documentation updated if needed
- [ ] `cargo clippy` passes with no warnings
- [ ] `pnpm typecheck` passes

---

## 🏗️ Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture documentation.

Key principle: **All business logic lives in `packages/core` (Rust)**. Platform apps are thin shells.

---

## 📄 License

By contributing, you agree that your contributions will be licensed under the GPL-3.0 License.
