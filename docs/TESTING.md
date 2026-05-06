# KeePassEx — Testing Guide

## Overview

KeePassEx has a multi-layer test strategy:
- **Rust unit tests** — core crypto, vault, OTP, generator, health, import, breach, passkey, emergency access, SSH, sync
- **TypeScript unit tests** — i18n parity, UI components, stores, utilities
- **Integration tests** — planned (cross-platform vault compatibility)

---

## Running Tests

### All Tests
```bash
make test
# or:
pnpm test:rust && pnpm test:ts
```

### Rust Tests Only
```bash
cargo test --all --all-features

# With output
cargo test --all --all-features -- --nocapture

# Specific test
cargo test vault_tests::test_create_vault

# Specific module
cargo test -p keepassex-core otp_tests
```

### TypeScript Tests Only
```bash
pnpm test:ts

# Watch mode
pnpm --filter '*' vitest

# With coverage
pnpm test:ts -- --coverage

# Specific package
pnpm --filter @keepassex/i18n test
pnpm --filter @keepassex/ui test
pnpm --filter @keepassex/utils test
```

---

## Test Structure

### Rust Tests (`packages/core/src/tests/`)

| File | Tests | Coverage |
|------|-------|----------|
| `vault_tests.rs` | 15 | CRUD, search, recycle bin, history |
| `crypto_tests.rs` | 10 | KDF, cipher, HMAC, key derivation |
| `otp_tests.rs` | 8 | TOTP/HOTP RFC vectors, URI parsing |
| `generator_tests.rs` | 10 | Random, passphrase, strength scoring |
| `health_tests.rs` | 5 | Weak, reused, expired detection |
| `import_tests.rs` | 8 | Bitwarden, Chrome, CSV import/export |
| `breach_tests.rs` | 6 | SHA-1, k-anonymity, offline check |
| `passkey_tests.rs` | 8 | FIDO2 credential management |
| `emergency_access_tests.rs` | 10 | Lifecycle, revoke, manager |
| `ssh_tests.rs` | 8 | Agent, key parsing, deduplication |
| `sync_tests.rs` | 6 | Merge, diff, conflict resolution |

**Total: 94+ Rust tests**

### TypeScript Tests

| Package | File | Tests |
|---------|------|-------|
| `@keepassex/i18n` | `__tests__/i18n.test.ts` | EN/VI parity, interpolation |
| `@keepassex/ui` | `__tests__/StrengthMeter.test.tsx` | Component rendering |
| `@keepassex/utils` | `__tests__/utils.test.ts` | All 13 utility functions |
| `@keepassex/desktop` | `__tests__/store.test.ts` | Vault + settings stores |

---

## Coverage Targets

| Layer | Target | Current |
|-------|--------|---------|
| Rust core | 80% | ~75% |
| TypeScript utils | 90% | ~85% |
| i18n | 100% | 100% |
| UI components | 60% | ~20% |

---

## Writing Tests

### Rust Tests

```rust
// In packages/core/src/tests/my_tests.rs
use crate::vault::Vault;

#[test]
fn test_my_feature() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    // Arrange
    let uuid = vault.create_entry(root).unwrap();

    // Act
    let entry = vault.get_entry(&uuid).unwrap();

    // Assert
    assert_eq!(entry.group_uuid, root);
}

#[tokio::test]
async fn test_async_feature() {
    // Async tests use tokio::test
}
```

Register in `packages/core/src/tests/mod.rs`:
```rust
#[cfg(test)]
mod my_tests;
```

### TypeScript Tests

```typescript
// In packages/ui/src/__tests__/MyComponent.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MyComponent } from '../components/MyComponent';

describe('MyComponent', () => {
  it('renders correctly', () => {
    render(<MyComponent value="test" />);
    expect(screen.getByText('test')).toBeTruthy();
  });
});
```

### i18n Tests

When adding new translation keys, the i18n parity test will automatically catch missing keys:

```bash
pnpm --filter @keepassex/i18n test
# Will fail if EN and VI keys don't match
```

---

## Security-Sensitive Tests

For tests involving cryptographic operations:

1. **Never use real passwords** in test fixtures — use `"test_password"` or similar
2. **Use small Argon2 parameters** for speed: `memory_kb: 1024, iterations: 1`
3. **Never commit real vault files** — use generated test vaults
4. **Zeroize test secrets** after use

Example:
```rust
#[test]
fn test_kdf_fast() {
    use crate::crypto::kdf::{KdfParams, ArgonParams, derive_master_key};

    let params = KdfParams::Argon2(ArgonParams {
        salt: vec![0u8; 32],
        iterations: 1,
        memory_kb: 1024, // Small for tests
        parallelism: 1,
        version: 19,
        secret_key: None,
        associated_data: None,
    });

    let key = derive_master_key(&[0u8; 32], &params).unwrap();
    assert_eq!(key.len(), 32);
}
```

---

## CI/CD Integration

Tests run automatically on:
- Every push to `main` or `develop`
- Every pull request

See `.github/workflows/ci.yml` for the full pipeline.

### Local Pre-commit Check
```bash
make check  # lint + typecheck + fmt-check
make test   # all tests
```

---

## Troubleshooting Tests

**Rust tests fail with "cannot find crate"**
```bash
cargo build -p keepassex-core  # Build first
cargo test --all
```

**TypeScript tests fail with "Cannot find module"**
```bash
pnpm install  # Install dependencies
pnpm build --filter='./packages/*'  # Build packages first
```

**i18n parity test fails**
```
Error: missingInVi: ["emergencyAccess.newKey"]
```
Add the missing key to `packages/i18n/src/locales/vi.ts`.
