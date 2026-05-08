/**
 * Web-only entry point for @keepassex/ui
 * Only exports components with no React Native dependencies.
 * Used by: apps/desktop, apps/browser-extension
 */
export { OtpDisplay } from './components/OtpDisplay';
export type { OtpCode } from './components/OtpDisplay';
export { PasswordField } from './components/PasswordField';
export { EntryHistoryViewer } from './components/EntryHistoryViewer';
