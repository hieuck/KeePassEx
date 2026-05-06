# KeePassEx CLI — Scripting & CI/CD Guide

## Overview

The `kpx` CLI is designed for scripting and automation. It supports:
- Environment variables for credentials (avoid shell history)
- JSON output for machine-readable results
- Pipe-friendly design
- Exit codes for error handling

---

## Environment Variables

```bash
export KPX_VAULT=/path/to/vault.kdbx
export KPX_PASSWORD=your-master-password  # Use with caution in CI
export KPX_KEY_FILE=/path/to/keyfile.keyx
```

---

## Basic Scripting

### Get a password and use it
```bash
# Get password for an entry
PASSWORD=$(kpx --vault vault.kdbx get abc123 --field password)

# Use in a command
curl -u "user:$PASSWORD" https://api.example.com/endpoint
```

### Copy password to clipboard
```bash
kpx get abc123 --field password --copy
```

### Get OTP code
```bash
OTP=$(kpx otp abc123)
echo "Current OTP: $OTP"
```

---

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/deploy.yml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install kpx
        run: |
          curl -L https://github.com/keepassex/keepassex/releases/latest/download/kpx-linux-x64 \
            -o /usr/local/bin/kpx
          chmod +x /usr/local/bin/kpx

      - name: Get deployment credentials
        env:
          KPX_VAULT: ${{ secrets.KPX_VAULT_PATH }}
          KPX_PASSWORD: ${{ secrets.KPX_MASTER_PASSWORD }}
        run: |
          # Get API key from vault
          API_KEY=$(kpx get "Production API Key" --field password)
          echo "::add-mask::$API_KEY"
          echo "API_KEY=$API_KEY" >> $GITHUB_ENV

      - name: Deploy
        run: |
          curl -H "Authorization: Bearer $API_KEY" \
            https://api.example.com/deploy
```

### GitLab CI

```yaml
# .gitlab-ci.yml
deploy:
  stage: deploy
  script:
    - |
      # Install kpx
      curl -L https://github.com/keepassex/keepassex/releases/latest/download/kpx-linux-x64 \
        -o /usr/local/bin/kpx
      chmod +x /usr/local/bin/kpx

      # Get credentials from vault
      export KPX_VAULT=$CI_PROJECT_DIR/vault.kdbx
      export KPX_PASSWORD=$VAULT_PASSWORD

      DB_PASSWORD=$(kpx get "Production Database" --field password)

      # Use credentials
      psql -h db.example.com -U admin -p "$DB_PASSWORD" -c "SELECT 1"
  variables:
    VAULT_PASSWORD: $VAULT_MASTER_PASSWORD
```

---

## Automation Scripts

### Rotate all expired passwords
```bash
#!/bin/bash
# rotate-expired.sh

export KPX_VAULT=/path/to/vault.kdbx
export KPX_PASSWORD="$1"  # Pass as argument

# Get expired entries as JSON
EXPIRED=$(kpx health --format json | jq -r '.expired_entries[].entry_uuid')

for UUID in $EXPIRED; do
  TITLE=$(kpx get "$UUID" --field title)
  echo "Rotating password for: $TITLE"

  # Generate new password
  NEW_PASS=$(kpx generate --length 24)

  # Update entry (requires interactive confirmation in current version)
  echo "New password: $NEW_PASS"
done
```

### Export and backup vault
```bash
#!/bin/bash
# backup-vault.sh

VAULT_PATH="$1"
BACKUP_DIR="$2"
DATE=$(date +%Y%m%d_%H%M%S)

# Create encrypted backup (copy the KDBX file)
cp "$VAULT_PATH" "$BACKUP_DIR/vault_backup_$DATE.kdbx"

# Also export to CSV for emergency access (store securely!)
kpx --vault "$VAULT_PATH" export \
  --output "$BACKUP_DIR/vault_export_$DATE.csv" \
  --format csv

echo "Backup created: vault_backup_$DATE.kdbx"
echo "WARNING: CSV export contains unencrypted passwords!"
```

### Check for breached passwords in CI
```bash
#!/bin/bash
# check-breaches.sh

export KPX_VAULT=/path/to/vault.kdbx
export KPX_PASSWORD="$VAULT_PASSWORD"

# Run offline breach check
RESULT=$(kpx breach --format json)
BREACHED=$(echo "$RESULT" | jq '.breached_count')

if [ "$BREACHED" -gt 0 ]; then
  echo "⚠️ WARNING: $BREACHED breached password(s) found!"
  echo "$RESULT" | jq '.breached[] | .title'
  exit 1
else
  echo "✅ No breached passwords found"
fi
```

### Import from multiple sources
```bash
#!/bin/bash
# migrate-to-keepassex.sh

VAULT="new_vault.kdbx"
PASSWORD="$1"

# Create new vault
kpx --vault "$VAULT" --password "$PASSWORD" create --name "My Vault"

# Import from Bitwarden
kpx --vault "$VAULT" --password "$PASSWORD" \
  import bitwarden_export.json --format bitwarden

# Import from Chrome
kpx --vault "$VAULT" --password "$PASSWORD" \
  import chrome_passwords.csv --format chrome

# Import from LastPass
kpx --vault "$VAULT" --password "$PASSWORD" \
  import lastpass_export.csv --format lastpass

echo "Migration complete!"
kpx --vault "$VAULT" --password "$PASSWORD" stats
```

---

## JSON Output

All commands support `--format json` for machine-readable output:

```bash
# List entries as JSON
kpx list --format json | jq '.[].title'

# Health report as JSON
kpx health --format json | jq '.score'

# Stats as JSON
kpx stats --format json | jq '.total_entries'

# Breach check as JSON
kpx breach --format json | jq '.breached_count'
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Vault not found |
| 3 | Wrong credentials |
| 4 | Entry not found |
| 5 | Vault corrupted |

```bash
kpx get abc123 --field password
if [ $? -ne 0 ]; then
  echo "Failed to get password"
  exit 1
fi
```

---

## Security Best Practices

1. **Never put passwords in shell history:**
   ```bash
   # BAD
   kpx --password "mypassword" list

   # GOOD
   export KPX_PASSWORD="mypassword"
   kpx list

   # BEST (prompt interactively)
   kpx list  # Will prompt for password
   ```

2. **Use key files in CI:**
   ```bash
   # Store key file as CI secret, not the password
   echo "$KEY_FILE_BASE64" | base64 -d > /tmp/vault.keyx
   kpx --vault vault.kdbx --key-file /tmp/vault.keyx list
   rm /tmp/vault.keyx
   ```

3. **Mask secrets in CI logs:**
   ```bash
   # GitHub Actions
   PASSWORD=$(kpx get abc123 --field password)
   echo "::add-mask::$PASSWORD"

   # GitLab CI
   PASSWORD=$(kpx get abc123 --field password)
   # Use masked variables in .gitlab-ci.yml
   ```

4. **Clean up after use:**
   ```bash
   # Unset environment variables when done
   unset KPX_PASSWORD
   unset KPX_VAULT
   ```
