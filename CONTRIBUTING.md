# Contributing to KeePassEx

Thank you for helping make KeePassEx better. This guide covers the development workflow, commit conventions, and quality standards.

## Development Setup

```bash
make install      # Install all dependencies (Node + Rust)
make desktop      # Run desktop dev server
make mobile       # Start React Native metro bundler
make cli ARGS="--help"  # Run CLI
make test         # Run all tests
make check        # Lint + typecheck + fmt-check
```

## Commit Conventions

KeePassEx uses [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) for automated changelog generation and semantic versioning.

### Format

```
<type>(<scope>): <short summary>

[optional body]

[optional footer(s)]
```

### Types

| Type       | When to use                     | Version bump |
| ---------- | ------------------------------- | ------------ |
| `feat`     | New feature                     | minor        |
| `fix`      | Bug fix                         | patch        |
| `perf`     | Performance improvement         | patch        |
| `refactor` | Code change without feature/fix | patch        |
| `docs`     | Documentation only              | none         |
| `test`     | Adding or fixing tests          | none         |
| `chore`    | Build, CI, tooling changes      | none         |
| `style`    | Formatting, whitespace          | none         |
| `revert`   | Revert a previous commit        | patch        |

Append `!` after the type/scope for **breaking changes** (triggers a major bump):

```
feat(vault)!: change KDBX encryption to require hardware key
```

### Scopes

Use the package or app name as scope:

- `core` — `packages/core` (Rust)
- `ui` — `packages/ui`
- `i18n` — `packages/i18n`
- `desktop` — `apps/desktop`
- `mobile` — `apps/mobile`
- `cli` — `apps/cli`
- `tui` — `apps/tui`
- `extension` — `apps/browser-extension`
- `watch` — `apps/watch`
- `ci` — GitHub Actions workflows
- `deps` — dependency updates

### Examples

```
feat(desktop): add split-view preview pane to vault page
fix(vault): preserve vaultPath through lock/unlock cycles
perf(core): cache Argon2id parameters between vault opens
docs(i18n): document key naming convention
chore(ci): add aarch64-linux to CLI build matrix
feat(i18n)!: rename all keys from camelCase to dot.notation
```

## i18n Rules

All user-facing strings **must** use i18n keys. Never hardcode English or Vietnamese text in components.

1. Add the key to `packages/i18n/src/locales/en.ts`
2. Add the **same key** with Vietnamese translation to `packages/i18n/src/locales/vi.ts`
3. Use `useTranslation()` in React components: `const { t } = useTranslation()`
4. Key format: `section.subsection.key` (e.g., `entry.copyPassword`)

The i18n parity test will fail CI if EN and VI keys are out of sync.

## Security Rules

- **Never log passwords, keys, or sensitive data** — not even in debug builds
- **Always use `ProtectedString`** for password/key fields in Rust
- **Always `zeroize` sensitive data on drop**
- **Clipboard auto-clear must default to 10 seconds** — do not change this default
- **No network calls with plaintext passwords**

## Pull Request Checklist

- [ ] `make check` passes (lint + typecheck + fmt)
- [ ] `make test` passes
- [ ] New features include tests
- [ ] i18n keys added to both `en.ts` and `vi.ts`
- [ ] No hardcoded strings in UI components
- [ ] Security rules followed
- [ ] `CHANGELOG.md` updated under `[Unreleased]`
- [ ] Commit messages follow Conventional Commits

## Versioning

Releases are tagged as `vMAJOR.MINOR.PATCH` and pushed to trigger the release workflow:

```bash
make version VERSION=1.2.0   # Updates version in all manifests + creates git tag
```

The release workflow builds and publishes:

- Desktop installers (Windows MSI/NSIS, macOS DMG, Linux AppImage/deb/rpm)
- CLI binaries (Linux x64/arm64, macOS x64/arm64, Windows x64)
- Browser extensions (Chrome, Firefox)
