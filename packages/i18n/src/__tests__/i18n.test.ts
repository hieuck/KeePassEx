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

// All supported locales with their translations
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

beforeAll(async () => {
  await initI18n('en');
});

describe('i18n — English', () => {
  it('translates basic keys', () => {
    expect(t('common.ok')).toBe('OK');
    expect(t('common.cancel')).toBe('Cancel');
    expect(t('app.name')).toBe('KeePassEx');
  });

  it('handles interpolation', () => {
    const result = t('app.version', { version: '1.0.0' });
    expect(result).toContain('1.0.0');
  });

  it('falls back to key for missing translations', () => {
    const result = t('nonexistent.key');
    expect(result).toBe('nonexistent.key');
  });
});

describe('i18n — Vietnamese', () => {
  it('switches to Vietnamese', async () => {
    await changeLocale('vi');
    expect(t('common.ok')).toBe('OK');
    expect(t('common.cancel')).toBe('Hủy');
    expect(t('app.name')).toBe('KeePassEx');
  });

  it('translates vault keys', () => {
    expect(t('vault.lock')).toBe('Khóa kho');
    expect(t('vault.unlock')).toBe('Mở khóa kho');
  });
});

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
    // Reset to English after all locale tests
    await changeLocale('en');
  });
});

describe('i18n — Key parity (all 10 languages vs EN)', () => {
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

  const enKeys = getAllKeys(en as unknown as Record<string, unknown>).sort();

  for (const { code, locale, name } of ALL_LOCALES.filter(l => l.code !== 'en')) {
    it(`${name} (${code}) has all EN keys`, () => {
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

  it('no empty translation values in EN', () => {
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
    expect(checkEmpty(en as unknown as Record<string, unknown>)).toEqual([]);
  });

  it('no empty translation values in VI', () => {
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
    expect(checkEmpty(viLocale as unknown as Record<string, unknown>)).toEqual([]);
  });

  it('EN and VI have matching interpolation variables', () => {
    const I18NEXT_SPECIAL_VARS = new Set(['plural', 'count_plural', 'ordinal']);

    function extractVars(str: string): string[] {
      const matches = str.match(/\{\{(\w+)\}\}/g) ?? [];
      return matches
        .map(m => m.replace(/\{\{|\}\}/g, ''))
        .filter(v => !I18NEXT_SPECIAL_VARS.has(v))
        .sort();
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

    const enLeaves = getAllLeafValues(en as unknown as Record<string, unknown>);
    const viLeaves = getAllLeafValues(viLocale as unknown as Record<string, unknown>);
    const viMap = new Map(viLeaves.map(l => [l.key, l.value]));
    const mismatches: Array<{ key: string; enVars: string[]; viVars: string[] }> = [];

    for (const { key, value: enValue } of enLeaves) {
      const viValue = viMap.get(key);
      if (!viValue) continue;
      const enVars = extractVars(enValue);
      const viVars = extractVars(viValue);
      const missingInVi = enVars.filter(v => !viVars.includes(v));
      const extraInVi = viVars.filter(v => !enVars.includes(v));
      if (missingInVi.length > 0 || extraInVi.length > 0) {
        mismatches.push({ key, enVars, viVars });
      }
    }

    if (mismatches.length > 0) {
      const details = mismatches
        .map(m => `  ${m.key}: EN=[${m.enVars.join(',')}] VI=[${m.viVars.join(',')}]`)
        .join('\n');
      throw new Error(`Interpolation variable mismatches:\n${details}`);
    }
  });
});

beforeAll(async () => {
  await initI18n('en');
});

describe('i18n — English', () => {
  it('translates basic keys', () => {
    expect(t('common.ok')).toBe('OK');
    expect(t('common.cancel')).toBe('Cancel');
    expect(t('app.name')).toBe('KeePassEx');
  });

  it('handles interpolation', () => {
    const result = t('app.version', { version: '1.0.0' });
    expect(result).toContain('1.0.0');
  });

  it('falls back to key for missing translations', () => {
    const result = t('nonexistent.key');
    expect(result).toBe('nonexistent.key');
  });
});

describe('i18n — Vietnamese', () => {
  it('switches to Vietnamese', async () => {
    await changeLocale('vi');
    expect(t('common.ok')).toBe('OK');
    expect(t('common.cancel')).toBe('Hủy');
    expect(t('app.name')).toBe('KeePassEx');
  });

  it('translates vault keys', () => {
    expect(t('vault.lock')).toBe('Khóa kho');
    expect(t('vault.unlock')).toBe('Mở khóa kho');
  });
});

describe('i18n — Key parity', () => {
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

  it('EN and VI have the same keys', () => {
    const enKeys = getAllKeys(en as unknown as Record<string, unknown>).sort();
    const viKeys = getAllKeys(viLocale as unknown as Record<string, unknown>).sort();

    const missingInVi = enKeys.filter(k => !viKeys.includes(k));
    const missingInEn = viKeys.filter(k => !enKeys.includes(k));

    expect(missingInVi).toEqual([]);
    expect(missingInEn).toEqual([]);
  });

  it('no empty translation values in EN', () => {
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

    const emptyKeys = checkEmpty(en as unknown as Record<string, unknown>);
    expect(emptyKeys).toEqual([]);
  });

  it('no empty translation values in VI', () => {
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

    const emptyKeys = checkEmpty(viLocale as unknown as Record<string, unknown>);
    expect(emptyKeys).toEqual([]);
  });

  it('EN and VI have matching interpolation variables', () => {
    /**
     * Extract all {{variable}} placeholders from a string.
     * Excludes i18next special suffixes like {{plural}} which are used for
     * English grammatical pluralization and are not required in Vietnamese.
     */
    const I18NEXT_SPECIAL_VARS = new Set(['plural', 'count_plural', 'ordinal']);

    function extractVars(str: string): string[] {
      const matches = str.match(/\{\{(\w+)\}\}/g) ?? [];
      return matches
        .map(m => m.replace(/\{\{|\}\}/g, ''))
        .filter(v => !I18NEXT_SPECIAL_VARS.has(v))
        .sort();
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

    const enLeaves = getAllLeafValues(en as unknown as Record<string, unknown>);
    const viLeaves = getAllLeafValues(viLocale as unknown as Record<string, unknown>);

    const viMap = new Map(viLeaves.map(l => [l.key, l.value]));

    const mismatches: Array<{ key: string; enVars: string[]; viVars: string[] }> = [];

    for (const { key, value: enValue } of enLeaves) {
      const viValue = viMap.get(key);
      if (!viValue) continue; // Missing key already caught by parity test

      const enVars = extractVars(enValue);
      const viVars = extractVars(viValue);

      // VI must contain all non-special EN variables (VI may omit plural suffixes)
      const missingInVi = enVars.filter(v => !viVars.includes(v));
      // VI must not introduce variables that EN doesn't have
      const extraInVi = viVars.filter(v => !enVars.includes(v));

      if (missingInVi.length > 0 || extraInVi.length > 0) {
        mismatches.push({ key, enVars, viVars });
      }
    }

    if (mismatches.length > 0) {
      const details = mismatches
        .map(m => `  ${m.key}: EN=[${m.enVars.join(',')}] VI=[${m.viVars.join(',')}]`)
        .join('\n');
      throw new Error(`Interpolation variable mismatches:\n${details}`);
    }
  });
});
