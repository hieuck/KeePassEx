/**
 * Entry detail screen (with full i18n EN/VI)
 */
import React, { useState, useEffect } from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, Alert } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation, useRoute } from '@react-navigation/native';
import type { NativeStackNavigationProp, RouteProp } from '@react-navigation/native-stack';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import Clipboard from '@react-native-clipboard/clipboard';
import ReactNativeHapticFeedback from 'react-native-haptic-feedback';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import { OtpCountdown } from '../components/OtpCountdown';
import { TagList } from '../components/TagList';
import type { RootStackParamList } from '../App';

const { KeePassExCore } = NativeModules;
type Nav = NativeStackNavigationProp<RootStackParamList>;
type Route = RouteProp<RootStackParamList, 'EntryDetail'>;

export function EntryDetailScreen() {
  const navigation = useNavigation<Nav>();
  const route = useRoute<Route>();
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { uuid } = route.params;

  const [showPassword, setShowPassword] = useState(false);
  const [password, setPassword] = useState('');
  const [copiedField, setCopiedField] = useState<string | null>(null);
  const [otpCode, setOtpCode] = useState<{ code: string; remaining: number } | null>(null);

  const { data: entry } = useQuery({
    queryKey: ['entry', uuid],
    queryFn: () => KeePassExCore.getEntry(uuid, false),
  });

  useEffect(() => {
    if (!entry?.hasOtp) return;
    const refresh = () => {
      KeePassExCore.generateTotp(uuid)
        .then((r: { code: string; remainingSeconds: number }) =>
          setOtpCode({ code: r.code, remaining: r.remainingSeconds })
        )
        .catch(() => {});
    };
    refresh();
    const interval = setInterval(refresh, 1000);
    return () => clearInterval(interval);
  }, [entry?.hasOtp, uuid]);

  const copyField = async (value: string, field: string) => {
    Clipboard.setString(value);
    ReactNativeHapticFeedback.trigger('impactLight');
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 2000);
    setTimeout(() => Clipboard.setString(''), 10_000);
  };

  const revealPassword = async () => {
    const pw = await KeePassExCore.getEntryPassword(uuid);
    setPassword(pw);
    setShowPassword(true);
  };

  const deleteMutation = useMutation({
    mutationFn: () => KeePassExCore.deleteEntry(uuid, false),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      navigation.goBack();
    },
  });

  const handleDelete = () => {
    Alert.alert(t('entry.delete'), t('entry.confirmDelete', { title: entry?.title ?? '' }), [
      { text: t('common.cancel'), style: 'cancel' },
      { text: t('common.delete'), style: 'destructive', onPress: () => deleteMutation.mutate() },
    ]);
  };

  if (!entry) return null;

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <TouchableOpacity
          onPress={() => navigation.goBack()}
          accessibilityRole="button"
          accessibilityLabel={t('common.back')}
        >
          <Text style={[styles.backButton, { color: theme.primary }]}>← {t('common.back')}</Text>
        </TouchableOpacity>
        <Text style={[styles.headerTitle, { color: theme.text }]} numberOfLines={1}>
          {entry.title}
        </Text>
        <TouchableOpacity
          onPress={() => navigation.navigate('EntryEdit', { uuid })}
          accessibilityRole="button"
          accessibilityLabel={t('entry.edit')}
        >
          <Text style={[styles.editButton, { color: theme.primary }]}>{t('common.edit')}</Text>
        </TouchableOpacity>
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.contentInner}>
        {entry.username && (
          <FieldCard
            label={t('entry.username')}
            value={entry.username}
            theme={theme}
            onCopy={() => copyField(entry.username, 'username')}
            copied={copiedField === 'username'}
          />
        )}

        <View
          style={[styles.fieldCard, { backgroundColor: theme.surface, borderColor: theme.border }]}
        >
          <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
            {t('entry.password')}
          </Text>
          <View style={styles.fieldRow}>
            <Text style={[styles.fieldValue, { color: theme.text }]}>
              {showPassword ? password : '••••••••••••'}
            </Text>
            <View style={styles.fieldActions}>
              <TouchableOpacity
                onPress={showPassword ? () => setShowPassword(false) : revealPassword}
                style={styles.actionButton}
                accessibilityRole="button"
                accessibilityLabel={
                  showPassword ? t('entry.hidePassword') : t('entry.showPassword')
                }
              >
                <Text style={{ fontSize: 18 }}>{showPassword ? '🙈' : '👁'}</Text>
              </TouchableOpacity>
              <TouchableOpacity
                onPress={() => copyField(showPassword ? password : '', 'password')}
                style={styles.actionButton}
                accessibilityRole="button"
                accessibilityLabel={t('entry.copyPassword')}
              >
                <Text
                  style={{
                    fontSize: 18,
                    color: copiedField === 'password' ? tokens.color.success : theme.textTertiary,
                  }}
                >
                  {copiedField === 'password' ? '✓' : '⎘'}
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>

        {entry.url && (
          <FieldCard
            label={t('entry.url')}
            value={entry.url}
            theme={theme}
            onCopy={() => copyField(entry.url, 'url')}
            copied={copiedField === 'url'}
          />
        )}

        {entry.hasOtp && otpCode && (
          <View
            style={[
              styles.fieldCard,
              { backgroundColor: theme.surface, borderColor: theme.border },
            ]}
          >
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('entry.otp')}
            </Text>
            <OtpCountdown
              code={otpCode.code}
              remainingSeconds={otpCode.remaining}
              period={30}
              onCopy={() => {
                Clipboard.setString(otpCode.code);
                ReactNativeHapticFeedback.trigger('impactLight');
                setCopiedField('otp');
                setTimeout(() => setCopiedField(null), 2000);
              }}
            />
            {copiedField === 'otp' && (
              <Text style={{ fontSize: tokens.fontSize.xs, color: tokens.color.success }}>
                ✓ {t('common.copied')}
              </Text>
            )}
          </View>
        )}

        {entry.notes && (
          <View
            style={[
              styles.fieldCard,
              { backgroundColor: theme.surface, borderColor: theme.border },
            ]}
          >
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('entry.notes')}
            </Text>
            <Text style={[styles.notesValue, { color: theme.text }]}>{entry.notes}</Text>
          </View>
        )}

        {entry.tags?.length > 0 && (
          <View
            style={[
              styles.fieldCard,
              { backgroundColor: theme.surface, borderColor: theme.border },
            ]}
          >
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('entry.tags')}
            </Text>
            <TagList tags={entry.tags} theme={theme} />
          </View>
        )}

        <TouchableOpacity
          style={[styles.deleteButton, { borderColor: tokens.color.danger }]}
          onPress={handleDelete}
          accessibilityRole="button"
          accessibilityLabel={t('entry.delete')}
        >
          <Text style={[styles.deleteButtonText, { color: tokens.color.danger }]}>
            🗑 {t('entry.delete')}
          </Text>
        </TouchableOpacity>
      </ScrollView>
    </SafeAreaView>
  );
}

