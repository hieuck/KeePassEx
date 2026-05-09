/**
 * Change master password screen
 * Missing from all competitors on mobile — KeePassEx exclusive
 */
import React, { useState } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  ScrollView,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import { NativeModules } from 'react-native';
import ReactNativeHapticFeedback from 'react-native-haptic-feedback';
import { useThemeStore } from '../store/theme';
import { useI18nStore } from '../store/i18n';
import { tokens } from '@keepassex/ui';

const { KeePassExCore } = NativeModules;

type StrengthLevel = 'weak' | 'fair' | 'good' | 'strong' | 'very-strong';

function getStrength(password: string): { level: StrengthLevel; score: number; label: string } {
  if (!password) return { level: 'weak', score: 0, label: '' };
  let score = 0;
  if (password.length >= 8) score++;
  if (password.length >= 12) score++;
  if (password.length >= 16) score++;
  if (/[A-Z]/.test(password)) score++;
  if (/[a-z]/.test(password)) score++;
  if (/[0-9]/.test(password)) score++;
  if (/[^A-Za-z0-9]/.test(password)) score++;

  if (score <= 2) return { level: 'weak', score, label: 'Weak' };
  if (score <= 3) return { level: 'fair', score, label: 'Fair' };
  if (score <= 4) return { level: 'good', score, label: 'Good' };
  if (score <= 5) return { level: 'strong', score, label: 'Strong' };
  return { level: 'very-strong', score, label: 'Very Strong' };
}

const STRENGTH_COLORS: Record<StrengthLevel, string> = {
  weak: '#EF4444',
  fair: '#F97316',
  good: '#EAB308',
  strong: '#22C55E',
  'very-strong': '#16A34A',
};

