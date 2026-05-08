# KeePassEx — Native Platform Extensions

Tài liệu này mô tả các nền tảng native mở rộng không có trong bất kỳ đối thủ nào.

---

## macOS Menu Bar App

**Location**: `apps/macos-menubar/KeePassExMenuBar/`
**Stack**: SwiftUI + AppKit
**Competitor gap**: ❌ KeePass, KeePassXC, Keepassium đều không có

### Tính năng

- Luôn accessible từ menu bar — không cần mở app chính
- Global shortcut: **⌘⇧K**
- Real-time entry search với debounce 200ms
- Recent entries list (5 entries gần nhất)
- One-click copy password/username/OTP với 10s auto-clear
- OTP countdown ring (green → red khi ≤5 giây)
- Status indicator: 🟢 unlocked / 🟠 locked / 🔴 not connected
- IPC với KeePassEx desktop app qua WebSocket (port 27015)

### Files

| File                        | Mô tả                               |
| --------------------------- | ----------------------------------- |
| `KeePassExMenuBarApp.swift` | App entry point, NSStatusItem setup |
| `MenuBarView.swift`         | SwiftUI popover UI (320px wide)     |
| `MenuBarViewModel.swift`    | State management, IPC client        |

### IPC Protocol

Menu bar app kết nối với desktop app qua WebSocket:

```
ws://127.0.0.1:27015/menubar
```

**Actions:**

```json
{ "action": "get_vault_status" }
{ "action": "search_entries", "query": "github" }
{ "action": "get_recent_entries", "limit": 5 }
{ "action": "get_entry_password", "uuid": "..." }
{ "action": "get_entry_username", "uuid": "..." }
{ "action": "generate_totp", "uuid": "..." }
{ "action": "lock_vault" }
```

### Build

```bash
# Open in Xcode
open apps/macos-menubar/KeePassExMenuBar.xcodeproj

# Build from CLI
xcodebuild -project apps/macos-menubar/KeePassExMenuBar.xcodeproj \
  -scheme KeePassExMenuBar \
  -configuration Release \
  -archivePath build/KeePassExMenuBar.xcarchive \
  archive
```

---

## Windows Credential Provider

**Location**: `apps/windows-credprov/src/`
**Stack**: Rust cdylib + Windows COM
**Competitor gap**: ❌ Không đối thủ nào có

### Tính năng

- Unlock Windows login screen bằng KeePassEx vault master password
- ZKPV pre-check: xác minh password nhanh không cần full Argon2id (~50ms vs ~2s)
- Windows credentials lưu encrypted trong vault
- Hiển thị tile "KeePassEx" trên Windows login screen
- Hỗ trợ Windows 10/11

### Security Model

```
Windows Login Screen
        │
        ▼
KeePassEx Credential Provider tile
        │
        ▼
User enters vault master password
        │
        ▼
ZKPV pre-check (BLAKE3 commitment, ~1ms)
  → Wrong password: reject immediately
  → Correct password: proceed
        │
        ▼
Retrieve Windows credentials from vault
(encrypted with master key)
        │
        ▼
Windows authentication
```

### Files

| File            | Mô tả                                          |
| --------------- | ---------------------------------------------- |
| `lib.rs`        | DLL entry point, COM exports                   |
| `provider.rs`   | `ICredentialProvider` implementation           |
| `credential.rs` | `ICredentialProviderCredential` implementation |
| `tile.rs`       | Login screen tile UI                           |
| `registry.rs`   | DLL registration/unregistration                |

### Installation

```powershell
# Build
cargo build --release -p keepassex-credprov

# Install (requires admin)
regsvr32 target\release\keepassex_credprov.dll

# Uninstall
regsvr32 /u target\release\keepassex_credprov.dll
```

### Build

```bash
cargo build --release -p keepassex-credprov \
  --target x86_64-pc-windows-msvc
```

---

## watchOS App

**Location**: `apps/watch/watchos/KeePassExWatch/`
**Stack**: SwiftUI + WatchConnectivity
**Competitor gap**: ❌ Không đối thủ nào có native watchOS app

### Tính năng

- Entry list với Digital Crown scroll
- Search (Digital Crown + keyboard)
- Favorites filter (⭐ button)
- OTP countdown ring với haptic warning khi ≤5 giây
- Copy password/username/OTP → phone clipboard
- Watch face complications (circular, rectangular, corner)
- Lock vault remotely từ watch
- Haptic feedback: success/failure/warning

### Files

| File                           | Mô tả                              |
| ------------------------------ | ---------------------------------- |
| `KeePassExWatchApp.swift`      | App entry point                    |
| `ContentView.swift`            | Root view, entry list, detail, OTP |
| `ComplicationController.swift` | Watch face complications           |

### WatchConnectivity Protocol

Watch app giao tiếp với iPhone app qua `WCSession`:

```swift
// Watch → Phone
session.sendMessage(["action": "getEntries"], replyHandler: ...)
session.sendMessage(["action": "getOtp", "uuid": "..."], replyHandler: ...)
session.sendMessage(["action": "copyField", "uuid": "...", "field": "password"], replyHandler: ...)
session.sendMessage(["action": "unlock"], replyHandler: ...)

// Phone → Watch (push)
session.sendMessage(["action": "vaultLocked"], replyHandler: nil)
```

