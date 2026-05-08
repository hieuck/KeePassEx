/**
 * AnalyticsScreen — Vault Analytics Dashboard (Mobile)
 * Exclusive KeePassEx feature — no competitor has this on mobile
 */
import React, { useCallback, useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  ActivityIndicator,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useQuery } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';

const { KeePassExCore } = NativeModules;

interface StrengthDist {
  very_weak: number;
  weak: number;
  fair: number;
  strong: number;
  very_strong: number;
  no_password: number;
}
interface SecuritySummary {
  health_score: number;
  weak_count: number;
  reused_count: number;
  expired_count: number;
  expiring_soon_count: number;
  breached_count: number;
  no_password_count: number;
}
interface FeatureUsage {
  with_otp: number;
  with_passkey: number;
  with_ssh_key: number;
  with_attachment: number;
  favorites: number;
}
interface PasswordAge {
  average_days: number;
  oldest_days: number;
  older_than_1_year: number;
  changed_last_30_days: number;
}
interface Analytics {
  total_entries: number;
  strength_distribution: StrengthDist;
  security_summary: SecuritySummary;
  feature_usage: FeatureUsage;
  password_age: PasswordAge;
  generated_at: string;
}

export function AnalyticsScreen() {
  const { theme } = useThemeStore();
  const { t } = useTranslation();

  const { data, isLoading, refetch } = useQuery<Analytics>({
    queryKey: ['analytics'],
    queryFn: () => KeePassExCore.getVaultAnalytics(),
    staleTime: 5 * 60_000,
  });

  const healthColor = (score: number) =>
    score >= 80 ? '#16a34a' : score >= 60 ? '#d97706' : '#dc2626';

  if (isLoading || !data) {
    return (
      <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
        <ActivityIndicator size="large" color={theme.primary} style={styles.loader} />
      </SafeAreaView>
    );
  }

  const { strength_distribution: sd, security_summary: ss, feature_usage: fu } = data;
  const totalWithPwd = sd.very_weak + sd.weak + sd.fair + sd.strong + sd.very_strong;
  const pctStrong =
    totalWithPwd > 0 ? Math.round(((sd.strong + sd.very_strong) / totalWithPwd) * 100) : 0;

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <ScrollView
        refreshControl={
          <RefreshControl refreshing={isLoading} onRefresh={refetch} tintColor={theme.primary} />
        }
        contentContainerStyle={styles.content}
      >
        {/* Header */}
        <View style={styles.header}>
          <Text style={[styles.title, { color: theme.text }]}>📊 {t('analytics.title')}</Text>
          <Text style={[styles.subtitle, { color: theme.textSecondary }]}>
            {t('analytics.subtitle')}
          </Text>
          <View style={[styles.exclusiveBadge, { backgroundColor: theme.primary + '20' }]}>
            <Text style={[styles.exclusiveText, { color: theme.primary }]}>
              ✨ {t('analytics.uniqueFeature')}
            </Text>
          </View>
        </View>

        {/* Health Score */}
        <View style={[styles.card, { backgroundColor: theme.surface }]}>
          <View style={styles.healthRow}>
            <View style={[styles.healthCircle, { borderColor: healthColor(ss.health_score) }]}>
              <Text style={[styles.healthScore, { color: healthColor(ss.health_score) }]}>
                {ss.health_score}
              </Text>
              <Text style={[styles.healthMax, { color: theme.textTertiary }]}>/100</Text>
            </View>
            <View style={styles.healthInfo}>
              <Text style={[styles.cardTitle, { color: theme.text }]}>{t('health.score')}</Text>
              <Text style={[styles.cardSubtitle, { color: theme.textSecondary }]}>
                {data.total_entries} {t('statistics.totalEntries').toLowerCase()}
              </Text>
              <Text style={[styles.cardSubtitle, { color: theme.textSecondary }]}>
                {pctStrong}% {t('analytics.strengthStrong').toLowerCase()}
              </Text>
            </View>
          </View>
        </View>

        {/* Security Issues */}
        <Text style={[styles.sectionTitle, { color: theme.text }]}>Security Issues</Text>
        <View style={styles.issueGrid}>
          <IssueCard
            icon="💪"
            label={t('health.weakPasswords')}
            count={ss.weak_count}
            color={ss.weak_count === 0 ? '#16a34a' : '#dc2626'}
            theme={theme}
          />
          <IssueCard
            icon="🔄"
            label={t('health.reusedPasswords')}
            count={ss.reused_count}
            color={ss.reused_count === 0 ? '#16a34a' : '#f97316'}
            theme={theme}
          />
          <IssueCard
            icon="⏰"
            label={t('health.expiredEntries')}
            count={ss.expired_count}
            color={ss.expired_count === 0 ? '#16a34a' : '#dc2626'}
            theme={theme}
          />
          <IssueCard
            icon="🚨"
            label={t('health.breachCheck')}
            count={ss.breached_count}
            color={ss.breached_count === 0 ? '#16a34a' : '#dc2626'}
            theme={theme}
          />
          <IssueCard
            icon="🔑"
            label={t('health.noPassword')}
            count={ss.no_password_count}
            color={ss.no_password_count === 0 ? '#16a34a' : '#f97316'}
            theme={theme}
          />
          <IssueCard
            icon="⚠️"
            label={t('health.expiringSoon')}
            count={ss.expiring_soon_count}
            color={ss.expiring_soon_count === 0 ? '#16a34a' : '#d97706'}
            theme={theme}
          />
        </View>

        {/* Strength Distribution */}
        <Text style={[styles.sectionTitle, { color: theme.text }]}>
          💪 {t('analytics.passwordStrength')}
        </Text>
        <View style={[styles.card, { backgroundColor: theme.surface }]}>
          {[
            { label: t('analytics.strengthVeryWeak'), count: sd.very_weak, color: '#ef4444' },
            { label: t('analytics.strengthWeak'), count: sd.weak, color: '#f97316' },
            { label: t('analytics.strengthFair'), count: sd.fair, color: '#eab308' },
            { label: t('analytics.strengthStrong'), count: sd.strong, color: '#22c55e' },
            { label: t('analytics.strengthVeryStrong'), count: sd.very_strong, color: '#16a34a' },
            { label: t('health.noPassword'), count: sd.no_password, color: '#6b7280' },
          ].map(({ label, count, color }) => {
            const pct = data.total_entries > 0 ? Math.round((count / data.total_entries) * 100) : 0;
            return (
              <View key={label} style={styles.strengthRow}>
                <Text style={[styles.strengthLabel, { color: theme.textSecondary }]}>{label}</Text>
                <View
                  style={[styles.strengthTrack, { backgroundColor: theme.backgroundSecondary }]}
                >
                  <View
                    style={[styles.strengthFill, { width: `${pct}%`, backgroundColor: color }]}
                  />
                </View>
                <Text style={[styles.strengthCount, { color: theme.textTertiary }]}>
                  {count} ({pct}%)
                </Text>
              </View>
            );
          })}
        </View>

        {/* Feature Usage */}
        <Text style={[styles.sectionTitle, { color: theme.text }]}>⚙️ Feature Usage</Text>
        <View style={styles.featureGrid}>
          <FeatureCard
            icon="🔐"
            label={t('otp.title')}
            count={fu.with_otp}
            total={data.total_entries}
            theme={theme}
          />
          <FeatureCard
            icon="🗝️"
            label={t('passkey.title')}
            count={fu.with_passkey}
            total={data.total_entries}
            theme={theme}
          />
          <FeatureCard
            icon="🖥️"
            label={t('ssh.title')}
            count={fu.with_ssh_key}
            total={data.total_entries}
            theme={theme}
          />
          <FeatureCard
            icon="📎"
            label={t('entry.attachments')}
            count={fu.with_attachment}
            total={data.total_entries}
            theme={theme}
          />
          <FeatureCard
            icon="⭐"
            label={t('vaultFilter.favorites')}
            count={fu.favorites}
            total={data.total_entries}
            theme={theme}
          />
        </View>

        {/* Password Age */}
        <Text style={[styles.sectionTitle, { color: theme.text }]}>🕐 Password Age</Text>
        <View style={[styles.card, { backgroundColor: theme.surface }]}>
          <View style={styles.ageRow}>
            <AgeItem
              label={t('analytics.averageAge', { days: '' }).split('{{')[0].trim() || 'Average'}
              value={`${Math.round(data.password_age.average_days)}d`}
              theme={theme}
            />
            <AgeItem
              label={`> 1 ${t('common.unknown').toLowerCase()}`}
              value={String(data.password_age.older_than_1_year)}
              theme={theme}
            />
            <AgeItem
              label={
                t('analytics.entriesModified', { count: '' }).split('{{')[0].trim() || 'Changed 30d'
              }
              value={String(data.password_age.changed_last_30_days)}
              theme={theme}
            />
          </View>
        </View>

        <View style={styles.footer}>
          <Text style={[styles.footerText, { color: theme.textTertiary }]}>
            Generated {new Date(data.generated_at).toLocaleString()}
          </Text>
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

function IssueCard({
  icon,
  label,
  count,
  color,
  theme,
}: {
  icon: string;
  label: string;
  count: number;
  color: string;
  theme: ReturnType<typeof useThemeStore>['theme'];
}) {
  return (
    <View style={[styles.issueCard, { backgroundColor: theme.surface }]}>
      <Text style={styles.issueIcon}>{icon}</Text>
      <Text style={[styles.issueCount, { color }]}>{count}</Text>
      <Text style={[styles.issueLabel, { color: theme.textSecondary }]} numberOfLines={2}>
        {label}
      </Text>
    </View>
  );
}

function FeatureCard({
  icon,
  label,
  count,
  total,
  theme,
}: {
  icon: string;
  label: string;
  count: number;
  total: number;
  theme: ReturnType<typeof useThemeStore>['theme'];
}) {
  const pct = total > 0 ? Math.round((count / total) * 100) : 0;
  return (
    <View style={[styles.featureCard, { backgroundColor: theme.surface }]}>
      <Text style={styles.featureIcon}>{icon}</Text>
      <Text style={[styles.featureCount, { color: theme.text }]}>{count}</Text>
      <Text style={[styles.featureLabel, { color: theme.textSecondary }]}>{label}</Text>
      <Text style={[styles.featurePct, { color: theme.textTertiary }]}>{pct}%</Text>
    </View>
  );
}

function AgeItem({
  label,
  value,
  theme,
}: {
  label: string;
  value: string;
  theme: ReturnType<typeof useThemeStore>['theme'];
}) {
  return (
    <View style={styles.ageItem}>
      <Text style={[styles.ageValue, { color: theme.text }]}>{value}</Text>
      <Text style={[styles.ageLabel, { color: theme.textSecondary }]}>{label}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  loader: { flex: 1, justifyContent: 'center' },
  content: { padding: tokens.space.lg, gap: tokens.space.md },
  header: { gap: tokens.space.xs },
  title: { fontSize: tokens.fontSize['2xl'], fontWeight: tokens.fontWeight.bold },
  subtitle: { fontSize: tokens.fontSize.sm },
  exclusiveBadge: {
    alignSelf: 'flex-start',
    paddingHorizontal: tokens.space.sm,
    paddingVertical: 2,
    borderRadius: tokens.radius.full,
    marginTop: tokens.space.xs,
  },
  exclusiveText: { fontSize: tokens.fontSize.xs, fontWeight: tokens.fontWeight.semibold },
  sectionTitle: {
    fontSize: tokens.fontSize.lg,
    fontWeight: tokens.fontWeight.semibold,
    marginTop: tokens.space.sm,
  },
  card: { borderRadius: tokens.radius.lg, padding: tokens.space.lg },
  cardTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.bold },
  cardSubtitle: { fontSize: tokens.fontSize.sm, marginTop: 2 },
  healthRow: { flexDirection: 'row', alignItems: 'center', gap: tokens.space.lg },
  healthCircle: {
    width: 80,
    height: 80,
    borderRadius: 40,
    borderWidth: 4,
    alignItems: 'center',
    justifyContent: 'center',
  },
  healthScore: { fontSize: 28, fontWeight: tokens.fontWeight.bold },
  healthMax: { fontSize: 11 },
  healthInfo: { flex: 1, gap: 4 },
  issueGrid: { flexDirection: 'row', flexWrap: 'wrap', gap: tokens.space.sm },
  issueCard: {
    width: '30%',
    borderRadius: tokens.radius.md,
    padding: tokens.space.md,
    alignItems: 'center',
    gap: 4,
  },
  issueIcon: { fontSize: 20 },
  issueCount: { fontSize: tokens.fontSize.xl, fontWeight: tokens.fontWeight.bold },
  issueLabel: { fontSize: 10, textAlign: 'center' },
  strengthRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.sm,
    marginBottom: tokens.space.sm,
  },
  strengthLabel: { width: 80, fontSize: tokens.fontSize.xs },
  strengthTrack: { flex: 1, height: 8, borderRadius: 4, overflow: 'hidden' },
  strengthFill: { height: '100%', borderRadius: 4 },
  strengthCount: { width: 60, fontSize: tokens.fontSize.xs, textAlign: 'right' },
  featureGrid: { flexDirection: 'row', flexWrap: 'wrap', gap: tokens.space.sm },
  featureCard: {
    width: '30%',
    borderRadius: tokens.radius.md,
    padding: tokens.space.md,
    alignItems: 'center',
    gap: 2,
  },
  featureIcon: { fontSize: 22 },
  featureCount: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.bold },
  featureLabel: { fontSize: 10, textAlign: 'center' },
  featurePct: { fontSize: 10 },
  ageRow: { flexDirection: 'row', justifyContent: 'space-around' },
  ageItem: { alignItems: 'center', gap: 4 },
  ageValue: { fontSize: tokens.fontSize.xl, fontWeight: tokens.fontWeight.bold },
  ageLabel: { fontSize: tokens.fontSize.xs },
  footer: { alignItems: 'center', paddingVertical: tokens.space.md },
  footerText: { fontSize: tokens.fontSize.xs },
});
