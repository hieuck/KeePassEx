/**
 * KeePassEx shared TypeScript types
 * Comprehensive type definitions for all platforms
 */

// ─── Core Vault Entities ──────────────────────────────────────────────────────

export interface Entry {
  uuid: string;
  groupUuid: string;
  title: string;
  username: string;
  password?: string;
  url: string;
  /** Additional URLs for multi-URL entries */
  additionalUrls?: string[];
  notes: string;
  iconId: number;
  /** Custom icon UUID (overrides iconId) */
  customIconUuid?: string;
  tags: string[];
  hasPassword: boolean;
  hasOtp: boolean;
  hasPasskey: boolean;
  hasSshKey: boolean;
  hasAttachments: boolean;
  isExpired: boolean;
  expiry?: string;
  createdAt: string;
  modifiedAt: string;
  accessedAt?: string;
  usageCount: number;
  customFields: CustomField[];
  autoTypeEnabled: boolean;
  autoTypeSequence?: string;
  autoTypeObfuscation: boolean;
  qualityCheck: boolean;
}

export interface CustomField {
  key: string;
  value: string;
  protected: boolean;
}

export interface Group {
  uuid: string;
  parentUuid?: string;
  name: string;
  notes: string;
  iconId: number;
  customIconUuid?: string;
  isExpanded: boolean;
  entryCount: number;
  childGroupCount: number;
  defaultAutoTypeSequence?: string;
  enableAutoType?: boolean;
  enableSearching?: boolean;
  lastTopVisibleEntry?: string;
}

export interface VaultMeta {
  name: string;
  description: string;
  entryCount: number;
  groupCount: number;
  path: string;
  kdbxVersion: '3.1' | '4.0' | '4.1';
  generator: string;
  recycleBinEnabled: boolean;
  recycleBinUuid?: string;
  historyMaxItems: number;
  historyMaxSize: number;
  maintenanceHistoryDays: number;
  masterKeyChanged?: string;
  masterKeyChangeRec?: number;
  masterKeyChangeForce?: number;
}

export interface CustomIcon {
  uuid: string;
  data: string; // base64 PNG
  name?: string;
  lastModified?: string;
}

// ─── OTP ──────────────────────────────────────────────────────────────────────

export interface OtpCode {
  code: string;
  remainingSeconds: number;
  period: number;
  progress: number;
  issuer?: string;
  account?: string;
}

export interface OtpConfig {
  secret: string;
  algorithm: 'SHA1' | 'SHA256' | 'SHA512';
  digits: number;
  period: number;
  otpType: 'Totp' | 'Hotp';
  counter?: number; // for HOTP
  issuer?: string;
  account?: string;
}

// ─── Password Generator ───────────────────────────────────────────────────────

export interface PasswordGeneratorOptions {
  mode: 'random' | 'passphrase' | 'pronounceable';
  length: number;
  useUppercase: boolean;
  useLowercase: boolean;
  useDigits: boolean;
  useSymbols: boolean;
  customSymbols?: string;
  excludeAmbiguous: boolean;
  excludeChars: string;
  minUppercase: number;
  minLowercase: number;
  minDigits: number;
  minSymbols: number;
  wordCount: number;
  wordSeparator: string;
  capitalizeWords: boolean;
  includeNumber: boolean;
  wordlist?: 'eff' | 'bip39' | 'custom';
}

export interface GeneratedPassword {
  password: string;
  entropy: number;
  strengthScore: 0 | 1 | 2 | 3 | 4;
  strengthLabel: string;
}

// ─── Health & Security ────────────────────────────────────────────────────────

export interface HealthReport {
  totalEntries: number;
  score: number;
  weakCount: number;
  reusedCount: number;
  expiredCount: number;
  expiringSoonCount: number;
  noPasswordCount: number;
  oldPasswordCount: number;
  noUrlCount: number;
  weakPasswords: WeakPasswordIssue[];
  reusedPasswords: ReusedPasswordGroup[];
  expiredEntries: ExpiredEntry[];
  expiringSoon: ExpiringEntry[];
  noPasswordEntries: EntryRef[];
  oldPasswordEntries: OldPasswordEntry[];
}

export interface WeakPasswordIssue {
  entryUuid: string;
  entryTitle: string;
  strengthScore: number;
  strengthLabel: string;
}

export interface ReusedPasswordGroup {
  entries: EntryRef[];
  passwordHash: string; // SHA-256 prefix for display
}

export interface EntryRef {
  uuid: string;
  title: string;
  groupName?: string;
}

export interface ExpiredEntry {
  entryUuid: string;
  entryTitle: string;
  expiredAt: string;
}

