# KeePassEx

> **Vượt trội hơn KeePass, KeePassXC, Keepassium, KeePass2Android**
> Password manager native đa nền tảng — offline-first, zero-trust, open source.

---

## 🌟 Tại sao KeePassEx?

| Tính năng                      | KeePass | KeePassXC | Keepassium | KeePass2Android | **KeePassEx** |
| ------------------------------ | ------- | --------- | ---------- | --------------- | ------------- |
| **Nền tảng**                   |         |           |            |                 |               |
| Native Windows                 | ✅      | ✅        | ❌         | ❌              | ✅            |
| Native macOS                   | ❌      | ✅        | ✅         | ❌              | ✅            |
| Native Linux                   | ❌      | ✅        | ❌         | ❌              | ✅            |
| Native iOS/iPadOS              | ❌      | ❌        | ✅         | ❌              | ✅            |
| Native Android                 | ❌      | ❌        | ❌         | ✅              | ✅            |
| Native watchOS                 | ❌      | ❌        | ❌         | ❌              | ✅            |
| Native WearOS                  | ❌      | ❌        | ❌         | ❌              | ✅            |
| **Bảo mật**                    |         |           |            |                 |               |
| KDBX 4.x (Argon2id)            | ✅      | ✅        | ✅         | ✅              | ✅            |
| YubiKey / Hardware Key         | ✅      | ✅        | ❌         | ❌              | ✅            |
| Passkey (FIDO2/WebAuthn)       | ❌      | ✅        | ❌         | ❌              | ✅            |
| SSH Agent tích hợp             | ❌      | ✅        | ❌         | ❌              | ✅            |
| Biometric unlock               | ❌      | ❌        | ✅         | ✅              | ✅            |
| Screen capture protection      | ❌      | ❌        | ✅         | ✅              | ✅            |
| Memory protection (zeroize)    | ✅      | ✅        | ✅         | ✅              | ✅            |
| Decoy vault (duress)           | ✅      | ❌        | ❌         | ❌              | ✅            |
| Audit log                      | ❌      | ❌        | ❌         | ❌              | ✅            |
| **Tính năng nâng cao**         |         |           |            |                 |               |
| Breach Monitor (HIBP)          | ❌      | ✅        | ❌         | ❌              | ✅            |
| Emergency Access               | ❌      | ❌        | ❌         | ❌              | ✅            |
| Plugin System (WASM)           | ✅      | ❌        | ❌         | ❌              | ✅            |
| Entry Templates (12 types)     | ✅      | ❌        | ❌         | ❌              | ✅            |
| Password Policies              | ❌      | ❌        | ❌         | ❌              | ✅            |
| Vault Statistics               | ✅      | ✅        | ❌         | ❌              | ✅            |
| Vault Comparison/Diff          | ❌      | ❌        | ❌         | ❌              | ✅            |
| Scheduled Backup               | ❌      | ❌        | ❌         | ❌              | ✅            |
| **Tích hợp**                   |         |           |            |                 |               |
| Browser Extension              | ❌      | ✅        | ❌         | ❌              | ✅            |
| iOS AutoFill                   | ❌      | ❌        | ✅         | ❌              | ✅            |
| Android AutoFill               | ❌      | ❌        | ❌         | ✅              | ✅            |
| iOS Widget (WidgetKit)         | ❌      | ❌        | ❌         | ❌              | ✅            |
| Android Widget                 | ❌      | ❌        | ❌         | ❌              | ✅            |
| Watch Complications            | ❌      | ❌        | ❌         | ❌              | ✅            |
| **UX**                         |         |           |            |                 |               |
| Command Palette                | ❌      | ❌        | ❌         | ❌              | ✅            |
| Entry Preview Pane             | ❌      | ❌        | ❌         | ❌              | ✅            |
| Dark/Light/OLED themes         | ❌      | ✅        | ✅         | ✅              | ✅            |
| Notifications (expiry, breach) | ❌      | ❌        | ✅         | ❌              | ✅            |
| Import (10+ formats)           | ✅      | ✅        | ✅         | ✅              | ✅            |
| Export (CSV, JSON, HTML)       | ✅      | ✅        | ✅         | ✅              | ✅            |
| Sync (8 providers)             | ❌      | ✅        | ✅         | ✅              | ✅            |
| CLI đầy đủ (18 commands)       | ❌      | ✅        | ❌         | ❌              | ✅            |
| **Bản địa hóa**                |         |           |            |                 |               |
| Tiếng Anh                      | ✅      | ✅        | ✅         | ✅              | ✅            |
| Tiếng Việt                     | ❌      | ❌        | ❌         | ❌              | ✅            |
| Kiến trúc i18n mở rộng         | ❌      | ❌        | ❌         | ❌              | ✅            |

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
│   ├── browser-extension/    # Chrome/Firefox/Edge/Safari extension
│   └── cli/                  # Rust CLI — kpx (18 commands)
├── packages/
│   ├── core/                 # Rust: crypto, KDBX engine, vault logic (25+ modules)
│   ├── ui/                   # React + Tamagui: shared design system
│   └── i18n/                 # Bản địa hóa EN/VI (900+ keys)
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
- **Biometric unlock** — Face ID, Touch ID, Fingerprint, Face Unlock
- **Decoy vault** — kho giả để tiết lộ khi bị ép buộc
- **Audit log** — theo dõi 24 loại sự kiện bảo mật

