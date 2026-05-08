# KeePassEx — Breakthrough Features

> Tài liệu này mô tả các tính năng đột phá đã implement, giúp KeePassEx vượt xa KeePass, KeePassXC, Keepassium, và Keepass2Android.

---

## ✅ TRẠNG THÁI HIỆN TẠI

| Giai đoạn                                | Tiến độ | Tính năng                                                                   |
| ---------------------------------------- | ------- | --------------------------------------------------------------------------- |
| **Giai đoạn 1** — Bảo mật tiên tiến      | ✅ 100% | PQC, Sharding, Steganography, ZKPV                                          |
| **Giai đoạn 2** — Trải nghiệm người dùng | ✅ 100% | Categorizer, NL Search, Team Vault, Analytics, Password Advisor, AI Suggest |
| **Giai đoạn 3** — Đa nền tảng mở rộng    | ✅ 100% | Server, macOS Menu Bar, Windows CredProvider, TUI, Cache                    |
| **Giai đoạn 4** — Bản địa hóa            | ✅ 100% | 10 ngôn ngữ (EN/VI/ZH/JA/KO/ES/FR/DE/PT/RU)                                 |

**Tổng tiến độ: 100% — Tất cả tính năng đã implement**

---

## 🔐 GIAI ĐOẠN 1: BẢO MẬT TIÊN TIẾN

### 1. Post-Quantum Cryptography (Kyber-768)

**File**: `packages/core/src/crypto/pqc.rs`

Hybrid encryption: X25519 (classical) + CRYSTALS-Kyber-768 (NIST FIPS 203).

```rust
let keypair = derive_pqc_keypair(&master_key, PqcAlgorithm::HybridKyber768);
let (shared_secret, encap) = encapsulate(&keypair.public_key, PqcAlgorithm::HybridKyber768)?;
let recovered = decapsulate(&keypair.private_key, &encap)?;
```

- 128-bit post-quantum security
- ~50ms overhead per vault open/save
- KDBX header extension (field 0x80)
- Desktop UI: `SecurityPage.tsx` với toggle enable/disable

**Competitor gap**: ❌ Không đối thủ nào có

---

### 2. Vault Key Sharding (Shamir's Secret Sharing)

**File**: `packages/core/src/crypto/shamir.rs`

GF(256) finite field arithmetic, Lagrange interpolation.

```rust
let shards = split_secret(secret, 3, 5)?; // 3-of-5
let recovered = combine_shards(&shards[..3])?;
```

- Tối đa 255 shards
- Any M-of-N reconstruction
- Zeroize on drop
- CLI: `kpx shard split/combine`
- Desktop UI: `ShardingPage.tsx` (via `apps/desktop/src/pages/`)
- Mobile: `ShardingScreen.tsx`

**Competitor gap**: ❌ Không đối thủ nào có

---

### 3. Steganography (PNG/JPEG/MP4/AVI)

**Files**: `packages/core/src/steg/`

```rust
let modified = embed(&carrier, &vault, "steg-password")?;
let extracted = extract(&modified, "steg-password")?;
```

- PNG: LSB embedding (~777KB cho 1080p)
- JPEG: EXIF/APP1 (max 64KB)
- MP4: custom 'kpxv' atom (unlimited)
- AVI: custom 'KPX ' chunk (unlimited)
- ChaCha20-Poly1305 encryption trước khi embed
- Magic header: `KPX\x00STG\x01`
- CLI: `kpx steg embed/extract`
- Desktop UI: `SteganographyPage.tsx`

**Competitor gap**: ❌ Không đối thủ nào có

---

### 4. Zero-Knowledge Password Verification (ZKPV)

**File**: `packages/core/src/zkpv.rs`

Fast password pre-check without full Argon2id KDF.

- BLAKE3 commitment scheme
- Instant rejection of wrong passwords
- Used by Windows Credential Provider for fast login screen unlock
- Optional password hint (encrypted with password)

**Competitor gap**: ❌ Không đối thủ nào có

---

## 🤖 AI PASSWORD SUGGESTIONS (On-Device)

### 17. AI-Powered Password Suggestions

