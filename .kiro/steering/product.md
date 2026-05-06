# KeePassEx — Product Overview

KeePassEx is a native, cross-platform password manager built around the KDBX 4.x format, ensuring full interoperability with KeePass and KeePassXC. It is licensed under GPL-3.0.

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

## Key Features

- KDBX 4.x read/write, TOTP/HOTP, Passkeys, SSH Agent
- Breach monitoring (HIBP k-anonymity), Emergency Access
- Plugin system (WASM sandbox), Import/Export (Bitwarden, LastPass, 1Password, Chrome, Firefox, CSV)
- Sync via WebDAV or local, iOS/Android AutoFill and Widgets
- Steganography vault embedding, Shamir secret sharing
- **Entry Field References** `{REF:F@I:uuid}` — KeePass-compatible cross-entry field linking
- **Favicon auto-fetch** — automatic site icons (privacy-safe, domain-only)
- **Multi-vault tabs** — open multiple vaults simultaneously (unique vs all competitors)
- **Quantum-resistant encryption** — X25519 + CRYSTALS-Kyber-768 hybrid (NIST FIPS 203)
- **TUI** — full terminal UI with vim keybindings
- **ZKPV** — zero-knowledge password pre-check without decrypting vault
- **Smart categorizer** — auto-categorize entries by URL/title
- **Natural language search** — "find weak passwords in Banking"
- **Password rotation engine** — category-based rotation schedules

## Competitive Advantages vs KeePass / KeePassXC / Keepassium / KeePass2Android

| Feature                  | KeePass | KeePassXC | Keepassium | KeePass2Android | **KeePassEx** |
| ------------------------ | ------- | --------- | ---------- | --------------- | ------------- |
| Multi-vault tabs         | ❌      | ✅        | ❌         | ❌              | ✅            |
| Favicon auto-fetch       | ❌      | ✅        | ✅         | ❌              | ✅            |
| Entry field references   | ✅      | ✅        | ✅         | ✅              | ✅            |
| Quantum-resistant crypto | ❌      | ❌        | ❌         | ❌              | ✅            |
| Steganography            | ❌      | ❌        | ❌         | ❌              | ✅            |
| Shamir key sharding      | ❌      | ❌        | ❌         | ❌              | ✅            |
| TUI (terminal UI)        | ❌      | ❌        | ❌         | ❌              | ✅            |
| ZKPV                     | ❌      | ❌        | ❌         | ❌              | ✅            |
| Smart categorizer        | ❌      | ❌        | ❌         | ❌              | ✅            |
| Natural language search  | ❌      | ❌        | ❌         | ❌              | ✅            |
| WearOS native            | ❌      | ❌        | ❌         | ❌              | ✅            |
| watchOS native           | ❌      | ❌        | ✅         | ❌              | ✅            |
| EN + VI i18n             | ❌      | ❌        | ❌         | ❌              | ✅            |