Xem [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md) để biết chi tiết.

---

## 🚀 Tính năng nổi bật

### Core Engine (Rust — 25+ modules, 200+ unit tests)

- Vault KDBX 4.x với nested groups không giới hạn
- Custom fields, attachments, history entries
- TOTP/HOTP tích hợp (RFC 6238/4226) — SHA-1/256/512
- Passkey store (FIDO2/WebAuthn) với assertion verification
- SSH Agent (thay thế ssh-agent) — Ed25519, RSA, ECDSA
- Password generator: random, passphrase (EFF/BIP-39), pronounceable
- Password health audit (độ mạnh, trùng lặp, hết hạn, cũ)
- **Password policy engine** — 4 built-in policies + custom rules
- Breach monitor (HIBP k-anonymity, offline mode)
- **Import: 10+ formats** — Bitwarden, LastPass, Chrome, Firefox, 1Password, Dashlane, NordPass, Enpass, RoboForm, CSV
- Export: CSV, JSON, HTML
- Emergency access (trusted contact với waiting period)
- Plugin system (WASM sandbox)
- **Vault comparison** — UUID-based diff + CRDT-inspired merge
- **Scheduled backup** — OnSave/Daily/Weekly/Monthly + retention policy
- **Audit log** — 24 event types, ring-buffer, filter, export
- **Decoy vault** — 10 realistic fake entries

### Sync & Sharing (8 providers)

- WebDAV, iCloud Drive, Google Drive, OneDrive, Dropbox, S3 (Signature V4), SFTP, Local
- Conflict resolution thông minh (CRDT-inspired merge)
- Emergency access E2EE với người dùng khác

### Desktop (Tauri v2 — 18 pages)

- Windows, macOS, Linux native
- System tray với lock/unlock
- Global shortcut (Ctrl+Alt+K)
- **Command palette** (Ctrl+K) — quick access to all features
- **Entry preview pane** — split-view với live OTP, copy fields
- **Filter chips** — All / Favorites / OTP / Expiring / No Password
- **Auto-lock** on idle / screen lock
- Clipboard auto-clear
- Auto-Type với placeholder engine
- Browser extension native messaging
- **Audit Log page** — filter, search, export
- **Backup page** — configure, backup now, restore, history
- **Vault Compare page** — diff + merge với 4 strategies
- **Password Policy page** — enable/disable, test passwords

### Mobile (React Native — 14 screens)

