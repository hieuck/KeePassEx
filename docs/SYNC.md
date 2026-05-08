# KeePassEx — Sync Guide

## Overview

KeePassEx supports syncing your vault across devices using 9 providers. All sync is **optional** — KeePassEx works fully offline.

**Security guarantee**: The vault file is always encrypted before sync. Your master password never leaves your device.

---

## Supported Providers

| Provider             | Desktop  | iOS | Android | Notes                                            |
| -------------------- | -------- | --- | ------- | ------------------------------------------------ |
| **KeePassEx Server** | ✅       | ✅  | ✅      | Self-hosted, zero-knowledge, real-time WebSocket |
| WebDAV               | ✅       | ✅  | ✅      | Nextcloud, ownCloud, etc.                        |
| iCloud Drive         | ✅ macOS | ✅  | ❌      | Apple ecosystem                                  |
| Google Drive         | ✅       | ✅  | ✅      | Requires OAuth2                                  |
| OneDrive             | ✅       | ✅  | ✅      | Requires OAuth2                                  |
| Dropbox              | ✅       | ✅  | ✅      | Requires OAuth2                                  |
| Amazon S3            | ✅       | ❌  | ❌      | S3-compatible (MinIO, Backblaze B2)              |
| SFTP                 | ✅       | ❌  | ❌      | SSH File Transfer Protocol                       |
| Local Folder         | ✅       | ❌  | ❌      | USB/network share                                |

---

## KeePassEx Server (Recommended) 🆕

> **No competitor has this.** KeePass/KeePassXC have no server. Bitwarden has one but requires complex setup.

The KeePassEx Server is a self-hosted, zero-knowledge sync server. The server stores only encrypted vault blobs — it cannot read your passwords.

### Quick Start (Docker)

```bash
# One-command setup
docker compose -f apps/server/docker-compose.yml up -d

# Check health
curl http://localhost:8080/health
# → {"status":"ok","version":"1.0.0"}
```

### Manual Setup

```bash
# Build and run
cargo build --release -p keepassex-server
./target/release/keepassex-server \
  --port 8080 \
  --db ./keepassex.db \
  --jwt-secret "$(openssl rand -hex 32)"
```

### Environment Variables

| Variable            | Default          | Description                                     |
| ------------------- | ---------------- | ----------------------------------------------- |
| `KPX_PORT`          | `8080`           | HTTP port                                       |
| `KPX_HOST`          | `0.0.0.0`        | Bind address                                    |
| `KPX_DB`            | `./keepassex.db` | SQLite database path                            |
| `KPX_JWT_SECRET`    | (random)         | JWT signing secret — **set this in production** |
| `KPX_MAX_VAULT_MB`  | `100`            | Maximum vault size in MB                        |
| `KPX_ADMIN_ENABLED` | `false`          | Enable admin API                                |
| `KPX_ADMIN_KEY`     | —                | Admin API key (required if admin enabled)       |

### Connecting from KeePassEx Desktop

1. Open **Settings → Sync**
2. Select **KeePassEx Server** (shown first, highlighted)
3. Enter your server URL: `https://your-server.example.com`
4. Click **Sign In** or **Create Account**
5. Enter email + password
6. Click **Connect** — you'll see "✅ Connected to https://..."
7. Click **Save Configuration**
8. Click **Sync Now** for the first sync

### API Endpoints

```
POST /api/v1/auth/register    — Create account
POST /api/v1/auth/login       — Get JWT token
POST /api/v1/auth/refresh     — Refresh token
POST /api/v1/auth/logout      — Invalidate token

GET  /api/v1/vault            — Get vault metadata
PUT  /api/v1/vault            — Upload encrypted vault
GET  /api/v1/vault/download   — Download vault
GET  /api/v1/vault/history    — List vault versions (last 10)
GET  /api/v1/vault/history/:v — Download specific version

GET  /ws?token=<jwt>          — WebSocket real-time sync

GET  /health                  — Server health check
GET  /api/v1/server/info      — Server version info

# Admin (requires KPX_ADMIN_ENABLED=true + X-Admin-Key header)
GET  /api/v1/admin/users      — List users
DEL  /api/v1/admin/users/:id  — Delete user
GET  /api/v1/admin/stats      — Server statistics
```

### Security Model

```
Client                          KeePassEx Server
  │                                    │
  │  vault.kdbx (Argon2id encrypted)   │
  │ ─────────────────────────────────► │  ← Server stores only ciphertext
  │                                    │  ← Server CANNOT decrypt
  │  vault.kdbx (same ciphertext)      │
  │ ◄───────────────────────────────── │
  │                                    │
  │  Decrypt locally with master key   │
```

