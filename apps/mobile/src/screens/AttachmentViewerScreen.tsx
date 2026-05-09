/**
 * Attachment Viewer screen — mobile
 * View, download, and manage entry attachments
 * KeePassEx exclusive on mobile: no competitor has attachment management on mobile
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
  Share,
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
type Route = RouteProp<RootStackParamList, 'AttachmentViewer'>;

interface Attachment {
  name: string;
  sizeBytes: number;
  mimeType?: string;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function getFileIcon(name: string, mimeType?: string): string {
  const ext = name.split('.').pop()?.toLowerCase() ?? '';
  if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'].includes(ext)) return '🖼️';
  if (['pdf'].includes(ext)) return '📄';
  if (['doc', 'docx', 'odt'].includes(ext)) return '📝';
  if (['xls', 'xlsx', 'csv'].includes(ext)) return '📊';
  if (['zip', 'tar', 'gz', '7z', 'rar'].includes(ext)) return '📦';
  if (['mp3', 'wav', 'ogg', 'flac'].includes(ext)) return '🎵';
  if (['mp4', 'mov', 'avi', 'mkv'].includes(ext)) return '🎬';
  if (['txt', 'md', 'log'].includes(ext)) return '📃';
  if (['key', 'pem', 'crt', 'p12'].includes(ext)) return '🔑';
  return '📎';
}

export function AttachmentViewerScreen() {
  const navigation = useNavigation<Nav>();
  const route = useRoute<Route>();
  const { entryUuid, entryTitle } = route.params;
  const { theme } = useThemeStore();
  const { t } = useI18nStore();
  const queryClient = useQueryClient();
  const [savingName, setSavingName] = useState<string | null>(null);

  const { data: attachments = [], isLoading } = useQuery<Attachment[]>({
    queryKey: ['attachments', entryUuid],
    queryFn: () => KeePassExCore.getEntryAttachments(entryUuid),
  });

  const removeMutation = useMutation({
    mutationFn: (name: string) => KeePassExCore.removeAttachment(entryUuid, name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['attachments', entryUuid] });
      queryClient.invalidateQueries({ queryKey: ['entry', entryUuid] });
      ReactNativeHapticFeedback.trigger('notificationSuccess');
    },
  });

  const handleSave = async (attachment: Attachment) => {
    setSavingName(attachment.name);
    try {
      const path = await KeePassExCore.saveAttachment(entryUuid, attachment.name);
      await Share.share({ url: `file://${path}`, title: attachment.name });
    } catch (e: any) {
      Alert.alert(t('common.error'), e?.message ?? t('errors.generic'));
    } finally {
      setSavingName(null);
    }
  };

  const handleRemove = (attachment: Attachment) => {
    Alert.alert(t('entry.attachments'), `${t('common.delete')} "${attachment.name}"?`, [
      { text: t('common.cancel'), style: 'cancel' },
      {
        text: t('common.delete'),
        style: 'destructive',
        onPress: () => removeMutation.mutate(attachment.name),
      },
    ]);
  };

  const renderItem = ({ item }: { item: Attachment }) => {
    const isSaving = savingName === item.name;
    return (
      <View style={[styles.item, { backgroundColor: theme.surface, borderColor: theme.border }]}>
        <Text style={styles.itemIcon}>{getFileIcon(item.name, item.mimeType)}</Text>
        <View style={styles.itemInfo}>
          <Text style={[styles.itemName, { color: theme.text }]} numberOfLines={1}>
            {item.name}
          </Text>
          <Text style={[styles.itemMeta, { color: theme.textSecondary }]}>
            {formatBytes(item.sizeBytes)}
            {item.mimeType ? ` · ${item.mimeType}` : ''}
          </Text>
        </View>
        <View style={styles.itemActions}>
          <TouchableOpacity
            style={[styles.actionBtn, { backgroundColor: theme.primary + '20' }]}
            onPress={() => handleSave(item)}
            disabled={isSaving}
            accessibilityRole="button"
            accessibilityLabel={`${t('common.export')} ${item.name}`}
          >
            {isSaving ? (
              <ActivityIndicator size="small" color={theme.primary} />
            ) : (
              <Text style={[styles.actionBtnText, { color: theme.primary }]}>⬇</Text>
            )}
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.actionBtn, { backgroundColor: '#FEE2E2' }]}
            onPress={() => handleRemove(item)}
            accessibilityRole="button"
            accessibilityLabel={`${t('common.delete')} ${item.name}`}
          >
            <Text style={[styles.actionBtnText, { color: '#EF4444' }]}>🗑</Text>
          </TouchableOpacity>
        </View>
      </View>
    );
  };

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Header */}
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <TouchableOpacity onPress={() => navigation.goBack()} accessibilityRole="button">
          <Text style={[styles.backBtn, { color: theme.primary }]}>← {t('common.back')}</Text>
        </TouchableOpacity>
        <View style={styles.headerCenter}>
          <Text style={[styles.headerTitle, { color: theme.text }]}>{t('entry.attachments')}</Text>
          <Text style={[styles.headerSubtitle, { color: theme.textSecondary }]} numberOfLines={1}>
            {entryTitle}
          </Text>
        </View>
        <View style={{ width: 60 }} />
      </View>

      {/* Content */}
      {isLoading ? (
        <View style={styles.centered}>
          <ActivityIndicator color={theme.primary} />
        </View>
      ) : attachments.length === 0 ? (
        <View style={styles.centered}>
          <Text style={styles.emptyIcon}>📎</Text>
          <Text style={[styles.emptyTitle, { color: theme.text }]}>{t('common.none')}</Text>
          <Text style={[styles.emptyDesc, { color: theme.textSecondary }]}>
            {t('entry.attachments')}
          </Text>
        </View>
      ) : (
        <FlatList
          data={attachments}
          keyExtractor={item => item.name}
          renderItem={renderItem}
          contentContainerStyle={styles.list}
          ItemSeparatorComponent={() => (
            <View style={[styles.separator, { backgroundColor: theme.border }]} />
          )}
        />
      )}

      {/* Total size */}
      {attachments.length > 0 && (
        <View style={[styles.footer, { borderTopColor: theme.border }]}>
          <Text style={[styles.footerText, { color: theme.textTertiary }]}>
            {attachments.length} {t('entry.attachments').toLowerCase()} ·{' '}
            {formatBytes(attachments.reduce((sum, a) => sum + a.sizeBytes, 0))}
          </Text>
        </View>
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
  centered: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    gap: tokens.space.md,
    padding: tokens.space.xl,
  },
  emptyIcon: { fontSize: 48 },
  emptyTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.semibold },
  emptyDesc: { fontSize: tokens.fontSize.sm, textAlign: 'center' },
  list: { padding: tokens.space.md },
  item: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.md,
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
    borderWidth: StyleSheet.hairlineWidth,
  },
  itemIcon: { fontSize: 28, flexShrink: 0 },
  itemInfo: { flex: 1, minWidth: 0 },
  itemName: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.medium },
  itemMeta: { fontSize: tokens.fontSize.xs, marginTop: 2 },
  itemActions: { flexDirection: 'row', gap: tokens.space.xs, flexShrink: 0 },
  actionBtn: {
    width: 36,
    height: 36,
    borderRadius: tokens.radius.sm,
    alignItems: 'center',
    justifyContent: 'center',
  },
  actionBtnText: { fontSize: 16 },
  separator: { height: StyleSheet.hairlineWidth, marginHorizontal: tokens.space.md },
  footer: {
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.sm,
    borderTopWidth: StyleSheet.hairlineWidth,
  },
  footerText: { fontSize: 11, textAlign: 'center' },
});