- iOS 15+ và Android 8.0+ (API 26+)
- Biometric unlock (Face ID, Touch ID, Fingerprint)
- iOS AutoFill Extension
- Android AutoFill Service
- Android Quick Settings tile
- **iOS WidgetKit** (small, medium, large, lock screen)
- **Android home screen widget**
- Screen capture protection
- **Swipe actions** — copy/favorite/edit/delete
- **Filter chips** — All / Favorites / OTP / Expiring / No Password
- **Multi-select mode** — long press, bulk delete
- **Language switcher** — EN/VI trong Settings

### Watch Apps

- **watchOS**: SwiftUI + WatchConnectivity + Complications + Search + Favorites + Haptic
- **WearOS**: Jetpack Compose + Wearable Data Layer + Tiles + Rotary Input + Haptic

### Browser Extension

- Chrome/Edge (Manifest V3) + Firefox (Manifest V2)
- Form detection và credential filling
- **OTP inline display** với live countdown
- **Password save detection** — offer to save new credentials
- **Smart URL matching** — exact/domain/subdomain
- **Keyboard navigation** — Arrow keys + Enter + Escape
- **Dark mode** + i18n EN/VI
- Native messaging với desktop app

### CLI (`kpx` — 18 commands)

- `list`, `get`, `add`, `edit`, `delete`, `generate`
- `health`, `otp`, `export`, `import`, `sync`, `breach`, `stats`
- `template list/show` — entry templates
- `hardware-key list/test/setup` — hardware key management
- `compare` — diff two vaults
- `audit` — show audit log

---

## 📊 Số liệu dự án

| Thành phần          | Số lượng |
| ------------------- | -------- |
| Core Rust modules   | 25+      |
| Core unit tests     | 200+     |
| Desktop pages       | 18       |
| Mobile screens      | 14       |
| CLI commands        | 18       |
| Import formats      | 10+      |
| Sync providers      | 8        |
| Entry templates     | 12       |
| i18n keys (EN + VI) | 900+     |
| UI components       | 15+      |
| Docs                | 12       |

---

## 🌐 Bản địa hóa

- 🇺🇸 **English** (mặc định) — 900+ keys
- 🇻🇳 **Tiếng Việt** — parity đầy đủ với English
- Kiến trúc i18n mở rộng dễ dàng thêm ngôn ngữ mới

---

## 📦 Tech Stack

| Layer              | Technology                                      |
| ------------------ | ----------------------------------------------- |
| Core crypto/engine | **Rust** (argon2, chacha20poly1305, sha2, hmac) |
| Desktop UI         | **Tauri v2** + React + TypeScript               |
| Mobile             | **React Native** (New Architecture)             |
| watchOS            | **SwiftUI** native                              |
| WearOS             | **Jetpack Compose** native                      |
| CLI                | **Rust** (clap v4)                              |
| Browser Extension  | **Vite** + React + WebExtension Polyfill        |
| State management   | Zustand + TanStack Query                        |
| Build system       | **Turborepo** + Cargo workspace                 |
| i18n               | i18next                                         |
| Testing            | Vitest + cargo test                             |
| CI/CD              | GitHub Actions                                  |

---

## 🛠️ Bắt đầu phát triển

```bash
# Clone
git clone https://github.com/keepassex/keepassex
cd keepassex

# Cài dependencies
pnpm install

# Build core Rust
cargo build -p keepassex-core

# Chạy desktop (dev)
make desktop

# Chạy mobile (dev)
make mobile

# Chạy CLI
make cli ARGS="--help"

# Chạy tất cả tests
make test

# Xem tất cả lệnh
make help
```

Xem [docs/BUILD.md](docs/BUILD.md) để biết hướng dẫn build chi tiết cho từng nền tảng.

---

## 📚 Tài liệu

| Tài liệu                                                 | Mô tả                        |
| -------------------------------------------------------- | ---------------------------- |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)             | Kiến trúc hệ thống           |
| [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md)         | Mô hình bảo mật chi tiết     |
| [docs/API.md](docs/API.md)                               | Tauri command API reference  |
| [docs/BUILD.md](docs/BUILD.md)                           | Hướng dẫn build              |
| [docs/SYNC.md](docs/SYNC.md)                             | Cấu hình sync                |
| [docs/IMPORT_EXPORT.md](docs/IMPORT_EXPORT.md)           | Import/Export formats        |
| [docs/ROADMAP.md](docs/ROADMAP.md)                       | Roadmap & feature comparison |
| [docs/PLUGIN_DEVELOPMENT.md](docs/PLUGIN_DEVELOPMENT.md) | Plugin development guide     |

