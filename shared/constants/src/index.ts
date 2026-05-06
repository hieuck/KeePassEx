/**
 * KeePassEx app-wide constants
 */

// ─── App identity ─────────────────────────────────────────────────────────────

export const APP_NAME = 'KeePassEx';
export const APP_VERSION = '1.0.0';
export const APP_BUNDLE_ID = 'com.keepassex.app';

// ─── File formats ─────────────────────────────────────────────────────────────

export const KDBX_EXTENSION = '.kdbx';
export const KEY_FILE_EXTENSION = '.keyx';
export const KDBX_MIME_TYPE = 'application/x-keepass2';

// ─── Security defaults ────────────────────────────────────────────────────────

export const DEFAULT_CLIPBOARD_CLEAR_SECONDS = 10;
export const MAX_CLIPBOARD_CLEAR_SECONDS = 300;
export const DEFAULT_LOCK_AFTER_IDLE_MINUTES = 5;
export const MIN_PASSWORD_LENGTH = 8;

// ─── History ─────────────────────────────────────────────────────────────────

export const DEFAULT_HISTORY_MAX_ITEMS = 10;
export const DEFAULT_HISTORY_MAX_SIZE_BYTES = 6 * 1024 * 1024; // 6 MB
export const DEFAULT_MAINTENANCE_HISTORY_DAYS = 365;

// ─── Argon2id KDF defaults ────────────────────────────────────────────────────

export const DEFAULT_ARGON2_MEMORY_KB = 65536;   // 64 MB
export const DEFAULT_ARGON2_ITERATIONS = 2;
export const DEFAULT_ARGON2_PARALLELISM = 1;

// ─── Password generator defaults ─────────────────────────────────────────────

export const DEFAULT_GENERATOR_LENGTH = 20;
export const DEFAULT_GENERATOR_WORD_COUNT = 6;

// ─── HIBP breach monitor ──────────────────────────────────────────────────────

export const HIBP_API_URL = 'https://api.pwnedpasswords.com/range/';
export const HIBP_HASH_PREFIX_LENGTH = 5;
export const HIBP_USER_AGENT = 'KeePassEx/1.0 (https://github.com/keepassex/keepassex)';

// ─── Browser extension / native messaging ────────────────────────────────────

export const NATIVE_MESSAGING_HOST = 'com.keepassex.app';
export const BROWSER_EXTENSION_ID_CHROME = 'keepassex-chrome-extension-id';
export const BROWSER_EXTENSION_ID_FIREFOX = '{keepassex-firefox-extension-uuid}';

// ─── Sync ─────────────────────────────────────────────────────────────────────

export const SYNC_TIMEOUT_SECONDS = 30;
export const DEFAULT_SYNC_INTERVAL_SECONDS = 300;
export const MAX_SYNC_RETRY_ATTEMPTS = 3;
export const SYNC_CONFLICT_MARKER = '_conflict_';

// ─── OTP ─────────────────────────────────────────────────────────────────────

export const OTP_PERIOD_SECONDS = 30;
export const DEFAULT_OTP_DIGITS = 6;
export const OTP_REFRESH_INTERVAL_MS = 1000;

// ─── Emergency access ─────────────────────────────────────────────────────────

export const EMERGENCY_ACCESS_DEFAULT_WAIT_DAYS = 7;
export const MIN_EMERGENCY_WAIT_DAYS = 1;
export const MAX_EMERGENCY_WAIT_DAYS = 90;

// ─── Plugin system ────────────────────────────────────────────────────────────

export const PLUGIN_MAX_SIZE_BYTES = 10 * 1024 * 1024; // 10 MB
export const PLUGIN_FILE_EXTENSION = '.kpxplugin';
export const PLUGIN_API_VERSION = '1.0';

// ─── Icons ────────────────────────────────────────────────────────────────────

/** Number of standard KeePass built-in icons (0–68) */
export const ICON_COUNT = 69;

// Standard icon IDs
export const ICON_KEY = 0;
export const ICON_WORLD = 1;
export const ICON_WARNING = 2;
export const ICON_NETWORK_SERVER = 3;
export const ICON_MARKS_EMAIL = 8;
export const ICON_CREDIT_CARD = 9;
export const ICON_BANK = 10;
export const ICON_MOBILE = 11;
export const ICON_FOLDER = 48;
export const ICON_FOLDER_OPEN = 49;
export const ICON_RECYCLE_BIN = 43;

