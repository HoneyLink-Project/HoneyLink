/**
 * i18next configuration for HoneyLink UI
 * 
 * Supported languages: en (English), ja (日本語), es (Español), zh (中文)
 * Default: Japanese (ja)
 */

import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

// Translation resources
import en from './locales/en.json';
import ja from './locales/ja.json';
import es from './locales/es.json';
import zh from './locales/zh.json';

i18n
  .use(LanguageDetector) // Detect user language from browser
  .use(initReactI18next) // React integration
  .init({
    resources: {
      en: { translation: en },
      ja: { translation: ja },
      es: { translation: es },
      zh: { translation: zh },
    },
    fallbackLng: 'ja', // Default to Japanese (primary development language)
    interpolation: {
      escapeValue: false, // React already escapes values
    },
    detection: {
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
    },
  });

export default i18n;
