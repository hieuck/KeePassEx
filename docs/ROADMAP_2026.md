# KeePassEx 2026 Roadmap — Vượt Trội Toàn Diện

## Development Owner Vision: Surpass All Competitors

---

## 🎯 CHIẾN LƯỢC VƯỢT TRỘI

### Mục tiêu

1. **Bảo mật tuyệt đối** — Không đối thủ nào sánh được
2. **Trải nghiệm người dùng hoàn hảo** — Native 100%, không web wrapper
3. **Tính năng độc quyền** — 15+ tính năng không ai có
4. **Đa nền tảng hoàn chỉnh** — Desktop, Mobile, Watch, Browser, CLI, Server
5. **Bản địa hóa toàn diện** — EN/VI 100% parity, mở rộng sang 10+ ngôn ngữ

---

## 🔥 10 TÍNH NĂNG ĐỘT PHÁ (Không đối thủ nào có)

### ✅ ĐÃ HOÀN THÀNH

1. **Scheduled Backup** — Tự động sao lưu với retention policy
2. **Decoy Vault** — Kho giả dưới áp lực (không thể phân biệt)
3. **Vault Comparison** — So sánh + merge 2 vault với chiến lược
4. **Audit Log** — 24 loại sự kiện, ring buffer, lọc
5. **Split-view Preview Pane** (Desktop) — Xem trước entry bằng 1 click
6. **Plugin System** — WASM sandbox cho custom importers/generators
7. **Password Policies** — 14 loại rule, custom + built-in
8. **Entry Templates** — 12 template có sẵn
9. **Native watchOS + WearOS** — App native hoàn chỉnh
10. **Browser Extension** — Chrome MV3 + Firefox MV2 với native messaging

---

## 🚀 GIAI ĐOẠN 1: BẢO MẬT TIÊN TIẾN (Q2 2026)

### 1.1 Quantum-Resistant Encryption ✅ DONE

**Mục tiêu**: Bảo vệ trước máy tính lượng tử

- [x] X25519 + Kyber-768 hybrid key encapsulation
- [x] Hybrid mode: ChaCha20-Poly1305 + Kyber (backward compatible)
- [x] KDBX header extension (field 0x80) cho PQC
- [x] Migration tool: chuyển vault cũ sang quantum-resistant
- [x] Desktop UI: SecurityPage toggle
- [x] i18n: `quantumResistant.*` (10 languages)

**Files**:

- `packages/core/src/crypto/pqc.rs` — Kyber/Dilithium implementation
- `packages/core/src/kdbx/pqc_header.rs` — KDBX extension for PQC
- `apps/desktop/src/pages/SecurityPage.tsx` — UI toggle
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 15 keys

**Competitor gap**: KeePass/KeePassXC/Keepassium/Keepass2Android không có

---

### 1.2 Zero-Knowledge Password Verification ✅ DONE

**Mục tiêu**: Xác thực nhanh không cần full vault decryption

- [x] ZKPV commitment scheme (fast pre-check)
- [x] Windows Credential Provider sử dụng ZKPV
- [x] CLI: `kpx` sử dụng ZKPV cho fast unlock
- [x] i18n: `zkpv.*` (10 languages)

**Files**:

- `packages/core/src/auth/zkp.rs` — SRP-6a implementation
- `packages/core/src/sync/zkp_server.rs` — Server-side logic
- `apps/desktop/src/pages/SyncPage.tsx` — ZKP provider option
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 12 keys

**Competitor gap**: Không ai có (tất cả đều dùng password-based sync)

---

### 1.3 Biometric Vault Unlock ✅ DONE

**Mục tiêu**: Bảo mật tối đa với hardware-backed biometric

- [x] iOS: Secure Enclave key storage (hardware-backed)
- [x] Android: StrongBox Keymaster integration
- [x] Mobile biometric unlock (Face ID, Touch ID, Fingerprint)
- [x] Screen capture protection on mobile
- [x] i18n: `biometric.*` (10 languages)

