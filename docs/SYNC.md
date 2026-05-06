# KeePassEx — Sync Guide

## Overview

KeePassEx supports syncing your vault across devices using multiple providers. All sync is **optional** — KeePassEx works fully offline.

**Security guarantee**: The vault file is always encrypted before sync. Your master password never leaves your device.

---

## Supported Providers

| Provider | Desktop | iOS | Android | Notes |
|----------|---------|-----|---------|-------|
| WebDAV | ✅ | ✅ | ✅ | Nextcloud, ownCloud, etc. |
| iCloud Drive | ✅ macOS | ✅ | ❌ | Apple ecosystem |
| Google Drive | ✅ | ✅ | ✅ | Requires OAuth |
| OneDrive | ✅ | ✅ | ✅ | Requires OAuth |
| Dropbox | ✅ | ✅ | ✅ | Requires OAuth |
| Amazon S3 | ✅ | ❌ | ❌ | Power users |
| SFTP | ✅ | ❌ | ❌ | Self-hosted |
| Local Folder | ✅ | ❌ | ❌ | USB/network share |

---

## Setup

### Desktop (Tauri)

1. Open **Settings → Sync** (or navigate to `/sync`)
2. Select your provider
3. Enter the remote path (e.g., `/keepassex/vault.kdbx`)
4. For WebDAV: enter server URL, username, password
5. Click **Test Connection** to verify
6. Click **Save Configuration**
7. Click **Sync Now** to perform the first sync

### Mobile (iOS/Android)

1. Open **Settings → Sync**
2. Select provider
3. Configure credentials
4. Enable **Auto Sync** to sync on open/close

### CLI

```bash
# Push local vault to remote
kpx --vault vault.kdbx sync --provider local --remote /backup/vault.kdbx --direction push

# Pull from remote
kpx --vault vault.kdbx sync --provider local --remote /backup/vault.kdbx --direction pull

# Merge (recommended)
kpx --vault vault.kdbx sync --provider local --remote /backup/vault.kdbx --direction merge
```

---

## WebDAV Setup

### Nextcloud
```
Server URL: https://your-nextcloud.com/remote.php/dav/files/USERNAME/
Remote path: /keepassex/vault.kdbx
Username: your-nextcloud-username
Password: your-nextcloud-password (or app password)
```

### ownCloud
```
Server URL: https://your-owncloud.com/remote.php/webdav/
Remote path: /keepassex/vault.kdbx
```

### Generic WebDAV
```
Server URL: https://dav.example.com/
Remote path: /vault.kdbx
```

---

## Conflict Resolution

When both local and remote vaults have changed since the last sync:

| Strategy | Behavior |
|----------|----------|
| **Merge** (default) | CRDT-inspired: keeps newest modification per entry |
| **Keep Local** | Overwrites remote with local version |
| **Keep Remote** | Overwrites local with remote version |
| **Ask Me** | Prompts user to choose |

### Merge Algorithm

KeePassEx uses a last-write-wins strategy per entry:
- For each entry UUID, the version with the newer `modified_at` timestamp wins
- New entries from both sides are kept
- Deleted entries (moved to recycle bin) are preserved

---

## Auto Sync

When enabled, KeePassEx syncs:
- When the vault is opened
- When the vault is closed/locked
- Periodically (configurable: 1, 5, 15, 60 minutes)

---

## Troubleshooting

**"Connection failed"**
- Check server URL (must include protocol: `https://`)
- Verify credentials
- Check firewall/VPN settings

**"Sync conflict"**
- Use **Merge** strategy for automatic resolution
- Or manually choose which version to keep

**"File not found"**
- Ensure the remote path exists
- Create the directory on the server first
