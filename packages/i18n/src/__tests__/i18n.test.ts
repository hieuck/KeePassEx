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
});
