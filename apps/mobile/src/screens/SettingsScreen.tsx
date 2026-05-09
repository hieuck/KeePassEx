/**
 * Settings screen (mobile) — with full i18n EN/VI, language switcher,
 * biometric unlock, clipboard settings, and all tool links
 */
import React, { useState } from 'react';
import { View, Text, ScrollView, Switch, TouchableOpacity, StyleSheet, Alert } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { useThemeStore } from '../store/theme';
import { useMobileSettingsStore as useSettingsStore } from '../store/settings';
import { useI18nStore, useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import type { RootStackParamList } from '../App';

type Nav = NativeStackNavigationProp<RootStackParamList>;

export function SettingsScreen() {
  const { theme, mode, setMode } = useThemeStore();
  const { settings, update } = useSettingsStore();
  const { locale, setLocale } = useI18nStore();
  const { t } = useTranslation();
  const navigation = useNavigation<Nav>();

  const THEME_OPTIONS = [
    { id: 'light' as const, icon: '☀️', label: t('settings.themeLight') },
    { id: 'dark' as const, icon: '🌙', label: t('settings.themeDark') },
    { id: 'oled' as const, icon: '⚫', label: t('settings.themeOled') },
    { id: 'system' as const, icon: '📱', label: t('settings.themeSystem') },
  ];

  const LANGUAGE_OPTIONS = [
    { id: 'en' as const, flag: '🇺🇸', label: 'English' },
    { id: 'vi' as const, flag: '🇻🇳', label: 'Tiếng Việt' },
    { id: 'zh' as const, flag: '🇨🇳', label: '简体中文' },
    { id: 'ja' as const, flag: '🇯🇵', label: '日本語' },
    { id: 'ko' as const, flag: '🇰🇷', label: '한국어' },
    { id: 'es' as const, flag: '🇪🇸', label: 'Español' },
    { id: 'fr' as const, flag: '🇫🇷', label: 'Français' },
    { id: 'de' as const, flag: '🇩🇪', label: 'Deutsch' },
    { id: 'pt' as const, flag: '🇧🇷', label: 'Português' },
    { id: 'ru' as const, flag: '🇷🇺', label: 'Русский' },
  ];

  const CLIPBOARD_OPTIONS = [
    { value: 10, label: '10s' },
    { value: 30, label: '30s' },
    { value: 60, label: '60s' },
    { value: 0, label: t('settings.clipboardClearNever') },
  ];

  const IDLE_OPTIONS = [
    {
      value: 1,
      label: `1 ${t('settings.lockAfterIdleMinutes').replace('{{minutes}}', '').trim()}`,
    },
    { value: 5, label: `5 min` },
    { value: 15, label: `15 min` },
    { value: 0, label: t('settings.clipboardClearNever') },
  ];

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <Text style={[styles.headerTitle, { color: theme.text }]}>⚙️ {t('settings.title')}</Text>
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.contentInner}>
        {/* ── Appearance ── */}
        <SectionHeader title={t('settings.appearance')} theme={theme} />
        <View style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}>
          <View style={styles.themeRow}>
            {THEME_OPTIONS.map(opt => (
              <TouchableOpacity
                key={opt.id}
                style={[
                  styles.themeButton,
                  { borderColor: theme.border },
                  mode === opt.id && { backgroundColor: theme.primary, borderColor: theme.primary },
                ]}
                onPress={() => setMode(opt.id)}
                accessibilityRole="radio"
                accessibilityState={{ checked: mode === opt.id }}
                accessibilityLabel={opt.label}
              >
                <Text style={styles.themeButtonIcon}>{opt.icon}</Text>
                <Text
                  style={[
                    styles.themeButtonLabel,
                    { color: mode === opt.id ? 'white' : theme.textSecondary },
                  ]}
                >
                  {opt.label}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
        </View>

        {/* ── Language ── */}
        <SectionHeader title={t('settings.language')} theme={theme} />
        <View style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}>
          <View style={styles.langRow}>
            {LANGUAGE_OPTIONS.map((opt, idx) => (
              <React.Fragment key={opt.id}>
                <TouchableOpacity
                  style={[
                    styles.langButton,
                    { borderColor: theme.border },
                    locale === opt.id && {
                      backgroundColor: theme.primary,
                      borderColor: theme.primary,
                    },
                  ]}
                  onPress={() => setLocale(opt.id)}
                  accessibilityRole="radio"
                  accessibilityState={{ checked: locale === opt.id }}
                  accessibilityLabel={opt.label}
                >
                  <Text style={styles.langFlag}>{opt.flag}</Text>
                  <Text
                    style={[styles.langLabel, { color: locale === opt.id ? 'white' : theme.text }]}
                  >
                    {opt.label}
                  </Text>
                </TouchableOpacity>
                {idx < LANGUAGE_OPTIONS.length - 1 && (
                  <View style={[styles.langDivider, { backgroundColor: theme.border }]} />
                )}
              </React.Fragment>
            ))}
          </View>
        </View>

        {/* ── Security ── */}
        <SectionHeader title={t('settings.security')} theme={theme} />
        <View style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}>
          <SettingRow
            label={t('settings.biometricUnlock')}
            description={t('settings.biometricUnlockDesc')}
            theme={theme}
          >
            <Switch
              value={settings.biometricEnabled ?? false}
              onValueChange={v => update({ biometricEnabled: v })}
              trackColor={{ false: theme.border, true: theme.primary }}
              thumbColor="white"
              accessibilityLabel={t('settings.biometricUnlock')}
            />
          </SettingRow>

          <Divider theme={theme} />

          <SettingRow
            label={t('settings.screenCaptureProtection')}
            description={t('settings.screenCaptureProtectionDesc')}
            theme={theme}
          >
            <Switch
              value={settings.screenCaptureProtection ?? true}
              onValueChange={v => update({ screenCaptureProtection: v })}
              trackColor={{ false: theme.border, true: theme.primary }}
              thumbColor="white"
              accessibilityLabel={t('settings.screenCaptureProtection')}
            />
          </SettingRow>

          <Divider theme={theme} />

          <SettingRow
            label={t('settings.lockOnScreenLock')}
            description={t('settings.lockOnScreenLock')}
            theme={theme}
          >
            <Switch
              value={settings.lockOnBackground ?? true}
              onValueChange={v => update({ lockOnBackground: v })}
              trackColor={{ false: theme.border, true: theme.primary }}
              thumbColor="white"
              accessibilityLabel={t('settings.lockOnScreenLock')}
            />
          </SettingRow>

          <Divider theme={theme} />

          {/* Clipboard clear */}
          <View style={styles.settingRow}>
            <View style={styles.settingInfo}>
              <Text style={[styles.settingLabel, { color: theme.text }]}>
                {t('settings.clipboardClearDelay')}
              </Text>
            </View>
            <View style={styles.chipRow}>
              {CLIPBOARD_OPTIONS.map(opt => (
                <TouchableOpacity
                  key={opt.value}
                  style={[
                    styles.chip,
                    { borderColor: theme.border },
                    (settings.clipboardClearSeconds ?? 10) === opt.value && {
                      backgroundColor: theme.primary,
                      borderColor: theme.primary,
                    },
                  ]}
                  onPress={() => update({ clipboardClearSeconds: opt.value })}
                  accessibilityRole="radio"
                  accessibilityState={{
                    checked: (settings.clipboardClearSeconds ?? 10) === opt.value,
                  }}
                >
                  <Text
                    style={[
                      styles.chipText,
                      {
                        color:
                          (settings.clipboardClearSeconds ?? 10) === opt.value
                            ? 'white'
                            : theme.textSecondary,
                      },
                    ]}
                  >
                    {opt.label}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>

          <Divider theme={theme} />

          {/* Auto-lock */}
          <View style={styles.settingRow}>
            <View style={styles.settingInfo}>
              <Text style={[styles.settingLabel, { color: theme.text }]}>
                {t('settings.lockAfterIdle')}
              </Text>
            </View>
            <View style={styles.chipRow}>
              {IDLE_OPTIONS.map(opt => (
                <TouchableOpacity
                  key={opt.value}
                  style={[
                    styles.chip,
                    { borderColor: theme.border },
                    (settings.lockAfterIdleMinutes ?? 5) === opt.value && {
                      backgroundColor: theme.primary,
                      borderColor: theme.primary,
                    },
                  ]}
                  onPress={() => update({ lockAfterIdleMinutes: opt.value })}
                  accessibilityRole="radio"
                  accessibilityState={{
                    checked: (settings.lockAfterIdleMinutes ?? 5) === opt.value,
                  }}
                >
                  <Text
                    style={[
                      styles.chipText,
                      {
                        color:
                          (settings.lockAfterIdleMinutes ?? 5) === opt.value
                            ? 'white'
                            : theme.textSecondary,
                      },
                    ]}
                  >
                    {opt.label}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>
        </View>

        {/* ── Tools ── */}
        <SectionHeader title={t('settings.advanced')} theme={theme} />
        <View style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}>
          {[
            { icon: '📁', key: 'group.manage', screen: 'Groups' as const },
            { icon: '🔑', key: 'vault.changePassword', screen: 'ChangePassword' as const },
            { icon: '🔐', key: 'settings.security', screen: 'SecuritySettings' as const },
            { icon: '🔄', key: 'rotation.title', screen: 'Rotation' as const },
            { icon: '🔄', key: 'sync.title', screen: 'Sync' as const },
            { icon: '📥', key: 'importExport.import', screen: 'ImportExport' as const },
            { icon: '🛡️', key: 'breach.title', screen: 'Breach' as const },
            { icon: '🆘', key: 'emergencyAccess.title', screen: 'EmergencyAccess' as const },
            { icon: '🔧', key: 'plugins.title', screen: 'Plugins' as const },
          ].map(({ icon, key, screen }, idx, arr) => (
            <React.Fragment key={screen}>
              <TouchableOpacity
                style={styles.navRow}
                onPress={() => navigation.navigate(screen)}
                accessibilityRole="button"
                accessibilityLabel={t(key)}
              >
                <Text style={[styles.navRowLabel, { color: theme.text }]}>
                  {icon} {t(key)}
                </Text>
                <Text style={[styles.navRowChevron, { color: theme.textTertiary }]}>›</Text>
              </TouchableOpacity>
              {idx < arr.length - 1 && <Divider theme={theme} />}
            </React.Fragment>
          ))}
        </View>

        {/* ── About ── */}
        <SectionHeader title={t('settings.about')} theme={theme} />
        <View style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}>
          <AboutRow label={t('settings.version')} value="1.0.0" theme={theme} />
          <Divider theme={theme} />
          <AboutRow label="License" value="GPL-3.0" theme={theme} />
          <Divider theme={theme} />
          <AboutRow label="Format" value="KDBX 4.x" theme={theme} />
          <Divider theme={theme} />
          <TouchableOpacity
            style={styles.navRow}
            onPress={() => Alert.alert('KeePassEx', 'Open source password manager')}
            accessibilityRole="button"
          >
            <Text style={[styles.navRowLabel, { color: theme.primary }]}>
              {t('settings.reportBug')}
            </Text>
          </TouchableOpacity>
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function SectionHeader({
  title,
  theme,
}: {
  title: string;
  theme: ReturnType<typeof useThemeStore>['theme'];
}) {
  return (
    <Text style={[styles.sectionHeader, { color: theme.textSecondary }]}>
      {title.toUpperCase()}
    </Text>
  );
}

function Divider({ theme }: { theme: ReturnType<typeof useThemeStore>['theme'] }) {
  return <View style={[styles.divider, { backgroundColor: theme.border }]} />;
}

function SettingRow({
  label,
  description,
  theme,
  children,
}: {
  label: string;
  description?: string;
  theme: ReturnType<typeof useThemeStore>['theme'];
  children: React.ReactNode;
}) {
  return (
    <View style={styles.settingRow}>
      <View style={styles.settingInfo}>
        <Text style={[styles.settingLabel, { color: theme.text }]}>{label}</Text>
        {description && (
          <Text style={[styles.settingDesc, { color: theme.textSecondary }]}>{description}</Text>
        )}
      </View>
      {children}
    </View>
  );
}

function AboutRow({
  label,
  value,
  theme,
}: {
  label: string;
  value: string;
  theme: ReturnType<typeof useThemeStore>['theme'];
}) {
  return (
    <View style={styles.aboutRow}>
      <Text style={[styles.settingLabel, { color: theme.text }]}>{label}</Text>
      <Text style={[styles.aboutValue, { color: theme.textSecondary }]}>{value}</Text>
    </View>
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
  sectionHeader: {
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
  settingRow: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: tokens.space.md,
    gap: tokens.space.md,
  },
  settingInfo: { flex: 1 },
  settingLabel: { fontSize: tokens.fontSize.md },
  settingDesc: { fontSize: tokens.fontSize.sm, marginTop: 2 },
  divider: { height: StyleSheet.hairlineWidth, marginLeft: tokens.space.md },
  themeRow: {
    flexDirection: 'row',
    gap: tokens.space.sm,
    padding: tokens.space.md,
  },
  themeButton: {
    flex: 1,
    alignItems: 'center',
    paddingVertical: tokens.space.sm,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
    gap: 2,
  },
  themeButtonIcon: { fontSize: 20 },
  themeButtonLabel: { fontSize: 10, fontWeight: '500' },
  langRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  langButton: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: tokens.space.sm,
    paddingVertical: tokens.space.md,
    borderRadius: 0,
    borderWidth: 0,
  },
  langFlag: { fontSize: 20 },
  langLabel: { fontSize: tokens.fontSize.md, fontWeight: '500' },
  langDivider: { width: StyleSheet.hairlineWidth, height: 40 },
  chipRow: {
    flexDirection: 'row',
    gap: 4,
    flexWrap: 'wrap',
    justifyContent: 'flex-end',
  },
  chip: {
    paddingHorizontal: tokens.space.sm,
    paddingVertical: 3,
    borderRadius: tokens.radius.full,
    borderWidth: 1,
  },
  chipText: { fontSize: 11, fontWeight: '500' },
  aboutRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: tokens.space.md,
  },
  aboutValue: { fontSize: tokens.fontSize.md },
  navRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: tokens.space.md,
  },
  navRowLabel: { fontSize: tokens.fontSize.md },
  navRowChevron: { fontSize: 20 },
});