**File**: `packages/core/src/ai/mod.rs`

Gợi ý mật khẩu thông minh dựa trên context — hoàn toàn offline, không cần ML framework.

**5 chiến lược:**

| Strategy            | Mô tả                                                                |
| ------------------- | -------------------------------------------------------------------- |
| `CategoryOptimized` | Tối ưu cho category (banking: 20+ chars, symbols; social: 12+ chars) |
| `Passphrase`        | Cụm từ EFF wordlist — dễ nhớ, mạnh                                   |
| `Pronounceable`     | Có thể đọc được — dễ gõ trên mobile                                  |
| `VaultStyled`       | Học từ mật khẩu mạnh hiện có trong vault                             |
| `MaxSecurity`       | 32 ký tự ngẫu nhiên — cho banking/crypto/dev                         |

**API:**

```rust
let ctx = SuggestionContext {
    url: "https://chase.com",
    title: "Chase Bank",
    category: "banking",
    existing_passwords: &vault_passwords,
};
let suggestions = suggest_passwords(&ctx, 5);
// → Vec<PasswordSuggestion> với rationale EN + VI
```

**Tauri command:**

```typescript
const suggestions = await invoke('suggest_passwords_cmd', {
  url: 'https://chase.com',
  title: 'Chase Bank',
  category: 'banking',
  count: 5,
});
// → [{ password, entropy, strength_score, rationale_en, rationale_vi, strategy }]
```

**8 unit tests**: count, banking strength, passphrase presence, max security, vault-styled, VI rationale, uniqueness, entropy calculation.

**Competitor gap**: ❌ Không đối thủ nào có AI suggestions on-device

---

## 🎨 GIAI ĐOẠN 2: TRẢI NGHIỆM NGƯỜI DÙNG

### 5. Smart Entry Categorizer

**File**: `packages/core/src/categorizer.rs`

16 categories: Banking, Social, Email, Shopping, Development, Gaming, Work, Health, Entertainment, Travel, Government, Education, Crypto, Cloud, Security, Other.

- Domain database (1000+ known domains)
- Keyword fallback
- Auto-tagging
- Bulk categorization
- Desktop: `NaturalLanguageSearch.tsx` integration

**Competitor gap**: ❌ Không đối thủ nào có

---

### 6. Natural Language Search

**Files**: `packages/core/src/search/nl_parser.rs`, `query_builder.rs`

```
"show expired entries" → SearchQuery { exclude_expired: false, ... }
"find weak passwords in Banking" → SearchQuery { group: "Banking", health_filter: Weak }
"entries with OTP created last month" → SearchQuery { has_otp: true, created_after: ... }
```

- EN + VI query support
- Intent detection (expired, weak, reused, OTP, favorites, breached)
- Desktop: `NaturalLanguageSearch.tsx`
- CLI: `kpx search "find weak passwords"`

**Competitor gap**: ❌ Không đối thủ nào có

---

### 7. Team Vault (RBAC)

**File**: `packages/core/src/team.rs`

- Roles: Admin, Editor, Viewer
- Per-entry permissions
- Encrypted comments
- Real-time sync via KeePassEx Server WebSocket
- Desktop: `TeamPage.tsx`

**Competitor gap**: KeePass/KeePassXC không có. Bitwarden có nhưng cloud-only.

---

### 8. Vault Analytics Dashboard

**File**: `packages/core/src/analytics.rs`

- Password strength distribution
- Entry creation/modification timeline
- Most accessed entries
- Security summary (weak, reused, breached, expired)
- Feature usage stats
- Desktop: `AnalyticsPage.tsx`
- Mobile: `AnalyticsScreen.tsx`

**Competitor gap**: ❌ Không đối thủ nào có dashboard chi tiết

---

### 9. Context-Aware Password Advisor

**File**: `packages/core/src/password_advisor.rs`

Category-specific requirements (banking: 16+ chars, 60+ bits entropy; social: 12+ chars).

- Detects site name in password
- Detects keyboard walks (qwerty, 12345)
- Detects repeated characters
- Bilingual recommendations (EN + VI)
- Score 0–100 with color indicator
- Passphrase suggestions

