/**
 * Main vault screen - enhanced with swipe actions, filter chips, multi-select, i18n EN/VI
 */
import React, { useState, useCallback, useMemo } from 'react';
import {
  View, Text, FlatList, TouchableOpacity, StyleSheet,
  TextInput, RefreshControl, ScrollView, Alert,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import Clipboard from '@react-native-clipboard/clipboard';
import ReactNativeHapticFeedback from 'react-native-haptic-feedback';
import { Swipeable } from 'react-native-gesture-handler';
import { useVaultStore } from '../store/vault';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import { EmptyState } from '../components/EmptyState';
import type { RootStackParamList } from '../App';

const { KeePassExCore } = NativeModules;
type Nav = NativeStackNavigationProp<RootStackParamList>;
type FilterType = 'all' | 'favorites' | 'otp' | 'expiring' | 'noPassword';
type SortField = 'title' | 'modified' | 'created' | 'username';
type SortDir = 'asc' | 'desc';

interface EntryDto {
  uuid: string; title: string; username: string; url: string;
  iconId: number; hasOtp: boolean; hasPasskey: boolean; hasSshKey: boolean;
  isExpired: boolean; isExpiringSoon: boolean; hasPassword: boolean;
  isFavorite: boolean; modifiedAt: string; createdAt: string;
  lastUsedAt?: string; groupName?: string;
}

const FILTERS = [
  { id: 'all' as FilterType,        label: 'All',         labelVi: 'Tat ca',      icon: 'all' },
  { id: 'favorites' as FilterType,  label: 'Favorites',   labelVi: 'Yeu thich',   icon: 'fav' },
  { id: 'otp' as FilterType,        label: 'OTP',         labelVi: 'OTP',         icon: 'otp' },
  { id: 'expiring' as FilterType,   label: 'Expiring',    labelVi: 'Sap het han', icon: 'exp' },
  { id: 'noPassword' as FilterType, label: 'No Password', labelVi: 'Khong co MK', icon: 'nopw' },
];

export function VaultScreen() {
  const navigation = useNavigation<Nav>();
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const { selectedGroupUuid } = useVaultStore();
  const queryClient = useQueryClient();
  const [search, setSearch] = useState('');
  const [activeFilter, setActiveFilter] = useState<FilterType>('all');
  const [sortField, setSortField] = useState<SortField>('title');
  const [sortDir, setSortDir] = useState<SortDir>('asc');
  const [selectedUuids, setSelectedUuids] = useState<Set<string>>(new Set());
  const [isMultiSelect, setIsMultiSelect] = useState(false);

  const { data: entries = [], isLoading, refetch } = useQuery<EntryDto[]>({
    queryKey: ['entries', selectedGroupUuid, search],
    queryFn: async () => {
      if (search.trim()) return KeePassExCore.searchEntries(search);
      return KeePassExCore.getEntries(selectedGroupUuid ?? null);
    },
  });

  const { data: recentEntries = [] } = useQuery<EntryDto[]>({
    queryKey: ['recent-entries'],
    queryFn: () => KeePassExCore.getRecentEntries(3),
    staleTime: 60_000,
  });

  const deleteMutation = useMutation({
    mutationFn: (uuids: string[]) =>
      Promise.all(uuids.map(uuid => KeePassExCore.deleteEntry(uuid, false))),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      setSelectedUuids(new Set());
      setIsMultiSelect(false);
    },
  });

  const favoriteMutation = useMutation({
    mutationFn: ({ uuid, fav }: { uuid: string; fav: boolean }) =>
      KeePassExCore.setFavorite(uuid, fav),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['entries'] }),
  });

  const displayEntries = useMemo(() => {
    let filtered = [...entries];
    switch (activeFilter) {
      case 'favorites':  filtered = filtered.filter(e => e.isFavorite); break;
      case 'otp':        filtered = filtered.filter(e => e.hasOtp); break;
      case 'expiring':   filtered = filtered.filter(e => e.isExpiringSoon || e.isExpired); break;
    }
    filtered.sort((a, b) => {
      let cmp = 0;
      if (sortField === 'title')    cmp = a.title.localeCompare(b.title);
      if (sortField === 'username') cmp = a.username.localeCompare(b.username);
      if (sortField === 'modified') cmp = a.modifiedAt.localeCompare(b.modifiedAt);
      if (sortField === 'created')  cmp = a.createdAt.localeCompare(b.createdAt);
      return sortDir === 'asc' ? cmp : -cmp;
    });
    return filtered;
  }, [entries, activeFilter, sortField, sortDir]);

  const handleCopyPassword = useCallback(async (uuid: string) => {
    try {
      const password = await KeePassExCore.getEntryPassword(uuid);
      Clipboard.setString(password);
      ReactNativeHapticFeedback.trigger('impactLight');
      setTimeout(() => Clipboard.setString(''), 10_000);
    } catch { /* silent */ }
  }, []);

  const handleLongPress = useCallback((uuid: string) => {
    ReactNativeHapticFeedback.trigger('impactMedium');
    setIsMultiSelect(true);
    setSelectedUuids(new Set([uuid]));
  }, []);

  const handleToggleSelect = useCallback((uuid: string) => {
    setSelectedUuids(prev => {
      const next = new Set(prev);
      if (next.has(uuid)) next.delete(uuid); else next.add(uuid);
      return next;
    });
  }, []);

  const handleDeleteSelected = useCallback(() => {
    const count = selectedUuids.size;
    Alert.alert(
      t('bulk.confirmDelete', { count }),
      t('bulk.confirmDeleteDesc'),
      [
        { text: t('common.cancel'), style: 'cancel' },
        { text: t('common.delete'), style: 'destructive',
          onPress: () => deleteMutation.mutate(Array.from(selectedUuids)) },
      ],
    );
  }, [selectedUuids, deleteMutation, t]);

  const renderItem = useCallback(({ item }: { item: EntryDto }) => (
    <SwipeableEntryRow
      entry={item} theme={theme}
      isSelected={selectedUuids.has(item.uuid)} isMultiSelect={isMultiSelect}
      onPress={() => {
        if (isMultiSelect) { handleToggleSelect(item.uuid); return; }
        navigation.navigate('EntryDetail', { uuid: item.uuid });
      }}
      onLongPress={() => handleLongPress(item.uuid)}
      onCopyPassword={() => handleCopyPassword(item.uuid)}
      onEdit={() => navigation.navigate('EntryEdit', { uuid: item.uuid })}
      onDelete={() => deleteMutation.mutate([item.uuid])}
    />
  ), [navigation, handleCopyPassword, handleLongPress, handleToggleSelect,
      selectedUuids, isMultiSelect, theme, deleteMutation, favoriteMutation]);

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        {isMultiSelect ? (
          <>
            <TouchableOpacity onPress={() => { setIsMultiSelect(false); setSelectedUuids(new Set()); }}>
              <Text style={[styles.headerAction, { color: theme.primary }]}>{t('common.cancel')}</Text>
            </TouchableOpacity>
            <Text style={[styles.headerTitle, { color: theme.text }]}>
              {selectedUuids.size} {t('bulk.selected', { count: selectedUuids.size })}
            </Text>
            <TouchableOpacity onPress={handleDeleteSelected}>
              <Text style={[styles.headerAction, { color: '#EF4444' }]}>{t('common.delete')}</Text>
            </TouchableOpacity>
          </>
        ) : (
          <>
            <Text style={[styles.headerTitle, { color: theme.text }]}>{t('vault.open').replace('Open ', '')}</Text>
            <View style={styles.headerRight}>
              <TouchableOpacity style={styles.sortBtn}
                onPress={() => setSortDir(d => d === 'asc' ? 'desc' : 'asc')}>
                <Text style={[styles.sortBtnText, { color: theme.textSecondary }]}>
                  {sortDir === 'asc' ? 'A-Z' : 'Z-A'}
                </Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[styles.addButton, { backgroundColor: theme.primary }]}
                onPress={() => navigation.navigate('EntryEdit', {})}
                accessibilityRole="button"
                accessibilityLabel={t('entry.new')}>
                <Text style={styles.addButtonText}>+</Text>
              </TouchableOpacity>
            </View>
          </>
        )}
      </View>

      <View style={[styles.searchContainer, { backgroundColor: theme.backgroundSecondary }]}>
        <Text style={styles.searchIcon}>🔍</Text>
        <TextInput
          style={[styles.searchInput, { color: theme.text }]}
          placeholder={t('entry.searchPlaceholder')}
          placeholderTextColor={theme.textTertiary}
          value={search} onChangeText={setSearch}
          returnKeyType="search" clearButtonMode="while-editing"
          accessibilityLabel={t('common.search')}
        />
      </View>

      <ScrollView horizontal showsHorizontalScrollIndicator={false}
        style={styles.filterRow} contentContainerStyle={styles.filterContent}>
        {FILTERS.map(f => (
          <TouchableOpacity key={f.id}
            style={[styles.filterChip, { borderColor: theme.border, backgroundColor: theme.backgroundSecondary },
              activeFilter === f.id && { backgroundColor: theme.primary, borderColor: theme.primary }]}
            onPress={() => setActiveFilter(f.id)}
            accessibilityRole="button"
            accessibilityState={{ selected: activeFilter === f.id }}>
            <Text style={[styles.filterChipText,
              { color: activeFilter === f.id ? '#fff' : theme.textSecondary }]}>
              {f.id === 'all' ? t('vaultFilter.all')
                : f.id === 'favorites' ? t('vaultFilter.favorites')
                : f.id === 'otp' ? t('vaultFilter.withOtp')
                : f.id === 'expiring' ? t('vaultFilter.expiring')
                : t('vaultFilter.noPassword')}
            </Text>
          </TouchableOpacity>
        ))}
      </ScrollView>

        <View style={[styles.recentSection, { borderBottomColor: theme.border }]}>
          <Text style={[styles.sectionLabel, { color: theme.textTertiary }]}>{t('vaultFilter.recentlyUsed')}</Text>
          <ScrollView horizontal showsHorizontalScrollIndicator={false}
            contentContainerStyle={styles.recentList}>
            {recentEntries.map(entry => (
              <TouchableOpacity key={entry.uuid}
                style={[styles.recentChip, { backgroundColor: theme.backgroundSecondary, borderColor: theme.border }]}
                onPress={() => navigation.navigate('EntryDetail', { uuid: entry.uuid })}
                accessibilityRole="button"
                accessibilityLabel={entry.title}>
                <Text style={[styles.recentChipText, { color: theme.text }]} numberOfLines={1}>
                  {entry.title}
                </Text>
              </TouchableOpacity>
            ))}
          </ScrollView>
        </View>
      )}

      <FlatList
        data={displayEntries} keyExtractor={item => item.uuid} renderItem={renderItem}
        refreshControl={<RefreshControl refreshing={isLoading} onRefresh={refetch} tintColor={theme.primary} />}
          <EmptyState icon="key"
            title={search ? t('entry.searchPlaceholder').replace('...', '') : t('entry.noEntries')}
            description={search ? `${t('common.search')}: "${search}"` : t('entry.noEntriesInGroup')}
            actionLabel={search ? undefined : t('entry.new')}
            onAction={search ? undefined : () => navigation.navigate('EntryEdit', {})}
            theme={theme} />
        ) : null}
        ItemSeparatorComponent={() => <View style={[styles.separator, { backgroundColor: theme.border }]} />}
      />
    </SafeAreaView>
  );
}