---

## 🤝 Đóng góp

Xem [CONTRIBUTING.md](CONTRIBUTING.md) để biết hướng dẫn đóng góp.

Báo cáo lỗi bảo mật: security@keepassex.app (xem [SECURITY.md](SECURITY.md))

---

## 📄 License

GNU General Public License v3.0 — xem [LICENSE](LICENSE)

Copyright © 2025 KeePassEx Team

---

## 🌟 Tại sao KeePassEx?

| Tính năng                      | KeePass | KeePassXC | Keepassium | KeePass2Android | **KeePassEx** |
| ------------------------------ | ------- | --------- | ---------- | --------------- | ------------- |
| **Nền tảng**                   |         |           |            |                 |               |
| Native Windows                 | ✅      | ✅        | ❌         | ❌              | ✅            |
| Native macOS                   | ❌      | ✅        | ✅         | ❌              | ✅            |
| Native Linux                   | ❌      | ✅        | ❌         | ❌              | ✅            |
| Native iOS/iPadOS              | ❌      | ❌        | ✅         | ❌              | ✅            |
| Native Android                 | ❌      | ❌        | ❌         | ✅              | ✅            |
| Native watchOS                 | ❌      | ❌        | ❌         | ❌              | ✅            |
| Native WearOS                  | ❌      | ❌        | ❌         | ❌              | ✅            |
| **Bảo mật**                    |         |           |            |                 |               |
| KDBX 4.x (Argon2id)            | ✅      | ✅        | ✅         | ✅              | ✅            |
| YubiKey / Hardware Key         | ✅      | ✅        | ❌         | ❌              | ✅            |
| Passkey (FIDO2/WebAuthn)       | ❌      | ✅        | ❌         | ❌              | ✅            |
| SSH Agent tích hợp             | ❌      | ✅        | ❌         | ❌              | ✅            |
| Biometric unlock               | ❌      | ❌        | ✅         | ✅              | ✅            |
| Screen capture protection      | ❌      | ❌        | ✅         | ✅              | ✅            |
| Memory protection (zeroize)    | ✅      | ✅        | ✅         | ✅              | ✅            |
| **Tính năng nâng cao**         |         |           |            |                 |               |
| Breach Monitor (HIBP)          | ❌      | ✅        | ❌         | ❌              | ✅            |
| Emergency Access               | ❌      | ❌        | ❌         | ❌              | ✅            |
| Plugin System (WASM)           | ✅      | ❌        | ❌         | ❌              | ✅            |
| Entry Templates                | ✅      | ❌        | ❌         | ❌              | ✅            |
| Vault Statistics               | ✅      | ✅        | ❌         | ❌              | ✅            |
| Password Audit Rules           | ❌      | ❌        | ❌         | ❌              | ✅            |
| Decoy Vault (duress)           | ✅      | ❌        | ❌         | ❌              | ✅            |
| Vault Comparison/Diff          | ❌      | ❌        | ❌         | ❌              | ✅            |
| Audit Log                      | ❌      | ❌        | ❌         | ❌              | ✅            |
| Scheduled Backup               | ❌      | ❌        | ❌         | ❌              | ✅            |
| **Tích hợp**                   |         |           |            |                 |               |
| Browser Extension              | ❌      | ✅        | ❌         | ❌              | ✅            |
| iOS AutoFill                   | ❌      | ❌        | ✅         | ❌              | ✅            |
| Android AutoFill               | ❌      | ❌        | ❌         | ✅              | ✅            |
| iOS Widget (WidgetKit)         | ❌      | ❌        | ❌         | ❌              | ✅            |
| Android Widget                 | ❌      | ❌        | ❌         | ❌              | ✅            |
| Watch Complications            | ❌      | ❌        | ❌         | ❌              | ✅            |
| **UX**                         |         |           |            |                 |               |
| Command Palette                | ❌      | ❌        | ❌         | ❌              | ✅            |
| Dark/Light/OLED themes         | ❌      | ✅        | ✅         | ✅              | ✅            |
| Notifications (expiry, breach) | ❌      | ❌        | ✅         | ❌              | ✅            |
| Import (7+ formats)            | ✅      | ✅        | ✅         | ✅              | ✅            |
| Export (CSV, JSON, HTML)       | ✅      | ✅        | ✅         | ✅              | ✅            |
| Sync (8 providers)             | ❌      | ✅        | ✅         | ✅              | ✅            |
| CLI đầy đủ                     | ❌      | ✅        | ❌         | ❌              | ✅            |
| **Bản địa hóa**                |         |           |            |                 |               |
| Tiếng Anh                      | ✅      | ✅        | ✅         | ✅              | ✅            |
| Tiếng Việt                     | ❌      | ❌        | ❌         | ❌              | ✅            |
| Kiến trúc i18n mở rộng         | ❌      | ❌        | ❌         | ❌              | ✅            |

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
│   ├── browser-extension/    # Chrome/Firefox/Edge/Safari extension
│   └── cli/                  # Rust CLI — kpx
├── packages/
│   ├── core/                 # Rust: crypto, KDBX engine, vault logic
│   ├── ui/                   # React + Tamagui: shared design system
│   └── i18n/                 # Bản địa hóa EN/VI
└── shared/
    ├── types/                # TypeScript types dùng chung
    ├── constants/            # App-wide constants
    └── utils/                # Shared utilities