export interface ExpiringEntry {
  entryUuid: string;
  entryTitle: string;
  expiresAt: string;
  daysRemaining: number;
}

export interface OldPasswordEntry {
  entryUuid: string;
  entryTitle: string;
  lastChangedAt: string;
  daysSinceChange: number;
}

/** Custom audit rule for the plugin system / advanced health checks */
export interface PasswordAuditRule {
  id: string;
  name: string;
  description: string;
  minLength?: number;
  maxLength?: number;
  requireUppercase?: boolean;
  requireLowercase?: boolean;
  requireDigits?: boolean;
  requireSymbols?: boolean;
  forbiddenPatterns?: string[];
  maxAgeDays?: number;
  enabled: boolean;
}

// ─── Sync ─────────────────────────────────────────────────────────────────────

export type SyncProvider =
  | 'webdav'
  | 'icloud'
  | 'gdrive'
  | 'onedrive'
  | 'dropbox'
  | 's3'
  | 'sftp'
  | 'local';

export interface SyncConfig {
  provider: SyncProvider;
  remotePath: string;
  autoSync: boolean;
  syncIntervalSeconds: number;
  conflictResolution: 'keepLocal' | 'keepRemote' | 'merge' | 'askUser';
  /** Provider-specific credentials (stored encrypted) */
  credentials?: SyncCredentials;
}

export interface SyncCredentials {
  username?: string;
  password?: string;
  token?: string;
  /** S3 / compatible */
  accessKeyId?: string;
  secretAccessKey?: string;
  region?: string;
  bucket?: string;
  endpoint?: string;
  /** SFTP */
  privateKeyPath?: string;
  hostFingerprint?: string;
}

export interface SyncStatus {
  configured: boolean;
  provider?: SyncProvider;
  remotePath?: string;
  autoSync: boolean;
  lastSync?: string;
  lastSyncStatus?: 'success' | 'error' | 'conflict' | 'no_changes';
  lastError?: string;
}

export interface SyncResult {
  status: 'success' | 'error' | 'conflict' | 'no_changes';
  entriesUploaded: number;
  entriesDownloaded: number;
  conflicts: number;
  error?: string;
  durationMs?: number;
}

// ─── Passkey (FIDO2 / WebAuthn) ───────────────────────────────────────────────

export interface PasskeyEntry {
  credentialId: string; // base64url
  rpId: string;
  rpName: string;
  userId: string; // base64url
  userName: string;
  userDisplayName: string;
  signCount: number;
  createdAt: string;
  lastUsedAt?: string;
  backupEligible: boolean;
  backupState: boolean;
  /** COSE algorithm identifier (e.g. -7 = ES256, -8 = EdDSA) */
  algorithm?: number;
  transports?: Array<'usb' | 'nfc' | 'ble' | 'internal' | 'hybrid'>;
}

export interface PasskeyRegistrationOptions {
  rp: { id: string; name: string };
  user: { id: string; name: string; displayName: string };
  challenge: string; // base64url
  pubKeyCredParams: Array<{ alg: number; type: string }>;
  timeout: number;
  excludeCredentials: CredentialDescriptor[];
  authenticatorSelection: {
    authenticatorAttachment?: 'platform' | 'cross-platform';
    residentKey: 'required' | 'preferred' | 'discouraged';
    userVerification: 'required' | 'preferred' | 'discouraged';
  };
  attestation: 'none' | 'indirect' | 'direct';
}

export interface CredentialDescriptor {
  id: string; // base64url
  type: 'public-key';
  transports?: Array<'usb' | 'nfc' | 'ble' | 'internal' | 'hybrid'>;
}

// ─── SSH ──────────────────────────────────────────────────────────────────────

export type SshKeyType = 'Ed25519' | 'Rsa2048' | 'Rsa4096' | 'EcdsaP256' | 'EcdsaP384';

export interface SshKeyEntry {
  keyType: SshKeyType;
  publicKey: string;
  comment: string;
  fingerprint: string;
  addToAgent: boolean;
  agentDuration?: number; // seconds, undefined = forever
  confirmBeforeUse: boolean;
}

// ─── Hardware Key (YubiKey / FIDO2 / Smart Card) ──────────────────────────────

export type HardwareKeyType =
  | 'yubikey_hmac' // YubiKey HMAC-SHA1 challenge-response (slot 1 or 2)
  | 'yubikey_otp' // YubiKey OTP (Yubico OTP protocol)
  | 'fido2' // FIDO2 / WebAuthn hardware key
  | 'smart_card' // PIV smart card / CAC
  | 'onlykey'; // OnlyKey