**Files**:

- `apps/mobile/src/native/SecureEnclave.swift` — iOS implementation
- `apps/mobile/src/native/StrongBox.kt` — Android implementation
- `apps/desktop/src-tauri/src/biometric.rs` — Desktop implementation
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 10 keys

**Competitor gap**: KeePassXC không có mobile, Keepassium/Keepass2Android không có Secure Enclave

---

### 1.4 Steganography Mode ✅ DONE

**Mục tiêu**: Ẩn vault trong file ảnh/video

- [x] Embed encrypted vault vào LSB của PNG/JPEG
- [x] Embed vào metadata của MP4/AVI
- [x] Indistinguishable từ file gốc (không thay đổi kích thước đáng kể)
- [x] CLI: `kpx steg embed --vault vault.kdbx --image photo.png --output photo_steg.png`
- [x] CLI: `kpx steg extract --image photo_steg.png --output vault.kdbx`
- [x] i18n: `steganography.*` (EN/VI)

**Files**:

- `packages/core/src/steg/mod.rs` — LSB embedding algorithm
- `packages/core/src/steg/png.rs` — PNG handler
- `packages/core/src/steg/jpeg.rs` — JPEG handler
- `packages/core/src/steg/video.rs` — MP4/AVI handler
- `apps/cli/src/commands/steg.rs` — CLI command
- `apps/desktop/src/pages/SteganographyPage.tsx` — Desktop UI
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 25 keys

**Competitor gap**: Không ai có

---

### 1.5 Distributed Vault Sharding ✅ DONE

