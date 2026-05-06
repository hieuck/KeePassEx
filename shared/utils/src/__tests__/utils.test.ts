/**
 * shared/utils tests — comprehensive coverage
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  extractDomain,
  isValidUrl,
  matchesUrl,
  truncate,
  formatBytes,
  formatRelativeTime,
  formatDate,
  getStrengthColor,
  isValidEmail,
  isValidOtpSecret,
  safeEqual,
  generateId,
  generateUuid,
  maskPassword,
  calculateEntropy,
  classifyPasswordStrength,
  parseOtpUri,
  buildOtpUri,
  filterEntries,
  sortEntries,
  slugify,
  debounce,
  throttle,
} from '../index';
import type { Entry } from '@keepassex/types';

// ─── URL utilities ────────────────────────────────────────────────────────────

describe('extractDomain', () => {
  it('extracts domain from https URL', () => {
    expect(extractDomain('https://github.com/user/repo')).toBe('github.com');
  });

  it('strips www prefix', () => {
    expect(extractDomain('https://www.google.com')).toBe('google.com');
  });

  it('handles http', () => {
    expect(extractDomain('http://example.com/path')).toBe('example.com');
  });

  it('returns input for invalid URL', () => {
    expect(extractDomain('not-a-url')).toBe('not-a-url');
  });

  it('handles URL without path', () => {
    expect(extractDomain('https://keepassex.app')).toBe('keepassex.app');
  });
});

describe('isValidUrl', () => {
  it('accepts valid https URL', () => {
    expect(isValidUrl('https://example.com')).toBe(true);
  });

  it('accepts valid http URL', () => {
    expect(isValidUrl('http://localhost:3000')).toBe(true);
  });

  it('rejects plain text', () => {
    expect(isValidUrl('not a url')).toBe(false);
  });

  it('rejects empty string', () => {
    expect(isValidUrl('')).toBe(false);
  });
});

// ─── String utilities ─────────────────────────────────────────────────────────

describe('truncate', () => {
  it('returns string unchanged if within limit', () => {
    expect(truncate('hello', 10)).toBe('hello');
  });

  it('truncates long string with ellipsis', () => {
    // Default suffix is '…' (1 char), so maxLength 8 → 'hello w' (7 chars) + '…' = 'hello w…'
    expect(truncate('hello world', 8)).toBe('hello w…');
  });

  it('uses custom suffix', () => {
    // Custom suffix '…' (1 char), maxLength 7 → 'hello ' (6 chars) + '…' = 'hello …'
    expect(truncate('hello world', 7, '…')).toBe('hello …');
  });

  it('handles exact length', () => {
    expect(truncate('hello', 5)).toBe('hello');
  });
});

describe('formatBytes', () => {
  it('formats bytes', () => {
    expect(formatBytes(0)).toBe('0 B');
    expect(formatBytes(512)).toBe('512 B');
  });

  it('formats kilobytes', () => {
    expect(formatBytes(1024)).toBe('1 KB');
    expect(formatBytes(2048)).toBe('2 KB');
  });

  it('formats megabytes', () => {
    expect(formatBytes(1024 * 1024)).toBe('1 MB');
  });

  it('formats gigabytes', () => {
    expect(formatBytes(1024 * 1024 * 1024)).toBe('1 GB');
  });
});

// ─── Date utilities ───────────────────────────────────────────────────────────

describe('formatRelativeTime', () => {
  it('returns "Just now" for current time', () => {
    const today = new Date();
    expect(formatRelativeTime(today, 'en')).toBe('Just now');
  });

  it('returns "Vừa xong" for current time in Vietnamese', () => {
    const today = new Date();
    expect(formatRelativeTime(today, 'vi')).toBe('Vừa xong');
  });

  it('returns "Yesterday" for yesterday', () => {
    const yesterday = new Date(Date.now() - 86_400_000);
    expect(formatRelativeTime(yesterday, 'en')).toBe('Yesterday');
  });

  it('returns days ago for recent dates', () => {
    const threeDaysAgo = new Date(Date.now() - 3 * 86_400_000);
    expect(formatRelativeTime(threeDaysAgo, 'en')).toBe('3 days ago');
  });

  it('accepts string input', () => {
    const result = formatRelativeTime(new Date().toISOString(), 'en');
    expect(result).toBe('Just now');
  });
});

describe('formatDate', () => {
  it('formats date in English', () => {
    const date = new Date('2025-01-15');
    const result = formatDate(date, 'en');
    expect(result).toContain('2025');
    expect(result).toContain('Jan');
  });

  it('formats date in Vietnamese', () => {
    const date = new Date('2025-01-15');
    const result = formatDate(date, 'vi');
    expect(result).toContain('2025');
  });
});

// ─── Color utilities ──────────────────────────────────────────────────────────

describe('getStrengthColor', () => {
  it('returns red for score 0', () => {
    expect(getStrengthColor(0)).toBe('#DC2626');
  });

  it('returns green for score 3', () => {
    expect(getStrengthColor(3)).toBe('#16A34A');
  });

  it('returns dark green for score 4', () => {
    expect(getStrengthColor(4)).toBe('#059669');
  });
});

// ─── Validation ───────────────────────────────────────────────────────────────

describe('isValidEmail', () => {
  it('accepts valid email', () => {
    expect(isValidEmail('user@example.com')).toBe(true);
    expect(isValidEmail('user+tag@sub.domain.com')).toBe(true);
  });

  it('rejects invalid email', () => {
    expect(isValidEmail('not-an-email')).toBe(false);
    expect(isValidEmail('@example.com')).toBe(false);
    expect(isValidEmail('user@')).toBe(false);
    expect(isValidEmail('')).toBe(false);
  });
});

describe('isValidOtpSecret', () => {
  it('accepts valid base32 secret', () => {
    expect(isValidOtpSecret('JBSWY3DPEHPK3PXP')).toBe(true);
    expect(isValidOtpSecret('GEZDGNBVGY3TQOJQ')).toBe(true);
  });

  it('accepts secret with spaces', () => {
    expect(isValidOtpSecret('JBSW Y3DP EHPK 3PXP')).toBe(true);
  });

  it('rejects too-short secret', () => {
    expect(isValidOtpSecret('JBSWY3D')).toBe(false);
  });

  it('rejects invalid characters', () => {
    expect(isValidOtpSecret('INVALID!@#$%^&*()')).toBe(false);
  });
});

// ─── Security utilities ───────────────────────────────────────────────────────

describe('safeEqual', () => {
  it('returns true for equal strings', () => {
    expect(safeEqual('hello', 'hello')).toBe(true);
    expect(safeEqual('', '')).toBe(true);
  });

  it('returns false for different strings', () => {
    expect(safeEqual('hello', 'world')).toBe(false);
    expect(safeEqual('abc', 'abcd')).toBe(false);
  });

  it('returns false for different lengths', () => {
    expect(safeEqual('short', 'longer string')).toBe(false);
  });

  it('is case sensitive', () => {
    expect(safeEqual('Hello', 'hello')).toBe(false);
  });
});

describe('generateId', () => {
  it('generates a string of correct length', () => {
    const id = generateId(16);
    expect(id).toHaveLength(32); // hex encoding doubles length
  });

  it('generates unique IDs', () => {
    const ids = new Set(Array.from({ length: 100 }, () => generateId()));
    expect(ids.size).toBe(100);
  });

  it('generates hex string', () => {
    const id = generateId(8);
    expect(/^[0-9a-f]+$/.test(id)).toBe(true);
  });
});

// ─── URL matching ─────────────────────────────────────────────────────────────

describe('matchesUrl', () => {
  it('matches exact URL', () => {
    expect(matchesUrl('https://github.com', 'https://github.com')).toBe(true);
  });

  it('matches same domain', () => {
    expect(matchesUrl('https://github.com/login', 'https://github.com/dashboard')).toBe(true);
  });

  it('matches subdomain', () => {
    expect(matchesUrl('https://github.com', 'https://api.github.com')).toBe(true);
  });

  it('does not match different domain', () => {
    expect(matchesUrl('https://github.com', 'https://gitlab.com')).toBe(false);
  });

  it('does not match phishing domain', () => {
    expect(matchesUrl('https://github.com', 'https://github.com.evil.com')).toBe(false);
  });

  it('handles empty strings', () => {
    expect(matchesUrl('', 'https://github.com')).toBe(false);
    expect(matchesUrl('https://github.com', '')).toBe(false);
  });

  it('strips www prefix for comparison', () => {
    expect(matchesUrl('https://www.github.com', 'https://github.com')).toBe(true);
  });
});

// ─── Password utilities ───────────────────────────────────────────────────────

describe('maskPassword', () => {
  it('masks entire password by default', () => {
    expect(maskPassword('password123')).toBe('•••••••••••');
  });

  it('shows last N characters', () => {
    expect(maskPassword('password123', 3)).toBe('••••••••123');
  });

  it('handles empty password', () => {
    expect(maskPassword('')).toBe('');
  });

  it('handles showLast >= length', () => {
    expect(maskPassword('abc', 5)).toBe('abc');
  });
});

describe('calculateEntropy', () => {
  it('returns 0 for empty password', () => {
    expect(calculateEntropy('')).toBe(0);
  });

  it('returns higher entropy for longer passwords', () => {
    const short = calculateEntropy('abc');
    const long = calculateEntropy('abcdefghijklmnop');
    expect(long).toBeGreaterThan(short);
  });

  it('returns higher entropy for mixed character sets', () => {
    const lower = calculateEntropy('aaaaaaaaaa');
    const mixed = calculateEntropy('Aa1!Aa1!Aa');
    expect(mixed).toBeGreaterThan(lower);
  });
});

describe('classifyPasswordStrength', () => {
  it('classifies very weak password', () => {
    expect(classifyPasswordStrength(10)).toBe(0);
  });

  it('classifies weak password', () => {
    expect(classifyPasswordStrength(30)).toBe(1);
  });

  it('classifies fair password', () => {
    expect(classifyPasswordStrength(50)).toBe(2);
  });

  it('classifies strong password', () => {
    expect(classifyPasswordStrength(80)).toBe(3);
  });

  it('classifies very strong password', () => {
    expect(classifyPasswordStrength(130)).toBe(4);
  });
});

// ─── OTP utilities ────────────────────────────────────────────────────────────

describe('parseOtpUri', () => {
  it('parses valid TOTP URI', () => {
    const uri = 'otpauth://totp/Example:alice@example.com?secret=JBSWY3DPEHPK3PXP&issuer=Example';
    const result = parseOtpUri(uri);
    expect(result).not.toBeNull();
    expect(result!.type).toBe('totp');
    expect(result!.secret).toBe('JBSWY3DPEHPK3PXP');
    expect(result!.issuer).toBe('Example');
  });

  it('parses HOTP URI with counter', () => {
    const uri = 'otpauth://hotp/Example?secret=JBSWY3DPEHPK3PXP&counter=5';
    const result = parseOtpUri(uri);
    expect(result).not.toBeNull();
    expect(result!.type).toBe('hotp');
    expect(result!.counter).toBe(5);
  });

  it('returns null for invalid URI', () => {
    expect(parseOtpUri('not-a-uri')).toBeNull();
    expect(parseOtpUri('https://example.com')).toBeNull();
    expect(parseOtpUri('')).toBeNull();
  });

  it('returns null for URI without secret', () => {
    expect(parseOtpUri('otpauth://totp/Example?issuer=Test')).toBeNull();
  });

  it('normalizes secret to uppercase', () => {
    const uri = 'otpauth://totp/Example?secret=jbswy3dpehpk3pxp';
    const result = parseOtpUri(uri);
    expect(result!.secret).toBe('JBSWY3DPEHPK3PXP');
  });

  it('uses default values for missing params', () => {
    const uri = 'otpauth://totp/Example?secret=JBSWY3DPEHPK3PXP';
    const result = parseOtpUri(uri);
    expect(result!.digits).toBe(6);
    expect(result!.period).toBe(30);
    expect(result!.algorithm).toBe('SHA1');
  });
});

describe('buildOtpUri', () => {
  it('builds valid TOTP URI', () => {
    const uri = buildOtpUri({
      type: 'totp',
      label: 'Example:alice',
      secret: 'JBSWY3DPEHPK3PXP',
      issuer: 'Example',
    });
    expect(uri).toContain('otpauth://totp/');
    expect(uri).toContain('secret=JBSWY3DPEHPK3PXP');
    expect(uri).toContain('issuer=Example');
  });

  it('round-trips through parse and build', () => {
    const original = 'otpauth://totp/Example%3Aalice?secret=JBSWY3DPEHPK3PXP&issuer=Example';
    const parsed = parseOtpUri(original);
    expect(parsed).not.toBeNull();
    const rebuilt = buildOtpUri(parsed!);
    const reparsed = parseOtpUri(rebuilt);
    expect(reparsed!.secret).toBe(parsed!.secret);
    expect(reparsed!.issuer).toBe(parsed!.issuer);
  });
});

// ─── Entry utilities ──────────────────────────────────────────────────────────

function makeEntry(overrides: Partial<Entry> = {}): Entry {
  return {
    uuid: generateUuid(),
    groupUuid: 'root',
    title: 'Test Entry',
    username: 'user@example.com',
    url: 'https://example.com',
    notes: '',
    iconId: 0,
    tags: [],
    hasPassword: true,
    hasOtp: false,
    hasPasskey: false,
    hasSshKey: false,
    hasAttachments: false,
    isExpired: false,
    createdAt: new Date().toISOString(),
    modifiedAt: new Date().toISOString(),
    usageCount: 0,
    customFields: [],
    autoTypeEnabled: true,
    autoTypeObfuscation: false,
    qualityCheck: true,
    ...overrides,
  };
}

describe('filterEntries', () => {
  const entries = [
    makeEntry({ title: 'GitHub', username: 'user@github.com', url: 'https://github.com' }),
    makeEntry({ title: 'Gmail', username: 'user@gmail.com', url: 'https://mail.google.com' }),
    makeEntry({
      title: 'Amazon',
      username: 'shopper',
      url: 'https://amazon.com',
      tags: ['shopping'],
    }),
  ];

  it('returns all entries for empty query', () => {
    expect(filterEntries(entries, '')).toHaveLength(3);
  });

  it('filters by title', () => {
    const result = filterEntries(entries, 'github');
    expect(result).toHaveLength(1);
    expect(result[0].title).toBe('GitHub');
  });

  it('filters by username', () => {
    const result = filterEntries(entries, 'gmail.com');
    expect(result).toHaveLength(1);
    expect(result[0].title).toBe('Gmail');
  });

  it('filters by URL', () => {
    const result = filterEntries(entries, 'amazon');
    expect(result).toHaveLength(1);
  });

  it('filters by tag', () => {
    const result = filterEntries(entries, 'shopping');
    expect(result).toHaveLength(1);
    expect(result[0].title).toBe('Amazon');
  });

  it('is case-insensitive', () => {
    expect(filterEntries(entries, 'GITHUB')).toHaveLength(1);
    expect(filterEntries(entries, 'github')).toHaveLength(1);
  });

  it('returns empty for no matches', () => {
    expect(filterEntries(entries, 'nonexistent')).toHaveLength(0);
  });
});

describe('sortEntries', () => {
  const entries = [
    makeEntry({ title: 'Zebra', modifiedAt: '2024-01-03T00:00:00Z' }),
    makeEntry({ title: 'Apple', modifiedAt: '2024-01-01T00:00:00Z' }),
    makeEntry({ title: 'Mango', modifiedAt: '2024-01-02T00:00:00Z' }),
  ];

  it('sorts by title ascending', () => {
    const sorted = sortEntries(entries, 'title', 'asc');
    expect(sorted[0].title).toBe('Apple');
    expect(sorted[2].title).toBe('Zebra');
  });

  it('sorts by title descending', () => {
    const sorted = sortEntries(entries, 'title', 'desc');
    expect(sorted[0].title).toBe('Zebra');
    expect(sorted[2].title).toBe('Apple');
  });

  it('sorts by modifiedAt ascending', () => {
    const sorted = sortEntries(entries, 'modifiedAt', 'asc');
    expect(sorted[0].title).toBe('Apple');
    expect(sorted[2].title).toBe('Zebra');
  });

  it('does not mutate original array', () => {
    const original = [...entries];
    sortEntries(entries, 'title', 'asc');
    expect(entries[0].title).toBe(original[0].title);
  });
});

// ─── String utilities (extended) ─────────────────────────────────────────────

describe('slugify', () => {
  it('converts to lowercase slug', () => {
    expect(slugify('Hello World')).toBe('hello-world');
  });

  it('removes special characters', () => {
    expect(slugify('Hello, World!')).toBe('hello-world');
  });

  it('handles multiple spaces', () => {
    expect(slugify('Hello   World')).toBe('hello-world');
  });

  it('handles empty string', () => {
    expect(slugify('')).toBe('');
  });
});

// ─── Function utilities ───────────────────────────────────────────────────────

describe('debounce', () => {
  it('delays function execution', async () => {
    vi.useFakeTimers();
    const fn = vi.fn();
    const debounced = debounce(fn, 100);

    debounced();
    debounced();
    debounced();

    expect(fn).not.toHaveBeenCalled();
    vi.advanceTimersByTime(100);
    expect(fn).toHaveBeenCalledTimes(1);

    vi.useRealTimers();
  });
});

describe('throttle', () => {
  it('limits function calls', async () => {
    vi.useFakeTimers();
    const fn = vi.fn();
    const throttled = throttle(fn, 100);

    throttled();
    throttled();
    throttled();

    expect(fn).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(100);
    throttled();
    expect(fn).toHaveBeenCalledTimes(2);

    vi.useRealTimers();
  });
});

// ─── UUID ─────────────────────────────────────────────────────────────────────

describe('generateUuid', () => {
  it('generates valid UUID v4 format', () => {
    const uuid = generateUuid();
    expect(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(uuid)).toBe(
      true
    );
  });

  it('generates unique UUIDs', () => {
    const uuids = new Set(Array.from({ length: 100 }, () => generateUuid()));
    expect(uuids.size).toBe(100);
  });
});
