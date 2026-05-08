# KeePassEx — Tauri Command API Reference

All commands are invoked from the frontend via `invoke()` from `@tauri-apps/api/core`.

```typescript
import { invoke } from '@tauri-apps/api/core';
const result = await invoke<ReturnType>('command_name', { arg1, arg2 });
```

---

## Vault Commands

### `open_vault` → `VaultMetaDto`

Open an existing KDBX vault file.

```typescript
{ path: string; password?: string; key_file_data?: number[] }
```

### `create_vault` → `VaultMetaDto`

Create a new empty KDBX 4.x vault.

```typescript
{ path: string; name: string; password: string; key_file_data?: number[] }
```

### `save_vault` → `void`

Save the current vault to disk (atomic write via temp file + rename).

### `lock_vault` → `void`

Lock the vault (zeroes master key from memory, keeps vault path for re-unlock).

### `close_vault` → `void`

Close and unload the vault from memory.

### `change_credentials` → `void`

Change the master password.

```typescript
{
  old_password: string;
  new_password: string;
}
```

### `get_vault_meta` → `VaultMetaDto`

Get vault metadata without sensitive data.

### `open_vault_tab` → `VaultMetaDto`

Open a vault in a new multi-vault tab slot.

```typescript
{ path: string; password?: string; key_file_data?: number[] }
```

### `close_vault_tab` → `void`

Close a vault tab by path.

```typescript
{
  path: string;
}
```

### `lock_vault_tab` → `void`

Lock a specific vault tab.

```typescript
{
  path: string;
}
```

---

## Entry Commands

### `get_entries` → `EntryDto[]`

Get entries, optionally filtered by group.

```typescript
{ group_uuid?: string }
```

### `get_entry` → `EntryDto`

Get a single entry by UUID.

```typescript
{
  uuid: string;
  include_password: boolean;
}
```

### `get_entry_password` → `string`

Get the password for an entry (explicit request — logged in audit log).

```typescript
{
  uuid: string;
}
```

### `create_entry` → `string` (UUID)

```typescript
{ group_uuid: string; title: string; username: string; password: string;
  url: string; notes: string; tags: string[]; icon_id: number }
```

### `update_entry` → `void`

Update an existing entry (saves history snapshot automatically).

```typescript
{ uuid: string; title: string; username: string; password: string;
  url: string; notes: string; tags: string[]; icon_id: number;
  expiry?: string; custom_fields: CustomFieldDto[] }
```

### `delete_entry` → `void`

```typescript
{
  uuid: string;
  permanent: boolean;
}
```

### `move_entry` → `void`

```typescript
{
  uuid: string;
  new_group_uuid: string;
}
```

### `duplicate_entry` → `string` (new UUID)

```typescript
{
  uuid: string;
}
```

### `search_entries` → `EntryDto[]`

Full-text search across all entries.

```typescript
{
  query: string;
}
```

### `get_entry_history` → `EntryHistoryDto[]`

Get history snapshots, newest first.

```typescript
{
  uuid: string;
}
```

### `restore_entry_from_history` → `void`

Restore entry to a previous snapshot (current state saved to history first).

```typescript
{
  entry_uuid: string;
  history_uuid: string;
}
```

### `clear_entry_history` → `void`

```typescript
{
  uuid: string;
}
```

---

## Group Commands

### `get_groups` → `GroupDto[]`

### `create_group` → `string` (UUID)

```typescript
{ name: string; parent_uuid: string; icon_id?: number }
```

### `update_group` → `void`

### `delete_group` → `void`

```typescript
{
  uuid: string;
  permanent: boolean;
}
```

### `move_group` → `void`

```typescript
{
  uuid: string;
  new_parent_uuid: string;
}
```

---

## Generator Commands

### `generate_password` → `GeneratedPasswordDto`

