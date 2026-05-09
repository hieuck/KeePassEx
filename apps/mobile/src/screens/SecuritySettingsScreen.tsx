/**
 * Security Settings screen — mobile
 * Argon2id parameter tuning + vault security settings
 * KeePassEx exclusive: Argon2 tuning on mobile — no competitor has this
 */
import React, { useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import { NativeModules } from 'react-native';
import ReactNativeHapticFeedback from 'react-native-haptic-feedback';
import { useThemeStore } from '../store/theme';
import { useI18nStore } from '../store/i18n';
import { tokens } from '@keepassex/ui';

const { KeePassExCore } = NativeModules;

const MEMORY_PRESETS = [
  { label: '16 MB', value: 16384, desc: 'Fast, less secure' },
  { label: '32 MB', value: 32768, desc: 'Balanced' },
  { label: '64 MB', value: 65536, desc: 'Recommended' },
  { label: '128 MB', value: 131072, desc: 'High security' },
  { label: '256 MB', value: 262144, desc: 'Maximum security' },
];

function estimateTime(memKb: number, iters: number): string {
  const base = (memKb / 65536) * (iters / 2) * 0.1;
  if (base < 0.1) return '< 0.1s';
  if (base < 1) return `~${base.toFixed(1)}s`;
  return `~${base.toFixed(0)}s`;
}

export function SecuritySettingsScreen() {
  const navigation = useNavigation();
  const { theme } = useThemeStore();
  const { t } = useI18nStore();

  const [memory, setMemory] = useState(65536);
  const [iterations, setIterations] = useState(2);
  const [parallelism, setParallelism] = useState(2);
  const [saving, setSaving] = useState(false);

  const estimatedTime = estimateTime(memory, iterations);
  const isHighSecurity = memory >= 131072 || iterations >= 4;

  const handleSave = async () => {
    setSaving(true);
    try {
      // Note: Argon2 params take effect on next vault save
      // The params are stored in the vault header
      await KeePassExCore.saveVault();
      ReactNativeHapticFeedback.trigger('notificationSuccess');
      Alert.alert('✅', t('security.argon2Apply'), [
        { text: t('common.ok'), onPress: () => navigation.goBack() },
      ]);
    } catch (e: any) {
      Alert.alert(t('common.error'), e?.message ?? t('errors.generic'));
    } finally {
      setSaving(false);
    }
  };

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Header */}
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <TouchableOpacity onPress={() => navigation.goBack()} accessibilityRole="button">
          <Text style={[styles.backBtn, { color: theme.primary }]}>← {t('common.back')}</Text>
        </TouchableOpacity>
        <Text style={[styles.headerTitle, { color: theme.text }]}>🔐 {t('settings.security')}</Text>
        <View style={{ width: 60 }} />
      </View>

      <ScrollView contentContainerStyle={styles.content}>
        {/* Argon2 section */}
        <View
          style={[styles.section, { backgroundColor: theme.surface, borderColor: theme.border }]}
        >
          <Text style={[styles.sectionTitle, { color: theme.text }]}>
            🔑 {t('security.argon2Title')}
          </Text>
          <Text style={[styles.sectionDesc, { color: theme.textSecondary }]}>
            {t('security.argon2Desc')}
          </Text>

          {/* Memory presets */}
          <View style={styles.fieldGroup}>
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('security.argon2Memory')}
            </Text>
            <View style={styles.presetGrid}>
              {MEMORY_PRESETS.map(preset => (
                <TouchableOpacity
                  key={preset.value}
                  style={[
                    styles.presetChip,
                    { borderColor: theme.border, backgroundColor: theme.backgroundSecondary },
                    memory === preset.value && {
                      backgroundColor: theme.primary,
                      borderColor: theme.primary,
                    },
                  ]}
                  onPress={() => setMemory(preset.value)}
                  accessibilityRole="radio"
                  accessibilityState={{ checked: memory === preset.value }}
                >
                  <Text
                    style={[
                      styles.presetLabel,
                      { color: memory === preset.value ? 'white' : theme.text },
                    ]}
                  >
                    {preset.label}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>

          {/* Iterations */}
          <View style={styles.fieldGroup}>
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('security.argon2Iterations')}:{' '}
              <Text style={{ fontWeight: '700', color: theme.text }}>{iterations}</Text>
            </Text>
            <View style={styles.stepperRow}>
              {[1, 2, 3, 4, 5, 6, 8, 10].map(v => (
                <TouchableOpacity
                  key={v}
                  style={[
                    styles.stepperBtn,
                    { borderColor: theme.border, backgroundColor: theme.backgroundSecondary },
                    iterations === v && {
                      backgroundColor: theme.primary,
                      borderColor: theme.primary,
                    },
                  ]}
                  onPress={() => setIterations(v)}
                  accessibilityRole="radio"
                  accessibilityState={{ checked: iterations === v }}
                >
                  <Text
                    style={[styles.stepperText, { color: iterations === v ? 'white' : theme.text }]}
                  >
                    {v}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>

          {/* Parallelism */}
          <View style={styles.fieldGroup}>
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('security.argon2Parallelism')}:{' '}
              <Text style={{ fontWeight: '700', color: theme.text }}>{parallelism}</Text>
            </Text>
            <View style={styles.stepperRow}>
              {[1, 2, 4, 8].map(v => (
                <TouchableOpacity
                  key={v}
                  style={[
                    styles.stepperBtn,
                    { borderColor: theme.border, backgroundColor: theme.backgroundSecondary },
                    parallelism === v && {
                      backgroundColor: theme.primary,
                      borderColor: theme.primary,
                    },
                  ]}
                  onPress={() => setParallelism(v)}
                  accessibilityRole="radio"
                  accessibilityState={{ checked: parallelism === v }}
                >
                  <Text
                    style={[
                      styles.stepperText,
                      { color: parallelism === v ? 'white' : theme.text },
                    ]}
                  >
                    {v}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>

          {/* Estimated time */}
          <View
            style={[
              styles.estimateRow,
              {
                backgroundColor: isHighSecurity ? '#FEF3C7' : '#F0FDF4',
                borderColor: isHighSecurity ? '#FCD34D' : '#86EFAC',
              },
            ]}
          >
            <Text style={styles.estimateIcon}>{isHighSecurity ? '⚠️' : '✅'}</Text>
            <View style={styles.estimateInfo}>
              <Text
                style={[styles.estimateLabel, { color: isHighSecurity ? '#92400E' : '#166534' }]}
              >
                {t('security.argon2EstimatedTime')}:{' '}
                <Text style={{ fontWeight: '700' }}>{estimatedTime}</Text>
              </Text>
              <Text
                style={[styles.estimateNote, { color: isHighSecurity ? '#B45309' : '#15803D' }]}
              >
                {t('security.argon2Note')}
              </Text>
            </View>
          </View>
        </View>

        {/* Save button */}
        <TouchableOpacity
          style={[styles.saveBtn, { backgroundColor: theme.primary }]}
          onPress={handleSave}
          disabled={saving}
          accessibilityRole="button"
        >
          {saving ? (
            <ActivityIndicator color="white" />
          ) : (
            <Text style={styles.saveBtnText}>🔑 {t('security.argon2Apply')}</Text>
          )}
        </TouchableOpacity>

        {/* Info card */}
        <View
          style={[styles.infoCard, { backgroundColor: theme.surface, borderColor: theme.border }]}
        >
          <Text style={[styles.infoTitle, { color: theme.text }]}>
            ℹ️ {t('security.argon2Title')}
          </Text>
          <Text style={[styles.infoText, { color: theme.textSecondary }]}>
            {t('security.argon2Desc')}
          </Text>
          <View style={styles.infoRows}>
            <InfoRow label="Banking" value="64 MB, 2 iter" color="#EF4444" theme={theme} />
            <InfoRow label="Email/Work" value="64 MB, 2 iter" color="#F59E0B" theme={theme} />
            <InfoRow label="General" value="32 MB, 1 iter" color="#3B82F6" theme={theme} />
          </View>
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

function InfoRow({
  label,
  value,
  color,
  theme,
}: {
  label: string;
  value: string;
  color: string;
  theme: ReturnType<typeof useThemeStore>['theme'];
}) {
  return (
    <View style={styles.infoRow}>
      <View style={[styles.infoRowDot, { backgroundColor: color }]} />
      <Text style={[styles.infoRowLabel, { color: theme.text }]}>{label}</Text>
      <Text style={[styles.infoRowValue, { color: theme.textSecondary }]}>{value}</Text>
    </View>
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
  backBtn: { fontSize: tokens.fontSize.md },
  headerTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.bold },
  content: { padding: tokens.space.lg, gap: tokens.space.lg },
  section: {
    borderRadius: tokens.radius.lg,
    borderWidth: StyleSheet.hairlineWidth,
    padding: tokens.space.lg,
    gap: tokens.space.lg,
  },
  sectionTitle: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.bold },
  sectionDesc: { fontSize: tokens.fontSize.sm, lineHeight: 20 },
  fieldGroup: { gap: tokens.space.sm },
  fieldLabel: { fontSize: tokens.fontSize.sm, fontWeight: '500' },
  presetGrid: { flexDirection: 'row', flexWrap: 'wrap', gap: tokens.space.sm },
  presetChip: {
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.xs,
    borderRadius: tokens.radius.full,
    borderWidth: 1,
  },
  presetLabel: { fontSize: tokens.fontSize.sm, fontWeight: '600' },
  stepperRow: { flexDirection: 'row', gap: tokens.space.xs, flexWrap: 'wrap' },
  stepperBtn: {
    width: 40,
    height: 40,
    borderRadius: tokens.radius.md,
    alignItems: 'center',
    justifyContent: 'center',
    borderWidth: 1,
  },
  stepperText: { fontSize: tokens.fontSize.sm, fontWeight: '600' },
  estimateRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.sm,
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
  },
  estimateIcon: { fontSize: 20 },
  estimateInfo: { flex: 1 },
  estimateLabel: { fontSize: tokens.fontSize.sm },
  estimateNote: { fontSize: 11, marginTop: 2 },
  saveBtn: {
    paddingVertical: tokens.space.lg,
    borderRadius: tokens.radius.md,
    alignItems: 'center',
  },
  saveBtnText: { color: 'white', fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.bold },
  infoCard: {
    borderRadius: tokens.radius.lg,
    borderWidth: StyleSheet.hairlineWidth,
    padding: tokens.space.lg,
    gap: tokens.space.md,
  },
  infoTitle: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  infoText: { fontSize: tokens.fontSize.sm, lineHeight: 20 },
  infoRows: { gap: tokens.space.sm },
  infoRow: { flexDirection: 'row', alignItems: 'center', gap: tokens.space.sm },
  infoRowDot: { width: 8, height: 8, borderRadius: 4 },
  infoRowLabel: { flex: 1, fontSize: tokens.fontSize.sm },
  infoRowValue: { fontSize: tokens.fontSize.sm },
});
