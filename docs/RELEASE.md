# KeePassEx — Release Guide

## Version Strategy: v0.1.0 → v1.0.0

```
v0.1.0  ──► v0.2.0  ──► v0.3.0  ──► v0.9.0  ──► v1.0.0
  │            │            │            │            │
  │            │            │            │            └─ Production release
  │            │            │            └─ Release candidate
  │            │            └─ Feature complete
  │            └─ Safari ext, GTK4, memory encryption
  └─ MVP: all features buildable, 865 tests pass
```

---

## Current: v0.1.0 (MVP)

**Status**: ✅ Buildable, all tests pass

**What works:**

- ✅ `cargo check --workspace` — 0 errors
- ✅ `cargo test -p keepassex-core --lib` — 709/709 pass
- ✅ `pnpm test:ts` — 156/156 pass
- ✅ Desktop app (Tauri v2) — compiles, ready for `tauri dev`
- ✅ CLI (`kpx`) — compiles, all 18 commands
- ✅ TUI (`kpx-tui`) — compiles
- ✅ Server — compiles, Docker-ready
- ✅ Browser extension — compiles
- ✅ Windows Credential Provider — compiles (Windows only)
- ✅ 10 languages, 1082 keys each, 100% parity

**What needs testing before v0.2.0:**

- [ ] Open/create/save KDBX vault end-to-end
- [ ] Browser extension autofill flow
- [ ] Sync with WebDAV/local folder
- [ ] OTP generation
- [ ] Import from Bitwarden/Chrome CSV

---

## How to Release

### 1. Create a release tag

```bash
# Bump version in all files
# Cargo.toml workspace version
# package.json
# apps/desktop/src-tauri/tauri.conf.json

git add -A
git commit -m "chore(release): v0.2.0"
git tag v0.2.0
git push origin main --tags
```

### 2. GitHub Actions auto-builds

When a `v*` tag is pushed, `.github/workflows/release.yml` automatically:

1. Creates a GitHub Release with changelog notes
2. Builds desktop app for Windows, macOS, Linux
3. Builds CLI binaries for all platforms
4. Builds browser extensions (Chrome + Firefox)
5. Uploads all artifacts to the release

### 3. Auto-update flow

Desktop app checks for updates via Tauri updater:

```
App startup → GET https://releases.keepassex.app/{target}/{arch}/latest.json
           → Compare version
           → If newer: show update notification
           → User clicks "Update" → download + install
```

To enable auto-update, set in `tauri.conf.json`:

```json
"plugins": {
  "updater": {
    "pubkey": "YOUR_TAURI_SIGNING_PUBLIC_KEY",
    "endpoints": ["https://releases.keepassex.app/{{target}}/{{arch}}/latest.json"]
  }
}
```

Generate signing keys:

```bash
pnpm tauri signer generate -w ~/.tauri/keepassex.key
# Add public key to tauri.conf.json
# Add private key to GitHub Secrets as TAURI_SIGNING_PRIVATE_KEY
```

---

## Version Checklist

### Before each release:

- [ ] `cargo check --workspace` — 0 errors
- [ ] `cargo test -p keepassex-core --lib` — all pass
- [ ] `pnpm test:ts` — all pass
- [ ] `pnpm typecheck` — 0 errors
- [ ] `pnpm lint` — 0 errors
- [ ] `cargo clippy --workspace -- -D warnings` — 0 warnings
- [ ] Update `CHANGELOG.md` with release notes
- [ ] Update version in `Cargo.toml`, `package.json`, `tauri.conf.json`
- [ ] `python scripts/gen_locales.py` — sync i18n if en.ts changed
- [ ] Test desktop app manually on target platform

### v0.2.0 checklist (additional):

- [ ] End-to-end vault open/save test
- [ ] Browser extension autofill test
- [ ] Sync test (WebDAV)
- [ ] OTP test
- [ ] Import test (Bitwarden JSON)

### v1.0.0 checklist (additional):

- [ ] Code signing (Windows: EV cert, macOS: Apple Developer ID)
- [ ] Notarization (macOS)
- [ ] Windows installer (NSIS/MSI)
- [ ] macOS DMG with background
- [ ] Linux AppImage + .deb + .rpm
- [ ] Auto-update server deployed
- [ ] Security audit (cargo audit + pnpm audit)
- [ ] Performance benchmarks
- [ ] Documentation review

---

## Versioning Rules

Following [Semantic Versioning](https://semver.org/):

| Change                            | Version bump | Example       |
| --------------------------------- | ------------ | ------------- |
| Bug fix                           | patch        | 0.1.0 → 0.1.1 |
| New feature (backward compatible) | minor        | 0.1.0 → 0.2.0 |
| Breaking change                   | major        | 0.x.x → 1.0.0 |

During v0.x.x phase:

- Minor bumps (0.1 → 0.2) = significant new features or fixes
- Patch bumps (0.1.0 → 0.1.1) = bug fixes only
- v1.0.0 = production-ready, signed, auto-update working

---

## Commit Convention

```
feat(scope): add new feature
fix(scope): fix a bug
chore(scope): tooling/CI changes
docs(scope): documentation only
refactor(scope): code refactoring
test(scope): add/fix tests
perf(scope): performance improvement
```

Examples:

```
feat(desktop): add AI password suggestions panel
fix(core): fix circular reference detection in field_references
chore(ci): add Windows build to release workflow
docs(api): update Tauri command reference
```

---

## Support Matrix

| Platform      | v0.1.0        | v0.2.0  | v1.0.0     |
| ------------- | ------------- | ------- | ---------- |
| Windows 10/11 | ✅ Build      | ✅ Test | ✅ Release |
| macOS 12+     | ✅ Build      | ✅ Test | ✅ Release |
| Ubuntu 22.04+ | ✅ Build      | ✅ Test | ✅ Release |
| iOS 15+       | 🔧 Code ready | ✅ Test | ✅ Release |
| Android 8+    | 🔧 Code ready | ✅ Test | ✅ Release |
| Chrome/Edge   | ✅ Build      | ✅ Test | ✅ Release |
| Firefox       | ✅ Build      | ✅ Test | ✅ Release |
| watchOS       | 🔧 Code ready | ✅ Test | ✅ Release |
| WearOS        | 🔧 Code ready | ✅ Test | ✅ Release |