```typescript
// Args: PasswordGeneratorConfig
{
  mode: 'random' | 'passphrase' | 'pronounceable';
  length: number;
  use_uppercase: boolean;
  use_lowercase: boolean;
  use_digits: boolean;
  use_symbols: boolean;
  exclude_ambiguous: boolean;
  exclude_chars: string;
  min_uppercase: number;
  min_lowercase: number;
  min_digits: number;
  min_symbols: number;
  word_count: number;
  word_separator: string;
  capitalize_words: boolean;
  include_number: boolean;
}
```

### `estimate_entropy` → `number` (bits)

```typescript
{
  password: string;
}
```

### `score_strength` → `number` (0–4)

```typescript
{
  password: string;
}
```

---

## AI Password Suggestion Commands 🆕

### `suggest_passwords_cmd` → `PasswordSuggestionDto[]`

Generate context-aware password suggestions using on-device AI.
Learns from existing vault passwords to match style preferences.

```typescript
{ url: string; title: string; category: string; count?: number }
// category: 'banking'|'email'|'social'|'development'|'security'|'general'|...
```

Returns up to `count` (default 5) suggestions with:

- `password` — the suggested password
- `entropy` — entropy in bits
- `strength_score` — 0–4
- `rationale_en` / `rationale_vi` — why this password was suggested
- `strategy` — `CategoryOptimized|Passphrase|Pronounceable|VaultStyled|MaxSecurity`

---

## OTP Commands

### `generate_totp` → `OtpCodeDto`

```typescript
{
  entry_uuid: string;
}
// Returns: { code: string; remaining_seconds: number; period: number; progress: number }
```

### `parse_otp_uri` → `OtpConfigDto`

```typescript
{
  uri: string;
}
```

---

## Health Commands

### `audit_vault` → `HealthReportDto`

Run full vault health audit (weak, reused, expired, old passwords).

---

## Breach Commands

### `check_vault_breaches` → `VaultBreachReport`

Check all vault passwords against HIBP (k-anonymity — passwords never sent).

```typescript
{
  online: boolean;
}
```

### `check_password_breach` → `BreachCheckResult`

```typescript
{
  password: string;
  online: boolean;
}
```

---

## Import/Export Commands

### `import_vault` → `ImportResultDto`

```typescript
{ file_path: string; format?: string; target_group_uuid?: string }
// format: 'auto'|'bitwarden'|'lastpass'|'chrome'|'firefox'|'1password'|
//         'dashlane'|'nordpass'|'enpass'|'roboform'|'keepass1'|'csv'
```

### `export_vault_cmd` → `number` (bytes written)

```typescript
{
  file_path: string;
  format: 'csv' | 'json' | 'html' | 'kdbx';
}
```

### `detect_import_format` → `string`

```typescript
{
  file_path: string;
}
```

---

## Sync Commands

### `get_sync_status` → `SyncStatusDto`

Returns current sync configuration and last sync result.

### `configure_sync` → `void`

Persist sync configuration to AppSettings.

```typescript
{ provider: string; remote_path: string; auto_sync: boolean;
  sync_interval_seconds: number; conflict_resolution: string;
  username?: string; password?: string; server_url?: string;
  token?: string; access_key_id?: string; secret_access_key?: string;
  region?: string; bucket?: string; endpoint?: string }
// provider: 'local'|'webdav'|'gdrive'|'onedrive'|'dropbox'|'s3'|'sftp'|
//           'icloud'|'keepassex_server'
// conflict_resolution: 'merge'|'keepLocal'|'keepRemote'|'askUser'
```

### `sync_now` → `SyncResultDto`

Perform a manual sync. Uploads local vault, downloads remote, merges if needed.

```typescript
// Returns: { status: string; entries_uploaded: number; entries_downloaded: number;
//            conflicts: number; duration_ms: number; error?: string }
```

### `test_sync_connection` → `boolean`

Test provider connectivity without syncing.

```typescript
{ provider: string; remote_path: string; username?: string;
  password?: string; token?: string; server_url?: string }
```

---

## Clipboard Commands

### `copy_to_clipboard` → `void`

Copy text with auto-clear after delay.

