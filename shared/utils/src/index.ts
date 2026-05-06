/**
 * KeePassEx shared utilities
 * Comprehensive utility functions for all platforms
 */

import type { Entry } from '@keepassex/types';

// ─── Types ────────────────────────────────────────────────────────────────────

export interface OtpUriParams {
  type: 'totp' | 'hotp';
  label: string;
  secret: string;
  issuer?: string;
  algorithm?: 'SHA1' | 'SHA256' | 'SHA512';
  digits?: number;
  period?: number;
  counter?: number;
}

// ─── Date utilities ───────────────────────────────────────────────────────────

/**
 * Format a date with locale support.
 * @example formatDate('2024-01-15', 'vi') → "15 tháng 1, 2024"
 */
export function formatDate(date: string | Date, locale: string = 'en'): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  const bcp47 = locale === 'vi' ? 'vi-VN' : 'en-US';
  return d.toLocaleDateString(bcp47, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

/**
 * Format a date as relative time ("2 hours ago" / "2 giờ trước").
 */
export function formatRelativeTime(date: string | Date, locale: string = 'en'): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  const now = new Date();
  const diffMs = now.getTime() - d.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);
  const diffWeeks = Math.floor(diffDays / 7);
  const diffMonths = Math.floor(diffDays / 30);
  const diffYears = Math.floor(diffDays / 365);

  if (locale === 'vi') {
    if (diffSecs < 60) return 'Vừa xong';
    if (diffMins < 60) return `${diffMins} phút trước`;
    if (diffHours < 24) return `${diffHours} giờ trước`;
    if (diffDays === 1) return 'Hôm qua';
    if (diffDays < 7) return `${diffDays} ngày trước`;
    if (diffWeeks < 5) return `${diffWeeks} tuần trước`;
    if (diffMonths < 12) return `${diffMonths} tháng trước`;
    return `${diffYears} năm trước`;
  }

  if (diffSecs < 60) return 'Just now';
  if (diffMins < 60) return `${diffMins} minute${diffMins !== 1 ? 's' : ''} ago`;
  if (diffHours < 24) return `${diffHours} hour${diffHours !== 1 ? 's' : ''} ago`;
  if (diffDays === 1) return 'Yesterday';
  if (diffDays < 7) return `${diffDays} days ago`;
  if (diffWeeks < 5) return `${diffWeeks} week${diffWeeks !== 1 ? 's' : ''} ago`;
  if (diffMonths < 12) return `${diffMonths} month${diffMonths !== 1 ? 's' : ''} ago`;
  return `${diffYears} year${diffYears !== 1 ? 's' : ''} ago`;
}

// ─── String utilities ─────────────────────────────────────────────────────────

/**
 * Format bytes to human-readable string with locale support.
 * @example formatBytes(1572864) → "1.5 MB"
 */
export function formatBytes(bytes: number, locale: string = 'en'): string {
  if (bytes === 0) return locale === 'vi' ? '0 B' : '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const value = parseFloat((bytes / Math.pow(k, i)).toFixed(1));
  return `${value} ${sizes[i]}`;
}

/**
 * Truncate a string to maxLength, appending suffix if truncated.
 * @param str The string to truncate
 * @param maxLength Maximum length including the suffix
 * @param suffix The suffix to append when truncated (default: "…")
 */
export function truncate(str: string, maxLength: number, suffix: string = '…'): string {
  if (str.length <= maxLength) return str;
  return str.slice(0, maxLength - suffix.length) + suffix;
}

/**
 * Convert a string to a URL-safe slug.
 * @example slugify("Hello World!") → "hello-world"
 */
