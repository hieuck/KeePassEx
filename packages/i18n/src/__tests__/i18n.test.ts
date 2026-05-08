/**
 * i18n tests — verify all 10 languages have parity and key completeness
 */
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { initI18n, t, changeLocale } from '../index';
import { en } from '../locales/en';
import { vi as viLocale } from '../locales/vi';
import { zh } from '../locales/zh';
import { ja } from '../locales/ja';
import { ko } from '../locales/ko';
import { es } from '../locales/es';
import { fr } from '../locales/fr';
import { de } from '../locales/de';
import { pt } from '../locales/pt';
import { ru } from '../locales/ru';

// All supported locales
const ALL_LOCALES = [
  { code: 'en', locale: en, name: 'English' },
  { code: 'vi', locale: viLocale, name: 'Vietnamese' },
  { code: 'zh', locale: zh, name: 'Chinese' },
  { code: 'ja', locale: ja, name: 'Japanese' },
  { code: 'ko', locale: ko, name: 'Korean' },
  { code: 'es', locale: es, name: 'Spanish' },
  { code: 'fr', locale: fr, name: 'French' },
  { code: 'de', locale: de, name: 'German' },
  { code: 'pt', locale: pt, name: 'Portuguese' },
  { code: 'ru', locale: ru, name: 'Russian' },
] as const;

// ─── Helpers ──────────────────────────────────────────────────────────────────

function getAllKeys(obj: Record<string, unknown>, prefix = ''): string[] {
  const keys: string[] = [];
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (typeof value === 'object' && value !== null) {
      keys.push(...getAllKeys(value as Record<string, unknown>, fullKey));
    } else {
      keys.push(fullKey);
    }
  }
  return keys;
}

function getAllLeafValues(
  obj: Record<string, unknown>,
  prefix = ''
): Array<{ key: string; value: string }> {
  const result: Array<{ key: string; value: string }> = [];
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (typeof value === 'string') {
      result.push({ key: fullKey, value });
    } else if (typeof value === 'object' && value !== null) {
      result.push(...getAllLeafValues(value as Record<string, unknown>, fullKey));
    }
  }
  return result;
}

function checkEmpty(obj: Record<string, unknown>, prefix = ''): string[] {
  const empty: string[] = [];
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (typeof value === 'string' && value.trim() === '') {
      empty.push(fullKey);
    } else if (typeof value === 'object' && value !== null) {
      empty.push(...checkEmpty(value as Record<string, unknown>, fullKey));
    }
  }
  return empty;
}

// i18next special plural suffixes — not required in all languages
const I18NEXT_SPECIAL_VARS = new Set(['plural', 'count_plural', 'ordinal']);

function extractVars(str: string): string[] {
  const matches = str.match(/\{\{(\w+)\}\}/g) ?? [];
  return matches
    .map(m => m.replace(/\{\{|\}\}/g, ''))
    .filter(v => !I18NEXT_SPECIAL_VARS.has(v))
    .sort();
}

// ─── Setup ────────────────────────────────────────────────────────────────────

beforeAll(async () => {
  await initI18n('en');
});

// ─── English baseline ─────────────────────────────────────────────────────────

describe('i18n — English baseline', () => {
  it('translates basic keys', () => {
    expect(t('common.ok')).toBe('OK');
    expect(t('common.cancel')).toBe('Cancel');
    expect(t('app.name')).toBe('KeePassEx');
  });

  it('handles interpolation', () => {
    expect(t('app.version', { version: '1.0.0' })).toContain('1.0.0');
  });

  it('falls back to key for missing translations', () => {
    expect(t('nonexistent.key')).toBe('nonexistent.key');
  });

  it('no empty values in EN', () => {
    expect(checkEmpty(en as unknown as Record<string, unknown>)).toEqual([]);
  });
});

// ─── Vietnamese ───────────────────────────────────────────────────────────────

describe('i18n — Vietnamese', () => {
  it('switches to Vietnamese', async () => {
    await changeLocale('vi');
    expect(t('common.cancel')).toBe('Hủy');
    expect(t('app.name')).toBe('KeePassEx');
  });

  it('translates vault keys', () => {
    expect(t('vault.lock')).toBe('Khóa kho');
    expect(t('vault.unlock')).toBe('Mở khóa kho');
  });

  it('no empty values in VI', () => {
    expect(checkEmpty(viLocale as unknown as Record<string, unknown>)).toEqual([]);
  });

  afterAll(async () => {
    await changeLocale('en');
  });
});

// ─── All 10 languages — spot checks ──────────────────────────────────────────