```typescript
{ text: string; clear_after_seconds?: number }
```

### `clear_clipboard` → `void`

---

## SSH Agent Commands

### `start_ssh_agent` → `string` (socket path)

### `stop_ssh_agent` → `void`

### `add_ssh_key` → `void`

```typescript
{ entry_uuid: string; duration_seconds?: number }
```

### `list_ssh_keys` → `SshKeyDto[]`

---

## Settings Commands

### `get_settings` → `AppSettings`

### `save_settings` → `void`

```typescript
{
  settings: AppSettings;
}
```

---

## Backup Commands

### `get_backup_config` → `BackupConfig`

### `save_backup_config` → `void`

### `backup_now` → `string` (backup file path)

### `list_backups_cmd` → `BackupEntry[]`

### `restore_from_backup_cmd` → `void`

```typescript
{
  backup_path: string;
}
```

### `delete_backup_cmd` → `void`

```typescript
{
  backup_path: string;
}
```

---

## Audit Log Commands

### `get_audit_log` → `AuditEvent[]`

```typescript
{ limit?: number; event_type?: string }
```

### `clear_audit_log` → `void`

### `export_audit_log` → `string` (file path)

### `record_audit_event` → `void`

```typescript
{ event_type: string; entry_uuid?: string; details?: string }
```

---

## Password Policy Commands

### `get_password_policies` → `PasswordPolicy[]`

### `set_policy_enabled` → `void`

```typescript
{
  policy_id: string;
  enabled: boolean;
}
```

### `evaluate_password_policies` → `PolicyViolation[]`

```typescript
{ password: string; entry_uuid?: string }
```

### `check_password_strength` → `StrengthResult`

```typescript
{
  password: string;
}
// Returns: { score: 0|1|2|3|4; entropy: number; strength_label: string }
```

---

## Vault Compare Commands

### `compare_vaults_cmd` → `VaultDiff`

```typescript
{
  vault2_path: string;
  vault2_password: string;
}
```

### `merge_vaults_cmd` → `void`

```typescript
{
  vault2_path: string;
  vault2_password: string;
  strategy: string;
}
```

---

## Analytics Commands

### `get_vault_analytics` → `VaultAnalytics`

### `export_analytics_report` → `string` (file path)

---

## Search Commands

### `nl_search` → `EntryDto[]`

Natural language search (EN/VI).

```typescript
{
  query: string;
}
// Examples: "find weak passwords", "tìm mật khẩu yếu", "expired entries in Banking"
```

### `parse_search_query` → `ParsedQuery`

```typescript
{
  query: string;
}
```

---

## Steganography Commands

### `detect_steg_carrier` → `StegCarrierInfo`

```typescript
{
  file_path: string;
}
// Returns: { format: string; has_vault: boolean; capacity_bytes: number }
```

### `steg_embed_vault` → `void`

```typescript
{
  carrier_path: string;
  vault_path: string;
  output_path: string;
  steg_password: string;
}
```

### `steg_extract_vault` → `string` (extracted vault path)

```typescript
{
  carrier_path: string;
  output_path: string;
  steg_password: string;
}
```

---

## Team Vault Commands

### `get_team_vault` → `TeamVault`

### `invite_team_member` → `void`

```typescript
{
  email: string;
  name: string;
  role: 'admin' | 'editor' | 'viewer';
}
```

### `change_team_member_role` → `void`

```typescript
{
  member_id: string;
  role: string;
}
```

### `remove_team_member` → `void`

```typescript
{
  member_id: string;
}
```

### `set_entry_permission` → `void`

```typescript
{
  entry_uuid: string;
  member_id: string;
  permission: string;
}
```

---

## Password Advisor Commands

### `advise_password_strength` → `PasswordAdvice`

Context-aware password analysis with EN/VI recommendations.

```typescript
{
  password: string;
  entry_title: string;
  entry_url: string;
  category: string;
}
```

---

## Field Reference Commands

### `resolve_entry_refs` → `ResolvedEntry`

