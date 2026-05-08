/**
 * Health screen (mobile) — full i18n (10 languages)
 */
import React from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, RefreshControl } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { useQuery } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import { EmptyState } from '../components/EmptyState';
import type { RootStackParamList } from '../App';

const { KeePassExCore } = NativeModules;
type Nav = NativeStackNavigationProp<RootStackParamList>;

interface WeakEntry {
  entryUuid: string;
  entryTitle: string;
  strengthLabel: string;
}
interface ExpiredEntry {
  entryUuid: string;
  entryTitle: string;
  expiredAt: string;
}
interface ExpiringEntry {
  entryUuid: string;
  entryTitle: string;
  daysRemaining: number;
}

interface HealthReport {
  totalEntries: number;
  score: number;
  weakCount: number;
  reusedCount: number;
  expiredCount: number;
  expiringSoonCount: number;
  noPasswordCount: number;
  weakPasswords: WeakEntry[];
  expiredEntries: ExpiredEntry[];
  expiringSoon: ExpiringEntry[];
}

export function HealthScreen() {
  const navigation = useNavigation<Nav>();
  const { theme } = useThemeStore();
  const { t } = useTranslation();

  const {
    data: report,
    isLoading,
    refetch,
  } = useQuery<HealthReport>({
    queryKey: ['health'],
    queryFn: () => KeePassExCore.auditVault(),
    staleTime: 60_000,
  });

  const scoreColor = !report
    ? theme.textTertiary
    : report.score >= 90
      ? tokens.color.strengthVeryStrong
      : report.score >= 70
        ? tokens.color.strengthStrong
        : report.score >= 50
          ? tokens.color.strengthFair
          : tokens.color.strengthVeryWeak;

  const scoreTitle = !report
    ? ''
    : report.score >= 90
      ? `✅ ${t('health.score')}: ${report.score}/100`
      : report.score >= 70
        ? `👍 ${t('health.score')}: ${report.score}/100`
        : report.score >= 50
          ? `⚠️ ${t('health.score')}: ${report.score}/100`
          : `🚨 ${t('health.score')}: ${report.score}/100`;

  const allGood =
    report &&
    report.weakCount === 0 &&
    report.reusedCount === 0 &&
    report.expiredCount === 0 &&
    report.expiringSoonCount === 0;

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <Text style={[styles.headerTitle, { color: theme.text }]}>🛡️ {t('health.title')}</Text>
        <TouchableOpacity
          onPress={() => refetch()}
          accessibilityRole="button"
          accessibilityLabel={t('health.runCheck')}
        >
          <Text style={[styles.refreshBtn, { color: theme.primary }]}>🔄</Text>
        </TouchableOpacity>
      </View>

      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.contentInner}
        refreshControl={
          <RefreshControl refreshing={isLoading} onRefresh={refetch} tintColor={theme.primary} />
        }
      >
        {report && (
          <>
            {/* Score card */}
            <View
              style={[
                styles.scoreCard,
                { backgroundColor: theme.surface, borderColor: theme.border },
              ]}
            >
              <View style={[styles.scoreCircle, { borderColor: scoreColor }]}>
                <Text style={[styles.scoreNumber, { color: scoreColor }]}>{report.score}</Text>
                <Text style={[styles.scoreLabel, { color: theme.textSecondary }]}>/100</Text>
              </View>
              <View style={styles.scoreInfo}>
                <Text style={[styles.scoreTitle, { color: theme.text }]}>{scoreTitle}</Text>
                <Text style={[styles.scoreDesc, { color: theme.textSecondary }]}>
                  {t('vault.statistics_entries', { count: report.totalEntries })}
                </Text>
              </View>
            </View>

            {/* Stats grid */}
            <View style={styles.grid}>
              <StatCard
                icon="🔓"
                count={report.weakCount}
                label={t('health.weakPasswords')}
                color={report.weakCount > 0 ? tokens.color.danger : tokens.color.success}
                theme={theme}
              />
              <StatCard
                icon="♻️"
                count={report.reusedCount}
                label={t('health.reusedPasswords')}
                color={report.reusedCount > 0 ? tokens.color.warning : tokens.color.success}
                theme={theme}
              />
              <StatCard
                icon="⏰"
                count={report.expiredCount}
                label={t('health.expiredEntries')}
                color={report.expiredCount > 0 ? tokens.color.danger : tokens.color.success}
                theme={theme}
              />
              <StatCard
                icon="📅"
                count={report.expiringSoonCount}
                label={t('health.expiringSoon')}
                color={report.expiringSoonCount > 0 ? tokens.color.warning : tokens.color.success}
                theme={theme}
              />
            </View>

            {/* Weak passwords */}
            {report.weakPasswords?.length > 0 && (
              <View style={styles.issueSection}>
                <Text style={[styles.issueSectionTitle, { color: theme.textSecondary }]}>
                  🔓 {t('health.weakPasswords')}
                </Text>
                {report.weakPasswords.map(w => (
                  <IssueRow
                    key={w.entryUuid}
                    title={w.entryTitle}
                    detail={w.strengthLabel}
                    detailColor={tokens.color.danger}
                    theme={theme}
                    onPress={() => navigation.navigate('EntryDetail', { uuid: w.entryUuid })}
                  />
                ))}
              </View>
            )}

            {/* Expired entries */}
            {report.expiredEntries?.length > 0 && (
              <View style={styles.issueSection}>
                <Text style={[styles.issueSectionTitle, { color: theme.textSecondary }]}>
                  ⏰ {t('health.expiredEntries')}
                </Text>
                {report.expiredEntries.map(e => (
                  <IssueRow
                    key={e.entryUuid}
                    title={e.entryTitle}
                    detail={t('entry.expired')}
                    detailColor={tokens.color.danger}
                    theme={theme}
                    onPress={() => navigation.navigate('EntryDetail', { uuid: e.entryUuid })}
                  />
                ))}
              </View>
            )}

            {/* Expiring soon */}
            {report.expiringSoon?.length > 0 && (
              <View style={styles.issueSection}>
                <Text style={[styles.issueSectionTitle, { color: theme.textSecondary }]}>
                  📅 {t('health.expiringSoon')}
                </Text>
                {report.expiringSoon.map(e => (
                  <IssueRow
                    key={e.entryUuid}
                    title={e.entryTitle}
                    detail={t('entry.expiresIn', { days: e.daysRemaining })}
                    detailColor={tokens.color.warning}
                    theme={theme}
                    onPress={() => navigation.navigate('EntryDetail', { uuid: e.entryUuid })}
                  />
                ))}
              </View>
            )}

            {/* All good */}
            {allGood && (
              <EmptyState
                icon="🎉"
                title={t('health.noIssues').split('!')[0]}
                description={t('health.noIssues')}
                theme={theme}
              />
            )}
          </>
        )}
      </ScrollView>
    </SafeAreaView>
  );
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function StatCard({
  icon,
  count,
  label,
  color,
  theme,
}: {
  icon: string;
  count: number;
  label: string;
  color: string;
  theme: ReturnType<typeof useThemeStore>['theme'];
}) {
  return (
    <View style={[styles.statCard, { backgroundColor: theme.surface, borderColor: theme.border }]}>
      <Text style={styles.statIcon} accessibilityHidden>
        {icon}
      </Text>
      <Text style={[styles.statCount, { color }]}>{count}</Text>
      <Text style={[styles.statLabel, { color: theme.textSecondary }]} numberOfLines={2}>
        {label}
      </Text>
    </View>
  );
}

