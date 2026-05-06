# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.x.x   | ✅ Active |

## Reporting a Vulnerability

**Please do NOT report security vulnerabilities through public GitHub issues.**

### Contact

Email: **security@keepassex.app**

PGP Key: Available at https://keepassex.app/.well-known/security.txt

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (optional)

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial assessment**: Within 7 days
- **Fix timeline**: Depends on severity (critical: 7 days, high: 30 days)
- **Disclosure**: Coordinated with reporter

## Security Model

KeePassEx is designed with these security guarantees:

### Zero-Knowledge
- Master password **never leaves the device**
- No telemetry, no analytics, no cloud accounts required
- All encryption/decryption happens locally

### Cryptography
- **KDF**: Argon2id (memory-hard, resistant to GPU/ASIC attacks)
- **Cipher**: ChaCha20-Poly1305 (authenticated encryption)
- **Integrity**: HMAC-SHA256 per block (tamper detection)
- **Key derivation**: SHA-256 composite key

### Memory Protection
- Sensitive strings use `ProtectedString` with `zeroize` on drop
- Passwords are never stored in plain `String` long-term
- Clipboard auto-clears after configurable timeout (default: 10s)

### Platform Security
- **iOS**: Keychain for biometric-protected master key
- **Android**: Android Keystore + BiometricPrompt
- **Desktop**: OS keychain (Keychain/DPAPI/libsecret)
- **Mobile**: Screen capture protection enabled by default

### KDBX Format
- Full KDBX 4.x support with all security features
- HMAC-authenticated blocks prevent silent corruption
- Inner stream encryption for in-memory field protection

## Known Limitations

- Browser extension requires native messaging host (desktop app must be running)
- Watch apps require paired phone for vault access
- Offline breach check uses a limited wordlist; online check (HIBP) is more comprehensive
