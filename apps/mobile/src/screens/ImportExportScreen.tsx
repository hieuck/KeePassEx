/**
 * Import/Export screen — mobile (with full i18n EN/VI)
 */
import React, { useState } from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, Alert } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import DocumentPicker from 'react-native-document-picker';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';

const { KeePassExCore } = NativeModules;

type ImportFormat = 'auto' | 'bitwarden' | 'lastpass' | 'chrome' | 'csv';

const IMPORT_FORMATS: { value: ImportFormat; label: string; desc: string }[] = [
  { value: 'auto', label: 'Auto-detect', desc: 'Automatically detect format' },
  { value: 'bitwarden', label: 'Bitwarden', desc: 'JSON export from Bitwarden' },
  { value: 'lastpass', label: 'LastPass', desc: 'CSV export from LastPass' },
  { value: 'chrome', label: 'Chrome / Edge', desc: 'CSV export from Chrome' },
  { value: 'csv', label: 'Generic CSV', desc: 'title, username, password, url' },
];

export function ImportExportScreen() {
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [importFormat, setImportFormat] = useState<ImportFormat>('auto');

  const IMPORT_FORMATS_I18N: { value: ImportFormat; label: string; desc: string }[] = [
    {
      value: 'auto',
      label:
        t('importExport.formats.kdbx4').split(' ')[0] === 'KDBX' ? 'Auto-detect' : 'Tự nhận dạng',
      desc: t('importExport.detectingFormat'),
    },
    { value: 'bitwarden', label: 'Bitwarden', desc: t('importExport.formats.bitwarden') },
    { value: 'lastpass', label: 'LastPass', desc: t('importExport.formats.lastpass') },
    { value: 'chrome', label: 'Chrome / Edge', desc: t('importExport.formats.chrome') },
    { value: 'csv', label: 'CSV', desc: t('importExport.formats.csv') },
  ];

  const importMutation = useMutation({
    mutationFn: async (filePath: string) => {
      return KeePassExCore.importVault({
        file_path: filePath,
        format: importFormat === 'auto' ? null : importFormat,
        target_group_uuid: null,
      });
    },
    onSuccess: (result: { entriesImported: number; entriesSkipped: number }) => {
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      queryClient.invalidateQueries({ queryKey: ['groups'] });
      Alert.alert('✅', t('importExport.importSuccess', { count: result.entriesImported }));
    },
    onError: (e: Error) => Alert.alert('❌', e.message),
  });

  const exportMutation = useMutation({
    mutationFn: async (format: 'csv' | 'json') => KeePassExCore.exportVault({ format }),
    onSuccess: () => Alert.alert('✅', t('importExport.exportSuccess', { path: '' })),
    onError: (e: Error) => Alert.alert('❌', e.message),
  });

  const handlePickFile = async () => {
    try {
      const result = await DocumentPicker.pickSingle({
        type: [DocumentPicker.types.allFiles],
        copyTo: 'cachesDirectory',
      });
      if (result.fileCopyUri) importMutation.mutate(result.fileCopyUri);
    } catch (e) {
      if (!DocumentPicker.isCancel(e)) Alert.alert(t('common.error'), t('errors.generic'));
    }
  };

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <Text style={[styles.headerTitle, { color: theme.text }]}>📥 Import / Export</Text>
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.contentInner}>
        {/* Import section */}
        <Text style={[styles.sectionTitle, { color: theme.textSecondary }]}>IMPORT</Text>
        <View style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}>
          <Text style={[styles.cardDesc, { color: theme.textSecondary }]}>
            Import from Bitwarden, LastPass, Chrome, or CSV
          </Text>

          {/* Format picker */}
          {IMPORT_FORMATS.map((fmt, i) => (
            <React.Fragment key={fmt.value}>
              <TouchableOpacity
                style={styles.formatRow}
                onPress={() => setImportFormat(fmt.value)}
                accessibilityRole="radio"
                accessibilityState={{ checked: importFormat === fmt.value }}
              >
                <View style={styles.formatInfo}>
                  <Text style={[styles.formatLabel, { color: theme.text }]}>{fmt.label}</Text>
                  <Text style={[styles.formatDesc, { color: theme.textSecondary }]}>
                    {fmt.desc}
                  </Text>
                </View>
                <View style={[styles.radio, { borderColor: theme.primary }]}>
                  {importFormat === fmt.value && (
                    <View style={[styles.radioDot, { backgroundColor: theme.primary }]} />
                  )}
                </View>
              </TouchableOpacity>
              {i < IMPORT_FORMATS.length - 1 && (
                <View style={[styles.divider, { backgroundColor: theme.border }]} />
              )}
            </React.Fragment>
          ))}
        </View>

        <TouchableOpacity
          style={[styles.actionBtn, { backgroundColor: theme.primary }]}
          onPress={handlePickFile}
          disabled={importMutation.isPending}
          accessibilityRole="button"
          accessibilityLabel="Choose file to import"
        >
          <Text style={styles.actionBtnText}>
            {importMutation.isPending ? '⏳ Importing...' : '📂 Choose File to Import'}
          </Text>
        </TouchableOpacity>

        {/* Export section */}
        <Text style={[styles.sectionTitle, { color: theme.textSecondary }]}>EXPORT</Text>
        <View style={[styles.warningBox, { backgroundColor: '#FFFBEB', borderColor: '#FCD34D' }]}>
          <Text style={styles.warningText}>
            ⚠️ Exported files contain unencrypted passwords. Keep them secure!
          </Text>
        </View>

        <View style={styles.exportBtns}>
          <TouchableOpacity
            style={[styles.exportBtn, { borderColor: theme.border }]}
            onPress={() => exportMutation.mutate('csv')}
            disabled={exportMutation.isPending}
            accessibilityRole="button"
          >
            <Text style={[styles.exportBtnText, { color: theme.text }]}>📄 Export CSV</Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.exportBtn, { borderColor: theme.border }]}
            onPress={() => exportMutation.mutate('json')}
            disabled={exportMutation.isPending}
            accessibilityRole="button"
          >
            <Text style={[styles.exportBtnText, { color: theme.text }]}>📋 Export JSON</Text>
          </TouchableOpacity>
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  header: {
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.md,
    borderBottomWidth: StyleSheet.hairlineWidth,
  },
  headerTitle: { fontSize: tokens.fontSize.xl, fontWeight: tokens.fontWeight.bold },
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
  cardDesc: { fontSize: tokens.fontSize.sm, padding: tokens.space.md, paddingBottom: 0 },
  formatRow: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: tokens.space.md,
    gap: tokens.space.md,
  },
  formatInfo: { flex: 1 },
  formatLabel: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.medium },
  formatDesc: { fontSize: tokens.fontSize.xs, marginTop: 2 },
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
  actionBtn: {
    borderRadius: tokens.radius.md,
    paddingVertical: tokens.space.md,
    alignItems: 'center',
    marginTop: tokens.space.sm,
  },
  actionBtnText: {
    color: 'white',
    fontWeight: tokens.fontWeight.semibold,
    fontSize: tokens.fontSize.md,
  },
  warningBox: {
    borderWidth: 1,
    borderRadius: tokens.radius.md,
    padding: tokens.space.md,
  },
  warningText: { fontSize: tokens.fontSize.sm, color: '#92400E' },
  exportBtns: { flexDirection: 'row', gap: tokens.space.sm },
  exportBtn: {
    flex: 1,
    borderWidth: 1,
    borderRadius: tokens.radius.md,
    paddingVertical: tokens.space.md,
    alignItems: 'center',
  },
  exportBtnText: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.medium },
});