// ─── KDBX signatures ─────────────────────────────────────────────────────────

export const KDBX_SIGNATURE_1 = 0x9aa2d903;
export const KDBX_SIGNATURE_2 = 0xb54bfb67;

// ─── UI ───────────────────────────────────────────────────────────────────────

export const RECENT_VAULTS_MAX = 10;
export const SEARCH_DEBOUNCE_MS = 200;
export const NOTIFICATION_DURATION_MS = 3000;
export const PASSWORD_AGE_WARNING_DAYS = 365;
export const EXPIRY_WARNING_DAYS = 30;

// ─── Auto-Type ────────────────────────────────────────────────────────────────

export const DEFAULT_AUTO_TYPE_SEQUENCE = '{USERNAME}{TAB}{PASSWORD}{ENTER}';
export const AUTO_TYPE_DELAY_MS = 50;

// ─── SSH Agent ────────────────────────────────────────────────────────────────

export const SSH_AGENT_SOCKET_NAME = 'keepassex-ssh-agent.sock';
export const SSH_AGENT_PIPE_NAME = 'keepassex-ssh-agent';

// ─── File size limits ─────────────────────────────────────────────────────────

export const MAX_ATTACHMENT_SIZE_MB = 50;
export const MAX_VAULT_SIZE_MB = 100;
export const MAX_KEY_FILE_SIZE_KB = 1024;

// ─── Supported locales ────────────────────────────────────────────────────────

export const SUPPORTED_LOCALES: Record<string, string> = {
  en: 'English',
  vi: 'Tiếng Việt',
};

// ─── Import formats ───────────────────────────────────────────────────────────

export interface ImportFormatDef {
  id: string;
  name: string;
  extension: string;
  description: string;
}

export const IMPORT_FORMATS: ImportFormatDef[] = [
  {
    id: 'kdbx',
    name: 'KDBX 4.x',
    extension: '.kdbx',
    description: 'KeePassEx, KeePassXC, KeePass 2.x',
  },
  {
    id: 'bitwarden',
    name: 'Bitwarden',
    extension: '.json',
    description: 'Bitwarden JSON export',
  },
  {
    id: 'lastpass',
    name: 'LastPass',
    extension: '.csv',
    description: 'LastPass CSV export',
  },
  {
    id: 'chrome',
    name: 'Google Chrome',
    extension: '.csv',
    description: 'Chrome saved passwords CSV',
  },
  {
    id: 'firefox',
    name: 'Mozilla Firefox',
    extension: '.csv',
    description: 'Firefox saved logins CSV',
  },
  {
    id: '1password',
    name: '1Password',
    extension: '.1pux',
    description: '1Password 1PUX export',
  },
  {
    id: 'dashlane',
    name: 'Dashlane',
    extension: '.json',
    description: 'Dashlane JSON export',
  },
  {
    id: 'nordpass',
    name: 'NordPass',
    extension: '.csv',
    description: 'NordPass CSV export',
  },
  {
    id: 'enpass',
    name: 'Enpass',
    extension: '.json',
    description: 'Enpass JSON export',
  },
  {
    id: 'roboform',
    name: 'RoboForm',
    extension: '.html',
    description: 'RoboForm HTML export',
  },
  {
    id: 'keepass1',
    name: 'KeePass 1.x',
    extension: '.kdb',
    description: 'Legacy KeePass 1.x database',
  },
  {
    id: 'csv',
    name: 'Generic CSV',
    extension: '.csv',
    description: 'Generic comma-separated values',
  },
];

// ─── Export formats ───────────────────────────────────────────────────────────

export interface ExportFormatDef {
  id: string;
  name: string;
  extension: string;
  description: string;
}

export const EXPORT_FORMATS: ExportFormatDef[] = [
  {
    id: 'kdbx',
    name: 'KDBX 4.x',
    extension: '.kdbx',
    description: 'Encrypted KeePass database (recommended)',
  },
  {
    id: 'csv',
    name: 'CSV',
    extension: '.csv',
    description: 'Comma-separated values (unencrypted)',
  },
  {
    id: 'json',
    name: 'JSON',
    extension: '.json',
    description: 'JSON format (unencrypted)',
  },
  {
    id: 'html',
    name: 'HTML',
    extension: '.html',
    description: 'Printable HTML report (unencrypted)',
  },
];

// ─── Sync providers ───────────────────────────────────────────────────────────