export interface HardwareKeyConfig {
  type: HardwareKeyType;
  /** YubiKey serial number or FIDO2 credential ID */
  deviceId?: string;
  /** YubiKey slot (1 or 2) for HMAC-SHA1 */
  slot?: 1 | 2;
  /** Human-readable label */
  label?: string;
  /** Whether to require touch confirmation */
  requireTouch: boolean;
}

export interface HardwareKeyInfo {
  type: HardwareKeyType;
  deviceId: string;
  label: string;
  firmwareVersion?: string;
  serialNumber?: string;
  isConnected: boolean;
}

// ─── Emergency Access ─────────────────────────────────────────────────────────

export type EmergencyAccessLevel = 'view' | 'takeover';

export type EmergencyAccessStatus =
  | 'invited'
  | 'confirmed'
  | 'recovery_initiated'
  | 'recovery_approved'
  | 'recovery_granted'
  | 'revoked';

export interface EmergencyAccess {
  id: string;
  granteeId: string;
  granteeName: string;
  granteeEmail: string;
  accessLevel: EmergencyAccessLevel;
  waitTimeDays: number;
  status: EmergencyAccessStatus;
  requestInitiatedAt?: string;
  accessGrantedAt?: string;
  daysRemaining?: number;
  createdAt: string;
  updatedAt: string;
}

export interface AddEmergencyAccessArgs {
  granteeName: string;
  granteeEmail: string;
  accessLevel: EmergencyAccessLevel;
  waitTimeDays: number;
}

// ─── Plugins ──────────────────────────────────────────────────────────────────

export type PluginCapability =
  | 'password_generator'
  | 'importer'
  | 'exporter'
  | 'health_check'
  | 'field_validator'
  | 'icon_set'
  | 'ui_extension'
  | 'sync_provider'
  | 'autotype_action';

export type PluginPermission =
  | 'read_entry_metadata'
  | 'read_passwords'
  | 'write_entries'
  | 'network'
  | 'file_system'
  | 'clipboard'
  | 'notifications';

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  license: string;
  homepage?: string;
  repository?: string;
  capabilities: PluginCapability[];
  permissions: PluginPermission[];
  entryPoint: string;
  minKeepassexVersion: string;
  /** SHA-256 hash of the WASM binary for integrity verification */
  wasmHash?: string;
}

export interface InstalledPlugin {
  manifest: PluginManifest;
  enabled: boolean;
  installPath: string;
  installedAt: string;
  lastUsed?: string;
  /** Plugin store catalog entry ID */
  catalogId?: string;
}

export interface PluginCatalogEntry {
  id: string;
  name: string;
  description: string;
  author: string;
  version: string;
  downloadUrl: string;
  wasmHash: string;
  capabilities: PluginCapability[];
  downloads: number;
  rating?: number;
  verified: boolean;
  tags: string[];
}

// ─── Breach ───────────────────────────────────────────────────────────────────

export interface BreachCheckResult {
  entryUuid: string;
  entryTitle: string;
  isBreached: boolean;
  breachCount: number;
  hashPrefix: string;
}

export interface VaultBreachReport {
  totalChecked: number;
  breachedCount: number;
  results: BreachCheckResult[];
  usedOnline: boolean;
  checkedAt: string;
}

// ─── Import/Export ────────────────────────────────────────────────────────────

export type ImportFormat =
  | 'auto'
  | 'kdbx'
  | 'bitwarden'
  | 'lastpass'
  | 'chrome'
  | 'firefox'
  | '1password'
  | 'dashlane'
  | 'nordpass'
  | 'enpass'
  | 'roboform'
  | 'csv';

export type ExportFormat = 'kdbx' | 'csv' | 'json' | 'html';

export interface ImportResult {
  entriesImported: number;
  groupsCreated: number;
  entriesSkipped: number;
  warnings: string[];
  errors: string[];
}

// ─── Auto-Type ────────────────────────────────────────────────────────────────

export interface AutoTypeSequence {
  /** Window title pattern (glob or regex) */
  windowPattern: string;
  /** Auto-type sequence string, e.g. "{USERNAME}{TAB}{PASSWORD}{ENTER}" */
  sequence: string;
  enabled: boolean;
}

export type AutoTypePlaceholder =
  | '{USERNAME}'
  | '{PASSWORD}'
  | '{URL}'
  | '{TITLE}'
  | '{NOTES}'
  | '{TAB}'
  | '{ENTER}'
  | '{DELAY 500}'
  | '{CLEARFIELD}'
  | string; // custom field {S:FieldName}

// ─── Vault Templates ──────────────────────────────────────────────────────────

export interface VaultTemplate {
  id: string;
  name: string;
  description: string;
  iconId: number;
  fields: VaultTemplateField[];
  isBuiltIn: boolean;
}

