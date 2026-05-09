/**
 * Entry History screen — view and restore previous versions of an entry
 * KeePassEx exclusive on mobile: no competitor shows entry history on mobile
 */
import React, { useState } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation, useRoute } from '@react-navigation/native';
import type { NativeStackNavigationProp, RouteProp } from '@react-navigation/native-stack';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import ReactNativeHapticFeedback from 'react-native-haptic-feedback';
import { useThemeStore } from '../store/theme';
import { useI18nStore } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import type { RootStackParamList } from '../App';

const { KeePassExCore } = NativeModules;

type Nav = NativeStackNavigationProp<RootStackParamList>;
type Route = RouteProp<RootStackParamList, 'EntryHistory'>;

interface HistoryEntry {
  uuid: string;
  modifiedAt: string;
  title: string;
  username: string;
  url: string;
  notes: string;
  hasPassword: boolean;
}

export function EntryHistoryScreen() {
  const navigation = useNavigation<Nav>();
  const route = useRoute<Route>();
  const { entryUuid, entryTitle } = route.params;
  const { theme } = useThemeStore();
  const { t } = useI18nStore();
  const queryClient = useQueryClient();
  const [expandedUuid, setExpandedUuid] = useState<string | null>(null);

  const { data: history = [], isLoading } = useQuery<HistoryEntry[]>({
    queryKey: ['entry-history', entryUuid],
    queryFn: () => KeePassExCore.getEntryHistory(entryUuid),
  });

  const restoreMutation = useMutation({
    mutationFn: (historyUuid: string) =>
      KeePassExCore.restoreEntryFromHistory(entryUuid, historyUuid),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['entry', entryUuid] });
      queryClient.invalidateQueries({ queryKey: ['entry-history', entryUuid] });
      ReactNativeHapticFeedback.trigger('notificationSuccess');
      Alert.alert('✅', t('entry.restoreFromHistory'), [
        { text: t('common.ok'), onPress: () => navigation.goBack() },
      ]);
    },
    onError: (e: any) => {
      Alert.alert(t('common.error'), e?.message ?? t('errors.generic'));
    },
  });

  const clearMutation = useMutation({
    mutationFn: () => KeePassExCore.clearEntryHistory(entryUuid),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['entry-history', entryUuid] });
      ReactNativeHapticFeedback.trigger('notificationSuccess');
    },
  });

  const handleRestore = (item: HistoryEntry) => {
    Alert.alert(
      t('entry.restoreFromHistory'),
      `${t('entry.restoreFromHistory')}: ${new Date(item.modifiedAt).toLocaleString()}?`,
      [
        { text: t('common.cancel'), style: 'cancel' },
        {
          text: t('entry.restoreFromHistory'),
          onPress: () => restoreMutation.mutate(item.uuid),
        },
      ]
    );
  };

  const handleClearAll = () => {
    Alert.alert(t('entry.clearHistory'), t('entry.clearHistory'), [
      { text: t('common.cancel'), style: 'cancel' },
      {
        text: t('common.delete'),
        style: 'destructive',
        onPress: () => clearMutation.mutate(),
      },
    ]);
  };

  const formatDate = (iso: string) => {
    try {
      return new Date(iso).toLocaleString();
    } catch {
      return iso;
    }
  };

  const renderItem = ({ item, index }: { item: HistoryEntry; index: number }) => {
    const isExpanded = expandedUuid === item.uuid;
    const isLatest = index === 0;

    return (
      <TouchableOpacity
        style={[
          styles.historyItem,
          { backgroundColor: theme.surface, borderColor: theme.border },
          isLatest && { borderLeftWidth: 3, borderLeftColor: theme.primary },
        ]}
        onPress={() => setExpandedUuid(isExpanded ? null : item.uuid)}
        accessibilityRole="button"
        accessibilityLabel={`${t('entry.history')} ${formatDate(item.modifiedAt)}`}
        accessibilityState={{ expanded: isExpanded }}
      >
        <View style={styles.historyHeader}>
          <View style={styles.historyMeta}>
            <Text style={[styles.historyDate, { color: theme.text }]}>
              {formatDate(item.modifiedAt)}
            </Text>
            {isLatest && (
              <View style={[styles.latestBadge, { backgroundColor: theme.primary + '20' }]}>
                <Text style={[styles.latestBadgeText, { color: theme.primary }]}>
                  {t('entry.history')} #{history.length - index}
                </Text>
              </View>
            )}
          </View>
          <Text style={[styles.expandIcon, { color: theme.textTertiary }]}>
            {isExpanded ? '▲' : '▼'}
          </Text>
        </View>

        {/* Summary row */}
        <View style={styles.historySummary}>
          {item.title && (
            <Text style={[styles.historyField, { color: theme.textSecondary }]} numberOfLines={1}>
              📝 {item.title}
            </Text>
          )}
          {item.username && (
            <Text style={[styles.historyField, { color: theme.textSecondary }]} numberOfLines={1}>
              👤 {item.username}
            </Text>
          )}
          {item.url && (
            <Text style={[styles.historyField, { color: theme.textSecondary }]} numberOfLines={1}>
              🌐 {item.url}
            </Text>
          )}
          {item.hasPassword && (
            <Text style={[styles.historyField, { color: theme.textSecondary }]}>
              🔑 {t('entry.password')}: ••••••••
            </Text>
          )}
        </View>

        {/* Expanded: restore button */}
        {isExpanded && (
          <View style={styles.historyActions}>
            <TouchableOpacity
              style={[styles.restoreBtn, { backgroundColor: theme.primary }]}
              onPress={() => handleRestore(item)}
              disabled={restoreMutation.isPending}
              accessibilityRole="button"
              accessibilityLabel={t('entry.restoreFromHistory')}
            >
              {restoreMutation.isPending ? (
                <ActivityIndicator color="white" size="small" />
              ) : (
                <Text style={styles.restoreBtnText}>↩ {t('entry.restoreFromHistory')}</Text>
              )}
            </TouchableOpacity>
          </View>
        )}
      </TouchableOpacity>
    );
  };

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Header */}
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <TouchableOpacity
          onPress={() => navigation.goBack()}
          accessibilityRole="button"
          accessibilityLabel={t('common.back')}
        >
          <Text style={[styles.backBtn, { color: theme.primary }]}>← {t('common.back')}</Text>
        </TouchableOpacity>
        <View style={styles.headerCenter}>
          <Text style={[styles.headerTitle, { color: theme.text }]}>{t('entry.history')}</Text>
          <Text style={[styles.headerSubtitle, { color: theme.textSecondary }]} numberOfLines={1}>
            {entryTitle}
          </Text>
        </View>
        {history.length > 0 && (
          <TouchableOpacity
            onPress={handleClearAll}
            disabled={clearMutation.isPending}
            accessibilityRole="button"
            accessibilityLabel={t('entry.clearHistory')}
          >
            <Text style={[styles.clearBtn, { color: '#EF4444' }]}>{t('entry.clearHistory')}</Text>
          </TouchableOpacity>
        )}
      </View>

      {/* Content */}
      {isLoading ? (
        <View style={styles.centered}>
          <ActivityIndicator color={theme.primary} />
          <Text style={[styles.loadingText, { color: theme.textSecondary }]}>
            {t('common.loading')}
          </Text>
        </View>
      ) : history.length === 0 ? (
        <View style={styles.centered}>
          <Text style={styles.emptyIcon}>📋</Text>
          <Text style={[styles.emptyTitle, { color: theme.text }]}>{t('common.none')}</Text>
          <Text style={[styles.emptyDesc, { color: theme.textSecondary }]}>
            {t('entry.history')}
          </Text>
        </View>
      ) : (
        <FlatList
          data={history}
          keyExtractor={item => item.uuid}
          renderItem={renderItem}
          contentContainerStyle={styles.list}
          ItemSeparatorComponent={() => (
            <View style={[styles.separator, { backgroundColor: theme.border }]} />
          )}
        />
      )}
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
  backBtn: { fontSize: tokens.fontSize.md },
  headerCenter: { flex: 1, alignItems: 'center', paddingHorizontal: tokens.space.sm },
  headerTitle: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.bold },
  headerSubtitle: { fontSize: tokens.fontSize.xs, marginTop: 1 },
  clearBtn: { fontSize: tokens.fontSize.sm },
  centered: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    gap: tokens.space.md,
    padding: tokens.space.xl,
  },
  loadingText: { fontSize: tokens.fontSize.sm },
  emptyIcon: { fontSize: 48 },
  emptyTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.semibold },
  emptyDesc: { fontSize: tokens.fontSize.sm, textAlign: 'center' },
  list: { padding: tokens.space.md, gap: tokens.space.sm },
  historyItem: {
    borderRadius: tokens.radius.md,
    borderWidth: StyleSheet.hairlineWidth,
    padding: tokens.space.md,
    gap: tokens.space.sm,
  },
  historyHeader: { flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between' },
  historyMeta: { flexDirection: 'row', alignItems: 'center', gap: tokens.space.sm, flex: 1 },
  historyDate: { fontSize: tokens.fontSize.sm, fontWeight: tokens.fontWeight.medium },
  latestBadge: { paddingHorizontal: 6, paddingVertical: 2, borderRadius: tokens.radius.full },
  latestBadgeText: { fontSize: 10, fontWeight: '700' },
  expandIcon: { fontSize: 12 },
  historySummary: { gap: 3 },
  historyField: { fontSize: tokens.fontSize.xs },
  historyActions: {
    paddingTop: tokens.space.sm,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: 'rgba(0,0,0,0.08)',
  },
  restoreBtn: {
    paddingVertical: tokens.space.sm,
    paddingHorizontal: tokens.space.lg,
    borderRadius: tokens.radius.md,
    alignItems: 'center',
    alignSelf: 'flex-start',
  },
  restoreBtnText: {
    color: 'white',
    fontSize: tokens.fontSize.sm,
    fontWeight: tokens.fontWeight.semibold,
  },
  separator: { height: StyleSheet.hairlineWidth },
});
