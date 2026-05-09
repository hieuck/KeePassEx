/**
 * Secure Note screen — create and view rich secure notes
 * KeePassEx exclusive on mobile: full-screen secure note editor
 * Notes are stored encrypted inside the KDBX vault
 */
import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  ScrollView,
  Alert,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation, useRoute } from '@react-navigation/native';
import type { NativeStackNavigationProp, RouteProp } from '@react-navigation/native-stack';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import ReactNativeHapticFeedback from 'react-native-haptic-feedback';
import Clipboard from '@react-native-clipboard/clipboard';
import { useThemeStore } from '../store/theme';
import { useI18nStore } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import type { RootStackParamList } from '../App';

const { KeePassExCore } = NativeModules;

type Nav = NativeStackNavigationProp<RootStackParamList>;
type Route = RouteProp<RootStackParamList, 'SecureNote'>;

interface NoteEntry {
  uuid: string;
  title: string;
  notes: string;
  tags: string[];
  createdAt: string;
  modifiedAt: string;
}

const NOTE_TEMPLATES = [
  { icon: '🔑', label: 'API Key', template: 'Service: \nAPI Key: \nSecret: \nNotes: ' },
  { icon: '💳', label: 'Credit Card', template: 'Card Number: \nExpiry: \nCVV: \nHolder: ' },
  { icon: '🏦', label: 'Bank Account', template: 'Bank: \nAccount: \nRouting: \nNotes: ' },
  { icon: '📝', label: 'General Note', template: '' },
  {
    icon: '🔐',
    label: 'Recovery Codes',
    template: 'Service: \nRecovery Codes:\n1. \n2. \n3. \n4. \n5. ',
  },
  { icon: '🌐', label: 'Server Info', template: 'Host: \nPort: \nUsername: \nPassword: \nNotes: ' },
];