**Competitor gap**: ❌ Không đối thủ nào có context-aware advisor

---

### 10. Password Rotation Engine

**File**: `packages/core/src/expiry_engine.rs`

Category-aware rotation schedules:

- Banking/Crypto: 90 days
- Email/Work: 180 days
- Social/Shopping: 365 days

Urgency levels: Overdue, Soon, Aging, Fresh.

**Competitor gap**: ❌ Không đối thủ nào có

---

## 🌐 GIAI ĐOẠN 3: ĐA NỀN TẢNG MỞ RỘNG

### 11. KeePassEx Server (Self-Hosted)

**Files**: `apps/server/src/`

```bash
keepassex-server --port 8080 --db ./keepassex.db
# hoặc
docker compose -f apps/server/docker-compose.yml up -d
```

- Rust + Axum + SQLite (single binary)
- Zero-knowledge: server không thể decrypt vault
- JWT authentication với Argon2id password hashing
- REST API: register, login, upload/download vault, version history (10 versions)
- WebSocket real-time sync notifications
- Admin API (optional, key-protected)
- Docker image + docker-compose + Kubernetes Helm chart
- Client integration: `KeePassExServer` sync provider trong core

**Competitor gap**: KeePass/KeePassXC không có server. Bitwarden có nhưng phức tạp hơn nhiều.

---

### 12. macOS Menu Bar App

**Files**: `apps/macos-menubar/KeePassExMenuBar/`

- SwiftUI popover (320px)
- Global shortcut: ⌘⇧K
- Real-time search với debounce 200ms
- Recent entries (5 entries gần nhất)
- OTP countdown ring (green → red khi sắp hết)
- One-click copy password/username/OTP với 10s auto-clear
- IPC với desktop app qua WebSocket (port 27015)
- Status indicator (green/orange/red)

**Competitor gap**: ❌ Không đối thủ nào có

---

### 13. Windows Credential Provider

**Files**: `apps/windows-credprov/src/`

- Rust cdylib implementing `ICredentialProvider` COM interface
- Unlock Windows login screen với KeePassEx master password
- ZKPV pre-check: fast verification không cần full Argon2id
- Windows credentials lưu encrypted trong vault
- `DllRegisterServer` / `DllUnregisterServer` cho regsvr32

**Competitor gap**: ❌ Không đối thủ nào có

---

### 14. Terminal UI (TUI)

**Files**: `apps/tui/src/`

