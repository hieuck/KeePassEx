/**
 * Groups management screen — create, rename, move, delete groups
 * KeePassEx exclusive: full group management on mobile
 * KeePassXC mobile doesn't exist; Keepassium has limited group management
 */
import React, { useState, useCallback } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  TextInput,
  Alert,
  Modal,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import ReactNativeHapticFeedback from 'react-native-haptic-feedback';
import { useThemeStore } from '../store/theme';
import { useI18nStore } from '../store/i18n';
import { tokens } from '@keepassex/ui';

const { KeePassExCore } = NativeModules;

interface GroupDto {
  uuid: string;
  parentUuid?: string;
  name: string;
  notes: string;
  iconId: number;
  entryCount: number;
  childGroupCount: number;
  isExpanded: boolean;
}

export function GroupsScreen() {
  const navigation = useNavigation();
  const { theme } = useThemeStore();
  const { t } = useI18nStore();
  const queryClient = useQueryClient();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingGroup, setEditingGroup] = useState<GroupDto | null>(null);
  const [groupName, setGroupName] = useState('');
  const [selectedParentUuid, setSelectedParentUuid] = useState<string | null>(null);

  const { data: groups = [], isLoading } = useQuery<GroupDto[]>({
    queryKey: ['groups'],
    queryFn: () => KeePassExCore.getGroups(),
  });

  const createMutation = useMutation({
    mutationFn: ({ name, parentUuid }: { name: string; parentUuid?: string }) =>
      KeePassExCore.createGroup({ name, parentUuid }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['groups'] });
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      setShowCreateModal(false);
      setGroupName('');
      ReactNativeHapticFeedback.trigger('notificationSuccess');
    },
  });

  const renameMutation = useMutation({
    mutationFn: ({ uuid, name }: { uuid: string; name: string }) =>
      KeePassExCore.updateGroup({ uuid, name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['groups'] });
      setEditingGroup(null);
      setGroupName('');
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (uuid: string) => KeePassExCore.deleteGroup(uuid),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['groups'] });
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      ReactNativeHapticFeedback.trigger('notificationSuccess');
    },
  });

  const handleDelete = useCallback(
    (group: GroupDto) => {
      const hasContent = group.entryCount > 0 || group.childGroupCount > 0;
      Alert.alert(
        t('group.delete'),
        hasContent
          ? t('group.deleteWithContent', {
              entries: group.entryCount,
              groups: group.childGroupCount,
            })
          : t('group.confirmDelete', { name: group.name }),
        [
          { text: t('common.cancel'), style: 'cancel' },
          {
            text: t('common.delete'),
            style: 'destructive',
            onPress: () => deleteMutation.mutate(group.uuid),
          },
        ]
      );
    },
    [deleteMutation, t]
  );

  const handleEdit = useCallback((group: GroupDto) => {
    setEditingGroup(group);
    setGroupName(group.name);
  }, []);

  const handleSave = useCallback(() => {
    if (!groupName.trim()) return;
    if (editingGroup) {
      renameMutation.mutate({ uuid: editingGroup.uuid, name: groupName.trim() });
    } else {
      createMutation.mutate({
        name: groupName.trim(),
        parentUuid: selectedParentUuid ?? undefined,
      });
    }
  }, [groupName, editingGroup, selectedParentUuid, createMutation, renameMutation]);

  const renderGroup = useCallback(
    ({ item }: { item: GroupDto }) => {
      const isRoot = !item.parentUuid;
      return (
        <View
          style={[styles.groupItem, { backgroundColor: theme.surface, borderColor: theme.border }]}
        >
          <View style={styles.groupIcon}>
            <Text style={styles.groupIconText}>{isRoot ? '🏠' : '📁'}</Text>
          </View>
          <View style={styles.groupInfo}>
            <Text style={[styles.groupName, { color: theme.text }]}>{item.name}</Text>
            <Text style={[styles.groupMeta, { color: theme.textSecondary }]}>
              {item.entryCount}{' '}
              {t('vault.statistics_entries', { count: item.entryCount })
                .replace(String(item.entryCount), '')
                .trim()}
              {item.childGroupCount > 0 &&
                ` · ${item.childGroupCount} ${t('vault.statistics_groups', { count: item.childGroupCount }).replace(String(item.childGroupCount), '').trim()}`}
            </Text>
          </View>
          {!isRoot && (
            <View style={styles.groupActions}>
              <TouchableOpacity
                style={[styles.actionBtn, { backgroundColor: theme.backgroundSecondary }]}
                onPress={() => handleEdit(item)}
                accessibilityRole="button"
                accessibilityLabel={`${t('common.edit')} ${item.name}`}
              >
                <Text style={[styles.actionBtnText, { color: theme.primary }]}>✏️</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[styles.actionBtn, { backgroundColor: '#FEE2E2' }]}
                onPress={() => handleDelete(item)}
                accessibilityRole="button"
                accessibilityLabel={`${t('common.delete')} ${item.name}`}
              >
                <Text style={styles.actionBtnText}>🗑</Text>
              </TouchableOpacity>
            </View>
          )}
        </View>
      );
    },
    [theme, t, handleEdit, handleDelete]
  );

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
        <Text style={[styles.headerTitle, { color: theme.text }]}>{t('group.manage')}</Text>
        <TouchableOpacity
          onPress={() => {
            setShowCreateModal(true);
            setGroupName('');
            setEditingGroup(null);
          }}
          accessibilityRole="button"
          accessibilityLabel={t('group.create')}
        >
          <Text style={[styles.addBtn, { color: theme.primary }]}>+ {t('group.create')}</Text>
        </TouchableOpacity>
      </View>

      {/* Group list */}
      <FlatList
        data={groups}
        keyExtractor={item => item.uuid}
        renderItem={renderGroup}
        contentContainerStyle={styles.list}
        ItemSeparatorComponent={() => (
          <View style={[styles.separator, { backgroundColor: theme.border }]} />
        )}
      />

      {/* Create/Edit Modal */}
      <Modal
        visible={showCreateModal || editingGroup !== null}
        transparent
        animationType="slide"
        onRequestClose={() => {
          setShowCreateModal(false);
          setEditingGroup(null);
        }}
      >
        <View style={styles.modalOverlay}>
          <View style={[styles.modalCard, { backgroundColor: theme.surface }]}>
            <Text style={[styles.modalTitle, { color: theme.text }]}>
              {editingGroup ? t('group.rename') : t('group.create')}
            </Text>

            <TextInput
              style={[
                styles.modalInput,
                { color: theme.text, borderColor: theme.border, backgroundColor: theme.background },
              ]}
              placeholder={t('group.name')}
              placeholderTextColor={theme.textTertiary}
              value={groupName}
              onChangeText={setGroupName}
              autoFocus
              returnKeyType="done"
              onSubmitEditing={handleSave}
              accessibilityLabel={t('group.name')}
            />

            <View style={styles.modalActions}>
              <TouchableOpacity
                style={[styles.modalBtn, { backgroundColor: theme.backgroundSecondary }]}
                onPress={() => {
                  setShowCreateModal(false);
                  setEditingGroup(null);
                  setGroupName('');
                }}
                accessibilityRole="button"
              >
                <Text style={[styles.modalBtnText, { color: theme.text }]}>
                  {t('common.cancel')}
                </Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[
                  styles.modalBtn,
                  { backgroundColor: theme.primary },
                  !groupName.trim() && styles.modalBtnDisabled,
                ]}
                onPress={handleSave}
                disabled={!groupName.trim() || createMutation.isPending || renameMutation.isPending}
                accessibilityRole="button"
              >
                <Text style={[styles.modalBtnText, { color: 'white' }]}>
                  {createMutation.isPending || renameMutation.isPending ? '...' : t('common.save')}
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      </Modal>
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
  headerTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.bold },
  addBtn: { fontSize: tokens.fontSize.sm, fontWeight: tokens.fontWeight.medium },
  list: { padding: tokens.space.md, gap: tokens.space.sm },
  groupItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.md,
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
  },
  groupIcon: {
    width: 44,
    height: 44,
    borderRadius: tokens.radius.md,
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: 'rgba(37,99,235,0.1)',
  },
  groupIconText: { fontSize: 22 },
  groupInfo: { flex: 1 },
  groupName: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  groupMeta: { fontSize: tokens.fontSize.sm, marginTop: 2 },
  groupActions: { flexDirection: 'row', gap: tokens.space.xs },
  actionBtn: {
    width: 36,
    height: 36,
    borderRadius: tokens.radius.sm,
    alignItems: 'center',
    justifyContent: 'center',
  },
  actionBtnText: { fontSize: 16 },
  separator: { height: StyleSheet.hairlineWidth },
  modalOverlay: { flex: 1, backgroundColor: 'rgba(0,0,0,0.5)', justifyContent: 'flex-end' },
  modalCard: {
    borderTopLeftRadius: tokens.radius.xl,
    borderTopRightRadius: tokens.radius.xl,
    padding: tokens.space.xl,
    gap: tokens.space.lg,
  },
  modalTitle: {
    fontSize: tokens.fontSize.xl,
    fontWeight: tokens.fontWeight.bold,
    textAlign: 'center',
  },
  modalInput: {
    borderWidth: 1,
    borderRadius: tokens.radius.md,
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.sm,
    fontSize: tokens.fontSize.md,
  },
  modalActions: { flexDirection: 'row', gap: tokens.space.md },
  modalBtn: {
    flex: 1,
    paddingVertical: tokens.space.md,
    borderRadius: tokens.radius.md,
    alignItems: 'center',
  },
  modalBtnDisabled: { opacity: 0.5 },
  modalBtnText: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
});
