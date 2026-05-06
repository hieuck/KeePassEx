# KeePassEx — Tauri Command API Reference

All commands are invoked from the frontend via `invoke()` from `@tauri-apps/api/core`.

---

## Vault Commands

### `open_vault`

Open an existing KDBX vault file.

**Args:**

```typescript
{
  path: string;           // Absolute path to .kdbx file
  password?: string;      // Master password
  key_file_data?: number[]; // Key file bytes (optional)
}
```

**Returns:** `VaultMetaDto`

---

### `create_vault`

Create a new empty KDBX 4.x vault.

**Args:**

```typescript
{ path: string; name: string; password: string; key_file_data?: number[] }
```

**Returns:** `VaultMetaDto`

---

### `save_vault`

Save the current vault to disk (atomic write).

**Returns:** `void`

---

### `lock_vault`

Lock the vault (keeps in memory, requires re-auth).

**Returns:** `void`

---

### `close_vault`

Close and unload the vault from memory.

**Returns:** `void`

---

### `change_credentials`

Change the master password.

**Args:** `{ old_password: string; new_password: string }`

**Returns:** `void`

---

### `get_vault_meta`

Get vault metadata without sensitive data.

**Returns:** `VaultMetaDto`

---

## Entry Commands

### `get_entries`

Get entries, optionally filtered by group.

**Args:** `{ group_uuid?: string }`

**Returns:** `EntryDto[]`

---

### `get_entry`

Get a single entry by UUID.

**Args:** `{ uuid: string; include_password: boolean }`

**Returns:** `EntryDto`

---

### `get_entry_password`

Get the password for an entry (explicit request required).

**Args:** `{ uuid: string }`

**Returns:** `string`

---

### `create_entry`

Create a new entry in a group.

**Args:**

```typescript
{
  group_uuid: string;
  title: string;
  username: string;
  password: string;
  url: string;
  notes: string;
  tags: string[];
  icon_id: number;
}
```

**Returns:** `string` (new entry UUID)

---

### `update_entry`

Update an existing entry (saves history snapshot).

**Args:** `UpdateEntryArgs` (same fields as create + `uuid`)

**Returns:** `void`

---

### `delete_entry`

Delete an entry (to recycle bin or permanently).

**Args:** `{ uuid: string; permanent: boolean }`

**Returns:** `void`

---

### `move_entry`

Move entry to a different group.

**Args:** `{ uuid: string; new_group_uuid: string }`

**Returns:** `void`

---

### `duplicate_entry`

Duplicate an entry with "(copy)" suffix.

**Args:** `{ uuid: string }`

**Returns:** `string` (new UUID)

---

### `search_entries`

Full-text search across all entries.

**Args:** `{ query: string }`

**Returns:** `EntryDto[]`

---

### `get_entry_history`

Get the history snapshots for an entry, newest first.

**Args:** `{ uuid: string }`

**Returns:** `EntryHistoryDto[]`

```typescript
interface EntryHistoryDto {
  uuid: string; // Synthetic: "<entry-uuid>-history-<index>"
  modified_at: string; // ISO 8601
  title: string;
  username: string;
  url: string;
  notes: string;
  has_password: boolean;
}
```

---

### `restore_entry_from_history`

Restore an entry to a previous history snapshot. The current state is saved to history before restoring.

**Args:** `{ entry_uuid: string; history_uuid: string }`

**Returns:** `void`

---

### `clear_entry_history`

Clear all history snapshots for an entry.

**Args:** `{ uuid: string }`

**Returns:** `void`

---

## Group Commands

### `get_groups` → `GroupDto[]`

### `create_group` → `string` (UUID)

### `update_group` → `void`

### `delete_group` → `void`

### `move_group` → `void`

---

## Generator Commands

### `generate_password`

Generate a password or passphrase.

**Args:** `GeneratePasswordArgs`

**Returns:** `{ password: string; entropy: number; strength_score: number; strength_label: string }`

---

### `estimate_entropy`

**Args:** `{ password: string }` → `number` (bits)

### `score_strength`

**Args:** `{ password: string }` → `number` (0–4)

---

## OTP Commands

### `generate_totp`

**Args:** `{ entry_uuid: string }`

**Returns:** `{ code: string; remaining_seconds: number; period: number; progress: number }`

---

### `parse_otp_uri`

**Args:** `{ uri: string }` → `OtpConfigDto`

---

## Health Commands

### `audit_vault`

Run full health audit.

**Returns:** `HealthReportDto`

---

## Breach Commands

### `check_vault_breaches`

Check all vault passwords against HIBP (k-anonymity).

**Args:** `{ online: boolean }`

**Returns:** `VaultBreachReport`

---

### `check_password_breach`

Check a single password.

**Args:** `{ password: string; online: boolean }`

**Returns:** `BreachCheckResult`

---

## Import/Export Commands

### `import_vault`

Import entries from an external file.

**Args:**

```typescript
{
  file_path: string;
  format?: 'bitwarden' | 'lastpass' | 'chrome' | 'firefox' | '1password' | 'csv';
  target_group_uuid?: string;
}
```

**Returns:** `ImportResultDto`

---

### `export_vault_cmd`

Export vault to CSV or JSON.

**Args:** `{ file_path: string; format: 'csv' | 'json' }`

**Returns:** `number` (bytes written)

---

### `detect_import_format`

**Args:** `{ file_path: string }` → `string` (format name)

---

## Sync Commands

### `get_sync_status` → `SyncStatusDto`

### `configure_sync` → `void`

### `sync_now` → `SyncResultDto`

### `test_sync_connection` → `boolean`

---

## Clipboard Commands

### `copy_to_clipboard`

Copy text with auto-clear.

**Args:** `{ text: string; clear_after_seconds?: number }`

**Returns:** `void`

---

### `clear_clipboard` → `void`

---

## SSH Agent Commands

### `start_ssh_agent` → `string`

### `stop_ssh_agent` → `void`

### `add_ssh_key` → `void`

### `list_ssh_keys` → `SshKeyDto[]`

---

## Settings Commands

### `get_settings` → `AppSettings`

### `save_settings` → `void` (persists to disk)

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
  expiry?: string;
  created_at: string;
  modified_at: string;
  custom_fields: CustomFieldDto[];
}
```
