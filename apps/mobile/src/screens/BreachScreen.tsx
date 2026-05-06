/**
 * Breach monitor screen — mobile (with full i18n EN/VI)
 */
import React, { useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  Switch,
  RefreshControl,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { useMutation } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import { EmptyState } from '../components/EmptyState';
import type { RootStackParamList } from '../App';

const { KeePassExCore } = NativeModules;
type Nav = NativeStackNavigationProp<RootStackParamList>;

interface BreachResult {
  entryUuid: string;
  entryTitle: string;
  isBreached: boolean;
  breachCount: number;
  hashPrefix: string;
}

interface VaultBreachReport {
  totalChecked: number;
  breachedCount: number;
  results: BreachResult[];
  usedOnline: boolean;
}

export function BreachScreen() {
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const navigation = useNavigation<Nav>();
  const [useOnline, setUseOnline] = useState(false);

  const checkMutation = useMutation({
    mutationFn: (online: boolean) => KeePassExCore.checkVaultBreaches({ online }),
  });

  const report = checkMutation.data as VaultBreachReport | undefined;

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <Text style={[styles.headerTitle, { color: theme.text }]}>🛡️ {t('breach.title')}</Text>
      </View>

      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.contentInner}
        refreshControl={
          <RefreshControl
            refreshing={checkMutation.isPending}
            onRefresh={() => checkMutation.mutate(useOnline)}
            tintColor={theme.primary}
          />
        }
      >
        {/* Info */}
        <View
          style={[styles.infoCard, { backgroundColor: theme.surface, borderColor: theme.border }]}
        >
          <Text style={[styles.infoTitle, { color: theme.text }]}>{t('breach.howItWorks')}</Text>
          <Text style={[styles.infoDesc, { color: theme.textSecondary }]}>
            {t('breach.howItWorksDesc')}
          </Text>

          <View style={styles.modeRow}>
            <View>
              <Text style={[styles.modeLabel, { color: theme.text }]}>
                {useOnline ? `🌐 ${t('breach.onlineMode')}` : `📴 ${t('breach.offlineMode')}`}
              </Text>
              <Text style={[styles.modeDesc, { color: theme.textSecondary }]}>
                {useOnline ? t('breach.onlineModeDesc') : t('breach.offlineModeDesc')}
              </Text>
            </View>
            <Switch
              value={useOnline}
              onValueChange={setUseOnline}
              trackColor={{ false: theme.border, true: theme.primary }}
              thumbColor="white"
              accessibilityLabel={t('breach.onlineMode')}
            />
          </View>
        </View>

        {/* Run check button */}
        <TouchableOpacity
          style={[styles.checkBtn, { backgroundColor: theme.primary }]}
          onPress={() => checkMutation.mutate(useOnline)}
          disabled={checkMutation.isPending}
          accessibilityRole="button"
          accessibilityLabel={t('breach.runCheck')}
        >
          <Text style={styles.checkBtnText}>
            {checkMutation.isPending ? `🔍 ${t('breach.checking')}` : `🔍 ${t('breach.runCheck')}`}
          </Text>
        </TouchableOpacity>

        {/* Results */}
        {report && (
          <>
            {/* Summary */}
            <View
              style={[
                styles.summaryCard,
                {
                  backgroundColor: report.breachedCount === 0 ? '#F0FDF4' : '#FEF2F2',
                  borderColor: report.breachedCount === 0 ? '#86EFAC' : '#FECACA',
                },
              ]}
            >
              <Text style={styles.summaryIcon}>{report.breachedCount === 0 ? '✅' : '⚠️'}</Text>
              <View style={styles.summaryInfo}>
                <Text
                  style={[
                    styles.summaryTitle,
                    {
                      color: report.breachedCount === 0 ? '#166534' : '#991B1B',
                    },
                  ]}
                >
                  {report.breachedCount === 0
                    ? 'No breaches found!'
                    : `${report.breachedCount} breached password${report.breachedCount !== 1 ? 's' : ''}!`}
                </Text>
                <Text
                  style={[
                    styles.summaryDesc,
                    {
                      color: report.breachedCount === 0 ? '#166534' : '#991B1B',
                    },
                  ]}
                >
                  Checked {report.totalChecked} passwords
                </Text>
              </View>
            </View>

            {/* Breached entries */}
            {report.results.length > 0 && (
              <>
                <Text style={[styles.sectionTitle, { color: theme.textSecondary }]}>
                  BREACHED PASSWORDS
                </Text>
                {report.results.map(r => (
                  <TouchableOpacity
                    key={r.entryUuid}
                    style={[
                      styles.breachItem,
                      { backgroundColor: theme.surface, borderColor: theme.border },
                    ]}
                    onPress={() => navigation.navigate('EntryDetail', { uuid: r.entryUuid })}
                    accessibilityRole="button"
                    accessibilityLabel={`${r.entryTitle} - breached`}
                  >
                    <View style={styles.breachItemInfo}>
                      <Text style={styles.breachIcon}>⚠️</Text>
                      <View>
                        <Text style={[styles.breachTitle, { color: theme.text }]}>
                          {r.entryTitle}
                        </Text>
                        {r.breachCount > 0 && (
                          <Text style={styles.breachCount}>
                            Found {r.breachCount.toLocaleString()} times in breaches
                          </Text>
                        )}
                      </View>
                    </View>
                    <Text style={[styles.breachChevron, { color: theme.textTertiary }]}>›</Text>
                  </TouchableOpacity>
                ))}
              </>
            )}

            {report.breachedCount === 0 && (
              <EmptyState
                icon="✅"
                title="No breaches found!"
                description={`Checked ${report.totalChecked} passwords — all clear.`}
                theme={theme}
              />
            )}

            <Text style={[styles.footerNote, { color: theme.textTertiary }]}>
              {report.usedOnline ? '🌐 Checked against HIBP (online)' : '📴 Checked offline'}
            </Text>
          </>
        )}
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
  contentInner: { padding: tokens.space.lg, gap: tokens.space.md },
  infoCard: {
    borderRadius: tokens.radius.lg,
    borderWidth: StyleSheet.hairlineWidth,
    padding: tokens.space.lg,
    gap: tokens.space.md,
  },
  infoTitle: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  infoDesc: { fontSize: tokens.fontSize.sm, lineHeight: 20 },
  modeRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: tokens.space.md,
  },
  modeLabel: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.medium },
  modeDesc: { fontSize: tokens.fontSize.xs, marginTop: 2 },
  checkBtn: {
    borderRadius: tokens.radius.md,
    paddingVertical: tokens.space.md,
    alignItems: 'center',
  },
  checkBtnText: {
    color: 'white',
    fontWeight: tokens.fontWeight.semibold,
    fontSize: tokens.fontSize.md,
  },
  summaryCard: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.md,
    padding: tokens.space.lg,
    borderRadius: tokens.radius.lg,
    borderWidth: 1,
  },
  summaryIcon: { fontSize: 32 },
  summaryInfo: { flex: 1 },
  summaryTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.bold },
  summaryDesc: { fontSize: tokens.fontSize.sm, marginTop: 2 },
  sectionTitle: {
    fontSize: 11,
    fontWeight: '600',
    letterSpacing: 0.8,
    paddingHorizontal: tokens.space.xs,
  },
  breachItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
    borderWidth: StyleSheet.hairlineWidth,
    gap: tokens.space.sm,
  },
  breachItemInfo: { flex: 1, flexDirection: 'row', alignItems: 'center', gap: tokens.space.sm },
  breachIcon: { fontSize: 20 },
  breachTitle: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.medium },
  breachCount: { fontSize: tokens.fontSize.xs, color: tokens.color.danger, marginTop: 2 },
  breachChevron: { fontSize: 20 },
  footerNote: { fontSize: tokens.fontSize.xs, textAlign: 'center' },
});
