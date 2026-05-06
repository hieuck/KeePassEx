# KeePassEx — Troubleshooting Guide

## Common Issues

---

## Desktop (Tauri)

### "Vault file not found"
**Cause:** The vault file was moved, renamed, or deleted.

**Fix:**
1. Click "Open Vault" and navigate to the new location
2. The vault will be added to recent vaults at the new path

---

### "Wrong master password or key file"
**Cause:** Incorrect password, wrong key file, or corrupted vault.

**Fix:**
1. Verify you're using the correct master password (case-sensitive)
2. If using a key file, ensure you're selecting the correct `.keyx` file
3. Try opening the vault in KeePassXC to verify it's not corrupted

---

### "Vault file appears to be corrupted"
**Cause:** File was truncated, partially written, or disk error.

**Fix:**
1. Check if a `.kdbx.tmp` file exists — this is an incomplete save
2. Restore from backup (KeePassEx creates `.bak` files during sync)
3. Try opening with KeePassXC for additional diagnostics

---

### Browser extension not connecting
**Cause:** Native messaging host not registered, or desktop app not running.

**Fix:**
1. Ensure KeePassEx desktop app is running
2. Enable "Browser Integration" in Settings → Integration
3. Reinstall the browser extension
4. Check browser console for errors (F12 → Console)

**Manual registration (macOS):**
```bash
# Chrome
cp apps/desktop/src-tauri/native-messaging/com.keepassex.app.json \
   ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts/

# Firefox
cp apps/desktop/src-tauri/native-messaging/com.keepassex.app.firefox.json \
   ~/Library/Application\ Support/Mozilla/NativeMessagingHosts/
```

---

### Auto-Type not working
**Cause:** Window matching failed, or accessibility permissions not granted.

**Fix (macOS):**
1. System Preferences → Security & Privacy → Accessibility
2. Add KeePassEx to the allowed apps list

**Fix (Linux):**
```bash
# Install xdotool
sudo apt install xdotool  # Ubuntu/Debian
sudo pacman -S xdotool    # Arch
```

---

### System tray icon not showing (Linux)
**Cause:** Missing libappindicator.

**Fix:**
```bash
sudo apt install libappindicator3-1  # Ubuntu/Debian
sudo pacman -S libappindicator-gtk3  # Arch
```

---

### "Failed to save vault"
**Cause:** Disk full, permission denied, or file locked.

**Fix:**
1. Check available disk space
2. Verify write permissions on the vault directory
3. Ensure no other app has the vault file locked
4. Check if antivirus is blocking the write

---

## Mobile (iOS)

### AutoFill not appearing
**Cause:** AutoFill extension not enabled in iOS Settings.

**Fix:**
1. Settings → Passwords → AutoFill Passwords
2. Enable "KeePassEx"
3. Disable other password managers to avoid conflicts

---

### Biometric unlock not working
**Cause:** Face ID/Touch ID not enrolled, or permission denied.

**Fix:**
1. Settings → Face ID & Passcode → Other Apps → Enable KeePassEx
2. Re-enroll Face ID if needed

---

### Widget not updating
**Cause:** Widget refresh interval or background app refresh disabled.

**Fix:**
1. Settings → General → Background App Refresh → Enable KeePassEx
2. Long-press widget → Edit Widget to force refresh

---

## Mobile (Android)

### AutoFill not working
**Cause:** AutoFill service not enabled.

**Fix:**
1. Settings → System → Languages & Input → Autofill service
2. Select "KeePassEx"

---

### Biometric unlock not working
**Cause:** Biometrics not enrolled or app permission denied.

**Fix:**
1. Settings → Biometrics and Security → Fingerprints/Face Recognition
2. Enroll biometrics if not done
3. Settings → Apps → KeePassEx → Permissions → Enable Biometrics

---

### Quick Settings tile not appearing
**Cause:** Tile not added to Quick Settings panel.

**Fix:**
1. Pull down notification shade → Edit (pencil icon)
2. Find "KeePassEx" tile and drag to active tiles

---

## CLI (`kpx`)

### "No vault specified"
```bash
# Fix: use --vault flag or set environment variable
export KPX_VAULT=/path/to/vault.kdbx
kpx list

# Or per-command:
kpx --vault /path/to/vault.kdbx list
```

---

### "Failed to open vault: Invalid KDBX signature"
**Cause:** File is not a valid KDBX file.

**Fix:**
```bash
# Check file type
file vault.kdbx

# Should output: vault.kdbx: Keepass password database 2.x KDBX
```

---

### OTP command shows "Entry has no OTP configured"
**Cause:** The entry doesn't have TOTP set up.

**Fix:**
```bash
# List entries with OTP
kpx list | grep -i otp

# Or check entry details
kpx get <uuid>
```

---

### Breach check fails with network error
**Cause:** No internet connection or HIBP API unavailable.

**Fix:**
```bash
# Use offline mode
kpx breach  # offline by default

# Online mode requires internet
kpx breach --online
```

---

## Browser Extension

### "KeePassEx not running"
**Cause:** Desktop app is not open.

**Fix:** Open KeePassEx desktop app before using the extension.

---

### "Vault is locked"
**Cause:** Vault was locked due to idle timeout.

**Fix:** Click "Unlock Vault" in the extension popup, or unlock in the desktop app.

---

### Extension popup is blank
**Cause:** Extension failed to load, or content security policy issue.

**Fix:**
1. Reload the extension: `chrome://extensions` → Reload
2. Check browser console for errors
3. Reinstall the extension

---

## Build Issues

### "cargo: command not found"
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### "pnpm: command not found"
```bash
npm install -g pnpm
```

### Desktop build fails on Linux: "webkit2gtk not found"
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

### iOS build fails: "CocoaPods not found"
```bash
sudo gem install cocoapods
cd apps/mobile/ios && pod install
```

### Android build fails: "SDK not found"
```bash
# Set ANDROID_HOME
export ANDROID_HOME=$HOME/Android/Sdk
export PATH=$PATH:$ANDROID_HOME/tools:$ANDROID_HOME/platform-tools
```

---

## Getting Help

1. **Search existing issues**: https://github.com/keepassex/keepassex/issues
2. **Open a new issue**: Include OS, version, and steps to reproduce
3. **Security issues**: Email security@keepassex.app (do NOT open public issues)
4. **Community**: GitHub Discussions

### Collecting Debug Information

```bash
# Desktop version
kpx --version

# System info
uname -a  # Linux/macOS
winver     # Windows

# Rust version
rustc --version

# Node version
node --version
pnpm --version
```
