/**
 * Vault unlock screen — biometric + password (with full i18n EN/VI)
 */
import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  Alert,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import ReactNativeBiometrics from 'react-native-biometrics';
import { useVaultStore } from '../store/vault';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';

const rnBiometrics = new ReactNativeBiometrics();

export function UnlockScreen() {
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [biometricAvailable, setBiometricAvailable] = useState(false);
  const { unlockVault, meta } = useVaultStore();
  const { theme } = useThemeStore();
  const { t } = useTranslation();

  useEffect(() => {
    checkBiometrics();
  }, []);

  const checkBiometrics = async () => {
    try {
      const { available } = await rnBiometrics.isSensorAvailable();
      setBiometricAvailable(available);
      if (available) {
        // Auto-trigger biometric on mount
        handleBiometricUnlock();
      }
    } catch {
      setBiometricAvailable(false);
    }
  };

  const handleBiometricUnlock = async () => {
    try {
      const { success } = await rnBiometrics.simplePrompt({
        promptMessage: t('biometric.prompt'),
        cancelButtonText: t('biometric.fallback'),
      });

      if (success) {
        setLoading(true);
        // Retrieve stored master password from secure keychain
        const Keychain = require('react-native-keychain');
        const credentials = await Keychain.getGenericPassword({
          service: 'keepassex-vault',
        });

        if (credentials) {
          await unlockVault(credentials.password);
        }
      }
    } catch (e: any) {
      // Biometric failed — user can use password
    } finally {
      setLoading(false);
    }
  };

  const handlePasswordUnlock = async () => {
    if (!password.trim()) return;

    setLoading(true);
    try {
      await unlockVault(password);
    } catch (e: any) {
      Alert.alert(t('errors.wrongCredentials'), t('vault.wrongPassword'), [
        { text: t('common.ok') },
      ]);
    } finally {
      setLoading(false);
    }
  };

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <KeyboardAvoidingView
        behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
        style={styles.inner}
      >
        {/* Logo */}
        <View style={styles.hero}>
          <Text style={styles.logo}>🔐</Text>
          <Text style={[styles.title, { color: theme.text }]}>{t('app.name')}</Text>
          {meta && (
            <Text style={[styles.vaultName, { color: theme.textSecondary }]}>{meta.name}</Text>
          )}
        </View>

        {/* Biometric button */}
        {biometricAvailable && (
          <TouchableOpacity
            style={[styles.biometricButton, { borderColor: theme.primary }]}
            onPress={handleBiometricUnlock}
            disabled={loading}
            accessibilityLabel={t('biometric.prompt')}
            accessibilityRole="button"
          >
            <Text style={styles.biometricIcon}>{Platform.OS === 'ios' ? '👤' : '🔏'}</Text>
            <Text style={[styles.biometricText, { color: theme.primary }]}>
              {Platform.OS === 'ios' ? 'Face ID / Touch ID' : t('settings.biometricUnlock')}
            </Text>
          </TouchableOpacity>
        )}

        {/* Divider */}
        {biometricAvailable && (
          <View style={styles.divider}>
            <View style={[styles.dividerLine, { backgroundColor: theme.border }]} />
            <Text style={[styles.dividerText, { color: theme.textTertiary }]}>or</Text>
            <View style={[styles.dividerLine, { backgroundColor: theme.border }]} />
          </View>
        )}

        {/* Password input */}
        <View style={styles.passwordSection}>
          <TextInput
            style={[
              styles.passwordInput,
              {
                backgroundColor: theme.surface,
                borderColor: theme.border,
                color: theme.text,
              },
            ]}
            placeholder="Master Password"
            placeholderTextColor={theme.textTertiary}
            secureTextEntry
            value={password}
            onChangeText={setPassword}
            onSubmitEditing={handlePasswordUnlock}
            returnKeyType="go"
            autoFocus={!biometricAvailable}
            accessibilityLabel="Master password"
          />

          <TouchableOpacity
            style={[
              styles.unlockButton,
              { backgroundColor: theme.primary },
              loading && styles.unlockButtonDisabled,
            ]}
            onPress={handlePasswordUnlock}
            disabled={loading || !password.trim()}
            accessibilityLabel="Unlock vault"
            accessibilityRole="button"
          >
            <Text style={styles.unlockButtonText}>{loading ? '...' : 'Unlock'}</Text>
          </TouchableOpacity>
        </View>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  inner: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: tokens.space.xl,
    gap: tokens.space.xl,
  },
  hero: {
    alignItems: 'center',
    gap: tokens.space.sm,
  },
  logo: {
    fontSize: 64,
  },
  title: {
    fontSize: tokens.fontSize['2xl'],
    fontWeight: tokens.fontWeight.bold,
  },
  vaultName: {
    fontSize: tokens.fontSize.md,
  },
  biometricButton: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.sm,
    paddingHorizontal: tokens.space.xl,
    paddingVertical: tokens.space.md,
    borderRadius: tokens.radius.full,
    borderWidth: 2,
  },
  biometricIcon: {
    fontSize: 24,
  },
  biometricText: {
    fontSize: tokens.fontSize.md,
    fontWeight: tokens.fontWeight.semibold,
  },
  divider: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.md,
    width: '100%',
  },
  dividerLine: {
    flex: 1,
    height: 1,
  },
  dividerText: {
    fontSize: tokens.fontSize.sm,
  },
  passwordSection: {
    width: '100%',
    gap: tokens.space.md,
  },
  passwordInput: {
    borderWidth: 1,
    borderRadius: tokens.radius.md,
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.md,
    fontSize: tokens.fontSize.md,
    width: '100%',
  },
  unlockButton: {
    borderRadius: tokens.radius.md,
    paddingVertical: tokens.space.md,
    alignItems: 'center',
  },
  unlockButtonDisabled: {
    opacity: 0.5,
  },
  unlockButtonText: {
    color: 'white',
    fontSize: tokens.fontSize.md,
    fontWeight: tokens.fontWeight.semibold,
  },
});