const { KeePassExCore } = NativeModules;
type Nav = NativeStackNavigationProp<RootStackParamList>;
type Route = RouteProp<RootStackParamList, 'EntryDetail'>;

export function EntryDetailScreen() {
  const navigation = useNavigation<Nav>();
  const route = useRoute<Route>();
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { uuid } = route.params;

  const [showPassword, setShowPassword] = useState(false);
  const [password, setPassword] = useState('');
  const [copiedField, setCopiedField] = useState<string | null>(null);
  const [otpCode, setOtpCode] = useState<{ code: string; remaining: number } | null>(null);

  const { data: entry } = useQuery({
    queryKey: ['entry', uuid],
    queryFn: () => KeePassExCore.getEntry(uuid, false),
  });

  // OTP refresh
  useEffect(() => {
    if (!entry?.hasOtp) return;
    const refresh = () => {
      KeePassExCore.generateTotp(uuid)
        .then((r: { code: string; remainingSeconds: number }) =>
          setOtpCode({ code: r.code, remaining: r.remainingSeconds })
        )
        .catch(() => {});
    };
    refresh();
    const interval = setInterval(refresh, 1000);
    return () => clearInterval(interval);
  }, [entry?.hasOtp, uuid]);

  const copyField = async (value: string, field: string) => {
    Clipboard.setString(value);
    ReactNativeHapticFeedback.trigger('impactLight');
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 2000);
    setTimeout(() => Clipboard.setString(''), 10_000);
  };

  const revealPassword = async () => {
    const pw = await KeePassExCore.getEntryPassword(uuid);
    setPassword(pw);
    setShowPassword(true);
  };

  const deleteMutation = useMutation({
    mutationFn: () => KeePassExCore.deleteEntry(uuid, false),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      navigation.goBack();
    },
  });

  const handleDelete = () => {
    Alert.alert(t('entry.delete'), t('entry.confirmDelete', { title: entry?.title ?? '' }), [
      { text: t('common.cancel'), style: 'cancel' },
      { text: t('common.delete'), style: 'destructive', onPress: () => deleteMutation.mutate() },
    ]);
  };

  if (!entry) return null;

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Header */}
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <TouchableOpacity
          onPress={() => navigation.goBack()}
          accessibilityRole="button"
          accessibilityLabel="Back"
        >
          <Text style={[styles.backButton, { color: theme.primary }]}>← Back</Text>
        </TouchableOpacity>
        <Text style={[styles.headerTitle, { color: theme.text }]} numberOfLines={1}>
          {entry.title}
        </Text>
        <TouchableOpacity
          onPress={() => navigation.navigate('EntryEdit', { uuid })}
          accessibilityRole="button"
          accessibilityLabel="Edit entry"
        >
          <Text style={[styles.editButton, { color: theme.primary }]}>Edit</Text>
        </TouchableOpacity>
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.contentInner}>
        {/* Username */}
        {entry.username && (
          <FieldCard
            label="Username"
            value={entry.username}
            theme={theme}
            onCopy={() => copyField(entry.username, 'username')}
            copied={copiedField === 'username'}
          />
        )}

        {/* Password */}
        <View
          style={[styles.fieldCard, { backgroundColor: theme.surface, borderColor: theme.border }]}
        >
          <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>Password</Text>
          <View style={styles.fieldRow}>
            <Text style={[styles.fieldValue, { color: theme.text }]}>
              {showPassword ? password : '••••••••••••'}
            </Text>
            <View style={styles.fieldActions}>
              <TouchableOpacity
                onPress={showPassword ? () => setShowPassword(false) : revealPassword}
                style={styles.actionButton}
                accessibilityRole="button"
                accessibilityLabel={showPassword ? 'Hide password' : 'Show password'}
              >
                <Text style={{ fontSize: 18 }}>{showPassword ? '🙈' : '👁'}</Text>
              </TouchableOpacity>
              <TouchableOpacity
                onPress={() => copyField(showPassword ? password : '', 'password')}
                style={styles.actionButton}
                accessibilityRole="button"
                accessibilityLabel="Copy password"
              >
                <Text
                  style={{
                    fontSize: 18,
                    color: copiedField === 'password' ? tokens.color.success : theme.textTertiary,
                  }}
                >
                  {copiedField === 'password' ? '✓' : '⎘'}
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>

        {/* URL */}
        {entry.url && (
          <FieldCard
            label="URL"
            value={entry.url}
            theme={theme}
            onCopy={() => copyField(entry.url, 'url')}
            copied={copiedField === 'url'}
          />
        )}

        {/* OTP */}
        {entry.hasOtp && otpCode && (
          <View
            style={[
              styles.fieldCard,
              { backgroundColor: theme.surface, borderColor: theme.border },
            ]}
          >
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              One-Time Password
            </Text>
            <OtpCountdown
              code={otpCode.code}
              remainingSeconds={otpCode.remaining}
              period={30}
              onCopy={() => {
                Clipboard.setString(otpCode.code);
                ReactNativeHapticFeedback.trigger('impactLight');
                setCopiedField('otp');
                setTimeout(() => setCopiedField(null), 2000);
              }}
            />
            {copiedField === 'otp' && (
              <Text style={{ fontSize: tokens.fontSize.xs, color: tokens.color.success }}>
                ✓ Copied!
              </Text>
            )}
          </View>
        )}

        {/* Notes */}
        {entry.notes && (
          <View
            style={[
              styles.fieldCard,
              { backgroundColor: theme.surface, borderColor: theme.border },
            ]}
          >
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>Notes</Text>
            <Text style={[styles.notesValue, { color: theme.text }]}>{entry.notes}</Text>
          </View>
        )}

        {/* Tags */}
        {entry.tags?.length > 0 && (
          <View
            style={[
              styles.fieldCard,
              { backgroundColor: theme.surface, borderColor: theme.border },
            ]}
          >
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>Tags</Text>
            <TagList tags={entry.tags} theme={theme} />
          </View>
        )}

        {/* Delete */}
        <TouchableOpacity
          style={[styles.deleteButton, { borderColor: tokens.color.danger }]}
          onPress={handleDelete}
          accessibilityRole="button"
          accessibilityLabel="Delete entry"
        >
          <Text style={[styles.deleteButtonText, { color: tokens.color.danger }]}>
            🗑 Delete Entry
          </Text>
        </TouchableOpacity>
      </ScrollView>
    </SafeAreaView>
  );
}

