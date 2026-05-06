/**
 * KeePassEx i18n — Internationalization
 * Supports: English (en), Vietnamese (vi), Chinese Simplified (zh),
 *           Japanese (ja), Korean (ko), Spanish (es), French (fr)
 */

import i18next from 'i18next';
import { en } from './locales/en';
import { vi } from './locales/vi';
import { zh } from './locales/zh';
import { ja } from './locales/ja';
import { ko } from './locales/ko';
import { es } from './locales/es';
import { fr } from './locales/fr';

export type SupportedLocale = 'en' | 'vi' | 'zh' | 'ja' | 'ko' | 'es' | 'fr';

export const SUPPORTED_LOCALES: Record<SupportedLocale, string> = {
  en: 'English',
  vi: 'Tiếng Việt',
  zh: '简体中文',
  ja: '日本語',
  ko: '한국어',
  es: 'Español',
  fr: 'Français',
};

export async function initI18n(locale: SupportedLocale = 'en') {
  await i18next.init({
    lng: locale,
    fallbackLng: 'en',
    resources: {
      en: { translation: en },
      vi: { translation: vi },
      zh: { translation: zh },
      ja: { translation: ja },
      ko: { translation: ko },
      es: { translation: es },
      fr: { translation: fr },
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

export { en, vi, zh, ja, ko, es, fr };
export type { TranslationKeys } from './types';
