/**
 * Entry detail screen — full-featured with i18n, passkey, SSH key, custom fields,
 * attachments, history, and password advisor integration.
 */
import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  Alert,
  Linking,
  Share,
} from 'react-native';
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

interface CustomFieldDto {
  key: string;
  value: string;
  protected: boolean;
}

interface EntryDetailDto {
  uuid: string;
  title: string;
  username: string;
  url: string;
  notes: string;
  iconId: number;
  tags: string[];
  hasPassword: boolean;
  hasOtp: boolean;
  hasPasskey: boolean;
  hasSshKey: boolean;
  hasAttachments: boolean;
  isExpired: boolean;
  expiry?: string;
  createdAt: string;
  modifiedAt: string;
  customFields: CustomFieldDto[];
  groupName?: string;
}

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
  const [revealedCustomFields, setRevealedCustomFields] = useState<Set<string>>(new Set());

  const { data: entry, isLoading } = useQuery<EntryDetailDto>({
    queryKey: ['entry', uuid],
    queryFn: () => KeePassExCore.getEntry(uuid, false),
  });

  // OTP refresh every second
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

  const copyToClipboard = useCallback((value: string, fieldKey: string) => {
    if (!value) return;
    Clipboard.setString(value);
    ReactNativeHapticFeedback.trigger('impactLight');
    setCopiedField(fieldKey);
    setTimeout(() => setCopiedField(null), 2000);
    // Auto-clear clipboard after 10 seconds
    setTimeout(() => Clipboard.setString(''), 10_000);
  }, []);

  const revealPassword = useCallback(async () => {
    try {
      const pw = await KeePassExCore.getEntryPassword(uuid);
      setPassword(pw);
      setShowPassword(true);
    } catch {
      Alert.alert(t('errors.generic'));
    }
  }, [uuid, t]);

  const toggleRevealCustomField = useCallback((key: string) => {
    setRevealedCustomFields(prev => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }, []);

  const deleteMutation = useMutation({
    mutationFn: () => KeePassExCore.deleteEntry(uuid, false),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      navigation.goBack();
    },
  });

  const handleDelete = useCallback(() => {
    Alert.alert(t('entry.delete'), t('entry.confirmDelete', { title: entry?.title ?? '' }), [
      { text: t('common.cancel'), style: 'cancel' },
      {
        text: t('common.delete'),
        style: 'destructive',
        onPress: () => deleteMutation.mutate(),
      },
    ]);
  }, [entry?.title, deleteMutation, t]);

  const handleOpenUrl = useCallback(() => {
    if (!entry?.url) return;
    const url = entry.url.startsWith('http') ? entry.url : `https://${entry.url}`;
    Linking.openURL(url).catch(() => Alert.alert(t('errors.generic')));
  }, [entry?.url, t]);

  const formatDate = (iso: string) => {
    try {
      return new Date(iso).toLocaleDateString(undefined, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return iso;
    }
  };

  if (isLoading || !entry) {
    return (
      <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
        <View style={styles.loadingContainer}>
          <Text style={[styles.loadingText, { color: theme.textSecondary }]}>
            {t('common.loading')}
          </Text>
        </View>
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Header */}
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <TouchableOpacity
          onPress={() => navigation.goBack()}
          accessibilityRole="button"
          accessibilityLabel={t('common.back')}
          hitSlop={{ top: 8, bottom: 8, left: 8, right: 8 }}
        >
          <Text style={[styles.headerAction, { color: theme.primary }]}>← {t('common.back')}</Text>
        </TouchableOpacity>

        <View style={styles.headerCenter}>
          <Text style={[styles.headerTitle, { color: theme.text }]} numberOfLines={1}>
            {entry.title}
          </Text>
          {entry.groupName && (
            <Text style={[styles.headerSubtitle, { color: theme.textTertiary }]} numberOfLines={1}>
              {entry.groupName}
            </Text>
          )}
        </View>

        <TouchableOpacity
          onPress={() => navigation.navigate('EntryEdit', { uuid })}
          accessibilityRole="button"
          accessibilityLabel={t('entry.edit')}
          hitSlop={{ top: 8, bottom: 8, left: 8, right: 8 }}
        >
          <Text style={[styles.headerAction, { color: theme.primary }]}>{t('common.edit')}</Text>
        </TouchableOpacity>
      </View>

      {/* Quick action bar */}
      <View
        style={[
          styles.actionBar,
          { borderBottomColor: theme.border, backgroundColor: theme.surface },
        ]}
      >
        <TouchableOpacity
          style={styles.actionBarBtn}
          onPress={() =>
            navigation.navigate('EntryHistory', { entryUuid: uuid, entryTitle: entry.title })
          }
          accessibilityRole="button"
          accessibilityLabel={t('entry.history')}
        >
          <Text style={styles.actionBarIcon}>📋</Text>
          <Text style={[styles.actionBarLabel, { color: theme.textSecondary }]}>
            {t('entry.history')}
          </Text>
        </TouchableOpacity>
        {entry.hasOtp && (
          <TouchableOpacity
            style={styles.actionBarBtn}
            onPress={() => navigation.navigate('OtpDetail', { entryUuid: uuid })}
            accessibilityRole="button"
            accessibilityLabel={t('otp.title')}
          >
            <Text style={styles.actionBarIcon}>⏱</Text>
            <Text style={[styles.actionBarLabel, { color: theme.textSecondary }]}>
              {t('otp.title')}
            </Text>
          </TouchableOpacity>
        )}
        {entry.url ? (
          <TouchableOpacity
            style={styles.actionBarBtn}
            onPress={handleOpenUrl}
            accessibilityRole="button"
            accessibilityLabel={t('entry.openUrl')}
          >
            <Text style={styles.actionBarIcon}>🔗</Text>
            <Text style={[styles.actionBarLabel, { color: theme.textSecondary }]}>
              {t('entry.openUrl')}
            </Text>
          </TouchableOpacity>
        ) : null}
        <TouchableOpacity
          style={styles.actionBarBtn}
          onPress={() => {
            KeePassExCore.getEntryPassword(uuid)
              .then((pw: string) => copyToClipboard(pw, 'password'))
              .catch(() => {});
          }}
          accessibilityRole="button"
          accessibilityLabel={t('entry.copyPassword')}
        >
          <Text style={styles.actionBarIcon}>{copiedField === 'password' ? '✓' : '⎘'}</Text>
          <Text
            style={[
              styles.actionBarLabel,
              { color: copiedField === 'password' ? '#16A34A' : theme.textSecondary },
            ]}
          >
            {t('entry.copyPassword')}
          </Text>
        </TouchableOpacity>
      </View>

      <ScrollView
        style={styles.scroll}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
      >
        {/* Expiry warning */}
        {entry.isExpired && (
          <View
            style={[styles.warningBanner, { backgroundColor: '#FEF2F2', borderColor: '#FECACA' }]}
          >
            <Text style={styles.warningText}>⚠️ {t('entry.expired')}</Text>
          </View>
        )}

        {/* Username */}
        {entry.username ? (
          <FieldCard
            label={t('entry.username')}
            value={entry.username}
            theme={theme}
            onCopy={() => copyToClipboard(entry.username, 'username')}
            copied={copiedField === 'username'}
            monospace={false}
          />
        ) : null}

        {/* Password */}
        {entry.hasPassword && (
          <View
            style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}
          >
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('entry.password')}
            </Text>
            <View style={styles.fieldRow}>
              <Text
                style={[styles.fieldValueMono, { color: theme.text }]}
                numberOfLines={showPassword ? undefined : 1}
              >
                {showPassword ? password : '••••••••••••'}
              </Text>
              <View style={styles.fieldActions}>
                <TouchableOpacity
                  onPress={
                    showPassword
                      ? () => {
                          setShowPassword(false);
                          setPassword('');
                        }
                      : revealPassword
                  }
                  style={styles.iconBtn}
                  accessibilityRole="button"
                  accessibilityLabel={
                    showPassword ? t('entry.hidePassword') : t('entry.showPassword')
                  }
                >
                  <Text style={styles.iconBtnText}>{showPassword ? '🙈' : '👁'}</Text>
                </TouchableOpacity>
                <TouchableOpacity
                  onPress={() => {
                    if (showPassword && password) {
                      copyToClipboard(password, 'password');
                    } else {
                      // Fetch and copy without revealing
                      KeePassExCore.getEntryPassword(uuid)
                        .then((pw: string) => copyToClipboard(pw, 'password'))
                        .catch(() => {});
                    }
                  }}
                  style={styles.iconBtn}
                  accessibilityRole="button"
                  accessibilityLabel={t('entry.copyPassword')}
                >
                  <Text
                    style={[
                      styles.iconBtnText,
                      {
                        color:
                          copiedField === 'password' ? tokens.color.success : theme.textTertiary,
                      },
                    ]}
                  >
                    {copiedField === 'password' ? '✓' : '⎘'}
                  </Text>
                </TouchableOpacity>
              </View>
            </View>
            {copiedField === 'password' && (
              <Text style={[styles.copiedHint, { color: tokens.color.success }]}>
                ✓ {t('common.copiedToClipboard')} · {t('common.clearingIn', { seconds: 10 })}
              </Text>
            )}
          </View>
        )}

        {/* URL */}
        {entry.url ? (
          <View
            style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}
          >
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('entry.url')}
            </Text>
            <View style={styles.fieldRow}>
              <Text
                style={[styles.fieldValue, { color: theme.primary, flex: 1 }]}
                numberOfLines={1}
                onPress={handleOpenUrl}
                accessibilityRole="link"
                accessibilityLabel={`${t('entry.openUrl')}: ${entry.url}`}
              >
                {entry.url}
              </Text>
              <View style={styles.fieldActions}>
                <TouchableOpacity
                  onPress={handleOpenUrl}
                  style={styles.iconBtn}
                  accessibilityRole="button"
                  accessibilityLabel={t('entry.openUrl')}
                >
                  <Text style={styles.iconBtnText}>🔗</Text>
                </TouchableOpacity>
                <TouchableOpacity
                  onPress={() => copyToClipboard(entry.url, 'url')}
                  style={styles.iconBtn}
                  accessibilityRole="button"
                  accessibilityLabel={t('entry.copyUrl')}
                >
                  <Text
                    style={[
                      styles.iconBtnText,
                      { color: copiedField === 'url' ? tokens.color.success : theme.textTertiary },
                    ]}
                  >
                    {copiedField === 'url' ? '✓' : '⎘'}
                  </Text>
                </TouchableOpacity>
              </View>
            </View>
          </View>
        ) : null}

        {/* OTP */}
        {entry.hasOtp && otpCode && (
          <View
            style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}
          >
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('entry.otp')}
            </Text>
            <OtpCountdown
              code={otpCode.code}
              remainingSeconds={otpCode.remaining}
              period={30}
              onCopy={() => copyToClipboard(otpCode.code, 'otp')}
            />
            {copiedField === 'otp' && (
              <Text style={[styles.copiedHint, { color: tokens.color.success }]}>
                ✓ {t('common.copied')}
              </Text>
            )}
          </View>
        )}

        {/* Custom fields */}
        {entry.customFields.length > 0 && (
          <View
            style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}
          >
            <Text style={[styles.sectionTitle, { color: theme.text }]}>
              {t('entry.customFields')}
            </Text>
            {entry.customFields.map(field => {
              const isRevealed = revealedCustomFields.has(field.key);
              const displayValue = field.protected && !isRevealed ? '••••••••' : field.value;
              return (
                <View key={field.key} style={styles.customFieldRow}>
                  <View style={styles.customFieldInfo}>
                    <Text style={[styles.customFieldKey, { color: theme.textSecondary }]}>
                      {field.key}
                    </Text>
                    <Text
                      style={[styles.customFieldValue, { color: theme.text }]}
                      numberOfLines={field.protected && !isRevealed ? 1 : undefined}
                    >
                      {displayValue}
                    </Text>
                  </View>
                  <View style={styles.fieldActions}>
                    {field.protected && (
                      <TouchableOpacity
                        onPress={() => toggleRevealCustomField(field.key)}
                        style={styles.iconBtn}
                        accessibilityRole="button"
                        accessibilityLabel={
                          isRevealed ? t('entry.hidePassword') : t('entry.showPassword')
                        }
                      >
                        <Text style={styles.iconBtnText}>{isRevealed ? '🙈' : '👁'}</Text>
                      </TouchableOpacity>
                    )}
                    <TouchableOpacity
                      onPress={() => copyToClipboard(field.value, `cf_${field.key}`)}
                      style={styles.iconBtn}
                      accessibilityRole="button"
                      accessibilityLabel={`${t('common.copy')} ${field.key}`}
                    >
                      <Text
                        style={[
                          styles.iconBtnText,
                          {
                            color:
                              copiedField === `cf_${field.key}`
                                ? tokens.color.success
                                : theme.textTertiary,
                          },
                        ]}
                      >
                        {copiedField === `cf_${field.key}` ? '✓' : '⎘'}
                      </Text>
                    </TouchableOpacity>
                  </View>
                </View>
              );
            })}
          </View>
        )}

        {/* Notes */}
        {entry.notes ? (
          <View
            style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}
          >
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('entry.notes')}
            </Text>
            <Text style={[styles.notesText, { color: theme.text }]} selectable>
              {entry.notes}
            </Text>
          </View>
        ) : null}

        {/* Tags */}
        {entry.tags.length > 0 && (
          <View
            style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}
          >
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('entry.tags')}
            </Text>
            <TagList tags={entry.tags} theme={theme} />
          </View>
        )}

        {/* Feature badges */}
        {(entry.hasPasskey || entry.hasSshKey || entry.hasAttachments) && (
          <View style={styles.badgeRow}>
            {entry.hasPasskey && (
              <View style={[styles.featureBadge, { backgroundColor: theme.backgroundSecondary }]}>
                <Text style={[styles.featureBadgeText, { color: theme.textSecondary }]}>
                  🔑 {t('passkey.title')}
                </Text>
              </View>
            )}
            {entry.hasSshKey && (
              <View style={[styles.featureBadge, { backgroundColor: theme.backgroundSecondary }]}>
                <Text style={[styles.featureBadgeText, { color: theme.textSecondary }]}>
                  🖥 {t('ssh.title')}
                </Text>
              </View>
            )}
            {entry.hasAttachments && (
              <TouchableOpacity
                style={[
                  styles.featureBadge,
                  { backgroundColor: '#EFF6FF', borderWidth: 1, borderColor: '#BFDBFE' },
                ]}
                onPress={() =>
                  navigation.navigate('AttachmentViewer', {
                    entryUuid: uuid,
                    entryTitle: entry.title,
                  })
                }
                accessibilityRole="button"
                accessibilityLabel={t('entry.attachments')}
              >
                <Text style={[styles.featureBadgeText, { color: '#2563EB' }]}>
                  📎 {t('entry.attachments')} →
                </Text>
              </TouchableOpacity>
            )}
          </View>
        )}

        {/* Metadata */}
        <View style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}>
          <Text style={[styles.sectionTitle, { color: theme.text }]}>{t('statistics.title')}</Text>
          <MetaRow
            label={t('entry.expiry')}
            value={entry.expiry ? formatDate(entry.expiry) : t('entry.neverExpires')}
            theme={theme}
            danger={entry.isExpired}
          />
          <MetaRow
            label={t('auditLog.events.entry_modified')}
            value={formatDate(entry.modifiedAt)}
            theme={theme}
          />
          <MetaRow
            label={t('auditLog.events.entry_created')}
            value={formatDate(entry.createdAt)}
            theme={theme}
          />
        </View>

        {/* Delete */}
        <TouchableOpacity
          style={[styles.deleteBtn, { borderColor: tokens.color.danger }]}
          onPress={handleDelete}
          accessibilityRole="button"
          accessibilityLabel={t('entry.delete')}
        >
          <Text style={[styles.deleteBtnText, { color: tokens.color.danger }]}>
            🗑 {t('entry.delete')}
          </Text>
        </TouchableOpacity>
      </ScrollView>
    </SafeAreaView>
  );
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function FieldCard({
  label,
  value,
  theme,
  onCopy,
  copied,
  monospace = true,
}: {
  label: string;
  value: string;
  theme: ReturnType<typeof useThemeStore>['theme'];
  onCopy: () => void;
  copied: boolean;
  monospace?: boolean;
}) {
  return (
    <View style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}>
      <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>{label}</Text>
      <View style={styles.fieldRow}>
        <Text
          style={[
            monospace ? styles.fieldValueMono : styles.fieldValue,
            { color: theme.text, flex: 1 },
          ]}
          numberOfLines={2}
          selectable
        >
          {value}
        </Text>
        <TouchableOpacity
          onPress={onCopy}
          style={styles.iconBtn}
          accessibilityRole="button"
          accessibilityLabel={`Copy ${label}`}
        >
          <Text
            style={[
              styles.iconBtnText,
              { color: copied ? tokens.color.success : theme.textTertiary },
            ]}
          >
            {copied ? '✓' : '⎘'}
          </Text>
        </TouchableOpacity>
      </View>
    </View>
  );
}