- JWT tokens are signed with `KPX_JWT_SECRET` (Argon2id-hashed passwords)
- Vault data is opaque bytes to the server
- Version history: last 10 uploads retained per user
- No email verification required (self-hosted, you control the users)

### Kubernetes Deployment

```bash
helm install keepassex-server apps/server/helm/keepassex-server/ \
  --set jwtSecret="$(openssl rand -hex 32)" \
  --set persistence.size=10Gi
```

---

## WebDAV Setup

### Nextcloud

```
Server URL:  https://your-nextcloud.com/remote.php/dav/files/USERNAME/
Remote path: /keepassex/vault.kdbx
Username:    your-nextcloud-username
Password:    your-nextcloud-password (or app password)
```

### ownCloud

```
Server URL:  https://your-owncloud.com/remote.php/webdav/
Remote path: /keepassex/vault.kdbx
```

### Generic WebDAV

```
Server URL:  https://dav.example.com/
Remote path: /vault.kdbx
```

---

## Google Drive Setup

1. Select **Google Drive** provider
2. Click **Authorize with Google**
3. Complete OAuth2 flow in browser
4. Set remote path: `vault.kdbx` (filename in Drive root)
5. Click **Test Connection** → **Save**

---

## Amazon S3 Setup

```
Bucket:          my-keepassex-bucket
Region:          us-east-1
Access Key ID:   AKIAIOSFODNN7EXAMPLE
Secret Key:      wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
Remote path:     vault.kdbx
Custom Endpoint: (leave empty for AWS, or set for MinIO/Backblaze)
```

**MinIO example:**

```
Custom Endpoint: http://minio.example.com:9000
```

**Backblaze B2:**

```
Custom Endpoint: https://s3.us-west-004.backblazeb2.com
```

---

## SFTP Setup

```
Host:         sftp.example.com
Port:         22
Username:     your-username
Password:     your-password (or leave empty for key auth)
Remote path:  /home/user/keepassex/vault.kdbx
```

---

## Conflict Resolution

When both local and remote vaults have changed since the last sync:

| Strategy            | Behavior                                                |
| ------------------- | ------------------------------------------------------- |
| **Merge** (default) | CRDT-inspired: keeps newest modification per entry UUID |
| **Keep Local**      | Overwrites remote with local version                    |
| **Keep Remote**     | Overwrites local with remote version                    |
| **Ask Me**          | Prompts user to choose                                  |

### Merge Algorithm

```
Local vault entries:  {A@t=10, B@t=5,  C@t=8}
Remote vault entries: {A@t=7,  B@t=12, D@t=3}

Merged result:        {A@t=10, B@t=12, C@t=8, D@t=3}
                       ↑local   ↑remote  ↑local  ↑remote
```

- For each entry UUID, the version with the newer `modified_at` wins
- New entries from both sides are kept
- Deleted entries (moved to recycle bin) are preserved
- Group structure: last-write-wins per group UUID

---

## Auto Sync

When enabled, KeePassEx syncs:

- When the vault is opened
- When the vault is closed/locked
- Periodically (configurable: 1, 5, 15, 60 minutes)

**KeePassEx Server** additionally supports real-time push notifications via WebSocket — when another device uploads a new vault version, all connected clients are notified immediately.

---

## CLI Sync

```bash
# Push local vault to remote (local folder)
kpx --vault vault.kdbx sync \
  --provider local \
  --remote /backup/vault.kdbx \
  --direction push

# Pull from WebDAV
kpx --vault vault.kdbx sync \
  --provider webdav \
  --remote https://dav.example.com/vault.kdbx \
  --direction pull

# Merge (recommended)
kpx --vault vault.kdbx sync \
  --provider local \
  --remote /backup/vault.kdbx \
  --direction merge

# Sync with KeePassEx Server
kpx server login --url https://kpx.example.com --email user@example.com
kpx --vault vault.kdbx sync \
  --provider keepassex_server \
  --remote https://kpx.example.com \
  --direction merge
```

---

## Troubleshooting

**"Connection failed"**

- Check server URL (must include protocol: `https://`)
- Verify credentials
- Check firewall/VPN settings
- For KeePassEx Server: check `docker logs keepassex-server`

**"Sync conflict"**

- Use **Merge** strategy for automatic resolution
- Or manually choose which version to keep

**"File not found"**

- Ensure the remote path exists
- Create the directory on the server first
- For KeePassEx Server: the server creates the vault on first upload

**"JWT expired"**

- Re-authenticate in Settings → Sync → KeePassEx Server → Sign In

**"Vault too large"**

- Increase `KPX_MAX_VAULT_MB` on the server
- Or reduce vault size by removing large attachments
