# KeePassEx — Import & Export Guide

## Import

KeePassEx supports importing from **12 password managers** — more than any competitor.

### Supported Import Formats

| Format              | Source              | Notes                                                    |
| ------------------- | ------------------- | -------------------------------------------------------- |
| **Bitwarden JSON**  | Bitwarden           | Full support: folders, TOTP, custom fields, secure notes |
| **LastPass CSV**    | LastPass            | Groups, TOTP, notes, URLs                                |
| **Chrome CSV**      | Chrome, Edge, Brave | Name, URL, username, password                            |
| **Firefox CSV**     | Firefox             | URL, username, password                                  |
| **1Password 1PUX**  | 1Password           | Vaults, sections, custom fields, all item types          |
| **Dashlane JSON**   | Dashlane            | Credentials, secure notes, payment cards, IDs, OTP       |
| **NordPass CSV**    | NordPass            | Credentials, payment cards, folders                      |
| **Enpass JSON**     | Enpass              | All item types, folders, OTP, archived items excluded    |
| **RoboForm HTML**   | RoboForm            | HTML table export, folder mapping                        |
| **KeePass 1.x XML** | KeePass 1.x         | XML export from KeePass 1.x (File → Export → XML)        |
| **Generic CSV**     | Any                 | title, username, password, url, notes                    |
| **KDBX**            | KeePass, KeePassXC  | Native format — use File → Open                          |

### Auto-Detection

KeePassEx automatically detects the format from file content:

- KDBX signature bytes
- JSON structure (Bitwarden, Dashlane, Enpass, 1Password)
- XML structure (KeePass 1.x)
- HTML structure (RoboForm)
- CSV header row (Chrome, LastPass, Firefox, NordPass)

### How to Export from Other Managers

#### Bitwarden

1. Bitwarden Web → Tools → Export Vault
2. Format: **JSON (Unencrypted)**
3. Import into KeePassEx

#### LastPass

1. LastPass → Account Options → Advanced → Export
2. Save as CSV
3. Import into KeePassEx

#### Chrome / Edge / Brave

1. Chrome → Settings → Passwords → ⋮ → Export passwords
2. Save as CSV
3. Import into KeePassEx

#### Firefox

1. Firefox → Passwords → ⋮ → Export Logins
2. Save as CSV
3. Import into KeePassEx

#### 1Password

1. 1Password → File → Export → All Items
2. Format: **1PUX**
3. Import into KeePassEx

#### Dashlane

1. Dashlane → My Account → Export Data
2. Format: **JSON**
3. Import into KeePassEx

#### NordPass

1. NordPass → Settings → Export Items
2. Format: **CSV**
3. Import into KeePassEx

#### Enpass

1. Enpass → File → Export
2. Format: **JSON (.json)**
3. Import into KeePassEx

#### RoboForm

1. RoboForm → RoboForm Editor → File → Print List
2. Save as HTML
3. Import into KeePassEx

#### KeePass 1.x

1. KeePass 1.x → File → Export → XML Format
2. Save as XML
3. Import into KeePassEx

> **Note**: KeePass 1.x binary (.kdb) files use legacy encryption (RC4/Twofish).
> Export to XML first, then import the XML.

### Import Steps

**Desktop:**

1. Navigate to **Import/Export** page
2. Select format (or use Auto-detect)
3. Click **Choose File to Import**
4. Review the import summary

**CLI:**

```bash
# Auto-detect format
kpx --vault vault.kdbx import passwords.json

# Specify format
kpx --vault vault.kdbx import --format bitwarden passwords.json
kpx --vault vault.kdbx import --format lastpass lastpass_export.csv
kpx --vault vault.kdbx import --format chrome chrome_passwords.csv
kpx --vault vault.kdbx import --format dashlane dashlane_export.json
kpx --vault vault.kdbx import --format nordpass nordpass_export.csv
kpx --vault vault.kdbx import --format enpass enpass_export.json
kpx --vault vault.kdbx import --format roboform roboform_export.html
kpx --vault vault.kdbx import --format keepass1 keepass1_export.xml
```

**Mobile:**

1. Settings → Import/Export
2. Select format
3. Tap **Choose File to Import**

### Duplicate Handling

When importing, you can choose how to handle duplicate entries:

- **Skip duplicates** (default) — skip entries with the same title + username
- **Overwrite existing** — replace existing entries
- **Keep both** — import all entries, even duplicates

---

## Export

### ⚠️ Security Warning

Exported files contain **unencrypted passwords**. Handle with care:

- Delete the export file after use
- Never store it in cloud storage unencrypted
- Never email it
- Use KDBX export for secure backups

### Supported Export Formats

| Format       | Use Case                                | Encrypted |
| ------------ | --------------------------------------- | --------- |
| **KDBX 4.x** | Backup, migration to other KeePass apps | ✅ Yes    |
| **CSV**      | Spreadsheet analysis, migration         | ❌ No     |
| **JSON**     | Developer use, scripting                | ❌ No     |
| **HTML**     | Printable reference (read-only)         | ❌ No     |

### Export Steps

**Desktop:**

1. Navigate to **Import/Export** page
2. Select format (CSV, JSON, or HTML)
3. Click **Export Vault**
4. Choose save location

**CLI:**

```bash
# Export to CSV
kpx --vault vault.kdbx export --output backup.csv --format csv

# Export to JSON
kpx --vault vault.kdbx export --output backup.json --format json
```

### CSV Format

```csv
Title,Username,Password,URL,Notes,Tags,Modified
GitHub,user@example.com,SecurePass123!,https://github.com,,work,2025-01-15 10:30:00
Gmail,user@gmail.com,GmailPass456!,https://mail.google.com,Personal email,,2025-01-10 08:00:00
```

### JSON Format

```json
{
  "generator": "KeePassEx",
  "version": "1.0",
  "exported_at": "2025-01-15T10:30:00Z",
  "entries": [
    {
      "uuid": "550e8400-e29b-41d4-a716-446655440000",
      "title": "GitHub",
      "username": "user@example.com",
      "password": "SecurePass123!",
      "url": "https://github.com",
      "notes": "",
      "tags": ["work"],
      "created": "2025-01-01T00:00:00Z",
      "modified": "2025-01-15T10:30:00Z"
    }
  ]
}
```

---

## Scheduled Backup

KeePassEx includes a built-in scheduled backup feature (unique — no competitor has this):

- **Frequencies**: On every save, Daily, Weekly, Monthly
- **Destination**: Any local folder
- **Retention**: Keep last N backups (configurable)
- **Restore**: One-click restore from any backup

Configure in: Settings → Scheduled Backup

---

## Competitor Comparison

| Feature            | KeePass | KeePassXC | Keepassium | KeePass2Android | **KeePassEx** |
| ------------------ | ------- | --------- | ---------- | --------------- | ------------- |
| Import formats     | 5       | 6         | 4          | 3               | **12**        |
| Auto-detect format | ❌      | ✅        | ❌         | ❌              | ✅            |
| Dashlane import    | ❌      | ❌        | ❌         | ❌              | ✅            |
| NordPass import    | ❌      | ❌        | ❌         | ❌              | ✅            |
| Enpass import      | ❌      | ❌        | ❌         | ❌              | ✅            |
| RoboForm import    | ❌      | ❌        | ❌         | ❌              | ✅            |
| KeePass 1.x import | ✅      | ✅        | ✅         | ✅              | ✅            |
| Scheduled backup   | ❌      | ❌        | ❌         | ❌              | ✅            |
| Export to HTML     | ❌      | ❌        | ❌         | ❌              | ✅            |