```

---

## 🔐 Bảo mật

- **KDBX 4.x** — tương thích ngược KDBX 3.1, KDB
- **Argon2id** memory-hard KDF (64 MB, 2 iterations) — mặc định
- **ChaCha20-Poly1305** AEAD encryption
- **HMAC-SHA256** per-block integrity (tamper detection)
- **Zero-knowledge** — master password không bao giờ rời thiết bị
- **Memory protection** — `ZeroizeOnDrop`, mlock, guard pages
- **Key file + Hardware key** (YubiKey, FIDO2)
- **Clipboard auto-clear** (mặc định 10 giây)
- **Screen capture protection** trên mobile
- **Biometric unlock** — Face ID, Touch ID, Fingerprint, Face Unlock

Xem [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md) để biết chi tiết.

---

## 🚀 Tính năng nổi bật

### Core (Rust)

- Vault KDBX 4.x với nested groups không giới hạn
- Custom fields, attachments, history entries
- TOTP/HOTP tích hợp (RFC 6238/4226) — SHA-1/256/512
- Passkey store (FIDO2/WebAuthn) với assertion verification
- SSH Agent (thay thế ssh-agent) — Ed25519, RSA, ECDSA
- Password generator: random, passphrase (EFF), pronounceable
- Password health audit (độ mạnh, trùng lặp, hết hạn, cũ)
- Breach monitor (HIBP k-anonymity, offline mode)
- Import: Bitwarden, LastPass, Chrome, Firefox, 1Password, CSV
- Export: CSV, JSON
- Emergency access (trusted contact với waiting period)
- Plugin system (WASM sandbox)

### Sync & Sharing

- Sync đa chiều: WebDAV, iCloud Drive, Google Drive, OneDrive, Dropbox, S3, SFTP
- Conflict resolution thông minh (CRDT-inspired merge)
- Emergency access E2EE với người dùng khác

### Desktop (Tauri v2)

- Windows, macOS, Linux native
- System tray với lock/unlock
- Global shortcut (Ctrl+Alt+K)
- **Command palette** (Ctrl+K) — quick access to all features
- **Auto-lock** on idle / screen lock
- Clipboard auto-clear
- Auto-Type với placeholder engine
- Browser extension native messaging
- All pages: Vault, Health, Breach, Generator, Import/Export, Sync, Emergency Access, Plugins, Settings

### Mobile (React Native)

- iOS 15+ và Android 8.0+ (API 26+)
- Biometric unlock (Face ID, Touch ID, Fingerprint)
- iOS AutoFill Extension
- Android AutoFill Service
- Android Quick Settings tile
- **iOS WidgetKit** (small, medium, large, lock screen)
- **Android home screen widget**
- Screen capture protection

### Watch Apps

- **watchOS**: SwiftUI + WatchConnectivity + Complications
- **WearOS**: Jetpack Compose + Wearable Data Layer + Tiles + Complications

### Browser Extension

- Chrome/Edge (Manifest V3) + Firefox (Manifest V2)
- Form detection và credential filling
- Fill picker UI
- Context menu + keyboard shortcut
- Native messaging với desktop app

### CLI (`kpx`)

- `list`, `get`, `add`, `edit`, `delete`
- `generate` — password/passphrase
- `health` — vault health report
- `otp` — TOTP với countdown
- `export`, `import`
- `sync` — local/WebDAV
- `breach` — HIBP check
- `stats` — vault statistics

---

## 🌐 Bản địa hóa

- 🇺🇸 **English** (mặc định) — 400+ keys
- 🇻🇳 **Tiếng Việt** — parity đầy đủ với English
- Kiến trúc i18n mở rộng dễ dàng thêm ngôn ngữ mới

---

## 📦 Tech Stack

| Layer              | Technology                                      |
| ------------------ | ----------------------------------------------- |
| Core crypto/engine | **Rust** (argon2, chacha20poly1305, sha2, hmac) |
| Desktop UI         | **Tauri v2** + React + TypeScript               |
| Mobile             | **React Native** (New Architecture)             |
| watchOS            | **SwiftUI** native                              |
| WearOS             | **Jetpack Compose** native                      |
| CLI                | **Rust** (clap v4)                              |
| Browser Extension  | **Vite** + React + WebExtension Polyfill        |
| State management   | Zustand + TanStack Query                        |
| Build system       | **Turborepo** + Cargo workspace                 |
| i18n               | i18next                                         |
| Testing            | Vitest + cargo test                             |
| CI/CD              | GitHub Actions                                  |

---

## 🛠️ Bắt đầu phát triển

```bash
# Clone
git clone https://github.com/keepassex/keepassex
cd keepassex