export function ChangePasswordScreen() {
  const navigation = useNavigation();
  const { theme } = useThemeStore();
  const { t } = useI18nStore();

  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showCurrent, setShowCurrent] = useState(false);
  const [showNew, setShowNew] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const strength = getStrength(newPassword);
  const passwordsMatch = newPassword === confirmPassword;
  const canSubmit =
    currentPassword.length > 0 && newPassword.length >= 8 && passwordsMatch && !loading;

  const handleChange = async () => {
    if (!canSubmit) return;
    setLoading(true);
    setError(null);

    try {
      await KeePassExCore.changeCredentials(currentPassword, newPassword);
      ReactNativeHapticFeedback.trigger('notificationSuccess');
      Alert.alert(t('vault.changePassword'), t('vault.passwordChanged'), [
        { text: t('common.ok'), onPress: () => navigation.goBack() },
      ]);
    } catch (e: any) {
      setError(e?.message ?? t('vault.wrongPassword'));
      ReactNativeHapticFeedback.trigger('notificationError');
    } finally {
      setLoading(false);
    }
  };

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Header */}
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <TouchableOpacity
          onPress={() => navigation.goBack()}
          accessibilityRole="button"
          accessibilityLabel={t('common.cancel')}
        >
          <Text style={[styles.cancelBtn, { color: theme.primary }]}>{t('common.cancel')}</Text>
        </TouchableOpacity>
        <Text style={[styles.headerTitle, { color: theme.text }]}>{t('vault.changePassword')}</Text>
        <TouchableOpacity
          onPress={handleChange}
          disabled={!canSubmit}
          accessibilityRole="button"
          accessibilityLabel={t('common.save')}
        >
          <Text style={[styles.saveBtn, { color: canSubmit ? theme.primary : theme.textTertiary }]}>
            {t('common.save')}
          </Text>
        </TouchableOpacity>
      </View>

      <ScrollView contentContainerStyle={styles.content} keyboardShouldPersistTaps="handled">
        {/* Security notice */}
        <View style={[styles.notice, { backgroundColor: '#EFF6FF', borderColor: '#BFDBFE' }]}>
          <Text style={styles.noticeIcon}>🔐</Text>
          <Text style={[styles.noticeText, { color: '#1E40AF' }]}>
            {t('vault.changePasswordNotice')}
          </Text>
        </View>

        {/* Current password */}
        <View style={styles.fieldGroup}>
          <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
            {t('vault.currentPassword')}
          </Text>
          <View
            style={[
              styles.inputRow,
              { borderColor: error ? '#EF4444' : theme.border, backgroundColor: theme.surface },
            ]}
          >
            <TextInput
              style={[styles.input, { color: theme.text }]}
              value={currentPassword}
              onChangeText={v => {
                setCurrentPassword(v);
                setError(null);
              }}
              secureTextEntry={!showCurrent}
              placeholder={t('vault.currentPassword')}
              placeholderTextColor={theme.textTertiary}
              autoCapitalize="none"
              autoCorrect={false}
              accessibilityLabel={t('vault.currentPassword')}
            />
            <TouchableOpacity onPress={() => setShowCurrent(v => !v)} accessibilityRole="button">
              <Text style={[styles.eyeBtn, { color: theme.textSecondary }]}>
                {showCurrent ? '🙈' : '👁'}
              </Text>
            </TouchableOpacity>
          </View>
          {error && <Text style={styles.errorText}>{error}</Text>}
        </View>

        {/* New password */}
        <View style={styles.fieldGroup}>
          <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
            {t('vault.newPassword')}
          </Text>
          <View
            style={[styles.inputRow, { borderColor: theme.border, backgroundColor: theme.surface }]}
          >
            <TextInput
              style={[styles.input, { color: theme.text }]}
              value={newPassword}
              onChangeText={setNewPassword}
              secureTextEntry={!showNew}
              placeholder={t('vault.newPassword')}
              placeholderTextColor={theme.textTertiary}
              autoCapitalize="none"
              autoCorrect={false}
              accessibilityLabel={t('vault.newPassword')}
            />
            <TouchableOpacity onPress={() => setShowNew(v => !v)} accessibilityRole="button">
              <Text style={[styles.eyeBtn, { color: theme.textSecondary }]}>
                {showNew ? '🙈' : '👁'}
              </Text>
            </TouchableOpacity>
          </View>

          {/* Strength bar */}
          {newPassword.length > 0 && (
            <View style={styles.strengthContainer}>
              <View style={styles.strengthBar}>
                {[1, 2, 3, 4, 5].map(i => (
                  <View
                    key={i}
                    style={[
                      styles.strengthSegment,
                      {
                        backgroundColor:
                          i <= Math.ceil(strength.score / 1.4)
                            ? STRENGTH_COLORS[strength.level]
                            : theme.border,
                      },
                    ]}
                  />
                ))}
              </View>
              <Text style={[styles.strengthLabel, { color: STRENGTH_COLORS[strength.level] }]}>
                {strength.label}
              </Text>
            </View>
          )}

          {newPassword.length > 0 && newPassword.length < 8 && (
            <Text style={styles.hintText}>{t('vault.passwordTooShort')}</Text>
          )}
        </View>

        {/* Confirm password */}
        <View style={styles.fieldGroup}>
          <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
            {t('vault.confirmPassword')}
          </Text>
          <View
            style={[
              styles.inputRow,
              {
                borderColor: confirmPassword && !passwordsMatch ? '#EF4444' : theme.border,
                backgroundColor: theme.surface,
              },
            ]}
          >
            <TextInput
              style={[styles.input, { color: theme.text }]}
              value={confirmPassword}
              onChangeText={setConfirmPassword}
              secureTextEntry={!showConfirm}
              placeholder={t('vault.confirmPassword')}
              placeholderTextColor={theme.textTertiary}
              autoCapitalize="none"
              autoCorrect={false}
              returnKeyType="done"
              onSubmitEditing={handleChange}
              accessibilityLabel={t('vault.confirmPassword')}
            />
            <TouchableOpacity onPress={() => setShowConfirm(v => !v)} accessibilityRole="button">
              <Text style={[styles.eyeBtn, { color: theme.textSecondary }]}>
                {showConfirm ? '🙈' : '👁'}
              </Text>
            </TouchableOpacity>
          </View>
          {confirmPassword.length > 0 && !passwordsMatch && (
            <Text style={styles.errorText}>{t('vault.passwordsDoNotMatch')}</Text>
          )}
          {confirmPassword.length > 0 && passwordsMatch && newPassword.length >= 8 && (
            <Text style={styles.successText}>✓ {t('vault.passwordsMatch')}</Text>
          )}
        </View>

        {/* Submit button */}
        <TouchableOpacity
          style={[styles.submitBtn, { backgroundColor: canSubmit ? theme.primary : theme.border }]}
          onPress={handleChange}
          disabled={!canSubmit}
          accessibilityRole="button"
          accessibilityLabel={t('vault.changePassword')}
        >
          {loading ? (
            <ActivityIndicator color="white" />
          ) : (
            <Text style={styles.submitBtnText}>{t('vault.changePassword')}</Text>
          )}
        </TouchableOpacity>
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
  cancelBtn: { fontSize: tokens.fontSize.md },
  headerTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.bold },
  saveBtn: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  content: { padding: tokens.space.xl, gap: tokens.space.xl },
  notice: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    gap: tokens.space.sm,
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
  },
  noticeIcon: { fontSize: 18, marginTop: 1 },
  noticeText: { flex: 1, fontSize: tokens.fontSize.sm, lineHeight: 20 },
  fieldGroup: { gap: tokens.space.sm },
  fieldLabel: {
    fontSize: tokens.fontSize.sm,
    fontWeight: tokens.fontWeight.medium,
    textTransform: 'uppercase',
    letterSpacing: 0.5,
  },
  inputRow: {
    flexDirection: 'row',
    alignItems: 'center',
    borderWidth: 1,
    borderRadius: tokens.radius.md,
    paddingHorizontal: tokens.space.md,
  },
  input: { flex: 1, fontSize: tokens.fontSize.md, paddingVertical: tokens.space.md },
  eyeBtn: { fontSize: 18, padding: tokens.space.xs },
  strengthContainer: { flexDirection: 'row', alignItems: 'center', gap: tokens.space.sm },
  strengthBar: { flex: 1, flexDirection: 'row', gap: 3 },
  strengthSegment: { flex: 1, height: 4, borderRadius: 2 },
  strengthLabel: {
    fontSize: tokens.fontSize.xs,
    fontWeight: tokens.fontWeight.semibold,
    minWidth: 60,
    textAlign: 'right',
  },
  hintText: { fontSize: tokens.fontSize.xs, color: '#F97316' },
  errorText: { fontSize: tokens.fontSize.xs, color: '#EF4444' },
  successText: { fontSize: tokens.fontSize.xs, color: '#16A34A' },
  submitBtn: {
    paddingVertical: tokens.space.lg,
    borderRadius: tokens.radius.md,
    alignItems: 'center',
    marginTop: tokens.space.md,
  },
  submitBtnText: {
    color: 'white',
    fontSize: tokens.fontSize.md,
    fontWeight: tokens.fontWeight.bold,
  },
});
