# KeePassEx — Testing Guide

## Overview

KeePassEx has a multi-layer test strategy:

- **Rust unit tests** — core crypto, vault, OTP, generator, health, import, breach, passkey, emergency access, SSH, sync, analytics, categorizer, decoy vault, expiry engine, notifications, scheduled backup, ZKPV, steganography, team, templates, plugins, search, **AI suggestions**, cache
- **TypeScript unit tests** — i18n parity (all 10 languages), interpolation validation, UI components, stores, utilities
- **Integration tests** — planned (cross-platform vault compatibility)

---

## Running Tests

### All Tests

```bash
make test
# or:
cargo test --all --all-features && pnpm test:ts
```

### Rust Tests Only

```bash
cargo test --all --all-features

# With output
cargo test --all --all-features -- --nocapture

# Specific test module
cargo test -p keepassex-core ai_tests
cargo test -p keepassex-core vault_tests
cargo test -p keepassex-core crypto_tests

# Specific test function
cargo test -p keepassex-core ai_tests::test_banking_suggestions_are_strongest
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
pnpm --filter @keepassex/desktop test
pnpm --filter @keepassex/mobile test
```

---

## Test Structure

### Rust Tests (`packages/core/src/tests/`) — 650+ tests across 30 modules

| File                        | Tests | Coverage                                                                                        |
| --------------------------- | ----- | ----------------------------------------------------------------------------------------------- |
| `ai_tests.rs`               | 24    | AI suggestions: count, uniqueness, strength, strategies, bilingual rationale, category profiles |
| `analytics_tests.rs`        | 11    | `compute_analytics()`, strength distribution, group stats                                       |
| `audit_tests.rs`            | 15    | Audit log, all 24 event types                                                                   |
| `backup_tests.rs`           | 15    | Scheduled backup, all frequencies, edge cases                                                   |
| `breach_tests.rs`           | 6     | SHA-1, k-anonymity, offline check                                                               |
| `categorizer_tests.rs`      | 17    | `categorize_entry()`, domain matching, EN/VI labels                                             |
| `compare_tests.rs`          | 12    | Vault comparison, diff, merge                                                                   |
| `crypto_tests.rs`           | 10    | KDF, cipher, HMAC, key derivation                                                               |
| `decoy_vault_tests.rs`      | 8     | `generate_decoy_vault()`, fake entry quality                                                    |
| `emergency_access_tests.rs` | 10    | Lifecycle, revoke, manager                                                                      |
| `expiry_engine_tests.rs`    | 15    | `analyze_rotation()`, urgency levels, EN/VI messages                                            |
| `generator_tests.rs`        | 10    | Random, passphrase, pronounceable, strength scoring                                             |
| `health_tests.rs`           | 5     | Weak, reused, expired detection                                                                 |
| `import_tests.rs`           | 8     | Bitwarden, Chrome, CSV import/export                                                            |
| `kdbx_tests.rs`             | 12    | Format, XML, round-trip, KDBX 3.1                                                               |
| `new_import_tests.rs`       | 30+   | NordPass, Enpass, Dashlane, RoboForm, KeePass1                                                  |
| `notifications_tests.rs`    | 12    | `NotificationGenerator`, all notification types                                                 |
| `otp_tests.rs`              | 8     | TOTP/HOTP RFC vectors, URI parsing                                                              |
| `passkey_tests.rs`          | 8     | FIDO2 credential management                                                                     |
| `password_advisor_tests.rs` | 11    | Context-aware advice, EN/VI messages                                                            |
| `plugin_tests.rs`           | 13    | `PluginManifest`, `PluginRegistry`, capabilities                                                |
| `policy_tests.rs`           | 25    | Password policy engine, all 14 rule types                                                       |
| `scheduled_backup_tests.rs` | 11    | `is_backup_due()`, all frequencies                                                              |
| `search_tests.rs`           | 10    | `vault.search()`, `SearchQuery`, case-insensitive                                               |
| `ssh_tests.rs`              | 8     | Agent, key parsing, deduplication                                                               |
| `steg_tests.rs`             | 12    | `StegFormat::detect()`, embed/extract, wrong password                                           |
| `sync_tests.rs`             | 6     | Merge, diff, conflict resolution                                                                |
| `team_tests.rs`             | 16    | `TeamVault`, RBAC, entry overrides, comments                                                    |
| `templates_tests.rs`        | 14    | `TemplateManager`, all 12 built-in templates                                                    |
| `vault_tests.rs`            | 15    | CRUD, search, recycle bin, history                                                              |
| `zkpv_tests.rs`             | 10    | `ZkpvCommitment`, `PasswordHint`, serialization                                                 |