function SwipeableEntryRow({ entry, theme, isSelected, isMultiSelect,
  onPress, onLongPress, onCopyPassword, onEdit, onDelete, onToggleFavorite }: {
  entry: EntryDto; theme: ReturnType<typeof useThemeStore>['theme'];
  isSelected: boolean; isMultiSelect: boolean;
  onPress: () => void; onLongPress: () => void; onCopyPassword: () => void;
  onEdit: () => void; onDelete: () => void; onToggleFavorite: () => void;
}) {
  const { t } = useTranslation();
  const renderLeft = () => (
    <View style={styles.swipeLeft}>
      <TouchableOpacity style={[styles.swipeAction, { backgroundColor: '#2563EB' }]}
        onPress={onCopyPassword} accessibilityRole="button" accessibilityLabel={t('entry.copyPassword')}>
        <Text style={styles.swipeActionText}>{t('swipe.copyPassword')}</Text>
      </TouchableOpacity>
      <TouchableOpacity
        style={[styles.swipeAction, { backgroundColor: entry.isFavorite ? '#6B7280' : '#F59E0B' }]}
        onPress={onToggleFavorite} accessibilityRole="button"
        accessibilityLabel={entry.isFavorite ? t('swipe.unfavorite') : t('swipe.favorite')}>
        <Text style={styles.swipeActionText}>{entry.isFavorite ? t('swipe.unfavorite') : t('swipe.favorite')}</Text>
      </TouchableOpacity>
    </View>
  );

  const renderRight = () => (
    <View style={styles.swipeRight}>
      <TouchableOpacity style={[styles.swipeAction, { backgroundColor: '#10B981' }]}
        onPress={onEdit} accessibilityRole="button" accessibilityLabel={t('common.edit')}>
        <Text style={styles.swipeActionText}>{t('swipe.edit')}</Text>
      </TouchableOpacity>
      <TouchableOpacity style={[styles.swipeAction, { backgroundColor: '#EF4444' }]}
        onPress={onDelete} accessibilityRole="button" accessibilityLabel={t('common.delete')}>
        <Text style={styles.swipeActionText}>{t('swipe.delete')}</Text>
      </TouchableOpacity>
    </View>
  );

  return (
    <Swipeable renderLeftActions={renderLeft} renderRightActions={renderRight}
      overshootLeft={false} overshootRight={false}>
      <TouchableOpacity
        style={[styles.entryItem,
          { backgroundColor: isSelected ? theme.primary + '18' : theme.surface }]}
        onPress={onPress} onLongPress={onLongPress} delayLongPress={400}
        accessibilityRole="button" accessibilityLabel={entry.title + ', ' + entry.username}
        accessibilityState={{ selected: isSelected }}>
        {isMultiSelect && (
          <View style={[styles.checkbox,
            { borderColor: isSelected ? theme.primary : theme.border },
            isSelected && { backgroundColor: theme.primary }]}>
            {isSelected && <Text style={styles.checkmark}>✓</Text>}
          </View>
        )}
        <View style={[styles.entryIcon, { backgroundColor: theme.backgroundTertiary }]}>
          <Text style={styles.entryIconText}>{getIconEmoji(entry.iconId)}</Text>
        </View>
        <View style={styles.entryContent}>
          <View style={styles.entryTitleRow}>
            <Text style={[styles.entryTitle,
              { color: entry.isExpired ? theme.textTertiary : theme.text },
              entry.isExpired && styles.strikethrough]} numberOfLines={1}>
              {entry.isFavorite ? '⭐ ' : ''}{entry.title}
            </Text>
            <View style={styles.badges}>
              {entry.hasOtp && <View style={styles.badge}><Text style={styles.badgeText}>OTP</Text></View>}
              {entry.hasPasskey && <View style={styles.badge}><Text style={styles.badgeText}>PK</Text></View>}
              {entry.isExpired && <View style={[styles.badge, { backgroundColor: '#FEE2E2' }]}><Text style={[styles.badgeText, { color: '#EF4444' }]}>EXP</Text></View>}
            </View>
          </View>
          <Text style={[styles.entryUsername, { color: theme.textSecondary }]} numberOfLines={1}>
            {entry.username || '-'}
          </Text>
          {entry.url ? (
            <Text style={[styles.entryDomain, { color: theme.textTertiary }]} numberOfLines={1}>
              {extractDomain(entry.url)}
            </Text>
          ) : null}
        </View>
        <TouchableOpacity style={styles.copyButton} onPress={onCopyPassword}
          hitSlop={{ top: 8, bottom: 8, left: 8, right: 8 }}
          accessibilityRole="button" accessibilityLabel={'Copy password for ' + entry.title}>
          <Text style={[styles.copyIcon, { color: theme.textTertiary }]}>⎘</Text>
        </TouchableOpacity>
      </TouchableOpacity>
    </Swipeable>
  );
}