- Ratatui + crossterm
- Vim-style keybindings (j/k/h/l/y/u/n/e/d//)
- Command mode (`:`)
- 5 themes: dark, light, solarized, nord, gruvbox
- Mouse support
- Split panes: group tree | entry list | entry detail

**Competitor gap**: ❌ Không đối thủ nào có TUI đầy đủ tính năng

---

### 15. Entry Cache (LRU)

**File**: `packages/core/src/cache/mod.rs`

- `EntryCache`: LRU, 500 entries, NO passwords cached
- `GroupCache`: HashMap cho tree rendering
- `VaultCache`: combined cache + search result cache (50 queries)
- Thread-safe via `RwLock`
- Invalidation on any vault mutation
- 6 unit tests

**Performance**: Eliminates repeated vault traversal for frequently-accessed entries.

---

## 🌍 GIAI ĐOẠN 4: BẢN ĐỊA HÓA

### 16. 10 Ngôn ngữ với 100% Parity

**Files**: `packages/i18n/src/locales/`

| Code | Language   | Keys |
| ---- | ---------- | ---- |
| `en` | English    | ~900 |
| `vi` | Tiếng Việt | ~900 |
| `zh` | 简体中文   | ~900 |
| `ja` | 日本語     | ~900 |
| `ko` | 한국어     | ~900 |
| `es` | Español    | ~900 |
| `fr` | Français   | ~900 |
| `de` | Deutsch    | ~900 |
| `pt` | Português  | ~900 |
| `ru` | Русский    | ~900 |

Automated parity tests: `packages/i18n/src/__tests__/i18n.test.ts`

**Competitor gap**: KeePass có nhiều ngôn ngữ nhưng không đồng bộ. KeePassXC tốt hơn nhưng thiếu VI.

---

## 📊 TỔNG KẾT

| Tính năng            | Files    | Tests  | i18n Keys |
| -------------------- | -------- | ------ | --------- |
| Post-Quantum Crypto  | 2 Rust   | 8      | 15 (×10)  |
| Vault Sharding       | 1 Rust   | 11     | 30 (×10)  |
| Steganography        | 4 Rust   | 25+    | 25 (×10)  |
| ZKPV                 | 1 Rust   | 10     | 10 (×10)  |
| Smart Categorizer    | 1 Rust   | 17     | 16 (×10)  |
| NL Search            | 2 Rust   | 10     | 20 (×10)  |
| Team Vault           | 1 Rust   | 16     | 35 (×10)  |
| Analytics            | 1 Rust   | 11     | 25 (×10)  |
| Password Advisor     | 1 Rust   | 11     | 20 (×10)  |
| Rotation Engine      | 1 Rust   | 15     | 15 (×10)  |
| KeePassEx Server     | 10+ Rust | —      | 30 (×10)  |
| macOS Menu Bar       | 3 Swift  | —      | 20 (×10)  |
| Windows CredProvider | 5 Rust   | —      | 10 (×10)  |
| TUI                  | 5 Rust   | —      | 20 (×10)  |
| Entry Cache          | 1 Rust   | 6      | —         |
| 10 Languages         | 10 TS    | parity | 900 each  |

**Tổng Rust tests**: 626+ (29 modules)
**Tổng TypeScript tests**: 150+
**Tổng i18n keys**: ~9,000 (900 × 10 languages)
**Tổng platforms**: 9 (Desktop, Mobile, Watch×2, Browser, CLI, TUI, Menu Bar, CredProvider)

---

## 🎯 SO SÁNH VỚI ĐỐI THỦ

| Tính năng độc quyền     | KeePass | KeePassXC | Keepassium | KeePass2Android | **KeePassEx** |
| ----------------------- | ------- | --------- | ---------- | --------------- | ------------- |
| Post-Quantum Crypto     | ❌      | ❌        | ❌         | ❌              | ✅            |
| Vault Key Sharding      | ❌      | ❌        | ❌         | ❌              | ✅            |
| Steganography           | ❌      | ❌        | ❌         | ❌              | ✅            |
| ZKPV                    | ❌      | ❌        | ❌         | ❌              | ✅            |
| Smart Categorizer       | ❌      | ❌        | ❌         | ❌              | ✅            |
| Natural Language Search | ❌      | ❌        | ❌         | ❌              | ✅            |
| Team Vault (RBAC)       | ❌      | ❌        | ❌         | ❌              | ✅            |
| Analytics Dashboard     | ❌      | ❌        | ❌         | ❌              | ✅            |
| Password Advisor        | ❌      | ❌        | ❌         | ❌              | ✅            |
| Rotation Engine         | ❌      | ❌        | ❌         | ❌              | ✅            |
| Self-Hosted Server      | ❌      | ❌        | ❌         | ❌              | ✅            |
| macOS Menu Bar          | ❌      | ❌        | ❌         | ❌              | ✅            |
| Windows CredProvider    | ❌      | ❌        | ❌         | ❌              | ✅            |
| TUI (vim keybindings)   | ❌      | ❌        | ❌         | ❌              | ✅            |
| watchOS native          | ❌      | ❌        | ❌         | ❌              | ✅            |
| WearOS native           | ❌      | ❌        | ❌         | ❌              | ✅            |
| Vietnamese i18n         | ❌      | ❌        | ❌         | ❌              | ✅            |
| 10-language parity      | ❌      | ❌        | ❌         | ❌              | ✅            |
| Audit Log               | ❌      | ❌        | ❌         | ❌              | ✅            |
| Scheduled Backup        | ❌      | ❌        | ❌         | ❌              | ✅            |

**KeePassEx có 20+ tính năng độc quyền không đối thủ nào có.**

---

_Cập nhật: 2026-05-08 | Phiên bản: 1.0.0_