function FieldCard({
  label,
  value,
  theme,
  onCopy,
  copied,
}: {
  label: string;
  value: string;
  theme: ReturnType<typeof useThemeStore>['theme'];
  onCopy: () => void;
  copied: boolean;
}) {
  return (
    <View style={[styles.fieldCard, { backgroundColor: theme.surface, borderColor: theme.border }]}>
      <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>{label}</Text>
      <View style={styles.fieldRow}>
        <Text style={[styles.fieldValue, { color: theme.text }]} numberOfLines={2} selectable>
          {value}
        </Text>
        <TouchableOpacity
          onPress={onCopy}
          style={styles.actionButton}
          accessibilityRole="button"
          accessibilityLabel={`Copy ${label}`}
        >
          <Text style={{ fontSize: 18, color: copied ? tokens.color.success : theme.textTertiary }}>
            {copied ? '✓' : '⎘'}
          </Text>
        </TouchableOpacity>
      </View>
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
    gap: tokens.space.md,
  },
  backButton: { fontSize: tokens.fontSize.md },
  headerTitle: {
    flex: 1,
    fontSize: tokens.fontSize.lg,
    fontWeight: tokens.fontWeight.semibold,
    textAlign: 'center',
  },
  editButton: { fontSize: tokens.fontSize.md },
  content: { flex: 1 },
  contentInner: {
    padding: tokens.space.lg,
    gap: tokens.space.md,
  },
  fieldCard: {
    borderRadius: tokens.radius.md,
    borderWidth: StyleSheet.hairlineWidth,
    padding: tokens.space.md,
    gap: tokens.space.xs,
  },
  fieldLabel: {
    fontSize: tokens.fontSize.xs,
    fontWeight: tokens.fontWeight.semibold,
    textTransform: 'uppercase',
    letterSpacing: 0.8,
  },
  fieldRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.sm,
  },
  fieldValue: {
    flex: 1,
    fontSize: tokens.fontSize.md,
    fontFamily: 'Menlo',
  },
  fieldActions: {
    flexDirection: 'row',
    gap: tokens.space.xs,
  },
  actionButton: {
    padding: tokens.space.xs,
  },
  otpCode: {
    flex: 1,
    fontSize: tokens.fontSize['2xl'],
    fontWeight: tokens.fontWeight.bold,
    fontFamily: 'Menlo',
    letterSpacing: 4,
  },
  otpTimer: {
    fontSize: tokens.fontSize.sm,
    fontWeight: tokens.fontWeight.semibold,
    minWidth: 28,
    textAlign: 'right',
  },
  notesValue: {
    fontSize: tokens.fontSize.md,
    lineHeight: 22,
  },
  tagsRow: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: tokens.space.xs,
  },
  tag: {
    paddingHorizontal: tokens.space.sm,
    paddingVertical: 3,
    borderRadius: tokens.radius.full,
  },
  tagText: { fontSize: tokens.fontSize.xs },
  deleteButton: {
    borderWidth: 1,
    borderRadius: tokens.radius.md,
    paddingVertical: tokens.space.md,
    alignItems: 'center',
    marginTop: tokens.space.lg,
  },
  deleteButtonText: {
    fontSize: tokens.fontSize.md,
    fontWeight: tokens.fontWeight.medium,
  },
});