export interface SyncProviderDef {
  id: string;
  name: string;
  icon: string;
  description: string;
}

export const SYNC_PROVIDERS: SyncProviderDef[] = [
  {
    id: 'webdav',
    name: 'WebDAV',
    icon: '���',
    description: 'Any WebDAV-compatible server (Nextcloud, ownCloud, etc.)',
  },
  {
    id: 'gdrive',
    name: 'Google Drive',
    icon: '���',
    description: 'Sync via Google Drive (OAuth2)',
  },
  {
    id: 'onedrive',
    name: 'OneDrive',
    icon: '☁️',
    description: 'Sync via Microsoft OneDrive (OAuth2)',
  },
  {
    id: 'dropbox',
    name: 'Dropbox',
    icon: '���',
    description: 'Sync via Dropbox API v2',
  },
  {
    id: 's3',
    name: 'Amazon S3',
    icon: '���',
    description: 'AWS S3 or any S3-compatible storage (MinIO, Backblaze B2)',
  },
  {
    id: 'sftp',
    name: 'SFTP',
    icon: '���',
    description: 'SSH File Transfer Protocol',
  },
  {
    id: 'icloud',
    name: 'iCloud Drive',
    icon: '���',
    description: 'iCloud Drive (iOS and macOS only)',
  },
  {
    id: 'local',
    name: 'Local Folder',
    icon: '���',
    description: 'Sync to a local or network-mounted folder',
  },
];

// ─── Entry template IDs ───────────────────────────────────────────────────────

export const ENTRY_TEMPLATES_IDS: string[] = [
  'tpl-login',
  'tpl-credit-card',
  'tpl-bank-account',
  'tpl-identity',
  'tpl-secure-note',
  'tpl-software-license',
  'tpl-wireless-router',
  'tpl-passport',
  'tpl-driver-license',
  'tpl-ssh-key',
  'tpl-api-key',
  'tpl-crypto-wallet',
];

// ─── Keyboard shortcuts ───────────────────────────────────────────────────────

export interface KeyboardShortcutDef {
  id: string;
  action: string;
  defaultKeys: string;
  scope: 'global' | 'vault' | 'entry' | 'generator';
}

export const KEYBOARD_SHORTCUTS: KeyboardShortcutDef[] = [
  { id: 'lock-vault',        action: 'Lock Vault',              defaultKeys: 'Ctrl+L',       scope: 'global' },
  { id: 'new-entry',         action: 'New Entry',               defaultKeys: 'Ctrl+N',       scope: 'vault' },
  { id: 'new-group',         action: 'New Group',               defaultKeys: 'Ctrl+G',       scope: 'vault' },
  { id: 'search',            action: 'Focus Search',            defaultKeys: 'Ctrl+F',       scope: 'vault' },
  { id: 'copy-password',     action: 'Copy Password',           defaultKeys: 'Ctrl+C',       scope: 'entry' },
  { id: 'copy-username',     action: 'Copy Username',           defaultKeys: 'Ctrl+B',       scope: 'entry' },
  { id: 'copy-url',          action: 'Copy URL',                defaultKeys: 'Ctrl+U',       scope: 'entry' },
  { id: 'copy-otp',          action: 'Copy OTP',                defaultKeys: 'Ctrl+T',       scope: 'entry' },
  { id: 'open-url',          action: 'Open URL',                defaultKeys: 'Ctrl+Shift+U', scope: 'entry' },
  { id: 'edit-entry',        action: 'Edit Entry',              defaultKeys: 'Enter',        scope: 'entry' },
  { id: 'delete-entry',      action: 'Delete Entry',            defaultKeys: 'Delete',       scope: 'entry' },
  { id: 'autotype',          action: 'Auto-Type',               defaultKeys: 'Ctrl+Shift+A', scope: 'global' },
  { id: 'generate-password', action: 'Open Generator',          defaultKeys: 'Ctrl+Shift+G', scope: 'global' },
  { id: 'save-vault',        action: 'Save Vault',              defaultKeys: 'Ctrl+S',       scope: 'global' },
  { id: 'open-vault',        action: 'Open Vault',              defaultKeys: 'Ctrl+O',       scope: 'global' },
  { id: 'settings',          action: 'Open Settings',           defaultKeys: 'Ctrl+,',       scope: 'global' },
  { id: 'browser-fill',      action: 'Fill in Browser',         defaultKeys: 'Ctrl+Shift+F', scope: 'global' },
];