function IssueRow({
  title,
  detail,
  detailColor,
  theme,
  onPress,
}: {
  title: string;
  detail: string;
  detailColor: string;
  theme: ReturnType<typeof useThemeStore>['theme'];
  onPress: () => void;
}) {
  return (
    <TouchableOpacity
      style={[styles.issueRow, { backgroundColor: theme.surface, borderColor: theme.border }]}
      onPress={onPress}
      accessibilityRole="button"
      accessibilityLabel={`${title}: ${detail}`}
    >
      <Text style={[styles.issueTitle, { color: theme.text }]} numberOfLines={1}>
        {title}
      </Text>
      <Text style={[styles.issueDetail, { color: detailColor }]}>{detail}</Text>
      <Text style={[styles.issueChevron, { color: theme.textTertiary }]}>›</Text>
    </TouchableOpacity>
  );
}

// ─── Styles ───────────────────────────────────────────────────────────────────

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
  refreshBtn: { fontSize: 20, padding: tokens.space.xs },
  content: { flex: 1 },
  contentInner: { padding: tokens.space.lg, gap: tokens.space.md },
  scoreCard: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.lg,
    padding: tokens.space.lg,
    borderRadius: tokens.radius.lg,
    borderWidth: 1,
  },
  scoreCircle: {
    width: 72,
    height: 72,
    borderRadius: 36,
    borderWidth: 4,
    alignItems: 'center',
    justifyContent: 'center',
    flexShrink: 0,
  },
  scoreNumber: { fontSize: 26, fontWeight: tokens.fontWeight.bold, lineHeight: 30 },
  scoreLabel: { fontSize: 11 },
  scoreInfo: { flex: 1, gap: 4 },
  scoreTitle: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  scoreDesc: { fontSize: tokens.fontSize.sm },
  grid: { flexDirection: 'row', flexWrap: 'wrap', gap: tokens.space.sm },
  statCard: {
    flex: 1,
    minWidth: '45%',
    alignItems: 'center',
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
    gap: 4,
  },
  statIcon: { fontSize: 24 },
  statCount: { fontSize: tokens.fontSize['2xl'], fontWeight: tokens.fontWeight.bold },
  statLabel: { fontSize: tokens.fontSize.xs, textAlign: 'center' },
  issueSection: { gap: tokens.space.sm },
  issueSectionTitle: {
    fontSize: 11,
    fontWeight: '700',
    textTransform: 'uppercase',
    letterSpacing: 0.8,
    paddingHorizontal: tokens.space.xs,
  },
  issueRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.sm,
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
  },
  issueTitle: { flex: 1, fontSize: tokens.fontSize.md },
  issueDetail: { fontSize: tokens.fontSize.sm, fontWeight: tokens.fontWeight.medium },
  issueChevron: { fontSize: 20 },
});
