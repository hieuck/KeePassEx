/**
 * KeePassEx i18n — Internationalization
 * Supports: English (en), Vietnamese (vi)
 */

import i18next from 'i18next';
import { en } from './locales/en';
import { vi } from './locales/vi';

export type SupportedLocale = 'en' | 'vi';

export const SUPPORTED_LOCALES: Record<SupportedLocale, string> = {
  en: 'English',
  vi: 'Tiếng Việt',
};

export async function initI18n(locale: SupportedLocale = 'en') {
  await i18next.init({
    lng: locale,
    fallbackLng: 'en',
    resources: {
      en: { translation: en },
      vi: { translation: vi },
    },
    interpolation: {
      escapeValue: false,
    },
  });
}

export function t(key: string, options?: Record<string, unknown>): string {
  return i18next.t(key, options);
}

export function changeLocale(locale: SupportedLocale) {
  return i18next.changeLanguage(locale);
}

export function currentLocale(): string {
  return i18next.language;
}

export { en, vi };
export type { TranslationKeys } from './types';
