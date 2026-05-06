/**
 * Sync screen — mobile (with full i18n, all 10 languages)
 * Supports KeePassEx Server (self-hosted, zero-knowledge) + cloud providers
 */
import React, { useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  TextInput,
  Switch,
  StyleSheet,
  Alert,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useMutation } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';

const { KeePassExCore } = NativeModules;

type Provider =
  | 'keepassex_server'
  | 'icloud'
  | 'gdrive'
  | 'onedrive'
  | 'dropbox'
  | 'webdav'
  | 'local';

const PROVIDERS: { value: Provider; label: string; icon: string; featured?: boolean }[] = [
  { value: 'keepassex_server', label: 'KeePassEx Server', icon: '🔐', featured: true },
  { value: 'icloud', label: 'iCloud Drive', icon: '☁️' },
  { value: 'gdrive', label: 'Google Drive', icon: '🔵' },
  { value: 'onedrive', label: 'OneDrive', icon: '🔷' },
  { value: 'dropbox', label: 'Dropbox', icon: '📦' },
  { value: 'webdav', label: 'WebDAV', icon: '🌐' },
  { value: 'local', label: 'Local Folder', icon: '📁' },
];

export function SyncScreen() {
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const [provider, setProvider] = useState<Provider>('keepassex_server');
  const [remotePath, setRemotePath] = useState('');
  const [autoSync, setAutoSync] = useState(false);

  // KeePassEx Server auth state
  const [serverUrl, setServerUrl] = useState('');
  const [serverEmail, setServerEmail] = useState('');
  const [serverPassword, setServerPassword] = useState('');
  const [serverToken, setServerToken] = useState<string | null>(null);
  const [serverAuthMode, setServerAuthMode] = useState<'login' | 'register'>('login');
  const [serverAuthLoading, setServerAuthLoading] = useState(false);

  const syncMutation = useMutation({
    mutationFn: () => KeePassExCore.syncNow(),
    onSuccess: () => Alert.alert('✅', t('sync.syncSuccess')),
    onError: (e: Error) => Alert.alert('❌', t('sync.syncError', { error: e.message })),
  });

  const handleServerAuth = async () => {
    if (!serverUrl.trim() || !serverEmail.trim() || !serverPassword.trim()) return;
    setServerAuthLoading(true);
    try {
      const endpoint =
        serverAuthMode === 'login'
          ? `${serverUrl.trim().replace(/\/$/, '')}/api/v1/auth/login`
          : `${serverUrl.trim().replace(/\/$/, '')}/api/v1/auth/register`;

      const response = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: serverEmail, password: serverPassword }),
      });

      if (!response.ok) {
        const err = await response.json().catch(() => ({ error: 'Unknown error' }));
        throw new Error(err.error || `HTTP ${response.status}`);
      }

      const data = await response.json();
      setServerToken(data.access_token);
      Alert.alert('✅', t('server.connected', { url: serverUrl }));
    } catch (e: unknown) {
      Alert.alert('❌', e instanceof Error ? e.message : String(e));
    } finally {
      setServerAuthLoading(false);
    }
  };

  const inputStyle = [
    styles.input,
    { backgroundColor: theme.surface, borderColor: theme.border, color: theme.text },
  ];

  const isServerProvider = provider === 'keepassex_server';

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <Text style={[styles.headerTitle, { color: theme.text }]}>🔄 {t('sync.title')}</Text>
        <TouchableOpacity
          style={[styles.syncBtn, { backgroundColor: theme.primary }]}
          onPress={() => syncMutation.mutate()}
          disabled={syncMutation.isPending}
          accessibilityRole="button"
          accessibilityLabel={t('sync.syncNow')}
        >
          <Text style={styles.syncBtnText}>
            {syncMutation.isPending ? t('sync.syncing') : t('sync.syncNow')}
          </Text>
        </TouchableOpacity>
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.contentInner}>
        {/* Provider */}
        <Text style={[styles.sectionTitle, { color: theme.textSecondary }]}>
          {t('sync.provider').toUpperCase()}
        </Text>
        <View style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}>
          {PROVIDERS.map((p, i) => (
            <React.Fragment key={p.value}>
              <TouchableOpacity
                style={styles.providerRow}
                onPress={() => setProvider(p.value)}
                accessibilityRole="radio"
                accessibilityState={{ checked: provider === p.value }}
              >
                <Text style={styles.providerIcon}>{p.icon}</Text>
                <View style={{ flex: 1 }}>
                  <Text style={[styles.providerLabel, { color: theme.text }]}>{p.label}</Text>
                  {p.featured && (
                    <Text style={[styles.providerBadge, { color: theme.primary }]}>
                      {t('server.selfHosted')}
                    </Text>
                  )}
                </View>
                <View style={[styles.radio, { borderColor: theme.primary }]}>
                  {provider === p.value && (
                    <View style={[styles.radioDot, { backgroundColor: theme.primary }]} />
                  )}
                </View>
              </TouchableOpacity>
              {i < PROVIDERS.length - 1 && (
                <View style={[styles.divider, { backgroundColor: theme.border }]} />
              )}
            </React.Fragment>
          ))}
        </View>

        {/* KeePassEx Server auth flow */}
        {isServerProvider && (
          <>
            <Text style={[styles.sectionTitle, { color: theme.textSecondary }]}>
              {t('server.title').toUpperCase()}
            </Text>
            <View
              style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}
            >
              <View style={styles.fieldRow}>
                <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
                  {t('server.serverUrl')}
                </Text>
                <TextInput
                  style={[inputStyle, styles.fieldInput]}
                  value={serverUrl}
                  onChangeText={setServerUrl}
                  placeholder={t('server.serverUrlPlaceholder')}
                  placeholderTextColor={theme.textTertiary}
                  autoCapitalize="none"
                  autoCorrect={false}
                  keyboardType="url"
                />
              </View>

              {!serverToken ? (
                <>
                  {/* Auth mode tabs */}
                  <View style={[styles.authTabs, { borderColor: theme.border }]}>
                    <TouchableOpacity
                      style={[
                        styles.authTab,
                        serverAuthMode === 'login' && { backgroundColor: theme.primary },
                      ]}
                      onPress={() => setServerAuthMode('login')}
                    >
                      <Text
                        style={[
                          styles.authTabText,
                          { color: serverAuthMode === 'login' ? 'white' : theme.textSecondary },
                        ]}
                      >
                        {t('server.login')}
                      </Text>
                    </TouchableOpacity>
                    <TouchableOpacity
                      style={[
                        styles.authTab,
                        serverAuthMode === 'register' && { backgroundColor: theme.primary },
                      ]}
                      onPress={() => setServerAuthMode('register')}
                    >
                      <Text
                        style={[
                          styles.authTabText,
                          { color: serverAuthMode === 'register' ? 'white' : theme.textSecondary },
                        ]}
                      >
                        {t('server.register')}
                      </Text>
                    </TouchableOpacity>
                  </View>

                  <View style={styles.fieldRow}>
                    <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
                      {t('server.email')}
                    </Text>
                    <TextInput
                      style={[inputStyle, styles.fieldInput]}
                      value={serverEmail}
                      onChangeText={setServerEmail}
                      placeholder="you@example.com"
                      placeholderTextColor={theme.textTertiary}
                      autoCapitalize="none"
                      keyboardType="email-address"
                    />
                  </View>

                  <View style={styles.fieldRow}>
                    <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
                      {t('server.password')}
                    </Text>
                    <TextInput
                      style={[inputStyle, styles.fieldInput]}
                      value={serverPassword}
                      onChangeText={setServerPassword}
                      placeholder="••••••••"
                      placeholderTextColor={theme.textTertiary}
                      secureTextEntry
                    />
                  </View>

                  <TouchableOpacity
                    style={[
                      styles.authBtn,
                      { backgroundColor: theme.primary, opacity: serverAuthLoading ? 0.7 : 1 },
                    ]}
                    onPress={handleServerAuth}
                    disabled={serverAuthLoading || !serverUrl || !serverEmail || !serverPassword}
                    accessibilityRole="button"
                  >
                    <Text style={styles.authBtnText}>
                      {serverAuthLoading
                        ? t('common.loading')
                        : serverAuthMode === 'login'
                          ? t('server.login')
                          : t('server.register')}
                    </Text>
                  </TouchableOpacity>
                </>
              ) : (
                <View style={[styles.connectedCard, { backgroundColor: 'rgba(16,185,129,0.08)' }]}>
                  <Text style={styles.connectedIcon}>✅</Text>
                  <View style={{ flex: 1 }}>
                    <Text style={[styles.connectedLabel, { color: theme.text }]}>
                      {t('server.connected', { url: serverUrl })}
                    </Text>
                    <Text style={[styles.connectedSub, { color: theme.textSecondary }]}>
                      {t('server.zeroKnowledge')}
                    </Text>
                  </View>
                  <TouchableOpacity
                    onPress={() => {
                      setServerToken(null);
                      setServerEmail('');
                      setServerPassword('');
                    }}
                  >
                    <Text style={[styles.disconnectBtn, { color: theme.danger ?? '#ef4444' }]}>
                      {t('server.disconnect')}
                    </Text>
                  </TouchableOpacity>
                </View>
              )}
            </View>
          </>
        )}

        {/* Remote path (non-server providers) */}
        {!isServerProvider && (
          <>
            <Text style={[styles.sectionTitle, { color: theme.textSecondary }]}>
              {t('sync.remotePath').toUpperCase()}
            </Text>
            <TextInput
              style={inputStyle}
              value={remotePath}
              onChangeText={setRemotePath}
              placeholder="/keepassex/vault.kdbx"
              placeholderTextColor={theme.textTertiary}
              autoCapitalize="none"
              autoCorrect={false}
              accessibilityLabel={t('sync.remotePath')}
            />
          </>
        )}

        {/* Auto sync */}
        <Text style={[styles.sectionTitle, { color: theme.textSecondary }]}>
          {t('settings.advanced').toUpperCase()}
        </Text>
        <View style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}>
          <View style={styles.toggleRow}>
            <View>
              <Text style={[styles.toggleLabel, { color: theme.text }]}>{t('sync.autoSync')}</Text>
              <Text style={[styles.toggleDesc, { color: theme.textSecondary }]}>
                {t('sync.syncInterval')}
              </Text>
            </View>
            <Switch
              value={autoSync}
              onValueChange={setAutoSync}
              trackColor={{ false: theme.border, true: theme.primary }}
              thumbColor="white"
              accessibilityLabel={t('sync.autoSync')}
            />
          </View>
        </View>

        <TouchableOpacity
          style={[styles.saveBtn, { backgroundColor: theme.primary }]}
          onPress={() => Alert.alert('✅', t('sync.syncSuccess'))}
          accessibilityRole="button"
        >
          <Text style={styles.saveBtnText}>💾 {t('common.save')}</Text>
        </TouchableOpacity>
      </ScrollView>
    </SafeAreaView>
  );
}
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  TextInput,
  Switch,
  StyleSheet,
  Alert,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useMutation } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';