export interface VaultTemplateField {
  key: string;
  label: string;
  type: 'text' | 'password' | 'url' | 'email' | 'date' | 'number' | 'multiline';
  protected: boolean;
  required: boolean;
  placeholder?: string;
  defaultValue?: string;
}

// ─── Notifications ────────────────────────────────────────────────────────────

export interface NotificationSettings {
  enabled: boolean;
  expiryWarningDays: number;
  breachAlerts: boolean;
  syncErrors: boolean;
  emergencyAccessRequests: boolean;
  /** Platform-specific: show in system notification center */
  useSystemNotifications: boolean;
}

export interface AppNotification {
  id: string;
  type: 'expiry_warning' | 'breach_alert' | 'sync_error' | 'emergency_access' | 'update_available';
  title: string;
  body: string;
  entryUuid?: string;
  createdAt: string;
  read: boolean;
  actionUrl?: string;
}

// ─── App Settings ─────────────────────────────────────────────────────────────

export type ThemeMode = 'light' | 'dark' | 'oled' | 'system';
export type Language = 'en' | 'vi' | 'zh' | 'ja' | 'ko' | 'es' | 'fr' | 'de' | 'pt' | 'ru';

/** Mobile biometric security level */
export type BiometricSecurityLevel =
  | 'SecureEnclave' // iOS: hardware-backed, keys never leave chip
  | 'StrongBox' // Android: dedicated HSM (Pixel 3+, Galaxy S10+)
  | 'TEE' // Android: Trusted Execution Environment
  | 'Software' // Software-only (not recommended)
  | 'None'; // Not available

export interface AppSettings {
  language: Language;
  theme: ThemeMode;
  lockOnMinimize: boolean;
  lockOnScreenLock: boolean;
  lockAfterIdleMinutes?: number;
  clipboardClearSeconds?: number;
  showPasswordInList: boolean;
  minimizeToTray: boolean;
  startMinimized: boolean;
  checkForUpdates: boolean;
  browserIntegration: boolean;
  sshAgentEnabled: boolean;
  defaultAutoTypeSequence: string;
  /** Require confirmation before auto-type */
  autoTypeConfirm: boolean;
  /** Show entry count in group tree */
  showGroupEntryCount: boolean;
  /** Compact entry list mode */
  compactMode: boolean;
  notifications: NotificationSettings;
  /** Advanced: number of history items to keep per entry */
  historyMaxItems: number;
  /** Advanced: max total history size in bytes */
  historyMaxSizeBytes: number;
  /** Advanced: days before master password change is recommended */
  masterKeyChangeRecDays?: number;
}

// ─── Recent Vault ─────────────────────────────────────────────────────────────

export interface RecentVault {
  path: string;
  name: string;
  lastOpened: string;
  /** Whether biometric unlock is configured for this vault */
  hasBiometric: boolean;
  /** Whether a hardware key is required */
  requiresHardwareKey: boolean;
}

// ─── Keyboard Shortcuts ───────────────────────────────────────────────────────

export interface KeyboardShortcut {
  id: string;
  action: string;
  defaultKeys: string;
  currentKeys: string;
  scope: 'global' | 'vault' | 'entry' | 'generator';
}

// ─── Statistics ───────────────────────────────────────────────────────────────

export interface VaultStatistics {
  totalEntries: number;
  totalGroups: number;
  totalAttachments: number;
  totalAttachmentSize: number;
  totalCustomFields: number;
  entriesWithOtp: number;
  entriesWithPasskey: number;
  entriesWithSshKey: number;
  entriesWithExpiry: number;
  entriesExpired: number;
  entriesExpiringSoon: number;
  oldestEntry?: string;
  newestEntry?: string;
  mostUsedEntry?: EntryRef;
  averagePasswordStrength: number;
}

// ─── Field References ─────────────────────────────────────────────────────────

/** A resolved entry with all {REF:...} placeholders replaced */
export interface ResolvedEntry {
  uuid: string;
  title: string;
  username: string;
  password: string;
  url: string;
  notes: string;
}

/** Field codes for field references */
export type FieldRefCode = 'T' | 'U' | 'P' | 'A' | 'N' | 'I';

// ─── Favicon ──────────────────────────────────────────────────────────────────

export interface FaviconResult {
  /** Base64-encoded icon data */
  dataBase64: string;
  /** MIME type */
  mimeType: string;
  /** Domain the icon was fetched for */
  domain: string;
  /** Which strategy was used */
  source: string;
}

// ─── Multi-Vault Tabs ─────────────────────────────────────────────────────────

export interface VaultTabMeta {
  id: string;
  path: string;
  name: string;
  entryCount: number;
  groupCount: number;
  isLocked: boolean;
  isDirty: boolean;
  lastAccessed: string;
}
