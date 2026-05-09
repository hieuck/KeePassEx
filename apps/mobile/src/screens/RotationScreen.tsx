/**
 * Password Rotation screen — mobile
 * KeePassEx exclusive: proactive rotation recommendations on mobile
 * No competitor (KeePassXC, Keepassium, KeePass2Android) has this
 */
import React, { useState } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  ActivityIndicator,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { useQuery } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import { useThemeStore } from '../store/theme';
import { useI18nStore } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import type { RootStackParamList } from '../App';

const { KeePassExCore } = NativeModules;
type Nav = NativeStackNavigationProp<RootStackParamList>;

interface RotationRec {
  entryUuid: string;
  entryTitle: string;
  urgency: 'aging' | 'soon' | 'overdue' | 'expired';
  urgencyColor: string;
  ageDays: number;
  recommendedMaxDays: number;
  daysUntilOverdue: number;
  messageEn: string;
  messageVi: string;
}

const URGENCY_ICONS: Record<string, string> = {
  expired: '🚨',
  overdue: '🔴',
  soon: '🟡',
  aging: '🔵',
};

const URGENCY_LABELS: Record<string, string> = {
  expired: 'rotation.expired',
  overdue: 'rotation.overdue',
  soon: 'rotation.soon',
  aging: 'rotation.aging',
};

type FilterType = 'all' | 'overdue' | 'soon' | 'aging';