describe('i18n — All 10 languages switch correctly', () => {
  const SPOT_CHECKS: Record<string, { key: string; expected: string }[]> = {
    zh: [
      { key: 'common.cancel', expected: '取消' },
      { key: 'vault.lock', expected: '锁定密码库' },
    ],
    ja: [
      { key: 'common.cancel', expected: 'キャンセル' },
      { key: 'vault.lock', expected: 'ボルトをロック' },
    ],
    ko: [
      { key: 'common.cancel', expected: '취소' },
      { key: 'vault.lock', expected: '볼트 잠금' },
    ],
    es: [
      { key: 'common.cancel', expected: 'Cancelar' },
      { key: 'vault.lock', expected: 'Bloquear bóveda' },
    ],
    fr: [
      { key: 'common.cancel', expected: 'Annuler' },
      { key: 'vault.lock', expected: 'Verrouiller le coffre' },
    ],
    de: [
      { key: 'common.cancel', expected: 'Abbrechen' },
      { key: 'vault.lock', expected: 'Tresor sperren' },
    ],
    pt: [
      { key: 'common.cancel', expected: 'Cancelar' },
      { key: 'vault.lock', expected: 'Bloquear cofre' },
    ],
    ru: [
      { key: 'common.cancel', expected: 'Отмена' },
      { key: 'vault.lock', expected: 'Заблокировать хранилище' },
    ],
  };

  for (const { code, name } of ALL_LOCALES.filter(l => l.code !== 'en' && l.code !== 'vi')) {
    it(`switches to ${name} (${code})`, async () => {
      await changeLocale(code as never);
      expect(t('app.name')).toBe('KeePassEx'); // App name never changes
      const checks = SPOT_CHECKS[code];
      if (checks) {
        for (const { key, expected } of checks) {
          expect(t(key)).toBe(expected);
        }
      }
    });
  }

  afterAll(async () => {
    await changeLocale('en');
  });
});

// ─── Key parity — all 10 languages vs EN ─────────────────────────────────────

describe('i18n — Key parity (all 10 languages vs EN)', () => {
  const enKeys = getAllKeys(en as unknown as Record<string, unknown>).sort();

  for (const { code, locale, name } of ALL_LOCALES.filter(l => l.code !== 'en')) {
    it(`${name} (${code}) has all EN keys and no extra keys`, () => {
      const localeKeys = getAllKeys(locale as unknown as Record<string, unknown>).sort();
      const missingInLocale = enKeys.filter(k => !localeKeys.includes(k));
      const extraInLocale = localeKeys.filter(k => !enKeys.includes(k));

      if (missingInLocale.length > 0) {
        throw new Error(
          `${name} (${code}) is missing ${missingInLocale.length} keys:\n` +
            missingInLocale
              .slice(0, 10)
              .map(k => `  - ${k}`)
              .join('\n') +
            (missingInLocale.length > 10 ? `\n  ... and ${missingInLocale.length - 10} more` : '')
        );
      }
      if (extraInLocale.length > 0) {
        throw new Error(
          `${name} (${code}) has ${extraInLocale.length} extra keys not in EN:\n` +
            extraInLocale
              .slice(0, 5)
              .map(k => `  + ${k}`)
              .join('\n')
        );
      }
    });
  }
});

// ─── No empty values ──────────────────────────────────────────────────────────

describe('i18n — No empty values in any language', () => {
  for (const { code, locale, name } of ALL_LOCALES.filter(l => l.code !== 'en')) {
    it(`no empty values in ${name} (${code})`, () => {
      const empty = checkEmpty(locale as unknown as Record<string, unknown>);
      if (empty.length > 0) {
        throw new Error(
          `${name} (${code}) has ${empty.length} empty values:\n` +
            empty
              .slice(0, 10)
              .map(k => `  - ${k}`)
              .join('\n')
        );
      }
    });
  }
});

// ─── Interpolation variable parity (EN vs all other languages) ────────────────

describe('i18n — Interpolation variable parity', () => {
  const enLeaves = getAllLeafValues(en as unknown as Record<string, unknown>);

  for (const { code, locale, name } of ALL_LOCALES.filter(l => l.code !== 'en')) {
    it(`${name} (${code}) has matching interpolation variables`, () => {
      const localeLeaves = getAllLeafValues(locale as unknown as Record<string, unknown>);
      const localeMap = new Map(localeLeaves.map(l => [l.key, l.value]));
      const mismatches: Array<{ key: string; enVars: string[]; localeVars: string[] }> = [];

      for (const { key, value: enValue } of enLeaves) {
        const localeValue = localeMap.get(key);
        if (!localeValue) continue; // Missing key caught by parity test

        const enVars = extractVars(enValue);
        const localeVars = extractVars(localeValue);
        const missingInLocale = enVars.filter(v => !localeVars.includes(v));
        const extraInLocale = localeVars.filter(v => !enVars.includes(v));

        if (missingInLocale.length > 0 || extraInLocale.length > 0) {
          mismatches.push({ key, enVars, localeVars });
        }
      }

      if (mismatches.length > 0) {
        const details = mismatches
          .slice(0, 10)
          .map(
            m =>
              `  ${m.key}: EN=[${m.enVars.join(',')}] ${code.toUpperCase()}=[${m.localeVars.join(',')}]`
          )
          .join('\n');
        throw new Error(
          `${name} (${code}) has ${mismatches.length} interpolation variable mismatches:\n${details}`
        );
      }
    });
  }
});