const { KeePassExCore } = NativeModules;

type Provider = 'local' | 'webdav' | 'icloud' | 'gdrive' | 'onedrive' | 'dropbox';

const PROVIDERS: { value: Provider; label: string; icon: string }[] = [
  { value: 'icloud', label: 'iCloud Drive', icon: '☁️' },
  { value: 'gdrive', label: 'Google Drive', icon: '🔵' },
  { value: 'onedrive', label: 'OneDrive', icon: '🔷' },
  { value: 'dropbox', label: 'Dropbox', icon: '📦' },
  { value: 'webdav', label: 'WebDAV', icon: '🌐' },
  { value: 'local', label: 'Local Folder', icon: '📁' },
];

export function SyncScreen() {
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const [provider, setProvider] = useState<Provider>('icloud');
  const [remotePath, setRemotePath] = useState('');
  const [autoSync, setAutoSync] = useState(false);

  const syncMutation = useMutation({
    mutationFn: () => KeePassExCore.syncNow(),
    onSuccess: () => Alert.alert('✅', t('sync.syncSuccess')),
    onError: (e: Error) => Alert.alert('❌', t('sync.syncError', { error: e.message })),
  });

  const inputStyle = [
    styles.input,
    { backgroundColor: theme.surface, borderColor: theme.border, color: theme.text },
  ];

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <Text style={[styles.headerTitle, { color: theme.text }]}>🔄 Sync</Text>
        <TouchableOpacity
          style={[styles.syncBtn, { backgroundColor: theme.primary }]}
          onPress={() => syncMutation.mutate()}
          disabled={syncMutation.isPending}
          accessibilityRole="button"
          accessibilityLabel="Sync now"
        >
          <Text style={styles.syncBtnText}>{syncMutation.isPending ? '...' : 'Sync Now'}</Text>
        </TouchableOpacity>
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.contentInner}>
        {/* Provider */}
        <Text style={[styles.sectionTitle, { color: theme.textSecondary }]}>PROVIDER</Text>
        <View style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}>
          {PROVIDERS.map((p, i) => (
            <React.Fragment key={p.value}>
              <TouchableOpacity
                style={styles.providerRow}
                onPress={() => setProvider(p.value)}
                accessibilityRole="radio"
                accessibilityState={{ checked: provider === p.value }}
              >
                <Text style={styles.providerIcon}>{p.icon}</Text>
                <Text style={[styles.providerLabel, { color: theme.text }]}>{p.label}</Text>
                <View style={[styles.radio, { borderColor: theme.primary }]}>
                  {provider === p.value && (
                    <View style={[styles.radioDot, { backgroundColor: theme.primary }]} />
                  )}
                </View>
              </TouchableOpacity>
              {i < PROVIDERS.length - 1 && (
                <View style={[styles.divider, { backgroundColor: theme.border }]} />
              )}
            </React.Fragment>
          ))}
        </View>

        {/* Remote path */}
        <Text style={[styles.sectionTitle, { color: theme.textSecondary }]}>REMOTE PATH</Text>
        <TextInput
          style={inputStyle}
          value={remotePath}
          onChangeText={setRemotePath}
          placeholder="/keepassex/vault.kdbx"
          placeholderTextColor={theme.textTertiary}
          autoCapitalize="none"
          autoCorrect={false}
          accessibilityLabel="Remote path"
        />

        {/* Auto sync */}
        <Text style={[styles.sectionTitle, { color: theme.textSecondary }]}>OPTIONS</Text>
        <View style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}>
          <View style={styles.toggleRow}>
            <View>
              <Text style={[styles.toggleLabel, { color: theme.text }]}>Auto Sync</Text>
              <Text style={[styles.toggleDesc, { color: theme.textSecondary }]}>
                Sync when vault opens and closes
              </Text>
            </View>
            <Switch
              value={autoSync}
              onValueChange={setAutoSync}
              trackColor={{ false: theme.border, true: theme.primary }}
              thumbColor="white"
              accessibilityLabel="Auto sync"
            />
          </View>
        </View>

        <TouchableOpacity
          style={[styles.saveBtn, { backgroundColor: theme.primary }]}
          onPress={() => Alert.alert('Saved', 'Sync configuration saved')}
          accessibilityRole="button"
        >
          <Text style={styles.saveBtnText}>💾 Save Configuration</Text>
        </TouchableOpacity>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.md,
    borderBottomWidth: StyleSheet.hairlineWidth,
  },
  headerTitle: { fontSize: tokens.fontSize.xl, fontWeight: tokens.fontWeight.bold },
  syncBtn: {
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.sm,
    borderRadius: tokens.radius.md,
  },
  syncBtnText: {
    color: 'white',
    fontWeight: tokens.fontWeight.semibold,
    fontSize: tokens.fontSize.sm,
  },
  content: { flex: 1 },
  contentInner: { padding: tokens.space.lg, gap: tokens.space.sm },
  sectionTitle: {
    fontSize: 11,
    fontWeight: '600',
    letterSpacing: 0.8,
    paddingHorizontal: tokens.space.xs,
    paddingTop: tokens.space.md,
    paddingBottom: tokens.space.xs,
  },
  card: {
    borderRadius: tokens.radius.lg,
    borderWidth: StyleSheet.hairlineWidth,
    overflow: 'hidden',
  },
  providerRow: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: tokens.space.md,
    gap: tokens.space.md,
  },
  providerIcon: { fontSize: 20 },
  providerLabel: { fontSize: tokens.fontSize.md },
  providerBadge: { fontSize: 10, fontWeight: '600', marginTop: 1 },
  radio: {
    width: 20,
    height: 20,
    borderRadius: 10,
    borderWidth: 2,
    alignItems: 'center',
    justifyContent: 'center',
  },
  radioDot: { width: 10, height: 10, borderRadius: 5 },
  divider: { height: StyleSheet.hairlineWidth, marginLeft: tokens.space.lg },
  input: {
    borderWidth: 1,
    borderRadius: tokens.radius.md,
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.sm,
    fontSize: tokens.fontSize.md,
  },
  fieldRow: { padding: tokens.space.md, gap: tokens.space.xs },
  fieldLabel: { fontSize: 11, fontWeight: '600', letterSpacing: 0.5, textTransform: 'uppercase' },
  fieldInput: { marginTop: 4 },
  authTabs: {
    flexDirection: 'row',
    borderTopWidth: StyleSheet.hairlineWidth,
    borderBottomWidth: StyleSheet.hairlineWidth,
  },
  authTab: { flex: 1, paddingVertical: tokens.space.sm, alignItems: 'center' },
  authTabText: { fontSize: tokens.fontSize.sm, fontWeight: '600' },
  authBtn: {
    margin: tokens.space.md,
    borderRadius: tokens.radius.md,
    paddingVertical: tokens.space.sm,
    alignItems: 'center',
  },
  authBtnText: {
    color: 'white',
    fontWeight: tokens.fontWeight.semibold,
    fontSize: tokens.fontSize.md,
  },
  connectedCard: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.md,
    padding: tokens.space.md,
    margin: tokens.space.sm,
    borderRadius: tokens.radius.md,
  },
  connectedIcon: { fontSize: 24 },
  connectedLabel: { fontSize: tokens.fontSize.md, fontWeight: '600' },
  connectedSub: { fontSize: tokens.fontSize.sm, marginTop: 2 },
  disconnectBtn: { fontSize: tokens.fontSize.sm, fontWeight: '600' },
  toggleRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: tokens.space.md,
    gap: tokens.space.md,
  },
  toggleLabel: { fontSize: tokens.fontSize.md },
  toggleDesc: { fontSize: tokens.fontSize.sm, marginTop: 2 },
  saveBtn: {
    borderRadius: tokens.radius.md,
    paddingVertical: tokens.space.md,
    alignItems: 'center',
    marginTop: tokens.space.md,
  },
  saveBtnText: {
    color: 'white',
    fontWeight: tokens.fontWeight.semibold,
    fontSize: tokens.fontSize.md,
  },
});
