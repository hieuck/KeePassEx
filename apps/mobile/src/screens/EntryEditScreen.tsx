/**
 * Entry edit/create screen (with full i18n EN/VI)
 */
import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  TextInput,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation, useRoute } from '@react-navigation/native';
import type { NativeStackNavigationProp, RouteProp } from '@react-navigation/native-stack';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import { PasswordField } from '../components/PasswordField';
import type { RootStackParamList } from '../App';

const { KeePassExCore } = NativeModules;
type Nav = NativeStackNavigationProp<RootStackParamList>;
type Route = RouteProp<RootStackParamList, 'EntryEdit'>;

export function EntryEditScreen() {
  const navigation = useNavigation<Nav>();
  const route = useRoute<Route>();
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { uuid, groupUuid } = route.params ?? {};
  const isNew = !uuid;

  const [form, setForm] = useState({
    title: '',
    username: '',
    password: '',
    url: '',
    notes: '',
  });
  const [showPassword, setShowPassword] = useState(false);

  const { data: entry } = useQuery({
    queryKey: ['entry', uuid],
    queryFn: () => KeePassExCore.getEntry(uuid, true),
    enabled: !!uuid,
  });

  useEffect(() => {
    if (entry) {
      setForm({
        title: entry.title ?? '',
        username: entry.username ?? '',
        password: entry.password ?? '',
        url: entry.url ?? '',
        notes: entry.notes ?? '',
      });
    }
  }, [entry]);

  const saveMutation = useMutation({
    mutationFn: async () => {
      if (isNew) {
        await KeePassExCore.createEntry({
          group_uuid: groupUuid ?? '00000000-0000-0000-0000-000000000000',
          ...form,
          tags: [],
          icon_id: 0,
        });
      } else {
        await KeePassExCore.updateEntry({
          uuid,
          ...form,
          tags: entry?.tags ?? [],
          icon_id: entry?.iconId ?? 0,
        });
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      if (uuid) queryClient.invalidateQueries({ queryKey: ['entry', uuid] });
      navigation.goBack();
    },
  });

  const handleGeneratePassword = () => {
    navigation.navigate('Generator', {
      onSelect: (pw: string) => setForm(f => ({ ...f, password: pw })),
    });
  };

  const inputStyle = [
    styles.input,
    { backgroundColor: theme.surface, borderColor: theme.border, color: theme.text },
  ];

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <KeyboardAvoidingView
        behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
        style={{ flex: 1 }}
      >
        {/* Header */}
        <View style={[styles.header, { borderBottomColor: theme.border }]}>
          <TouchableOpacity
            onPress={() => navigation.goBack()}
            accessibilityRole="button"
            accessibilityLabel={t('common.cancel')}
          >
            <Text style={[styles.cancelButton, { color: theme.primary }]}>
              {t('common.cancel')}
            </Text>
          </TouchableOpacity>
          <Text style={[styles.headerTitle, { color: theme.text }]}>
            {isNew ? t('entry.new') : t('entry.edit')}
          </Text>
          <TouchableOpacity
            onPress={() => saveMutation.mutate()}
            disabled={saveMutation.isPending || !form.title.trim()}
            accessibilityRole="button"
            accessibilityLabel={t('common.save')}
          >
            <Text
              style={[
                styles.saveButton,
                { color: theme.primary },
                (saveMutation.isPending || !form.title.trim()) && styles.saveButtonDisabled,
              ]}
            >
              {saveMutation.isPending ? '...' : t('common.save')}
            </Text>
          </TouchableOpacity>
        </View>

        <ScrollView style={styles.content} keyboardShouldPersistTaps="handled">
          <View style={styles.form}>
            <FormField label={t('entry.title')} required>
              <TextInput
                style={inputStyle}
                value={form.title}
                onChangeText={v => setForm(f => ({ ...f, title: v }))}
                placeholder={t('entry.title')}
                placeholderTextColor={theme.textTertiary}
                autoFocus={isNew}
                returnKeyType="next"
                accessibilityLabel="Title"
              />
            </FormField>

            <FormField label={t('entry.username')}>
              <TextInput
                style={inputStyle}
                value={form.username}
                onChangeText={v => setForm(f => ({ ...f, username: v }))}
                placeholder={t('entry.username')}
                placeholderTextColor={theme.textTertiary}
                autoCapitalize="none"
                autoCorrect={false}
                keyboardType="email-address"
                returnKeyType="next"
                accessibilityLabel={t('entry.username')}
              />
            </FormField>

            <FormField label={t('entry.password')}>
              <PasswordField
                value={form.password}
                onChangeText={v => setForm(f => ({ ...f, password: v }))}
                placeholder={t('entry.password')}
                onGeneratePress={() =>
                  navigation.navigate('Generator', {
                    onSelect: (pw: string) => setForm(f => ({ ...f, password: pw })),
                  })
                }
                theme={theme}
                accessibilityLabel={t('entry.password')}
              />
            </FormField>

            <FormField label={t('entry.url')}>
              <TextInput
                style={inputStyle}
                value={form.url}
                onChangeText={v => setForm(f => ({ ...f, url: v }))}
                placeholder="https://"
                placeholderTextColor={theme.textTertiary}
                autoCapitalize="none"
                autoCorrect={false}
                keyboardType="url"
                returnKeyType="next"
                accessibilityLabel={t('entry.url')}
              />
            </FormField>

            <FormField label={t('entry.notes')}>
              <TextInput
                style={[inputStyle, styles.notesInput]}
                value={form.notes}
                onChangeText={v => setForm(f => ({ ...f, notes: v }))}
                placeholder={t('entry.notes')}
                placeholderTextColor={theme.textTertiary}
                multiline
                numberOfLines={4}
                textAlignVertical="top"
                accessibilityLabel={t('entry.notes')}
              />
            </FormField>
          </View>
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}

function FormField({
  label,
  required,
  children,
}: {
  label: string;
  required?: boolean;
  children: React.ReactNode;
}) {
  return (
    <View style={styles.formField}>
      <Text style={styles.formLabel}>
        {label}
        {required && <Text style={{ color: tokens.color.danger }}> *</Text>}
      </Text>
      {children}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.md,
    borderBottomWidth: StyleSheet.hairlineWidth,
  },
  cancelButton: { fontSize: tokens.fontSize.md },
  headerTitle: {
    flex: 1,
    fontSize: tokens.fontSize.lg,
    fontWeight: tokens.fontWeight.semibold,
    textAlign: 'center',
  },
  saveButton: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  saveButtonDisabled: { opacity: 0.4 },
  content: { flex: 1 },
  form: {
    padding: tokens.space.lg,
    gap: tokens.space.lg,
  },
  formField: { gap: tokens.space.xs },
  formLabel: {
    fontSize: tokens.fontSize.sm,
    fontWeight: tokens.fontWeight.medium,
    color: '#6B7280',
    textTransform: 'uppercase',
    letterSpacing: 0.5,
  },
  input: {
    borderWidth: 1,
    borderRadius: tokens.radius.md,
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.sm,
    fontSize: tokens.fontSize.md,
  },
  passwordRow: {
    flexDirection: 'row',
    gap: tokens.space.sm,
    alignItems: 'center',
  },
  passwordToggle: {
    padding: tokens.space.xs,
  },
  generateButton: {
    width: 40,
    height: 40,
    borderRadius: tokens.radius.md,
    alignItems: 'center',
    justifyContent: 'center',
  },
  generateButtonText: { fontSize: 18 },
  notesInput: {
    minHeight: 100,
    paddingTop: tokens.space.sm,
  },
});
