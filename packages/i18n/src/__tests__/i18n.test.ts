/**
 * i18n tests — verify EN/VI parity and key completeness
 */
import { describe, it, expect, beforeAll } from 'vitest';
import { initI18n, t, changeLocale } from '../index';
import { en } from '../locales/en';
import { vi as viLocale } from '../locales/vi';

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