export function SecureNoteScreen() {
  const navigation = useNavigation<Nav>();
  const route = useRoute<Route>();
  const { entryUuid } = route.params ?? {};
  const isNew = !entryUuid;
  const { theme } = useThemeStore();
  const { t } = useI18nStore();
  const queryClient = useQueryClient();

  const [title, setTitle] = useState('');
  const [notes, setNotes] = useState('');
  const [tags, setTags] = useState<string[]>([]);
  const [editing, setEditing] = useState(isNew);
  const [showTemplates, setShowTemplates] = useState(isNew);
  const [wordCount, setWordCount] = useState(0);

  const { data: entry } = useQuery<NoteEntry>({
    queryKey: ['entry', entryUuid],
    queryFn: () => KeePassExCore.getEntry(entryUuid, false),
    enabled: !isNew && !!entryUuid,
  });

  useEffect(() => {
    if (entry) {
      setTitle(entry.title);
      setNotes(entry.notes);
      setTags(entry.tags);
    }
  }, [entry]);

  useEffect(() => {
    const words = notes.trim().split(/\s+/).filter(Boolean).length;
    setWordCount(words);
  }, [notes]);

  const saveMutation = useMutation({
    mutationFn: async () => {
      if (isNew) {
        await KeePassExCore.createEntry({
          groupUuid: null, // root group
          title: title || t('entry.new'),
          username: '',
          password: '',
          url: '',
          notes,
          tags: [...tags, 'note'],
          iconId: 22, // note icon
        });
      } else {
        await KeePassExCore.updateEntry({
          uuid: entryUuid,
          title,
          username: entry?.username ?? '',
          password: '',
          url: '',
          notes,
          tags,
          iconId: 22,
          expiry: null,
          customFields: [],
        });
      }
      await KeePassExCore.saveVault();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      queryClient.invalidateQueries({ queryKey: ['entry', entryUuid] });
      ReactNativeHapticFeedback.trigger('notificationSuccess');
      setEditing(false);
      if (isNew) navigation.goBack();
    },
    onError: (e: any) => {
      Alert.alert(t('common.error'), e?.message ?? t('errors.generic'));
    },
  });

  const handleCopyAll = () => {
    Clipboard.setString(notes);
    ReactNativeHapticFeedback.trigger('impactLight');
    setTimeout(() => Clipboard.setString(''), 30_000);
  };

  const applyTemplate = (template: string) => {
    setNotes(template);
    setShowTemplates(false);
  };

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <KeyboardAvoidingView
        style={{ flex: 1 }}
        behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      >
        {/* Header */}
        <View style={[styles.header, { borderBottomColor: theme.border }]}>
          <TouchableOpacity
            onPress={() => {
              if (editing && (title !== entry?.title || notes !== entry?.notes)) {
                Alert.alert(t('vault.unsavedChanges'), t('vault.unsavedChanges'), [
                  { text: t('common.cancel'), style: 'cancel' },
                  { text: t('common.ok'), onPress: () => navigation.goBack() },
                ]);
              } else {
                navigation.goBack();
              }
            }}
            accessibilityRole="button"
          >
            <Text style={[styles.backBtn, { color: theme.primary }]}>← {t('common.back')}</Text>
          </TouchableOpacity>

          <View style={styles.headerCenter}>
            {editing ? (
              <TextInput
                style={[styles.titleInput, { color: theme.text, borderBottomColor: theme.border }]}
                value={title}
                onChangeText={setTitle}
                placeholder={t('entry.title')}
                placeholderTextColor={theme.textTertiary}
                autoCapitalize="sentences"
                accessibilityLabel={t('entry.title')}
              />
            ) : (
              <Text style={[styles.headerTitle, { color: theme.text }]} numberOfLines={1}>
                {title || t('entry.new')}
              </Text>
            )}
          </View>

          <View style={styles.headerActions}>
            {!editing ? (
              <>
                <TouchableOpacity onPress={handleCopyAll} accessibilityRole="button">
                  <Text style={[styles.headerBtn, { color: theme.primary }]}>⎘</Text>
                </TouchableOpacity>
                <TouchableOpacity onPress={() => setEditing(true)} accessibilityRole="button">
                  <Text style={[styles.headerBtn, { color: theme.primary }]}>
                    {t('common.edit')}
                  </Text>
                </TouchableOpacity>
              </>
            ) : (
              <TouchableOpacity
                onPress={() => saveMutation.mutate()}
                disabled={saveMutation.isPending}
                accessibilityRole="button"
              >
                <Text style={[styles.headerBtn, { color: theme.primary, fontWeight: '700' }]}>
                  {saveMutation.isPending ? '...' : t('common.save')}
                </Text>
              </TouchableOpacity>
            )}
          </View>
        </View>

        {/* Templates (new note only) */}
        {showTemplates && isNew && (
          <View style={[styles.templates, { borderBottomColor: theme.border }]}>
            <Text style={[styles.templatesLabel, { color: theme.textSecondary }]}>
              {t('secureNote.chooseTemplate')}
            </Text>
            <ScrollView
              horizontal
              showsHorizontalScrollIndicator={false}
              contentContainerStyle={styles.templatesList}
            >
              {NOTE_TEMPLATES.map(tmpl => (
                <TouchableOpacity
                  key={tmpl.label}
                  style={[
                    styles.templateChip,
                    { backgroundColor: theme.backgroundSecondary, borderColor: theme.border },
                  ]}
                  onPress={() => applyTemplate(tmpl.template)}
                  accessibilityRole="button"
                  accessibilityLabel={tmpl.label}
                >
                  <Text style={styles.templateIcon}>{tmpl.icon}</Text>
                  <Text style={[styles.templateLabel, { color: theme.text }]}>{tmpl.label}</Text>
                </TouchableOpacity>
              ))}
            </ScrollView>
          </View>
        )}

        {/* Note content */}
        <ScrollView style={styles.content} contentContainerStyle={styles.contentInner}>
          {editing ? (
            <TextInput
              style={[styles.noteInput, { color: theme.text, backgroundColor: theme.surface }]}
              value={notes}
              onChangeText={setNotes}
              multiline
              placeholder={t('entry.notes')}
              placeholderTextColor={theme.textTertiary}
              textAlignVertical="top"
              autoCapitalize="sentences"
              spellCheck
              accessibilityLabel={t('entry.notes')}
            />
          ) : (
            <View style={[styles.noteView, { backgroundColor: theme.surface }]}>
              {notes ? (
                <Text style={[styles.noteText, { color: theme.text }]}>{notes}</Text>
              ) : (
                <Text style={[styles.notePlaceholder, { color: theme.textTertiary }]}>
                  {t('entry.notes')}
                </Text>
              )}
            </View>
          )}
        </ScrollView>

        {/* Footer: word count + tags */}
        <View
          style={[styles.footer, { borderTopColor: theme.border, backgroundColor: theme.surface }]}
        >
          <Text style={[styles.wordCount, { color: theme.textTertiary }]}>
            {wordCount} {t('secureNote.words')} · {notes.length} {t('secureNote.chars')}
          </Text>
          {entry?.modifiedAt && (
            <Text style={[styles.modifiedAt, { color: theme.textTertiary }]}>
              {t('entry.sortByModified')}: {new Date(entry.modifiedAt).toLocaleDateString()}
            </Text>
          )}
        </View>
      </KeyboardAvoidingView>
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
  headerCenter: { flex: 1, paddingHorizontal: tokens.space.sm },
  headerTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.bold },
  titleInput: {
    fontSize: tokens.fontSize.lg,
    fontWeight: tokens.fontWeight.bold,
    borderBottomWidth: 1,
    paddingBottom: 2,
  },
  headerActions: { flexDirection: 'row', gap: tokens.space.md, alignItems: 'center' },
  headerBtn: { fontSize: tokens.fontSize.md },
  templates: { borderBottomWidth: StyleSheet.hairlineWidth, paddingVertical: tokens.space.sm },
  templatesLabel: {
    fontSize: tokens.fontSize.xs,
    paddingHorizontal: tokens.space.lg,
    marginBottom: 6,
    fontWeight: '600',
  },
  templatesList: { paddingHorizontal: tokens.space.lg, gap: tokens.space.sm },
  templateChip: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.xs,
    borderRadius: tokens.radius.full,
    borderWidth: 1,
  },
  templateIcon: { fontSize: 16 },
  templateLabel: { fontSize: tokens.fontSize.sm, fontWeight: '500' },
  content: { flex: 1 },
  contentInner: { padding: tokens.space.lg, flexGrow: 1 },
  noteInput: {
    flex: 1,
    minHeight: 400,
    fontSize: tokens.fontSize.md,
    lineHeight: 24,
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
    fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
  },
  noteView: {
    minHeight: 400,
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
  },
  noteText: {
    fontSize: tokens.fontSize.md,
    lineHeight: 24,
    fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
  },
  notePlaceholder: { fontSize: tokens.fontSize.md, fontStyle: 'italic' },
  footer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.sm,
    borderTopWidth: StyleSheet.hairlineWidth,
  },
  wordCount: { fontSize: 11 },
  modifiedAt: { fontSize: 11 },
});
