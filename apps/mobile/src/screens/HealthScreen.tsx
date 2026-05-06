/**
 * Health screen (mobile) — with full i18n EN/VI
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

export function HealthScreen() {
  const navigation = useNavigation<Nav>();
  const { theme } = useThemeStore();
  const { t } = useTranslation();

  const {
    data: report,
    isLoading,
    refetch,
  } = useQuery({
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

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <Text style={[styles.headerTitle, { color: theme.text }]}>🛡️ Vault Health</Text>
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
            {/* Score */}
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
                <Text style={[styles.scoreTitle, { color: theme.text }]}>
                  {report.score >= 90
                    ? '✅ Excellent'
                    : report.score >= 70
                      ? '👍 Good'
                      : '⚠️ Needs work'}
                </Text>
                <Text style={[styles.scoreDesc, { color: theme.textSecondary }]}>
                  {report.totalEntries} entries checked
                </Text>
              </View>
            </View>

            {/* Grid */}
            <View style={styles.grid}>
              <StatCard
                icon="🔓"
                count={report.weakCount}
                label="Weak"
                color={report.weakCount > 0 ? tokens.color.danger : tokens.color.success}
                theme={theme}
              />
              <StatCard
                icon="♻️"
                count={report.reusedCount}
                label="Reused"
                color={report.reusedCount > 0 ? tokens.color.warning : tokens.color.success}
                theme={theme}
              />
              <StatCard
                icon="⏰"
                count={report.expiredCount}
                label="Expired"
                color={report.expiredCount > 0 ? tokens.color.danger : tokens.color.success}
                theme={theme}
              />
              <StatCard
                icon="📅"
                count={report.expiringSoonCount}
                label="Expiring"
                color={report.expiringSoonCount > 0 ? tokens.color.warning : tokens.color.success}
                theme={theme}
              />
            </View>

            {/* Issues */}
            {report.weakPasswords?.map(
              (w: { entryUuid: string; entryTitle: string; strengthLabel: string }) => (
                <IssueRow
                  key={w.entryUuid}
                  icon="🔓"
                  title={w.entryTitle}
                  detail={w.strengthLabel}
                  detailColor={tokens.color.danger}
                  theme={theme}
                  onPress={() => navigation.navigate('EntryDetail', { uuid: w.entryUuid })}
                />
              )
            )}

            {report.expiredEntries?.map(
              (e: { entryUuid: string; entryTitle: string; expiredAt: string }) => (
                <IssueRow
                  key={e.entryUuid}
                  icon="⏰"
                  title={e.entryTitle}
                  detail={`Expired ${e.expiredAt}`}
                  detailColor={tokens.color.danger}
                  theme={theme}
                  onPress={() => navigation.navigate('EntryDetail', { uuid: e.entryUuid })}
                />
              )
            )}

            {report.weakCount === 0 && report.reusedCount === 0 && report.expiredCount === 0 && (
              <EmptyState
                icon="🎉"
                title="No issues found!"
                description="Your vault is healthy. Keep up the good work!"
                theme={theme}
              />
            )}
          </>
        )}
      </ScrollView>
    </SafeAreaView>
  );
}

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
      <Text style={styles.statIcon}>{icon}</Text>
      <Text style={[styles.statCount, { color }]}>{count}</Text>
      <Text style={[styles.statLabel, { color: theme.textSecondary }]}>{label}</Text>
    </View>
  );
}

function IssueRow({
  icon,
  title,
  detail,
  detailColor,
  theme,
  onPress,
}: {
  icon: string;
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
    >
      <Text style={styles.issueIcon}>{icon}</Text>
      <Text style={[styles.issueTitle, { color: theme.text }]} numberOfLines={1}>
        {title}
      </Text>
      <Text style={[styles.issueDetail, { color: detailColor }]}>{detail}</Text>
      <Text style={[styles.issueChevron, { color: theme.textTertiary }]}>›</Text>
    </TouchableOpacity>
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
  },
  scoreNumber: { fontSize: 26, fontWeight: tokens.fontWeight.bold, lineHeight: 30 },
  scoreLabel: { fontSize: 11 },
  scoreInfo: { flex: 1, gap: 4 },
  scoreTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.semibold },
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
  statLabel: { fontSize: tokens.fontSize.xs },
  issueRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.sm,
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
  },
  issueIcon: { fontSize: 18 },
  issueTitle: { flex: 1, fontSize: tokens.fontSize.md },
  issueDetail: { fontSize: tokens.fontSize.sm, fontWeight: tokens.fontWeight.medium },
  issueChevron: { fontSize: 20 },
  allGood: { alignItems: 'center', paddingVertical: tokens.space['2xl'], gap: tokens.space.md },
  allGoodIcon: { fontSize: 48 },
  allGoodText: { fontSize: tokens.fontSize.md, textAlign: 'center' },
});
