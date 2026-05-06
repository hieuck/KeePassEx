# KeePassEx — Product Overview

KeePassEx is a native, cross-platform password manager built around the KDBX 4.x format, ensuring full interoperability with KeePass and KeePassXC. Licensed under GPL-3.0.

## Core Values

- **Security first** — Argon2id KDF, ChaCha20-Poly1305 cipher, zero-knowledge architecture
- **Offline first** — fully functional without internet; sync is optional
- **Native everywhere** — no Electron, no web wrappers on mobile
- **Privacy** — no telemetry, no cloud accounts required

## Platforms

| Platform                  | Stack                     |
| ------------------------- | ------------------------- |
| Desktop (Win/macOS/Linux) | Tauri v2 + React          |
| Mobile (iOS/Android)      | React Native              |
| Watch (watchOS/WearOS)    | SwiftUI + Jetpack Compose |
| Browser extension         | Chrome MV3 + Firefox MV2  |
| CLI                       | Rust (`kpx`)              |
| TUI                       | Rust (ratatui)            |

## Feature Scope

All features are implemented. When adding code, assume these exist and work:

- KDBX 4.x read/write, TOTP/HOTP, Passkeys, SSH Agent
- Breach monitoring (HIBP k-anonymity), Emergency Access
- Plugin system (WASM sandbox)
- Import: Bitwarden, LastPass, 1Password, Chrome, Firefox, Dashlane, NordPass, Enpass, RoboForm, CSV
- Export: KDBX, CSV, JSON, HTML
- Sync: WebDAV, Google Drive, OneDrive, Dropbox, S3, SFTP, iCloud, Local, **KeePassEx Server** (self-hosted, zero-knowledge)
- iOS/Android AutoFill and Widgets
- Steganography (vault in PNG/JPEG/MP4), Shamir secret sharing (M-of-N)
- Entry field references `{REF:F@I:uuid}` — KeePass-compatible
- Favicon auto-fetch — privacy-safe (domain only, never full URL)
- Multi-vault tabs — multiple vaults open simultaneously
- Quantum-resistant encryption — X25519 + Kyber-768 hybrid (KDBX header integrated, migration tool)
- Mobile Secure Enclave (iOS) + StrongBox (Android) — hardware-backed biometric key storage
- TUI with vim keybindings, ZKPV, smart categorizer, natural language search
- Password rotation engine, context-aware password advisor

## Languages

10 languages fully implemented with ~400 keys each, all in parity:
`en` (English), `vi` (Tiếng Việt), `zh` (简体中文), `ja` (日本語), `ko` (한국어), `es` (Español), `fr` (Français), `de` (Deutsch), `pt` (Português), `ru` (Русский)

## New Platforms (beyond original scope)

| Platform                    | Stack                | Status                      |
| --------------------------- | -------------------- | --------------------------- |
| macOS Menu Bar              | SwiftUI              | ✅ `apps/macos-menubar/`    |
| Windows Credential Provider | Rust cdylib          | ✅ `apps/windows-credprov/` |
| KeePassEx Server            | Rust + Axum + SQLite | ✅ `apps/server/`           |