Resolve all `{REF:...}` placeholders in an entry.

```typescript
{
  uuid: string;
}
```

### `resolve_ref_string` → `string`

```typescript
{
  ref_string: string;
}
```

### `build_field_ref` → `string`

```typescript
{
  field_code: string;
  entry_uuid: string;
}
// field_code: 'T'|'U'|'P'|'A'|'N'|'I'
```

### `check_has_refs` → `boolean`

```typescript
{
  text: string;
}
```

---

## Favicon Commands

### `fetch_entry_favicon` → `FaviconResult`

```typescript
{
  entry_uuid: string;
}
```

### `fetch_all_favicons` → `number` (count updated)

### `get_domain_from_url` → `string`

```typescript
{
  url: string;
}
```

---

## PQC (Post-Quantum Crypto) Commands

### `migrate_to_pqc` → `void`

Migrate vault to X25519 + Kyber-768 hybrid encryption.

### `downgrade_from_pqc` → `void`

Revert to classical-only encryption.

### `check_pqc_status` → `boolean`

Returns `true` if vault uses PQC hybrid encryption.

---

## Hardware Key Commands

### `list_hardware_keys_cmd` → `HardwareKeyInfo[]`

### `test_hardware_key_cmd` → `boolean`

```typescript
{ slot?: number }
```

### `configure_hardware_key` → `void`

```typescript
{ key_type: string; slot?: number; require_touch: boolean; label?: string }
```

### `remove_hardware_key` → `void`

### `get_hardware_key_config` → `HardwareKeyConfig | null`

---

## Attachment Commands

### `read_file_bytes` → `number[]`

```typescript
{
  path: string;
}
```

### `save_attachment` → `void`

```typescript
{
  entry_uuid: string;
  attachment_name: string;
  output_path: string;
}
```

### `add_attachment` → `void`

```typescript
{
  entry_uuid: string;
  file_path: string;
}
```

### `remove_attachment` → `void`

```typescript
{
  entry_uuid: string;
  attachment_name: string;
}
```

---

## Type Definitions

```typescript
interface VaultMetaDto {
  name: string;
  description: string;
  entry_count: number;
  group_count: number;
  path: string;
}

interface EntryDto {
  uuid: string;
  group_uuid: string;
  title: string;
  username: string;
  url: string;
  notes: string;
  icon_id: number;
  tags: string[];
  has_password: boolean;
  has_otp: boolean;
  has_passkey: boolean;
  has_ssh_key: boolean;
  has_attachments: boolean;
  is_expired: boolean;
  expiry?: string; // ISO 8601
  created_at: string; // ISO 8601
  modified_at: string; // ISO 8601
  custom_fields: CustomFieldDto[];
}

interface CustomFieldDto {
  key: string;
  value: string;
  protected: boolean;
}

interface GroupDto {
  uuid: string;
  parent_uuid?: string;
  name: string;
  notes: string;
  icon_id: number;
  is_expanded: boolean;
  entry_count: number;
  child_group_count: number;
}

interface SyncStatusDto {
  configured: boolean;
  provider?: string;
  remote_path?: string;
  auto_sync: boolean;
  last_sync?: string;
  last_sync_status?: string;
}

interface SyncResultDto {
  status: string;
  entries_uploaded: number;
  entries_downloaded: number;
  conflicts: number;
  duration_ms: number;
  error?: string;
}

interface PasswordSuggestionDto {
  password: string;
  entropy: number;
  strength_score: number; // 0–4
  rationale_en: string;
  rationale_vi: string;
  strategy: string;
}

interface PasswordAdvice {
  score: number; // 0–100
  label_en: string;
  label_vi: string;
  color: string; // hex color
  recommendations: Recommendation[];
  suggestion_en?: string;
  suggestion_vi?: string;
  appropriate_for_category: boolean;
  min_recommended_length: number;
}

interface Recommendation {
  severity: 'info' | 'warning' | 'critical';
  code: string;
  message_en: string;
  message_vi: string;
}
```
