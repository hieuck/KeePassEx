/**
 * Welcome screen — open or create vault, full i18n (10 languages)
 * Uses proper modal dialogs instead of Alert.prompt (not available on Android)
 */
import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  Alert,
  Modal,
  KeyboardAvoidingView,
  Platform,
  ScrollView,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import DocumentPicker from 'react-native-document-picker';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { useVaultStore } from '../store/vault';
import { useThemeStore } from '../store/theme';
import { useI18nStore, useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import type { RootStackParamList } from '../App';

type Nav = NativeStackNavigationProp<RootStackParamList>;

export function WelcomeScreen() {
  const navigation = useNavigation<Nav>();
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const { init: initI18n } = useI18nStore();
  const { createVault } = useVaultStore();

  const [loading, setLoading] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [createForm, setCreateForm] = useState({
    name: '',
    password: '',
    confirmPassword: '',
  });
  const [createError, setCreateError] = useState<string | null>(null);

  useEffect(() => {
    initI18n();
  }, []);

  // ─── Open existing vault ────────────────────────────────────────────────

  const handleOpenVault = async () => {
    try {
      const result = await DocumentPicker.pickSingle({
        type: [DocumentPicker.types.allFiles],
        copyTo: 'cachesDirectory',
      });
      if (!result.fileCopyUri) return;
      navigation.navigate('Unlock', { vaultPath: result.fileCopyUri });
    } catch (e) {
      if (!DocumentPicker.isCancel(e)) {
        Alert.alert(t('common.error'), t('errors.generic'));
      }
    }
  };

  // ─── Create new vault ───────────────────────────────────────────────────

  const handleCreateSubmit = async () => {
    setCreateError(null);

    if (!createForm.name.trim()) {
      setCreateError(t('vault.name') + ' ' + t('common.required').toLowerCase());
      return;
    }
    if (!createForm.password) {
      setCreateError(t('vault.masterPassword') + ' ' + t('common.required').toLowerCase());
      return;
    }
    if (createForm.password !== createForm.confirmPassword) {
      setCreateError(t('vault.passwordsDoNotMatch'));
      return;
    }
    if (createForm.password.length < 8) {
      setCreateError(t('vault.passwordTooShort'));
      return;
    }

    setLoading(true);
    try {
      const RNFS = require('react-native-fs');
      const path = `${RNFS.DocumentDirectoryPath}/${createForm.name.trim()}.kdbx`;
      await createVault(path, createForm.name.trim(), createForm.password);
      setShowCreateModal(false);
      setCreateForm({ name: '', password: '', confirmPassword: '' });
    } catch (err: unknown) {
      setCreateError(err instanceof Error ? err.message : t('errors.generic'));
    } finally {
      setLoading(false);
    }
  };

  const handleCloseCreateModal = () => {
    setShowCreateModal(false);
    setCreateForm({ name: '', password: '', confirmPassword: '' });
    setCreateError(null);
  };

  // ─── Feature highlights ─────────────────────────────────────────────────

  const features = [
    { icon: '🔒', key: 'hardwareKey.title' },
    { icon: '📱', key: 'sync.title' },
    { icon: '🔑', key: 'passkey.title' },
    { icon: '🛡️', key: 'breach.title' },
    { icon: '🔄', key: 'emergencyAccess.title' },
    { icon: '🌐', key: 'browserExtension.fill' },
  ] as const;

  const inputStyle = [
    styles.input,
    { backgroundColor: theme.surface, borderColor: theme.border, color: theme.text },
  ];

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Create Vault Modal */}
      <Modal
        visible={showCreateModal}
        animationType="slide"
        presentationStyle="pageSheet"
        onRequestClose={handleCloseCreateModal}
      >
        <SafeAreaView style={[styles.modalContainer, { backgroundColor: theme.background }]}>
          <KeyboardAvoidingView
            behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
            style={{ flex: 1 }}
          >
            {/* Modal header */}
            <View style={[styles.modalHeader, { borderBottomColor: theme.border }]}>
              <TouchableOpacity
                onPress={handleCloseCreateModal}
                accessibilityRole="button"
                accessibilityLabel={t('common.cancel')}
              >
                <Text style={[styles.modalAction, { color: theme.primary }]}>
                  {t('common.cancel')}
                </Text>
              </TouchableOpacity>
              <Text style={[styles.modalTitle, { color: theme.text }]}>{t('vault.create')}</Text>
              <TouchableOpacity
                onPress={handleCreateSubmit}
                disabled={loading}
                accessibilityRole="button"
                accessibilityLabel={t('common.create')}
              >
                <Text
                  style={[
                    styles.modalAction,
                    styles.modalActionBold,
                    { color: theme.primary },
                    loading && styles.disabled,
                  ]}
                >
                  {loading ? t('common.loading') : t('common.create')}
                </Text>
              </TouchableOpacity>
            </View>

            <ScrollView
              style={styles.modalContent}
              contentContainerStyle={styles.modalContentInner}
              keyboardShouldPersistTaps="handled"
            >
              {/* Vault name */}
              <View style={styles.formGroup}>
                <Text style={[styles.formLabel, { color: theme.textSecondary }]}>
                  {t('vault.name')} *
                </Text>
                <TextInput
                  style={inputStyle}
                  value={createForm.name}
                  onChangeText={v => setCreateForm(f => ({ ...f, name: v }))}
                  placeholder={t('vault.name')}
                  placeholderTextColor={theme.textTertiary}
                  autoFocus
                  returnKeyType="next"
                  accessibilityLabel={t('vault.name')}
                />
              </View>

              {/* Master password */}
              <View style={styles.formGroup}>
                <Text style={[styles.formLabel, { color: theme.textSecondary }]}>
                  {t('vault.masterPassword')} *
                </Text>
                <TextInput
                  style={inputStyle}
                  value={createForm.password}
                  onChangeText={v => setCreateForm(f => ({ ...f, password: v }))}
                  placeholder={t('vault.masterPassword')}
                  placeholderTextColor={theme.textTertiary}
                  secureTextEntry
                  autoCapitalize="none"
                  autoCorrect={false}
                  returnKeyType="next"
                  accessibilityLabel={t('vault.masterPassword')}
                />
              </View>

              {/* Confirm password */}
              <View style={styles.formGroup}>
                <Text style={[styles.formLabel, { color: theme.textSecondary }]}>
                  {t('vault.confirmPassword')} *
                </Text>
                <TextInput
                  style={inputStyle}
                  value={createForm.confirmPassword}
                  onChangeText={v => setCreateForm(f => ({ ...f, confirmPassword: v }))}
                  placeholder={t('vault.confirmPassword')}
                  placeholderTextColor={theme.textTertiary}
                  secureTextEntry
                  autoCapitalize="none"
                  autoCorrect={false}
                  returnKeyType="done"
                  onSubmitEditing={handleCreateSubmit}
                  accessibilityLabel={t('vault.confirmPassword')}
                />
              </View>

              {/* Error */}
              {createError && (
                <View
                  style={[
                    styles.errorBanner,
                    { backgroundColor: '#FEF2F2', borderColor: '#FECACA' },
                  ]}
                >
                  <Text style={styles.errorText}>⚠️ {createError}</Text>
                </View>
              )}

              {/* Hint */}
              <Text style={[styles.hint, { color: theme.textTertiary }]}>
                {t('vault.passwordTooShort').replace('8', '12+')}
              </Text>
            </ScrollView>
          </KeyboardAvoidingView>
        </SafeAreaView>
      </Modal>

      {/* Main content */}
      <ScrollView contentContainerStyle={styles.scrollContent} showsVerticalScrollIndicator={false}>
        {/* Hero */}
        <View style={styles.hero}>
          <Text style={styles.logo} accessibilityHidden>
            🔐
          </Text>
          <Text style={[styles.title, { color: theme.text }]}>{t('app.name')}</Text>
          <Text style={[styles.tagline, { color: theme.textSecondary }]}>{t('app.tagline')}</Text>
        </View>

        {/* Actions */}
        <View style={styles.actions}>
          <TouchableOpacity
            style={[styles.primaryButton, { backgroundColor: theme.primary }]}
            onPress={handleOpenVault}
            disabled={loading}
            accessibilityRole="button"
            accessibilityLabel={t('vault.open')}
          >
            <Text style={styles.primaryButtonText} accessibilityHidden>
              📂{' '}
            </Text>
            <Text style={styles.primaryButtonText}>{t('vault.open')}</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[
              styles.secondaryButton,
              { borderColor: theme.border, backgroundColor: theme.surface },
            ]}
            onPress={() => setShowCreateModal(true)}
            disabled={loading}
            accessibilityRole="button"
            accessibilityLabel={t('vault.create')}
          >
            <Text style={[styles.secondaryButtonText, { color: theme.text }]} accessibilityHidden>
              ✨{' '}
            </Text>
            <Text style={[styles.secondaryButtonText, { color: theme.text }]}>
              {t('vault.create')}
            </Text>
          </TouchableOpacity>
        </View>

        {/* Feature highlights */}
        <View style={styles.features}>
          {features.map(({ icon, key }) => (
            <View key={key} style={styles.featureRow}>
              <Text style={styles.featureIcon} accessibilityHidden>
                {icon}
              </Text>
              <Text style={[styles.featureText, { color: theme.textSecondary }]}>{t(key)}</Text>
            </View>
          ))}
        </View>

        {/* Version */}
        <Text style={[styles.version, { color: theme.textTertiary }]}>
          {t('app.version', { version: '1.0.0' })}
        </Text>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  scrollContent: {
    flexGrow: 1,
    padding: tokens.space.xl,
    justifyContent: 'space-between',
    gap: tokens.space['2xl'],
  },
  hero: { alignItems: 'center', gap: tokens.space.sm, paddingTop: tokens.space['2xl'] },
  logo: { fontSize: 72 },
  title: { fontSize: tokens.fontSize['3xl'], fontWeight: tokens.fontWeight.bold },
  tagline: { fontSize: tokens.fontSize.md, textAlign: 'center' },
  actions: { gap: tokens.space.md },
  primaryButton: {
    borderRadius: tokens.radius.md,
    paddingVertical: tokens.space.md,
    alignItems: 'center',
    flexDirection: 'row',
    justifyContent: 'center',
  },
  primaryButtonText: {
    color: 'white',
    fontSize: tokens.fontSize.lg,
    fontWeight: tokens.fontWeight.semibold,
  },
  secondaryButton: {
    borderRadius: tokens.radius.md,
    paddingVertical: tokens.space.md,
    alignItems: 'center',
    borderWidth: 1,
    flexDirection: 'row',
    justifyContent: 'center',
  },
  secondaryButtonText: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.medium },
  features: { gap: tokens.space.sm },
  featureRow: { flexDirection: 'row', alignItems: 'center', gap: tokens.space.sm },
  featureIcon: { fontSize: 18, width: 24 },
  featureText: { fontSize: tokens.fontSize.sm, flex: 1 },
  version: { fontSize: tokens.fontSize.xs, textAlign: 'center', paddingBottom: tokens.space.md },
  // Modal
  modalContainer: { flex: 1 },
  modalHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.md,
    borderBottomWidth: StyleSheet.hairlineWidth,
  },
  modalTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.semibold },
  modalAction: { fontSize: tokens.fontSize.md },
  modalActionBold: { fontWeight: tokens.fontWeight.semibold },
  modalContent: { flex: 1 },
  modalContentInner: { padding: tokens.space.lg, gap: tokens.space.lg },
  formGroup: { gap: tokens.space.xs },
  formLabel: {
    fontSize: 11,
    fontWeight: '700',
    textTransform: 'uppercase',
    letterSpacing: 0.8,
  },
  input: {
    borderWidth: 1,
    borderRadius: tokens.radius.md,
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.sm,
    fontSize: tokens.fontSize.md,
  },
  errorBanner: {
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
  },
  errorText: { fontSize: tokens.fontSize.sm, color: '#DC2626' },
  hint: { fontSize: tokens.fontSize.xs, lineHeight: 18 },
  disabled: { opacity: 0.4 },
});
