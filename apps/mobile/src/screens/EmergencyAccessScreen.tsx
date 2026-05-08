/**
 * Emergency Access screen — mobile (with full i18n EN/VI)
 */
import React, { useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  TextInput,
  StyleSheet,
  Alert,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import type { EmergencyAccess, EmergencyAccessLevel } from '@keepassex/types';

const { KeePassExCore } = NativeModules;

const STATUS_COLORS: Record<string, string> = {
  invited: '#D97706',
  confirmed: '#16A34A',
  recovery_initiated: '#DC2626',
  recovery_approved: '#D97706',
  recovery_granted: '#DC2626',
  revoked: '#9CA3AF',
};

const STATUS_LABELS: Record<string, string> = {
  invited: 'Invited',
  confirmed: 'Confirmed',
  recovery_initiated: 'Request pending',
  recovery_approved: 'Waiting period',
  recovery_granted: 'Access granted',
  revoked: 'Revoked',
};

export function EmergencyAccessScreen() {
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const navigation = useNavigation();
  const queryClient = useQueryClient();

  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState({
    name: '',
    email: '',
    accessLevel: 'view' as EmergencyAccessLevel,
    waitDays: 7,
  });

  const { data: grants = [] } = useQuery<EmergencyAccess[]>({
    queryKey: ['emergency-access'],
    queryFn: () => Promise.resolve([]),
  });

  const addMutation = useMutation({
    mutationFn: () => Promise.resolve(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['emergency-access'] });
      setShowForm(false);
      setForm({ name: '', email: '', accessLevel: 'view', waitDays: 7 });
      Alert.alert('✅', t('emergencyAccess.sendInvitation'));
    },
  });

  const revokeMutation = useMutation({
    mutationFn: (id: string) => Promise.resolve(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['emergency-access'] }),
  });

  const inputStyle = [
    styles.input,
    { backgroundColor: theme.surface, borderColor: theme.border, color: theme.text },
  ];

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <TouchableOpacity
          onPress={() => navigation.goBack()}
          accessibilityRole="button"
          accessibilityLabel={t('common.back')}
        >
          <Text style={[styles.backBtn, { color: theme.primary }]}>← {t('common.back')}</Text>
        </TouchableOpacity>
        <Text style={[styles.headerTitle, { color: theme.text }]}>
          🆘 {t('emergencyAccess.title')}
        </Text>
        <TouchableOpacity
          onPress={() => setShowForm(true)}
          accessibilityRole="button"
          accessibilityLabel={t('emergencyAccess.addContact')}
        >
          <Text style={[styles.addBtn, { color: theme.primary }]}>+ {t('common.add')}</Text>
        </TouchableOpacity>
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.contentInner}>
        <View
          style={[styles.infoCard, { backgroundColor: theme.surface, borderColor: theme.border }]}
        >
          <Text style={[styles.infoTitle, { color: theme.text }]}>
            {t('emergencyAccess.howItWorks')}
          </Text>
          <Text style={[styles.infoDesc, { color: theme.textSecondary }]}>
            {t('emergencyAccess.step1')} {t('emergencyAccess.step4')}
          </Text>
        </View>

        {showForm && (
          <View
            style={[styles.formCard, { backgroundColor: theme.surface, borderColor: theme.border }]}
          >
            <Text style={[styles.formTitle, { color: theme.text }]}>
              {t('emergencyAccess.addContact')}
            </Text>

            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('emergencyAccess.granteeName').toUpperCase()}
            </Text>
            <TextInput
              style={inputStyle}
              value={form.name}
              onChangeText={v => setForm(f => ({ ...f, name: v }))}
              placeholder="John Doe"
              placeholderTextColor={theme.textTertiary}
              accessibilityLabel={t('emergencyAccess.granteeName')}
            />

            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('emergencyAccess.granteeEmail').toUpperCase()}
            </Text>
            <TextInput
              style={inputStyle}
              value={form.email}
              onChangeText={v => setForm(f => ({ ...f, email: v }))}
              placeholder="contact@example.com"
              placeholderTextColor={theme.textTertiary}
              keyboardType="email-address"
              autoCapitalize="none"
              accessibilityLabel={t('emergencyAccess.granteeEmail')}
            />

            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('emergencyAccess.accessLevel').toUpperCase()}
            </Text>
            <View style={styles.levelRow}>
              {(['view', 'takeover'] as EmergencyAccessLevel[]).map(level => (
                <TouchableOpacity
                  key={level}
                  style={[
                    styles.levelBtn,
                    { borderColor: theme.border },
                    form.accessLevel === level && {
                      backgroundColor: theme.primary,
                      borderColor: theme.primary,
                    },
                  ]}
                  onPress={() => setForm(f => ({ ...f, accessLevel: level }))}
                  accessibilityRole="radio"
                  accessibilityState={{ checked: form.accessLevel === level }}
                >
                  <Text
                    style={[
                      styles.levelBtnText,
                      { color: form.accessLevel === level ? 'white' : theme.text },
                    ]}
                  >
                    {level === 'view'
                      ? t('emergencyAccess.accessLevelView')
                      : t('emergencyAccess.accessLevelTakeover')}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>

            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('emergencyAccess.waitPeriod').toUpperCase()}
            </Text>
            <View style={styles.waitRow}>
              {[1, 3, 7, 14, 30].map(days => (
                <TouchableOpacity
                  key={days}
                  style={[
                    styles.waitBtn,
                    { borderColor: theme.border },
                    form.waitDays === days && {
                      backgroundColor: theme.primary,
                      borderColor: theme.primary,
                    },
                  ]}
                  onPress={() => setForm(f => ({ ...f, waitDays: days }))}
                  accessibilityRole="radio"
                  accessibilityState={{ checked: form.waitDays === days }}
                >
                  <Text
                    style={[
                      styles.waitBtnText,
                      { color: form.waitDays === days ? 'white' : theme.text },
                    ]}
                  >
                    {days}d
                  </Text>
                </TouchableOpacity>
              ))}
            </View>

            <View style={styles.formActions}>
              <TouchableOpacity
                style={[styles.sendBtn, { backgroundColor: theme.primary }]}
                onPress={() => addMutation.mutate()}
                disabled={!form.name.trim() || !form.email.trim() || addMutation.isPending}
                accessibilityRole="button"
              >
                <Text style={styles.sendBtnText}>
                  {addMutation.isPending
                    ? t('common.loading')
                    : `📧 ${t('emergencyAccess.sendInvitation')}`}
                </Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[styles.cancelBtn, { borderColor: theme.border }]}
                onPress={() => setShowForm(false)}
                accessibilityRole="button"
              >
                <Text style={[styles.cancelBtnText, { color: theme.text }]}>
                  {t('common.cancel')}
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        )}

        {/* Grants list */}
        {grants.length === 0 && !showForm ? (
          <View style={styles.emptyState}>
            <Text style={styles.emptyIcon} accessibilityHidden>
              🆘
            </Text>
            <Text style={[styles.emptyTitle, { color: theme.text }]}>
              {t('emergencyAccess.noContacts')}
            </Text>
            <Text style={[styles.emptyDesc, { color: theme.textSecondary }]}>
              {t('emergencyAccess.noContactsDesc')}
            </Text>
          </View>
        ) : (
          grants.map(grant => (
            <View
              key={grant.id}
              style={[
                styles.grantCard,
                { backgroundColor: theme.surface, borderColor: theme.border },
              ]}
            >
              <View style={styles.grantHeader}>
                <View>
                  <Text style={[styles.grantName, { color: theme.text }]}>{grant.granteeName}</Text>
                  <Text style={[styles.grantEmail, { color: theme.textSecondary }]}>
                    {grant.granteeEmail}
                  </Text>
                </View>
                <Text
                  style={[styles.grantStatus, { color: STATUS_COLORS[grant.status] ?? '#9CA3AF' }]}
                >
                  {STATUS_LABELS[grant.status] ?? grant.status}
                </Text>
              </View>

              <View style={styles.grantMeta}>
                <Text style={[styles.grantMetaText, { color: theme.textSecondary }]}>
                  {grant.accessLevel === 'view'
                    ? t('emergencyAccess.accessLevelView')
                    : t('emergencyAccess.accessLevelTakeover')}{' '}
                  · {grant.waitTimeDays} {t('emergencyAccess.waitPeriod').toLowerCase()}
                </Text>
                {grant.daysRemaining !== undefined && (
                  <Text style={[styles.grantMetaText, { color: tokens.color.warning }]}>
                    ⏳ {t('emergencyAccess.daysRemaining', { days: grant.daysRemaining })}
                  </Text>
                )}
              </View>

              {grant.status !== 'revoked' && (
                <TouchableOpacity
                  style={[styles.revokeBtn, { borderColor: tokens.color.danger }]}
                  onPress={() =>
                    Alert.alert(
                      t('emergencyAccess.revoke'),
                      t('emergencyAccess.confirmRevoke', { name: grant.granteeName }),
                      [
                        { text: t('common.cancel'), style: 'cancel' },
                        {
                          text: t('emergencyAccess.revoke'),
                          style: 'destructive',
                          onPress: () => revokeMutation.mutate(grant.id),
                        },
                      ]
                    )
                  }
                  accessibilityRole="button"
                  accessibilityLabel={t('emergencyAccess.revoke')}
                >
                  <Text style={[styles.revokeBtnText, { color: tokens.color.danger }]}>
                    {t('emergencyAccess.revoke')}
                  </Text>
                </TouchableOpacity>
              )}
            </View>
          ))
        )}
      </ScrollView>
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
  headerTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.semibold },
  addBtn: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  content: { flex: 1 },
  contentInner: { padding: tokens.space.lg, gap: tokens.space.md },
  infoCard: {
    borderRadius: tokens.radius.lg,
    borderWidth: StyleSheet.hairlineWidth,
    padding: tokens.space.lg,
    gap: tokens.space.sm,
  },
  infoTitle: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  infoDesc: { fontSize: tokens.fontSize.sm, lineHeight: 20 },
  formCard: {
    borderRadius: tokens.radius.lg,
    borderWidth: StyleSheet.hairlineWidth,
    padding: tokens.space.lg,
    gap: tokens.space.sm,
  },
  formTitle: {
    fontSize: tokens.fontSize.md,
    fontWeight: tokens.fontWeight.semibold,
    marginBottom: tokens.space.sm,
  },
  fieldLabel: { fontSize: 11, fontWeight: '600', letterSpacing: 0.8, marginTop: tokens.space.sm },
  input: {
    borderWidth: 1,
    borderRadius: tokens.radius.md,
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.sm,
    fontSize: tokens.fontSize.md,
  },
  levelRow: { flexDirection: 'row', gap: tokens.space.sm },
  levelBtn: {
    flex: 1,
    paddingVertical: tokens.space.sm,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
    alignItems: 'center',
  },
  levelBtnText: { fontSize: tokens.fontSize.sm, fontWeight: tokens.fontWeight.medium },
  waitRow: { flexDirection: 'row', gap: tokens.space.xs },
  waitBtn: {
    flex: 1,
    paddingVertical: tokens.space.sm,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
    alignItems: 'center',
  },
  waitBtnText: { fontSize: tokens.fontSize.sm, fontWeight: tokens.fontWeight.medium },
  formActions: { flexDirection: 'row', gap: tokens.space.sm, marginTop: tokens.space.sm },
  sendBtn: {
    flex: 1,
    borderRadius: tokens.radius.md,
    paddingVertical: tokens.space.md,
    alignItems: 'center',
  },
  sendBtnText: {
    color: 'white',
    fontWeight: tokens.fontWeight.semibold,
    fontSize: tokens.fontSize.md,
  },
  cancelBtn: {
    flex: 1,
    borderRadius: tokens.radius.md,
    paddingVertical: tokens.space.md,
    alignItems: 'center',
    borderWidth: 1,
  },
  cancelBtnText: { fontSize: tokens.fontSize.md },
  emptyState: { alignItems: 'center', paddingVertical: tokens.space['2xl'], gap: tokens.space.md },
  emptyIcon: { fontSize: 48 },
  emptyTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.semibold },
  emptyDesc: { fontSize: tokens.fontSize.sm, textAlign: 'center', lineHeight: 20 },
  grantCard: {
    borderRadius: tokens.radius.lg,
    borderWidth: StyleSheet.hairlineWidth,
    padding: tokens.space.md,
    gap: tokens.space.sm,
  },
  grantHeader: { flexDirection: 'row', alignItems: 'flex-start', justifyContent: 'space-between' },
  grantName: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  grantEmail: { fontSize: tokens.fontSize.sm, marginTop: 2 },
  grantStatus: { fontSize: tokens.fontSize.xs, fontWeight: '600' },
  grantMeta: { gap: 2 },
  grantMetaText: { fontSize: tokens.fontSize.xs },
  revokeBtn: {
    borderWidth: 1,
    borderRadius: tokens.radius.sm,
    paddingVertical: tokens.space.xs,
    paddingHorizontal: tokens.space.md,
    alignSelf: 'flex-start',
  },
  revokeBtnText: { fontSize: tokens.fontSize.sm, fontWeight: tokens.fontWeight.medium },
});