function getIconEmoji(iconId: number): string {
  const map: Record<number, string> = {
    0: 'key', 1: 'web', 2: 'warn', 5: 'pc', 8: 'mail', 9: 'card', 10: 'bank', 11: 'phone', 48: 'folder',
  };
  return map[iconId] ?? 'key';
}

function extractDomain(url: string): string {
  try { return new URL(url).hostname.replace(/^www\./, ''); }
  catch { return url; }
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  header: { flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between',
    paddingHorizontal: tokens.space.lg, paddingVertical: tokens.space.md, borderBottomWidth: StyleSheet.hairlineWidth },
  headerTitle: { fontSize: tokens.fontSize.xl, fontWeight: tokens.fontWeight.bold },
  headerAction: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.medium },
  headerRight: { flexDirection: 'row', alignItems: 'center', gap: tokens.space.sm },
  sortBtn: { paddingHorizontal: tokens.space.sm, paddingVertical: tokens.space.xs },
  sortBtnText: { fontSize: tokens.fontSize.xs },
  addButton: { width: 36, height: 36, borderRadius: 18, alignItems: 'center', justifyContent: 'center' },
  addButtonText: { color: 'white', fontSize: 22, lineHeight: 26 },
  searchContainer: { flexDirection: 'row', alignItems: 'center',
    marginHorizontal: tokens.space.md, marginTop: tokens.space.md,
    paddingHorizontal: tokens.space.md, paddingVertical: tokens.space.sm,
    borderRadius: tokens.radius.full, gap: tokens.space.sm },
  searchIcon: { fontSize: 14 },
  searchInput: { flex: 1, fontSize: tokens.fontSize.md, padding: 0 },
  filterRow: { flexGrow: 0, marginTop: tokens.space.sm },
  filterContent: { paddingHorizontal: tokens.space.md, gap: tokens.space.sm, paddingBottom: tokens.space.sm },
  filterChip: { flexDirection: 'row', alignItems: 'center', gap: 4,
    paddingHorizontal: tokens.space.md, paddingVertical: tokens.space.xs,
    borderRadius: tokens.radius.full, borderWidth: 1 },
  filterChipText: { fontSize: tokens.fontSize.xs, fontWeight: tokens.fontWeight.medium },
  recentSection: { paddingVertical: tokens.space.sm, borderBottomWidth: StyleSheet.hairlineWidth },
  sectionLabel: { fontSize: tokens.fontSize.xs, fontWeight: '600', paddingHorizontal: tokens.space.lg,
    marginBottom: 6, textTransform: 'uppercase', letterSpacing: 0.5 },
  recentList: { paddingHorizontal: tokens.space.md, gap: tokens.space.sm },
  recentChip: { flexDirection: 'row', alignItems: 'center', gap: 6,
    paddingHorizontal: tokens.space.md, paddingVertical: tokens.space.xs,
    borderRadius: tokens.radius.full, borderWidth: 1, maxWidth: 140 },
  recentChipText: { fontSize: tokens.fontSize.xs, fontWeight: '500', flex: 1 },
  entryItem: { flexDirection: 'row', alignItems: 'center',
    paddingHorizontal: tokens.space.lg, paddingVertical: tokens.space.md, gap: tokens.space.md },
  checkbox: { width: 22, height: 22, borderRadius: 11, borderWidth: 2, alignItems: 'center', justifyContent: 'center' },
  checkmark: { color: 'white', fontSize: 12, fontWeight: '700' },
  entryIcon: { width: 40, height: 40, borderRadius: tokens.radius.md, alignItems: 'center', justifyContent: 'center' },
  entryIconText: { fontSize: 20 },
  entryContent: { flex: 1, gap: 1 },
  entryTitleRow: { flexDirection: 'row', alignItems: 'center', gap: 4 },
  entryTitle: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold, flex: 1 },
  strikethrough: { textDecorationLine: 'line-through' },
  entryUsername: { fontSize: tokens.fontSize.sm },
  entryDomain: { fontSize: 11 },
  badges: { flexDirection: 'row', gap: 3 },
  badge: { backgroundColor: '#EFF6FF', paddingHorizontal: 4, paddingVertical: 1, borderRadius: 4 },
  badgeText: { fontSize: 9, fontWeight: '700', color: '#2563EB' },
  copyButton: { padding: tokens.space.xs },
  copyIcon: { fontSize: 18 },
  separator: { height: StyleSheet.hairlineWidth, marginLeft: 72 },
  swipeLeft: { flexDirection: 'row' },
  swipeRight: { flexDirection: 'row' },
  swipeAction: { width: 72, alignItems: 'center', justifyContent: 'center', gap: 2 },
  swipeActionText: { fontSize: 10, color: 'white', fontWeight: '600' },
});