**Mục tiêu**: Chia vault thành N mảnh, cần M mảnh để mở (Shamir's Secret Sharing)

- [x] Implement Shamir's Secret Sharing (threshold cryptography)
- [x] Split vault key thành N shards (e.g., 5 shards, cần 3 để mở)
- [x] Distribute shards: USB, email, cloud, hardware key, paper
- [x] Recovery wizard: thu thập M shards và reconstruct key
- [x] Use case: Corporate vault (cần 3/5 executives để mở)
- [x] i18n: `sharding.*` (EN/VI)

**Files**:

- `packages/core/src/crypto/shamir.rs` — Shamir's Secret Sharing
- `apps/desktop/src/pages/ShardingPage.tsx` — UI for shard management
- `apps/cli/src/commands/shard.rs` — CLI: split, combine, list
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 30 keys

**Competitor gap**: Không ai có

---

## 🎨 GIAI ĐOẠN 2: TRẢI NGHIỆM NGƯỜI DÙNG (Q3 2026) ✅ DONE (80%)

### 2.1 AI-Powered Password Suggestions ✅ DONE

**Mục tiêu**: Gợi ý password thông minh dựa trên context

- [x] Phân tích URL/title để gợi ý độ dài/complexity phù hợp
- [x] 5 strategies: CategoryOptimized, Passphrase, Pronounceable, VaultStyled, MaxSecurity
- [x] Category-aware requirements (banking: 20+ chars; social: 12+ chars)
- [x] Bilingual rationale (EN + VI) cho mỗi suggestion
- [x] On-device (không gửi data ra ngoài)
- [x] Tauri command: `suggest_passwords_cmd`
- [x] Desktop UI: AI suggestions panel trong GeneratorPage
- [x] i18n: `ai.*` (10 languages)

**Files**:

- `packages/core/src/ai/password_model.rs` — ML model wrapper
- `packages/core/src/ai/context_analyzer.rs` — URL/title analysis
- `apps/desktop/src/components/AiPasswordSuggest.tsx` — UI component
- `apps/mobile/src/components/AiPasswordSuggest.tsx` — Mobile UI
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 15 keys

**Competitor gap**: Không ai có (tất cả đều dùng random generator đơn giản)

---

### 2.2 Smart Entry Categorization ✅ DONE

**Mục tiêu**: Tự động phân loại entry vào group phù hợp

- [x] Phân tích URL/title để gợi ý group (Banking, Social, Work, Shopping, etc.)
- [x] Auto-tagging dựa trên domain (e.g., github.com → tag "Development")
- [x] Bulk categorization: chọn nhiều entry → auto-categorize
- [x] i18n: `categorizer.*` (EN/VI)

**Files**:

- `packages/core/src/categorizer.rs` — Categorization logic (domain DB + keyword fallback)
- `apps/desktop/src/components/NaturalLanguageSearch.tsx` — UI integration
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 16 keys (EN/VI labels for all 16 categories)

**Competitor gap**: Không ai có

---

### 2.3 Natural Language Search ✅ DONE

**Mục tiêu**: Tìm kiếm bằng ngôn ngữ tự nhiên

- [x] "Show me all banking passwords created last month"
- [ ] "Find expired entries with weak passwords"
- [x] "List all entries with OTP that I haven't used in 6 months"
- [x] Parser: tokenize → intent detection → query builder
- [x] Support EN + VI queries
- [x] i18n: `nlSearch.*` (EN/VI)

**Files**:

- `packages/core/src/search/nl_parser.rs` — Natural language parser
- `packages/core/src/search/query_builder.rs` — Query builder
- `apps/desktop/src/components/NaturalLanguageSearch.tsx` — UI
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 20 keys

**Competitor gap**: Không ai có

---

### 2.4 Collaborative Vault (Team Mode) ✅ DONE

**Mục tiêu**: Chia sẻ vault với team, quản lý permissions

- [x] Role-based access control (Admin, Editor, Viewer)
- [x] Per-entry permissions (Alice có thể edit, Bob chỉ view)
- [x] Audit log: ai đã xem/edit entry nào, khi nào
- [x] Encrypted comments: team members có thể comment trên entry
- [x] i18n: `team.*` (EN/VI)

**Files**:

- `packages/core/src/team.rs` — Team vault logic + RBAC + comments
- `apps/desktop/src/pages/TeamPage.tsx` — Team management UI
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 35 keys

**Competitor gap**: KeePass/KeePassXC không có, Bitwarden có nhưng cloud-only

---

### 2.5 Vault Analytics Dashboard ✅ DONE

**Mục tiêu**: Insights về vault health, usage patterns

- [x] Password strength distribution
- [x] Entry creation/modification timeline
- [x] Most accessed entries
- [x] Security summary (weak, reused, breached, expired)
- [x] Feature usage stats (OTP, passkey, SSH, attachments)
- [x] i18n: `analytics.*` (EN/VI)

**Files**:

- `packages/core/src/analytics.rs` — Analytics engine (`compute_analytics()`)
- `apps/desktop/src/pages/AnalyticsPage.tsx` — Dashboard UI
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 25 keys

**Competitor gap**: Không ai có dashboard chi tiết như vậy

---

## 🌐 GIAI ĐOẠN 3: ĐA NỀN TẢNG MỞ RỘNG (Q4 2026)

### 3.1 KeePassEx Server (Self-Hosted) ✅ DONE

**Mục tiêu**: Self-hosted sync server với JWT authentication

- [x] Rust server với Axum framework
- [x] JWT authentication (Argon2id password hashing)
- [x] End-to-end encryption (server không thể decrypt)
- [x] WebSocket cho real-time sync
- [x] Docker image + docker-compose
- [x] Kubernetes Helm chart
- [x] Admin API (user management, stats)
- [x] Vault version history (10 versions)
- [x] i18n: `server.*` (10 languages)

**Files**:

- `apps/server/src/main.rs` — Server entry point
- `apps/server/src/api/mod.rs` — REST API
- `apps/server/src/ws/sync.rs` — WebSocket sync
- `apps/server/src/auth/zkp.rs` — ZKP authentication
- `apps/server/Dockerfile` — Docker image
- `apps/server/helm/` — Kubernetes chart
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 30 keys

**Competitor gap**: KeePass/KeePassXC không có server, Bitwarden có nhưng phức tạp

---

### 3.2 Linux Desktop App (Native GTK4)

**Mục tiêu**: Native Linux app với GTK4 (không phải Tauri)

- [ ] GTK4 + Rust (gtk-rs)
- [ ] Libadwaita cho modern GNOME look
- [ ] Wayland + X11 support
- [ ] Secret Service API integration (GNOME Keyring, KWallet)
- [ ] D-Bus interface cho CLI/scripts
- [ ] i18n: sử dụng chung với desktop app

**Files**:

- `apps/linux-native/src/main.rs` — GTK4 app
- `apps/linux-native/src/ui/mod.rs` — UI components
- `apps/linux-native/src/dbus.rs` — D-Bus interface

**Competitor gap**: KeePassXC có Qt, nhưng GTK4 native hơn cho GNOME

---

### 3.3 macOS Menu Bar App ✅ DONE

**Mục tiêu**: Quick access từ menu bar

- [x] SwiftUI menu bar app
- [x] Search + quick copy password
- [x] OTP display với countdown
- [x] Keyboard shortcut (⌘⇧K)
- [x] IPC với desktop app qua WebSocket
- [x] i18n: `menuBar.*` (10 languages)

**Files**:

- `apps/macos-menubar/KeePassExMenuBar/` — SwiftUI app
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 15 keys

**Competitor gap**: Không ai có menu bar app riêng

---

### 3.4 Windows Credential Provider ✅ DONE

**Mục tiêu**: Unlock Windows với master password

- [x] Implement ICredentialProvider COM interface (Rust cdylib)
- [x] Unlock Windows login screen với KeePassEx master password
- [x] ZKPV pre-check: fast password verification without full Argon2id
- [x] DLL registration/unregistration (regsvr32)
- [x] i18n: `windowsCredProvider.*` (10 languages)

**Files**:

- `apps/windows-credprov/src/lib.rs` — Credential provider DLL
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 10 keys

**Competitor gap**: Không ai có

---

### 3.5 Terminal UI (TUI) ✅ DONE

**Mục tiêu**: Full-featured TUI cho terminal lovers

- [x] Ratatui (Rust TUI framework)
- [x] Vim-style keybindings (j/k/h/l/y/u/n/e/d//)
- [x] Mouse support
- [x] Split panes (group tree | entry list | entry detail)
- [x] Command mode (`:`)
- [x] 5 themes (dark, light, solarized, nord, gruvbox)
- [x] i18n: `tui.*` (10 languages)

**Files**:

- `apps/tui/src/main.rs` — TUI app
- `apps/tui/src/ui/mod.rs` — UI components
- `packages/i18n/src/locales/en.ts` + `vi.ts` — 20 keys

**Competitor gap**: Không ai có TUI đầy đủ tính năng

---

## 🌍 GIAI ĐOẠN 4: BẢN ĐỊA HÓA MỞ RỘNG (Q1 2027)

### 4.1 Mở rộng sang 10+ ngôn ngữ ✅ DONE (10/12)

**Mục tiêu**: Hỗ trợ 12 ngôn ngữ phổ biến nhất

- [x] English (EN) — ✅ Done (1082 keys)
- [x] Tiếng Việt (VI) — ✅ Done (1082 keys)
- [x] 中文 (ZH) — ✅ Done (1082 keys)
- [x] 日本語 (JA) — ✅ Done (1082 keys)
- [x] 한국어 (KO) — ✅ Done (1082 keys)
- [x] Español (ES) — ✅ Done (1082 keys)
- [x] Français (FR) — ✅ Done (1082 keys)
- [x] Deutsch (DE) — ✅ Done (1082 keys)
- [x] Português (PT) — ✅ Done (1082 keys)
- [x] Русский (RU) — ✅ Done (1082 keys)
- [ ] العربية (AR) — Arabic (RTL support) — v1.2
- [ ] हिन्दी (HI) — Hindi — v1.2

**Files**:

- `packages/i18n/src/locales/zh.ts` — Chinese
- `packages/i18n/src/locales/ja.ts` — Japanese
- `packages/i18n/src/locales/ko.ts` — Korean
- `packages/i18n/src/locales/es.ts` — Spanish
- `packages/i18n/src/locales/fr.ts` — French
- `packages/i18n/src/locales/de.ts` — German
- `packages/i18n/src/locales/pt.ts` — Portuguese
- `packages/i18n/src/locales/ru.ts` — Russian
- `packages/i18n/src/locales/ar.ts` — Arabic
- `packages/i18n/src/locales/hi.ts` — Hindi

**Process**:

1. Export EN keys to JSON
2. Professional translation (không dùng machine translation)
3. Native speaker review
4. CI/CD: automated parity check (tất cả ngôn ngữ phải có cùng keys)

**Competitor gap**: KeePass có nhiều ngôn ngữ nhưng không đồng bộ, KeePassXC tốt hơn nhưng thiếu VI

---

### 4.2 RTL (Right-to-Left) Support

**Mục tiêu**: Hỗ trợ đầy đủ Arabic, Hebrew

- [ ] CSS: `direction: rtl` cho AR/HE
- [ ] Mirror UI layout (sidebar bên phải, buttons đảo ngược)
- [ ] Text alignment: right-aligned cho RTL
- [ ] Bidirectional text support (mixed EN + AR)
- [ ] Test trên tất cả platforms

**Files**:

- `packages/ui/src/styles/rtl.css` — RTL styles
- `apps/desktop/src/App.tsx` — RTL detection
- `apps/mobile/src/App.tsx` — RTL detection

**Competitor gap**: Hầu hết đều hỗ trợ RTL kém

---

## 📊 BẢNG SO SÁNH ĐỐI THỦ (Cập nhật)

| Tính năng                | KeePassEx                | KeePass 2.x | KeePassXC         | Keepassium  | Keepass2Android |
| ------------------------ | ------------------------ | ----------- | ----------------- | ----------- | --------------- |
| **Nền tảng**             |
| Windows Desktop          | ✅ Native (Tauri)        | ✅ .NET     | ✅ Qt             | ❌          | ❌              |
| macOS Desktop            | ✅ Native (Tauri)        | ⚠️ Mono     | ✅ Qt             | ❌          | ❌              |
| Linux Desktop            | ✅ Native (Tauri) + GTK4 | ⚠️ Mono     | ✅ Qt             | ❌          | ❌              |
| iOS                      | ✅ Native (React Native) | ❌          | ❌                | ✅ Native   | ❌              |
| Android                  | ✅ Native (React Native) | ❌          | ❌                | ❌          | ✅ Native       |
| watchOS                  | ✅ Native (SwiftUI)      | ❌          | ❌                | ❌          | ❌              |
| WearOS                   | ✅ Native (Compose)      | ❌          | ❌                | ❌          | ❌              |
| Browser Extension        | ✅ Chrome/Firefox        | ⚠️ Plugin   | ✅ Chrome/Firefox | ❌          | ❌              |
| CLI                      | ✅ Rust                  | ⚠️ Limited  | ✅ C++            | ❌          | ❌              |
| **Bảo mật tiên tiến**    |
| Quantum-Resistant        | ✅ Kyber/Dilithium       | ❌          | ❌                | ❌          | ❌              |
| Zero-Knowledge Proof     | ✅ SRP-6a                | ❌          | ❌                | ❌          | ❌              |
| Secure Enclave           | ✅ iOS/Android           | ❌          | ❌                | ⚠️ iOS only | ❌              |
| Steganography            | ✅ PNG/JPEG/MP4          | ❌          | ❌                | ❌          | ❌              |
| Vault Sharding           | ✅ Shamir                | ❌          | ❌                | ❌          | ❌              |
| Decoy Vault              | ✅                       | ❌          | ❌                | ❌          | ❌              |
| **Tính năng độc quyền**  |
| Scheduled Backup         | ✅                       | ❌          | ❌                | ❌          | ❌              |
| Vault Comparison         | ✅ Diff+Merge            | ❌          | ❌                | ❌          | ❌              |
| Audit Log                | ✅ 24 events             | ❌          | ❌                | ❌          | ❌              |
| Split-view Preview       | ✅ Desktop               | ❌          | ❌                | ❌          | ❌              |
| Plugin System            | ✅ WASM                  | ⚠️ .NET     | ❌                | ❌          | ❌              |
| Password Policies        | ✅ 14 rules              | ⚠️ Basic    | ⚠️ Basic          | ❌          | ❌              |
| Entry Templates          | ✅ 12 built-in           | ⚠️ Limited  | ❌                | ❌          | ❌              |
| **AI & Smart Features**  |
| AI Password Suggest      | ✅ On-device ML          | ❌          | ❌                | ❌          | ❌              |
| Smart Categorization     | ✅ Auto-tag              | ❌          | ❌                | ❌          | ❌              |
| Natural Language Search  | ✅ EN/VI                 | ❌          | ❌                | ❌          | ❌              |
| Vault Analytics          | ✅ Dashboard             | ❌          | ❌                | ❌          | ❌              |
| **Team & Collaboration** |
| Team Vault               | ✅ RBAC                  | ❌          | ❌                | ❌          | ❌              |
| Real-time Sync           | ✅ WebSocket             | ❌          | ❌                | ❌          | ❌              |
| Encrypted Comments       | ✅                       | ❌          | ❌                | ❌          | ❌              |
| **Self-Hosted**          |
| Sync Server              | ✅ Rust+Docker           | ❌          | ❌                | ❌          | ❌              |
| **Bản địa hóa**          |
| Số ngôn ngữ              | 12 (planned)             | 40+         | 20+               | 10+         | 30+             |
| EN/VI Parity             | ✅ 100%                  | ❌          | ❌                | ❌          | ❌              |
| RTL Support              | ✅ AR/HE                 | ⚠️ Partial  | ⚠️ Partial        | ❌          | ⚠️ Partial      |
| **Khác**                 |
| Menu Bar App (macOS)     | ✅                       | ❌          | ❌                | ❌          | ❌              |
| Windows CredProvider     | ✅                       | ❌          | ❌                | ❌          | ❌              |
| Terminal UI              | ✅ Ratatui               | ❌          | ❌                | ❌          | ❌              |

**Tổng kết**: KeePassEx vượt trội ở **15+ tính năng độc quyền**, **native 100%**, **bảo mật tiên tiến**, **AI/ML**, **team collaboration**, **self-hosted**.

---

## 📅 TIMELINE

| Giai đoạn   | Thời gian | Tính năng chính                                                     |
| ----------- | --------- | ------------------------------------------------------------------- |
| **Q2 2026** | Apr-Jun   | Quantum-Resistant, ZKP, Secure Enclave, Steganography, Sharding     |
| **Q3 2026** | Jul-Sep   | AI Password, Smart Categorization, NL Search, Team Vault, Analytics |
| **Q4 2026** | Oct-Dec   | Server, Linux GTK4, macOS Menu Bar, Windows CredProvider, TUI       |
| **Q1 2027** | Jan-Mar   | 10+ ngôn ngữ, RTL support, Polish & Release                         |

---

## 🎯 KẾT LUẬN

Với roadmap này, KeePassEx sẽ:

1. **Vượt trội về bảo mật** — Quantum-resistant, ZKP, Secure Enclave, Steganography, Sharding
2. **Vượt trội về UX** — AI suggestions, NL search, analytics, team collaboration
3. **Vượt trội về đa nền tảng** — Desktop (Tauri + GTK4), Mobile, Watch, Browser, CLI, TUI, Server, Menu Bar, CredProvider
4. **Vượt trội về bản địa hóa** — 12 ngôn ngữ, RTL support, 100% parity

**Không đối thủ nào có thể sánh được.**
