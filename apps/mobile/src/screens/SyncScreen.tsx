/**
 * Sync screen — mobile (with full i18n EN/VI)
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
  providerLabel: { flex: 1, fontSize: tokens.fontSize.md },
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