export function slugify(str: string): string {
  return str
    .toLowerCase()
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '') // strip diacritics
    .replace(/[^a-z0-9\s-]/g, '')
    .trim()
    .replace(/[\s_-]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

// ─── UUID ─────────────────────────────────────────────────────────────────────

/**
 * Generate a cryptographically secure UUID v4.
 * Uses `crypto.randomUUID()` when available, falls back to manual generation.
 */
export function generateUuid(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  // Fallback: manual UUID v4
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  bytes[6] = (bytes[6] & 0x0f) | 0x40; // version 4
  bytes[8] = (bytes[8] & 0x3f) | 0x80; // variant bits
  const hex = Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

// ─── Function utilities ───────────────────────────────────────────────────────

/**
 * Debounce a function — delays invocation until after `ms` milliseconds
 * have elapsed since the last call.
 */
export function debounce<T extends (...args: unknown[]) => unknown>(fn: T, ms: number): T {
  let timer: ReturnType<typeof setTimeout> | undefined;
  return ((...args: unknown[]) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  }) as T;
}

/**
 * Throttle a function — ensures it is called at most once per `ms` milliseconds.
 */
export function throttle<T extends (...args: unknown[]) => unknown>(fn: T, ms: number): T {
  let lastCall = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  return ((...args: unknown[]) => {
    const now = Date.now();
    const remaining = ms - (now - lastCall);
    if (remaining <= 0) {
      clearTimeout(timer);
      lastCall = now;
      fn(...args);
    } else {
      clearTimeout(timer);
      timer = setTimeout(() => {
        lastCall = Date.now();
        fn(...args);
      }, remaining);
    }
  }) as T;
}

// ─── Password utilities ───────────────────────────────────────────────────────

/**
 * Classify password strength from entropy bits.
 * Returns 0 (very weak) → 4 (very strong).
 */
export function classifyPasswordStrength(entropy: number): 0 | 1 | 2 | 3 | 4 {
  if (entropy < 28) return 0;
  if (entropy < 36) return 1;
  if (entropy < 60) return 2;
  if (entropy < 128) return 3;
  return 4;
}

/**
 * Calculate Shannon entropy of a password in bits.
 * Accounts for character set size based on character classes used.
 */
export function calculateEntropy(password: string): number {
  if (!password) return 0;

  let poolSize = 0;
  if (/[a-z]/.test(password)) poolSize += 26;
  if (/[A-Z]/.test(password)) poolSize += 26;
  if (/[0-9]/.test(password)) poolSize += 10;
  if (/[^a-zA-Z0-9]/.test(password)) poolSize += 32;

  if (poolSize === 0) return 0;
  return Math.log2(poolSize) * password.length;
}

/**
 * Mask a password, optionally showing the last N characters.
 * @example maskPassword("password1", 2) → "•••••••d1"
 */
export function maskPassword(password: string, showLast: number = 0): string {
  if (!password) return '';
  if (showLast <= 0) return '•'.repeat(password.length);
  const masked = Math.max(0, password.length - showLast);
  return '•'.repeat(masked) + password.slice(masked);
}

// ─── OTP utilities ────────────────────────────────────────────────────────────

/**
 * Parse an `otpauth://` URI into structured parameters.
 * Returns null if the URI is invalid.
 *
 * @example
 * parseOtpUri("otpauth://totp/Example:alice@example.com?secret=JBSWY3DPEHPK3PXP&issuer=Example")
 */
export function parseOtpUri(uri: string): OtpUriParams | null {
  try {
    const url = new URL(uri);
    if (url.protocol !== 'otpauth:') return null;

    const type = url.hostname as 'totp' | 'hotp';
    if (type !== 'totp' && type !== 'hotp') return null;

    // Label is the pathname (URL-decoded), strip leading slash
    const label = decodeURIComponent(url.pathname.replace(/^\//, ''));
    const params = url.searchParams;

    const secret = params.get('secret');
    if (!secret) return null;

    const result: OtpUriParams = {
      type,
      label,
      secret: secret.toUpperCase().replace(/\s/g, ''),
      issuer: params.get('issuer') ?? undefined,
      algorithm: (params.get('algorithm') as OtpUriParams['algorithm']) ?? 'SHA1',
      digits: params.has('digits') ? parseInt(params.get('digits')!, 10) : 6,
      period: params.has('period') ? parseInt(params.get('period')!, 10) : 30,
    };

    if (type === 'hotp' && params.has('counter')) {
      result.counter = parseInt(params.get('counter')!, 10);
    }

    return result;
  } catch {
    return null;
  }
}

/**
 * Build an `otpauth://` URI from structured parameters.
 */
export function buildOtpUri(params: OtpUriParams): string {
  const label = encodeURIComponent(params.label);
  const base = `otpauth://${params.type}/${label}`;
  const qs = new URLSearchParams({ secret: params.secret });

  if (params.issuer) qs.set('issuer', params.issuer);
  if (params.algorithm && params.algorithm !== 'SHA1') qs.set('algorithm', params.algorithm);
  if (params.digits && params.digits !== 6) qs.set('digits', String(params.digits));
  if (params.type === 'totp' && params.period && params.period !== 30) {
    qs.set('period', String(params.period));
  }
  if (params.type === 'hotp' && params.counter !== undefined) {
    qs.set('counter', String(params.counter));
  }

  return `${base}?${qs.toString()}`;
}

// ─── URL utilities ────────────────────────────────────────────────────────────

/**
 * Check if a string is a valid URL.
 */
export function isValidUrl(url: string): boolean {
  try {
    const u = new URL(url);
    return u.protocol === 'http:' || u.protocol === 'https:';
  } catch {
    return false;
  }
}

/**
 * Extract the domain from a URL, stripping "www." prefix.
 * @example extractDomain("https://github.com/foo/bar") → "github.com"
 */
export function extractDomain(url: string): string {
  try {
    const u = new URL(url);
    return u.hostname.replace(/^www\./, '');
  } catch {
    // Fallback for bare domains or malformed URLs
    return url
      .replace(/^https?:\/\//, '')
      .replace(/^www\./, '')
      .split('/')[0]
      .split('?')[0];
  }
}

/**
 * Smart URL matching for autofill.
 * Returns true if `entryUrl` should be offered for autofill on `pageUrl`.
 *
 * Matching rules (in order):
 * 1. Exact URL match
 * 2. Same domain (ignoring www, path, query)
 * 3. Subdomain match (pageUrl is a subdomain of entryUrl's domain)
 */
export function matchesUrl(entryUrl: string, pageUrl: string): boolean {
  if (!entryUrl || !pageUrl) return false;

  // Exact match
  if (entryUrl === pageUrl) return true;

  try {
    const entry = new URL(entryUrl.includes('://') ? entryUrl : `https://${entryUrl}`);
    const page = new URL(pageUrl.includes('://') ? pageUrl : `https://${pageUrl}`);

    const entryDomain = entry.hostname.replace(/^www\./, '');
    const pageDomain = page.hostname.replace(/^www\./, '');

    // Same domain
    if (entryDomain === pageDomain) return true;

    // Subdomain: page is a subdomain of entry domain
    if (pageDomain.endsWith(`.${entryDomain}`)) return true;

    return false;
  } catch {
    return false;
  }
}

// ─── Entry utilities ──────────────────────────────────────────────────────────

type SortField = 'title' | 'username' | 'url' | 'modifiedAt' | 'createdAt' | 'usageCount';

/**
 * Sort entries by a field in ascending or descending order.
 */
export function sortEntries(entries: Entry[], field: string, dir: 'asc' | 'desc'): Entry[] {
  const sorted = [...entries].sort((a, b) => {
    const key = field as SortField;
    const aVal = String(a[key] ?? '').toLowerCase();
    const bVal = String(b[key] ?? '').toLowerCase();

    if (aVal < bVal) return -1;
    if (aVal > bVal) return 1;
    return 0;
  });

  return dir === 'desc' ? sorted.reverse() : sorted;
}

/**
 * Group entries by their `groupUuid`, returning a Map of groupUuid → entries.
 */
export function groupEntriesByGroup(entries: Entry[]): Map<string, Entry[]> {
  const map = new Map<string, Entry[]>();
  for (const entry of entries) {
    const group = map.get(entry.groupUuid) ?? [];
    group.push(entry);
    map.set(entry.groupUuid, group);
  }
  return map;
}

/**
 * Multi-field search filter.
 * Searches title, username, url, notes, and tags (case-insensitive).
 */
export function filterEntries(entries: Entry[], query: string): Entry[] {
  const q = query.trim().toLowerCase();
  if (!q) return entries;

  return entries.filter(entry => {
    if (entry.title.toLowerCase().includes(q)) return true;
    if (entry.username.toLowerCase().includes(q)) return true;
    if (entry.url.toLowerCase().includes(q)) return true;
    if (entry.notes.toLowerCase().includes(q)) return true;
    if (entry.tags.some(tag => tag.toLowerCase().includes(q))) return true;
    if (
      entry.customFields.some(
        f => f.key.toLowerCase().includes(q) || (!f.protected && f.value.toLowerCase().includes(q))
      )
    )
      return true;
    return false;
  });
}

// ─── Security utilities ───────────────────────────────────────────────────────

/**
 * Constant-time string comparison to prevent timing attacks.
 */
export function safeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let result = 0;
  for (let i = 0; i < a.length; i++) {
    result |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return result === 0;
}

/**
 * Generate a cryptographically secure random hex string.
 */
export function generateId(length: number = 16): string {
  const array = new Uint8Array(length);
  crypto.getRandomValues(array);
  return Array.from(array, b => b.toString(16).padStart(2, '0')).join('');
}

// ─── Clipboard utilities ──────────────────────────────────────────────────────

/**
 * Copy text to clipboard. Returns true on success.
 * Never logs the text value.
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    // Fallback for environments without Clipboard API
    try {
      const el = document.createElement('textarea');
      el.value = text;
      el.style.cssText = 'position:fixed;opacity:0;pointer-events:none';
      document.body.appendChild(el);
      el.select();
      const ok = document.execCommand('copy');
      document.body.removeChild(el);
      return ok;
    } catch {
      return false;
    }
  }
}

// ─── Color utilities ──────────────────────────────────────────────────────────

/**
 * Get a color for a password strength score (0–4).
 */
export function getStrengthColor(score: 0 | 1 | 2 | 3 | 4): string {
  const colors: Record<number, string> = {
    0: '#DC2626', // red
    1: '#EA580C', // orange
    2: '#D97706', // amber
    3: '#16A34A', // green
    4: '#059669', // emerald
  };
  return colors[score] ?? colors[0];
}

// ─── Validation utilities ─────────────────────────────────────────────────────

/**
 * Validate an email address format.
 */
export function isValidEmail(email: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
}

/**
 * Validate a base32-encoded OTP secret.
 */
export function isValidOtpSecret(secret: string): boolean {
  const cleaned = secret.toUpperCase().replace(/[\s=-]/g, '');
  return /^[A-Z2-7]+$/.test(cleaned) && cleaned.length >= 16;
}
