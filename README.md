# KeePassEx

> **Vượt trội hơn KeePass, KeePassXC, Keepassium, KeePass2Android**
> Password manager native đa nền tảng — offline-first, zero-trust, open source.

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.82+-orange.svg)](https://www.rust-lang.org)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.7-blue.svg)](https://www.typescriptlang.org)
[![Tests](https://img.shields.io/badge/Tests-626%20Rust%20%2B%20150%20TS-green.svg)](#)
[![Languages](https://img.shields.io/badge/Languages-10-purple.svg)](#-bản-địa-hóa)

---

## 🌟 Tại sao KeePassEx?

| Tính năng                                 | KeePass | KeePassXC | Keepassium | KeePass2Android | **KeePassEx** |
| ----------------------------------------- | ------- | --------- | ---------- | --------------- | ------------- |
| **Nền tảng**                              |         |           |            |                 |               |
| Native Windows                            | ✅      | ✅        | ❌         | ❌              | ✅            |
| Native macOS                              | ❌      | ✅        | ✅         | ❌              | ✅            |
| Native Linux                              | ❌      | ✅        | ❌         | ❌              | ✅            |
| Native iOS/iPadOS                         | ❌      | ❌        | ✅         | ❌              | ✅            |
| Native Android                            | ❌      | ❌        | ❌         | ✅              | ✅            |
| Native watchOS                            | ❌      | ❌        | ❌         | ❌              | ✅            |
| Native WearOS                             | ❌      | ❌        | ❌         | ❌              | ✅            |
| macOS Menu Bar App                        | ❌      | ❌        | ❌         | ❌              | ✅            |
| Windows Credential Provider               | ❌      | ❌        | ❌         | ❌              | ✅            |
| **Bảo mật**                               |         |           |            |                 |               |
| KDBX 4.x (Argon2id)                       | ✅      | ✅        | ✅         | ✅              | ✅            |
| YubiKey / Hardware Key                    | ✅      | ✅        | ❌         | ❌              | ✅            |
| Passkey (FIDO2/WebAuthn)                  | ❌      | ✅        | ❌         | ❌              | ✅            |
| SSH Agent tích hợp                        | ❌      | ✅        | ❌         | ❌              | ✅            |
| Biometric (Secure Enclave/StrongBox)      | ❌      | ❌        | ✅         | ✅              | ✅            |
| Screen capture protection                 | ❌      | ❌        | ✅         | ✅              | ✅            |
| Memory protection (zeroize)               | ✅      | ✅        | ✅         | ✅              | ✅            |
| Decoy vault (duress)                      | ✅      | ❌        | ❌         | ❌              | ✅            |
| Audit log                                 | ❌      | ❌        | ❌         | ❌              | ✅            |
| Quantum-Resistant Encryption              | ❌      | ❌        | ❌         | ❌              | ✅            |
| Zero-Knowledge Verification               | ❌      | ❌        | ❌         | ❌              | ✅            |
| **Tính năng nâng cao**                    |         |           |            |                 |               |
| Breach Monitor (HIBP)                     | ❌      | ✅        | ❌         | ❌              | ✅            |
| Emergency Access                          | ❌      | ❌        | ❌         | ❌              | ✅            |
| Plugin System (WASM)                      | ✅      | ❌        | ❌         | ❌              | ✅            |
| Entry Templates (12 types)                | ✅      | ❌        | ❌         | ❌              | ✅            |
| Password Policies                         | ❌      | ❌        | ❌         | ❌              | ✅            |
| Vault Comparison/Diff                     | ❌      | ❌        | ❌         | ❌              | ✅            |
| Scheduled Backup                          | ❌      | ❌        | ❌         | ❌              | ✅            |
| Steganography (PNG/JPEG/MP4)              | ❌      | ❌        | ❌         | ❌              | ✅            |
| Vault Key Sharding (Shamir)               | ❌      | ❌        | ❌         | ❌              | ✅            |
| Smart Categorizer                         | ❌      | ❌        | ❌         | ❌              | ✅            |
| Natural Language Search                   | ❌      | ❌        | ❌         | ❌              | ✅            |
| Password Rotation Engine                  | ❌      | ❌        | ❌         | ❌              | ✅            |
| Team Vault (RBAC)                         | ❌      | ❌        | ❌         | ❌              | ✅            |
| Vault Analytics Dashboard                 | ❌      | ❌        | ❌         | ❌              | ✅            |
| Entry Field References {REF:}             | ✅      | ✅        | ✅         | ✅              | ✅            |
| Favicon Auto-fetch                        | ❌      | ✅        | ✅         | ❌              | ✅            |
| Multi-Vault Tabs                          | ❌      | ✅        | ❌         | ❌              | ✅            |
| **Sync**                                  |         |           |            |                 |               |
| WebDAV / SFTP / S3                        | ❌      | ✅        | ✅         | ✅              | ✅            |
| Google Drive / OneDrive / Dropbox         | ❌      | ✅        | ✅         | ✅              | ✅            |
| **Self-hosted sync server**               | ❌      | ❌        | ❌         | ❌              | ✅            |
| **Tích hợp**                              |         |           |            |                 |               |
| Browser Extension                         | ❌      | ✅        | ❌         | ❌              | ✅            |
| iOS AutoFill                              | ❌      | ❌        | ✅         | ❌              | ✅            |
| Android AutoFill                          | ❌      | ❌        | ❌         | ✅              | ✅            |
| iOS Widget (WidgetKit)                    | ❌      | ❌        | ❌         | ❌              | ✅            |
| Android Widget                            | ❌      | ❌        | ❌         | ❌              | ✅            |
| Watch Complications + Tiles               | ❌      | ❌        | ❌         | ❌              | ✅            |
| **Bản địa hóa**                           |         |           |            |                 |               |
| Tiếng Anh                                 | ✅      | ✅        | ✅         | ✅              | ✅            |
| Tiếng Việt                                | ❌      | ❌        | ❌         | ❌              | ✅            |
| 8 ngôn ngữ khác (ZH/JA/KO/ES/FR/DE/PT/RU) | ❌      | ❌        | ❌         | ❌              | ✅            |

**KeePassEx là password manager duy nhất có tất cả các tính năng trên.**

---

## 🏗️ Kiến trúc Monorepo

```
keepassex/
├── apps/
│   ├── desktop/              # Tauri v2 — Windows, macOS, Linux
│   ├── mobile/               # React Native — iOS, Android
│   │   ├── ios/              # iOS native module + AutoFill + Widget
│   │   └── android/          # Android native module + AutoFill + Widget
│   ├── watch/
│   │   ├── watchos/          # SwiftUI native — Apple Watch
│   │   └── wearos/           # Jetpack Compose native — Wear OS
│   ├── browser-extension/    # Chrome/Firefox extension
│   ├── cli/                  # Rust CLI — kpx (18 commands)
│   ├── tui/                  # Rust TUI — ratatui + vim keybindings
│   ├── macos-menubar/        # SwiftUI menu bar app (⌘⇧K)
│   ├── windows-credprov/     # Windows Credential Provider DLL
│   └── server/               # Self-hosted sync server (Axum + SQLite)
├── packages/
│   ├── core/                 # Rust: crypto, KDBX engine, vault logic (29+ modules)
│   ├── ui/                   # React + Tamagui: shared design system
│   └── i18n/                 # 10 languages, 400+ keys each
└── shared/
    ├── types/                # TypeScript types dùng chung
    ├── constants/            # App-wide constants
    └── utils/                # Shared utilities
```

---

## 🔐 Bảo mật

- **KDBX 4.x** — tương thích ngược KDBX 3.1
- **Argon2id** memory-hard KDF (64 MB, 2 iterations) — mặc định
- **ChaCha20-Poly1305** AEAD encryption
- **HMAC-SHA256** per-block integrity (tamper detection)
- **Zero-knowledge** — master password không bao giờ rời thiết bị
- **Memory protection** — `ZeroizeOnDrop`, mlock, guard pages
- **Key file + Hardware key** (YubiKey HMAC-SHA1, FIDO2, Smart Card, OnlyKey)
- **Clipboard auto-clear** (mặc định 10 giây)
- **Screen capture protection** trên mobile
- **Biometric unlock** — iOS Secure Enclave / Android StrongBox (hardware-backed)
- **Decoy vault** — kho giả để tiết lộ khi bị ép buộc
- **Audit log** — theo dõi 24 loại sự kiện bảo mật
- **Quantum-resistant encryption** — X25519 + Kyber-768 hybrid (NIST FIPS 203)
- **ZKPV** — zero-knowledge password pre-check (fast unlock without full Argon2id)

Xem [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md) để biết chi tiết.

---

## 🚀 Tính năng nổi bật

### Core Engine (Rust — 29+ modules, 626 unit tests)

- Vault KDBX 4.x với nested groups không giới hạn
- Custom fields, attachments, history entries
- TOTP/HOTP tích hợp (RFC 6238/4226) — SHA-1/256/512
- Passkey store (FIDO2/WebAuthn) với assertion verification
- SSH Agent (thay thế ssh-agent) — Ed25519, RSA, ECDSA
- Password generator: random, passphrase (EFF/BIP-39), pronounceable
- Password health audit (độ mạnh, trùng lặp, hết hạn, cũ)
- **Password policy engine** — 4 built-in policies + custom rules
- Breach monitor (HIBP k-anonymity, offline mode)
- **Import: 12 formats** — Bitwarden, LastPass, Chrome, Firefox, 1Password, Dashlane, NordPass, Enpass, RoboForm, KeePass 1.x, CSV, KDBX
- Export: CSV, JSON, HTML
- Emergency access (trusted contact với waiting period)
- Plugin system (WASM sandbox)
- **Vault comparison** — UUID-based diff + CRDT-inspired merge
- **Scheduled backup** — OnSave/Daily/Weekly/Monthly + retention policy
- **Audit log** — 24 event types, ring-buffer, filter, export
- **Decoy vault** — 10 realistic fake entries
- **Entry field references** `{REF:P@I:uuid}` — KeePass-compatible
- **Favicon auto-fetch** — privacy-safe (domain only)
- **Quantum-resistant encryption** — X25519 + Kyber-768 hybrid
- **ZKPV** — fast password pre-check without full vault decryption

### KeePassEx Server (Self-hosted Sync) 🆕

> **Không đối thủ nào có tính năng này.** KeePass/KeePassXC không có server. Bitwarden có nhưng phức tạp.

```bash
# Chạy server với Docker (một lệnh)
docker compose -f apps/server/docker-compose.yml up -d

# Hoặc chạy trực tiếp
keepassex-server --port 8080 --db ./keepassex.db
```

- **Zero-knowledge**: server lưu chỉ encrypted blobs, không thể decrypt
- **End-to-end encrypted**: tất cả mã hóa ở phía client
- **Single binary**: SQLite, không cần external database
- **REST API**: register, login, upload/download vault, version history (10 versions)
- **WebSocket**: real-time sync notifications khi vault được cập nhật
- **Admin API**: quản lý users, xem stats (optional, key-protected)
- **Docker image** + docker-compose cho self-hosting dễ dàng
- **JWT authentication** với Argon2id password hashing
- **Vault version history**: khôi phục bất kỳ version nào trong 10 version gần nhất

### macOS Menu Bar App 🆕

> **Không đối thủ nào có tính năng này.**

- Luôn accessible từ menu bar — không cần mở app chính
- **Global shortcut**: ⌘⇧K
- Real-time entry search với debounce
- Recent entries list (5 entries gần nhất)
- One-click copy password/username/OTP với 10s auto-clear
- OTP countdown ring (green → red khi sắp hết)
- IPC với KeePassEx desktop app qua WebSocket

### Windows Credential Provider 🆕

> **Không đối thủ nào có tính năng này.**

- Unlock Windows login screen bằng KeePassEx vault master password
- ZKPV pre-check: xác minh password nhanh không cần full Argon2id
- Windows credentials lưu encrypted trong vault
- Cài đặt: `regsvr32 keepassex_credprov.dll`

### Sync & Sharing (9 providers)

- WebDAV, iCloud Drive, Google Drive, OneDrive, Dropbox, S3 (Signature V4), SFTP, Local
- **KeePassEx Server** (self-hosted, zero-knowledge) 🆕
- Conflict resolution thông minh (CRDT-inspired merge)

### Desktop (Tauri v2 — 20+ pages)

- Windows, macOS, Linux native
- System tray với lock/unlock
- Global shortcut (Ctrl+Alt+K)
- **Command palette** (Ctrl+K)
- **Entry preview pane** — split-view với live OTP
- **Multi-vault tabs** — mở nhiều vault đồng thời
- **SecurityPage** — toggle quantum-resistant encryption
- **Auto-lock** on idle / screen lock
- Auto-Type với placeholder engine
- Browser extension native messaging

### Mobile (React Native — 16 screens)

- iOS 15+ và Android 8.0+ (API 26+)
- **Biometric unlock** — iOS Secure Enclave / Android StrongBox (hardware-backed)
- iOS AutoFill Extension + Android AutoFill Service
- iOS WidgetKit + Android home screen widget
- Screen capture protection
- **10-language picker** trong Settings

### Watch Apps

- **watchOS**: SwiftUI + WatchConnectivity + Complications + Search + Favorites + Haptic
- **WearOS**: Jetpack Compose + Wearable Data Layer + Tiles + Rotary Input + Haptic

### Browser Extension

- Chrome/Edge (Manifest V3) + Firefox (Manifest V2)
- Form detection và credential filling
- OTP inline display với live countdown
- Password save detection
- Smart URL matching (exact/domain/subdomain)
- Native messaging với desktop app

### CLI (`kpx` — 18 commands)

```bash
kpx list / get / add / edit / delete / generate
kpx health / otp / export / import / sync / breach / stats
kpx template list/show
kpx hardware-key list/test/setup
kpx compare / audit
kpx steg embed/extract      # Steganography
kpx shard split/combine     # Vault key sharding
```

### TUI (`kpx-tui`)

- Full-featured terminal UI với vim keybindings (j/k/h/l//)
- Split panes: group tree | entry list | entry detail
- Command mode (`:`)

---

## 📊 Số liệu dự án

| Thành phần             | Số lượng                                                                |
| ---------------------- | ----------------------------------------------------------------------- |
| Core Rust modules      | 29+                                                                     |
| Core unit tests        | 626                                                                     |
| TypeScript tests       | 150+                                                                    |
| Desktop pages          | 20+                                                                     |
| Mobile screens         | 16                                                                      |
| CLI commands           | 18                                                                      |
| Import formats         | 12                                                                      |
| Sync providers         | 9 (bao gồm KeePassEx Server)                                            |
| Entry templates        | 12                                                                      |
| Languages              | **10** (EN, VI, ZH, JA, KO, ES, FR, DE, PT, RU)                         |
| i18n keys per language | 400+                                                                    |
| Platforms              | **9** (Desktop, Mobile, Watch×2, Browser, CLI, TUI, Menu Bar, CredProv) |
| Docs                   | 15                                                                      |

---

## 🌐 Bản địa hóa

KeePassEx hỗ trợ **10 ngôn ngữ** với parity đầy đủ (~400 keys mỗi ngôn ngữ):

| Flag | Language   | Code |
| ---- | ---------- | ---- |
| 🇺🇸   | English    | `en` |
| 🇻🇳   | Tiếng Việt | `vi` |
| 🇨🇳   | 简体中文   | `zh` |
| 🇯🇵   | 日本語     | `ja` |
| 🇰🇷   | 한국어     | `ko` |
| 🇪🇸   | Español    | `es` |
| 🇫🇷   | Français   | `fr` |
| 🇩🇪   | Deutsch    | `de` |
| 🇧🇷   | Português  | `pt` |
| 🇷🇺   | Русский    | `ru` |

Thêm ngôn ngữ mới: tạo `packages/i18n/src/locales/<code>.ts` và đăng ký trong `packages/i18n/src/index.ts`.

---

## 📦 Tech Stack

| Layer                       | Technology                                      |
| --------------------------- | ----------------------------------------------- |
| Core crypto/engine          | **Rust** (argon2, chacha20poly1305, sha2, hmac) |
| Desktop UI                  | **Tauri v2** + React + TypeScript               |
| Mobile                      | **React Native** (New Architecture)             |
| watchOS                     | **SwiftUI** native                              |
| WearOS                      | **Jetpack Compose** native                      |
| macOS Menu Bar              | **SwiftUI** native                              |
| Windows Credential Provider | **Rust** cdylib (COM interface)                 |
| Self-hosted Server          | **Rust** + Axum + SQLite                        |
| CLI                         | **Rust** (clap v4)                              |
| TUI                         | **Rust** (ratatui)                              |
| Browser Extension           | **Vite** + React + WebExtension Polyfill        |
| State management            | Zustand + TanStack Query                        |
| Build system                | **Turborepo** + Cargo workspace                 |
| i18n                        | i18next                                         |
| Testing                     | Vitest + cargo test                             |
| CI/CD                       | GitHub Actions                                  |

---

## 🛠️ Bắt đầu phát triển

```bash
# Clone
git clone https://github.com/keepassex/keepassex
cd keepassex

# Cài dependencies
make install

# Chạy desktop (dev)
make desktop

# Chạy mobile (dev)
make mobile

# Chạy server (dev)
cargo run -p keepassex-server -- --port 8080

# Chạy CLI
make cli ARGS="--help"

# Chạy tất cả tests
make test

# Xem tất cả lệnh
make help
```

### Self-hosted Server (Docker)

```bash
# Chạy với Docker Compose
docker compose -f apps/server/docker-compose.yml up -d

# Kiểm tra health
curl http://localhost:8080/health

# Xem logs
docker compose -f apps/server/docker-compose.yml logs -f
```

Xem [docs/BUILD.md](docs/BUILD.md) để biết hướng dẫn build chi tiết cho từng nền tảng.

---

## 📚 Tài liệu

| Tài liệu                                                 | Mô tả                       |
| -------------------------------------------------------- | --------------------------- |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)             | Kiến trúc hệ thống          |
| [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md)         | Mô hình bảo mật chi tiết    |
| [docs/API.md](docs/API.md)                               | Tauri command API reference |
| [docs/BUILD.md](docs/BUILD.md)                           | Hướng dẫn build             |
| [docs/SYNC.md](docs/SYNC.md)                             | Cấu hình sync               |
| [docs/IMPORT_EXPORT.md](docs/IMPORT_EXPORT.md)           | Import/Export formats       |
| [docs/ROADMAP_2026.md](docs/ROADMAP_2026.md)             | Roadmap 2026                |
| [docs/PLUGIN_DEVELOPMENT.md](docs/PLUGIN_DEVELOPMENT.md) | Plugin development guide    |
| [CHANGELOG.md](CHANGELOG.md)                             | Lịch sử thay đổi            |

---

## 🤝 Đóng góp

Xem [CONTRIBUTING.md](CONTRIBUTING.md) để biết hướng dẫn đóng góp.

Báo cáo lỗi bảo mật: security@keepassex.app (xem [SECURITY.md](SECURITY.md))

---

## 📄 License

GNU General Public License v3.0 — xem [LICENSE](LICENSE)

Copyright © 2025 KeePassEx Team