function MetaRow({
  label,
  value,
  theme,
  danger = false,
}: {
  label: string;
  value: string;
  theme: ReturnType<typeof useThemeStore>['theme'];
  danger?: boolean;
}) {
  return (
    <View style={styles.metaRow}>
      <Text style={[styles.metaLabel, { color: theme.textTertiary }]}>{label}</Text>
      <Text
        style={[styles.metaValue, { color: danger ? tokens.color.danger : theme.textSecondary }]}
      >
        {value}
      </Text>
    </View>
  );
}

// ─── Styles ───────────────────────────────────────────────────────────────────

const styles = StyleSheet.create({
  container: { flex: 1 },
  loadingContainer: { flex: 1, alignItems: 'center', justifyContent: 'center' },
  loadingText: { fontSize: tokens.fontSize.md },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.md,
    borderBottomWidth: StyleSheet.hairlineWidth,
    gap: tokens.space.sm,
  },
  headerAction: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.medium },
  headerCenter: { flex: 1, alignItems: 'center' },
  headerTitle: {
    fontSize: tokens.fontSize.lg,
    fontWeight: tokens.fontWeight.semibold,
    textAlign: 'center',
  },
  headerSubtitle: { fontSize: tokens.fontSize.xs, textAlign: 'center', marginTop: 1 },
  scroll: { flex: 1 },
  scrollContent: { padding: tokens.space.lg, gap: tokens.space.md, paddingBottom: 40 },
  warningBanner: {
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
    marginBottom: tokens.space.xs,
  },
  warningText: { fontSize: tokens.fontSize.sm, color: '#DC2626', fontWeight: '600' },
  card: {
    borderRadius: tokens.radius.md,
    borderWidth: StyleSheet.hairlineWidth,
    padding: tokens.space.md,
    gap: tokens.space.sm,
  },
  fieldLabel: {
    fontSize: 11,
    fontWeight: '700',
    textTransform: 'uppercase',
    letterSpacing: 0.8,
  },
  fieldRow: { flexDirection: 'row', alignItems: 'center', gap: tokens.space.sm },
  fieldValue: { fontSize: tokens.fontSize.md },
  fieldValueMono: { fontSize: tokens.fontSize.md, fontFamily: 'Menlo' },
  fieldActions: { flexDirection: 'row', gap: 2 },
  iconBtn: { padding: tokens.space.xs, minWidth: 32, alignItems: 'center' },
  iconBtnText: { fontSize: 18 },
  copiedHint: { fontSize: 11, marginTop: 2 },
  sectionTitle: { fontSize: tokens.fontSize.sm, fontWeight: tokens.fontWeight.semibold },
  customFieldRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.sm,
    paddingTop: tokens.space.xs,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: 'rgba(0,0,0,0.06)',
  },
  customFieldInfo: { flex: 1 },
  customFieldKey: {
    fontSize: 11,
    fontWeight: '600',
    textTransform: 'uppercase',
    letterSpacing: 0.5,
  },
  customFieldValue: { fontSize: tokens.fontSize.sm, fontFamily: 'Menlo', marginTop: 2 },
  notesText: { fontSize: tokens.fontSize.md, lineHeight: 22 },
  badgeRow: { flexDirection: 'row', flexWrap: 'wrap', gap: tokens.space.sm },
  featureBadge: {
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.xs,
    borderRadius: tokens.radius.full,
  },
  featureBadgeText: { fontSize: tokens.fontSize.xs, fontWeight: '600' },
  metaRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 3,
  },
  metaLabel: { fontSize: tokens.fontSize.xs },
  metaValue: { fontSize: tokens.fontSize.xs, fontWeight: '500' },
  deleteBtn: {
    borderWidth: 1,
    borderRadius: tokens.radius.md,
    paddingVertical: tokens.space.md,
    alignItems: 'center',
    marginTop: tokens.space.md,
  },
  deleteBtnText: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.medium },
  actionBar: {
    flexDirection: 'row',
    borderBottomWidth: StyleSheet.hairlineWidth,
    paddingVertical: tokens.space.sm,
  },
  actionBarBtn: {
    flex: 1,
    alignItems: 'center',
    gap: 3,
    paddingVertical: tokens.space.xs,
  },
  actionBarIcon: { fontSize: 20 },
  actionBarLabel: { fontSize: 10, fontWeight: '500' },
});
