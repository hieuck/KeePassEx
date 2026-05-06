/**
 * KeePassEx i18n — Internationalization
 * Supports: English (en), Vietnamese (vi), Chinese Simplified (zh),
 *           Japanese (ja), Korean (ko), Spanish (es), French (fr),
 *           German (de), Portuguese (pt), Russian (ru)
 */

import i18next from 'i18next';
import { en } from './locales/en';
import { vi } from './locales/vi';
import { zh } from './locales/zh';
import { ja } from './locales/ja';
import { ko } from './locales/ko';
import { es } from './locales/es';
import { fr } from './locales/fr';
import { de } from './locales/de';
import { pt } from './locales/pt';
import { ru } from './locales/ru';

export type SupportedLocale = 'en' | 'vi' | 'zh' | 'ja' | 'ko' | 'es' | 'fr' | 'de' | 'pt' | 'ru';

export const SUPPORTED_LOCALES: Record<SupportedLocale, string> = {
  en: 'English',
  vi: 'Tiếng Việt',
  zh: '简体中文',
  ja: '日本語',
  ko: '한국어',
  es: 'Español',
  fr: 'Français',
  de: 'Deutsch',
  pt: 'Português',
  ru: 'Русский',
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
      de: { translation: de },
      pt: { translation: pt },
      ru: { translation: ru },
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

export { en, vi, zh, ja, ko, es, fr, de, pt, ru };
export type { TranslationKeys } from './types';