### Complications

| Type        | Content                   |
| ----------- | ------------------------- |
| Circular    | Lock icon + vault status  |
| Rectangular | "KeePassEx" + entry count |
| Corner      | Lock icon                 |

---

## WearOS App

**Location**: `apps/watch/wearos/`
**Stack**: Jetpack Compose + Wearable Data Layer
**Competitor gap**: ❌ Không đối thủ nào có native WearOS app

### Tính năng

- Entry list với rotary input (Digital Crown/bezel)
- Favorites filter chip
- OTP countdown với haptic warning
- Quick Settings tile
- Watch face complication
- Haptic feedback: VibrationPattern enum

### Files

| File                                                    | Mô tả                   |
| ------------------------------------------------------- | ----------------------- |
| `app/src/main/java/.../MainActivity.kt`                 | Main Compose UI         |
| `app/src/main/java/.../KeePassExTileService.kt`         | Quick Settings tile     |
| `app/src/main/java/.../KeePassExComplicationService.kt` | Watch face complication |
| `app/src/main/java/.../WearDataListenerService.kt`      | Phone communication     |

### Wearable Data Layer Protocol

```kotlin
// WearOS → Phone
messageClient.sendMessage(phoneNodeId, "/keepassex/getEntries", null)
messageClient.sendMessage(phoneNodeId, "/keepassex/getOtp", uuid.toByteArray())
messageClient.sendMessage(phoneNodeId, "/keepassex/copyPassword", uuid.toByteArray())

// Phone → WearOS (push)
messageClient.sendMessage(wearNodeId, "/keepassex/vaultLocked", null)
messageClient.sendMessage(wearNodeId, "/keepassex/entriesUpdated", entriesJson)
```

---

## KeePassEx Server

**Location**: `apps/server/`
**Stack**: Rust + Axum + SQLite + JWT
**Competitor gap**: ❌ KeePass/KeePassXC không có server. Bitwarden có nhưng phức tạp hơn.

Xem [SYNC.md](SYNC.md) để biết chi tiết về setup và sử dụng.

### Architecture

```
┌─────────────────────────────────────────────────────┐
│  KeePassEx Server                                   │
│                                                     │
│  ┌─────────┐  ┌──────────┐  ┌──────────────────┐   │
│  │  Auth   │  │  Vault   │  │   WebSocket      │   │
│  │  (JWT)  │  │  (CRUD)  │  │  (Real-time)     │   │
│  └────┬────┘  └────┬─────┘  └────────┬─────────┘   │
│       │            │                  │             │
│  ┌────▼────────────▼──────────────────▼──────────┐  │
│  │              SQLite Database                  │  │
│  │  users | vault_data | vault_history           │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### Database Schema

```sql
-- Users
CREATE TABLE users (
  id TEXT PRIMARY KEY,
  email TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,  -- Argon2id
  created_at TEXT NOT NULL
);

-- Vault data (encrypted blobs)
CREATE TABLE vault_data (
  user_id TEXT PRIMARY KEY,
  data BLOB NOT NULL,           -- Encrypted KDBX bytes
  version INTEGER NOT NULL,
  size_bytes INTEGER NOT NULL,
  client_hash TEXT,             -- Optional SHA-256 for integrity
  uploaded_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Version history (last 10 per user)
CREATE TABLE vault_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id TEXT NOT NULL,
  data BLOB NOT NULL,
  version INTEGER NOT NULL,
  size_bytes INTEGER NOT NULL,
  client_hash TEXT,
  uploaded_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);
```

### Docker Compose

```yaml
# apps/server/docker-compose.yml
services:
  keepassex-server:
    image: keepassex/server:latest
    ports:
      - '8080:8080'
    environment:
      KPX_JWT_SECRET: '${KPX_JWT_SECRET}'
      KPX_DB: /data/keepassex.db
      KPX_MAX_VAULT_MB: '100'
    volumes:
      - keepassex_data:/data
    restart: unless-stopped

volumes:
  keepassex_data:
```

---

## So sánh với đối thủ

| Platform              | KeePass | KeePassXC | Keepassium | KeePass2Android | **KeePassEx** |
| --------------------- | ------- | --------- | ---------- | --------------- | ------------- |
| macOS Menu Bar        | ❌      | ❌        | ❌         | ❌              | ✅            |
| Windows CredProvider  | ❌      | ❌        | ❌         | ❌              | ✅            |
| watchOS native        | ❌      | ❌        | ❌         | ❌              | ✅            |
| WearOS native         | ❌      | ❌        | ❌         | ❌              | ✅            |
| Self-hosted server    | ❌      | ❌        | ❌         | ❌              | ✅            |
| TUI (vim keybindings) | ❌      | ❌        | ❌         | ❌              | ✅            |

**KeePassEx là password manager duy nhất có tất cả 6 nền tảng mở rộng này.**
