/**
 * KeePassEx Desktop — React entry point
 */
import { StrictMode } from 'react';
import ReactDOM from 'react-dom/client';
import { initI18n } from '@keepassex/i18n';
import { App } from './App';
import './styles/globals.css';

// Initialize i18n before rendering — default to 'en', settings store will
// call changeLocale() once persisted settings are loaded.
initI18n('en').then(() => {
  // Global error handler — log to console for debugging
  window.addEventListener('error', e => {
    console.error('[KeePassEx] Uncaught error:', e.message, e.filename, e.lineno);
  });
  window.addEventListener('unhandledrejection', e => {
    console.error('[KeePassEx] Unhandled promise rejection:', e.reason);
  });

  ReactDOM.createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App />
    </StrictMode>
  );
});