# Cài dependencies
pnpm install

# Build core Rust
cargo build -p keepassex-core

# Chạy desktop (dev)
make desktop
# hoặc: pnpm --filter @keepassex/desktop tauri:dev

# Chạy mobile (dev)
make mobile
# hoặc: pnpm --filter @keepassex/mobile start

# Chạy CLI
make cli ARGS="--help"
# hoặc: cargo run -p keepassex-cli -- --help

# Chạy tất cả tests
make test

# Xem tất cả lệnh
make help
```

Xem [docs/BUILD.md](docs/BUILD.md) để biết hướng dẫn build chi tiết cho từng nền tảng.

---

## 📚 Tài liệu

| Tài liệu                                         | Mô tả                       |
| ------------------------------------------------ | --------------------------- |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)     | Kiến trúc hệ thống          |
| [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md) | Mô hình bảo mật chi tiết    |
| [docs/API.md](docs/API.md)                       | Tauri command API reference |
| [docs/BUILD.md](docs/BUILD.md)                   | Hướng dẫn build             |
| [docs/SYNC.md](docs/SYNC.md)                     | Cấu hình sync               |
| [docs/IMPORT_EXPORT.md](docs/IMPORT_EXPORT.md)   | Import/Export formats       |

---

## 🤝 Đóng góp

Xem [CONTRIBUTING.md](CONTRIBUTING.md) để biết hướng dẫn đóng góp.

Báo cáo lỗi bảo mật: security@keepassex.app (xem [SECURITY.md](SECURITY.md))

---

## 📄 License

GNU General Public License v3.0 — xem [LICENSE](LICENSE)

Copyright © 2025 KeePassEx Team
