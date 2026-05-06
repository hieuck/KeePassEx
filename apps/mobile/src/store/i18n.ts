/**
 * KeePassEx Mobile — i18n store
 * Manages language selection and provides translation function
 */
import { create } from 'zustand';
import { initI18n, t as translate, changeLocale, type SupportedLocale } from '@keepassex/i18n';

interface I18nState {
  locale: SupportedLocale;
  initialized: boolean;
  init: (locale?: SupportedLocale) => Promise<void>;
  setLocale: (locale: SupportedLocale) => Promise<void>;
  t: (key: string, options?: Record<string, unknown>) => string;
}

export const useI18nStore = create<I18nState>((set, get) => ({
  locale: 'en',
  initialized: false,

  init: async (locale = 'en') => {
    await initI18n(locale);
    set({ locale, initialized: true });
  },

  setLocale: async (locale) => {
    await changeLocale(locale);
    set({ locale });
  },

  t: (key, options) => translate(key, options),
}));

/**
 * Convenience hook — returns translation function for current locale
 */
export function useTranslation() {
  const { t, locale } = useI18nStore();
  return { t, locale };
}