export function RotationScreen() {
  const navigation = useNavigation<Nav>();
  const { theme } = useThemeStore();
  const { t, locale } = useI18nStore();
  const [filter, setFilter] = useState<FilterType>('all');

  const {
    data: recommendations = [],
    isLoading,
    refetch,
  } = useQuery<RotationRec[]>({
    queryKey: ['rotation-recommendations'],
    queryFn: () => KeePassExCore.getRotationRecommendations(),
    staleTime: 60_000,
  });

  const filtered = recommendations.filter(r => {
    if (filter === 'all') return true;
    if (filter === 'overdue') return r.urgency === 'overdue' || r.urgency === 'expired';
    return r.urgency === filter;
  });

  const counts = {
    overdue: recommendations.filter(r => r.urgency === 'overdue' || r.urgency === 'expired').length,
    soon: recommendations.filter(r => r.urgency === 'soon').length,
    aging: recommendations.filter(r => r.urgency === 'aging').length,
  };

  const getMessage = (rec: RotationRec) => (locale === 'vi' ? rec.messageVi : rec.messageEn);

  const renderItem = ({ item }: { item: RotationRec }) => (
    <TouchableOpacity
      style={[styles.item, { backgroundColor: theme.surface, borderColor: theme.border }]}
      onPress={() => navigation.navigate('EntryDetail', { uuid: item.entryUuid })}
      accessibilityRole="button"
      accessibilityLabel={`${item.entryTitle} - ${item.urgency}`}
    >
      <View style={styles.itemLeft}>
        <Text style={styles.itemIcon}>{URGENCY_ICONS[item.urgency] ?? '⚪'}</Text>
        <View style={styles.itemInfo}>
          <Text style={[styles.itemTitle, { color: theme.text }]} numberOfLines={1}>
            {item.entryTitle}
          </Text>
          <Text style={[styles.itemMessage, { color: theme.textSecondary }]} numberOfLines={2}>
            {getMessage(item)}
          </Text>
        </View>
      </View>
      <View style={styles.itemRight}>
        <View style={[styles.urgencyBadge, { backgroundColor: item.urgencyColor + '20' }]}>
          <Text style={[styles.urgencyText, { color: item.urgencyColor }]}>{item.ageDays}d</Text>
        </View>
        <Text style={[styles.chevron, { color: theme.textTertiary }]}>›</Text>
      </View>
    </TouchableOpacity>
  );

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Header */}
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <Text style={[styles.headerTitle, { color: theme.text }]}>🔄 {t('rotation.title')}</Text>
        <Text style={[styles.headerSubtitle, { color: theme.textSecondary }]}>
          {t('rotation.subtitle')}
        </Text>
      </View>

      {/* Summary cards */}
      {!isLoading && recommendations.length > 0 && (
        <View style={styles.summaryRow}>
          {[
            {
              key: 'overdue' as FilterType,
              count: counts.overdue,
              color: '#EF4444',
              label: t('rotation.overdue'),
            },
            {
              key: 'soon' as FilterType,
              count: counts.soon,
              color: '#F59E0B',
              label: t('rotation.soon'),
            },
            {
              key: 'aging' as FilterType,
              count: counts.aging,
              color: '#3B82F6',
              label: t('rotation.aging'),
            },
          ].map(card => (
            <TouchableOpacity
              key={card.key}
              style={[
                styles.summaryCard,
                { backgroundColor: theme.surface, borderColor: theme.border },
                filter === card.key && { borderColor: card.color, borderWidth: 2 },
              ]}
              onPress={() => setFilter(filter === card.key ? 'all' : card.key)}
              accessibilityRole="button"
              accessibilityState={{ selected: filter === card.key }}
            >
              <Text style={[styles.summaryCount, { color: card.color }]}>{card.count}</Text>
              <Text style={[styles.summaryLabel, { color: theme.textSecondary }]}>
                {card.label}
              </Text>
            </TouchableOpacity>
          ))}
        </View>
      )}

      {/* List */}
      {isLoading ? (
        <View style={styles.centered}>
          <ActivityIndicator color={theme.primary} />
          <Text style={[styles.loadingText, { color: theme.textSecondary }]}>
            {t('common.loading')}
          </Text>
        </View>
      ) : recommendations.length === 0 ? (
        <View style={styles.centered}>
          <Text style={styles.emptyIcon}>✅</Text>
          <Text style={[styles.emptyTitle, { color: theme.text }]}>{t('rotation.allGood')}</Text>
          <Text style={[styles.emptyDesc, { color: theme.textSecondary }]}>
            {t('rotation.allGoodDesc')}
          </Text>
        </View>
      ) : (
        <FlatList
          data={filtered}
          keyExtractor={item => item.entryUuid}
          renderItem={renderItem}
          refreshControl={
            <RefreshControl refreshing={isLoading} onRefresh={refetch} tintColor={theme.primary} />
          }
          contentContainerStyle={styles.list}
          ItemSeparatorComponent={() => (
            <View style={[styles.separator, { backgroundColor: theme.border }]} />
          )}
          ListEmptyComponent={
            <View style={styles.centered}>
              <Text style={[styles.emptyDesc, { color: theme.textSecondary }]}>
                {t('rotation.noFilterResults')}
              </Text>
            </View>
          }
        />
      )}
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
  headerSubtitle: { fontSize: tokens.fontSize.sm, marginTop: 2 },
  summaryRow: {
    flexDirection: 'row',
    gap: tokens.space.sm,
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.md,
  },
  summaryCard: {
    flex: 1,
    alignItems: 'center',
    paddingVertical: tokens.space.md,
    borderRadius: tokens.radius.md,
    borderWidth: StyleSheet.hairlineWidth,
  },
  summaryCount: { fontSize: tokens.fontSize['2xl'], fontWeight: tokens.fontWeight.bold },
  summaryLabel: { fontSize: 10, fontWeight: '600', marginTop: 2, textAlign: 'center' },
  centered: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    gap: tokens.space.md,
    padding: tokens.space.xl,
  },
  loadingText: { fontSize: tokens.fontSize.sm },
  emptyIcon: { fontSize: 48 },
  emptyTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.bold },
  emptyDesc: { fontSize: tokens.fontSize.sm, textAlign: 'center' },
  list: { padding: tokens.space.md },
  item: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
    borderWidth: StyleSheet.hairlineWidth,
    gap: tokens.space.sm,
  },
  itemLeft: { flex: 1, flexDirection: 'row', alignItems: 'center', gap: tokens.space.sm },
  itemIcon: { fontSize: 20, flexShrink: 0 },
  itemInfo: { flex: 1 },
  itemTitle: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  itemMessage: { fontSize: tokens.fontSize.xs, marginTop: 2, lineHeight: 16 },
  itemRight: { flexDirection: 'row', alignItems: 'center', gap: tokens.space.xs, flexShrink: 0 },
  urgencyBadge: { paddingHorizontal: 8, paddingVertical: 3, borderRadius: tokens.radius.full },
  urgencyText: { fontSize: 11, fontWeight: '700' },
  chevron: { fontSize: 20 },
  separator: { height: StyleSheet.hairlineWidth, marginHorizontal: tokens.space.md },
});