**Total: 650+ Rust tests across 30 modules**

Also: `packages/core/src/cache/mod.rs` contains 9 inline unit tests for LRU cache, GroupCache, VaultCache, and search cache.

### TypeScript Tests

| Package                        | File                               | Tests                                                                                                                            |
| ------------------------------ | ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `@keepassex/i18n`              | `__tests__/i18n.test.ts`           | EN baseline, VI spot checks, all 10 languages parity, no empty values (all 10), interpolation variable parity (all 10)           |
| `@keepassex/ui`                | `__tests__/StrengthMeter.test.tsx` | Component rendering                                                                                                              |
| `@keepassex/ui`                | `__tests__/components.test.tsx`    | Button, Input, HealthBadge, OtpDisplay                                                                                           |
| `@keepassex/utils`             | `__tests__/utils.test.ts`          | All utility functions                                                                                                            |
| `@keepassex/desktop`           | `__tests__/store.test.ts`          | Vault + settings stores                                                                                                          |
| `@keepassex/desktop`           | `__tests__/components.test.tsx`    | CustomFieldEditor, SearchBar, EntryRow                                                                                           |
| `@keepassex/desktop`           | `__tests__/pages.test.tsx`         | WelcomePage, UnlockPage, GeneratorPage, SettingsPage, BreachPage, AuditLogPage, BackupPage, VaultComparePage, PasswordPolicyPage |
| `@keepassex/mobile`            | `__tests__/screens.test.tsx`       | VaultScreen, HealthScreen, GeneratorScreen, UnlockScreen, SettingsScreen                                                         |
| `@keepassex/browser-extension` | `__tests__/background.test.ts`     | URL extraction, form detection, credential filling, HTML escaping                                                                |

**Total: 150+ TypeScript tests**

---

## Coverage Targets

| Layer            | Target | Current                                        |
| ---------------- | ------ | ---------------------------------------------- |
| Rust core        | 80%    | ~85% (30/30 modules covered)                   |
| TypeScript utils | 90%    | ~90%                                           |
| i18n             | 100%   | 100% (all 10 languages parity + interpolation) |
| UI components    | 60%    | ~60%                                           |

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

    let uuid = vault.create_entry(root).unwrap();
    let entry = vault.get_entry(&uuid).unwrap();

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

When adding new translation keys, the i18n parity test automatically catches missing keys in all 10 languages:

```bash
pnpm --filter @keepassex/i18n test
# Will fail if any language is missing keys or has extra keys
```

---

## Security-Sensitive Tests

For tests involving cryptographic operations:

1. **Never use real passwords** — use `"test_password"` or similar
2. **Use small Argon2 parameters** for speed: `memory_kb: 1024, iterations: 1`
3. **Never commit real vault files** — use generated test vaults
4. **Zeroize test secrets** after use

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

## Troubleshooting

**Rust tests fail with "cannot find crate"**

```bash
cargo build -p keepassex-core
cargo test --all
```

**TypeScript tests fail with "Cannot find module"**

```bash
pnpm install
pnpm build --filter='./packages/*'
```

**i18n parity test fails**

```
Error: German (de) is missing 2 keys:
  - ai.suggestions.title
  - ai.suggestions.rationale
```

Add the missing keys to `packages/i18n/src/locales/de.ts` (and all other non-EN locales).

**AI tests fail with "generator not found"**

```bash
cargo build -p keepassex-core  # Ensure generator module is compiled
```
