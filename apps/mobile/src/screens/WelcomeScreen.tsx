/**
 * Welcome screen — open or create vault (with full i18n EN/VI)
 */
import React, { useState, useEffect } from 'react';
import { View, Text, TouchableOpacity, StyleSheet, Alert, Platform } from 'react-native';
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
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    initI18n();
  }, []);

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
        Alert.alert('Error', 'Could not open file');
      }
    }
  };

  const handleCreateVault = () => {
    Alert.prompt(
      'New Vault',
      'Enter vault name:',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Next',
          onPress: name => {
            if (!name?.trim()) return;
            Alert.prompt(
              'Master Password',
              'Choose a strong master password:',
              [
                { text: 'Cancel', style: 'cancel' },
                {
                  text: 'Create',
                  onPress: async password => {
                    if (!password?.trim()) return;
                    setLoading(true);
                    try {
                      const RNFS = require('react-native-fs');
                      const path = `${RNFS.DocumentDirectoryPath}/${name.trim()}.kdbx`;
                      const { createVault } = useVaultStore.getState();
                      await createVault(path, name.trim(), password);
                    } catch (err: unknown) {
                      Alert.alert(
                        'Error',
                        err instanceof Error ? err.message : 'Failed to create vault'
                      );
                    } finally {
                      setLoading(false);
                    }
                  },
                },
              ],
              'secure-text'
            );
          },
        },
      ],
      'plain-text'
    );
  };

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={styles.hero}>
        <Text style={styles.logo}>🔐</Text>
        <Text style={[styles.title, { color: theme.text }]}>KeePassEx</Text>
        <Text style={[styles.tagline, { color: theme.textSecondary }]}>
          Your passwords, your control
        </Text>
      </View>

      <View style={styles.actions}>
        <TouchableOpacity
          style={[styles.primaryButton, { backgroundColor: theme.primary }]}
          onPress={handleOpenVault}
          disabled={loading}
          accessibilityRole="button"
          accessibilityLabel="Open existing vault"
        >
          <Text style={styles.primaryButtonText}>📂 Open Vault</Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.secondaryButton, { borderColor: theme.border }]}
          onPress={handleCreateVault}
          disabled={loading}
          accessibilityRole="button"
          accessibilityLabel="Create new vault"
        >
          <Text style={[styles.secondaryButtonText, { color: theme.text }]}>
            ✨ Create New Vault
          </Text>
        </TouchableOpacity>
      </View>

      <View style={styles.features}>
        {[
          { icon: '🔒', text: 'Argon2id + ChaCha20 encryption' },
          { icon: '📱', text: 'Cross-platform sync' },
          { icon: '🔑', text: 'Passkey & SSH support' },
          { icon: '🛡️', text: 'Breach monitoring' },
        ].map(({ icon, text }) => (
          <View key={text} style={styles.featureRow}>
            <Text style={styles.featureIcon}>{icon}</Text>
            <Text style={[styles.featureText, { color: theme.textSecondary }]}>{text}</Text>
          </View>
        ))}
      </View>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: tokens.space.xl,
    justifyContent: 'space-between',
  },
  hero: {
    alignItems: 'center',
    paddingTop: tokens.space['2xl'],
    gap: tokens.space.sm,
  },
  logo: { fontSize: 72 },
  title: {
    fontSize: tokens.fontSize['3xl'],
    fontWeight: tokens.fontWeight.bold,
  },
  tagline: {
    fontSize: tokens.fontSize.md,
    textAlign: 'center',
  },
  actions: {
    gap: tokens.space.md,
  },
  primaryButton: {
    borderRadius: tokens.radius.md,
    paddingVertical: tokens.space.md,
    alignItems: 'center',
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
  },
  secondaryButtonText: {
    fontSize: tokens.fontSize.lg,
    fontWeight: tokens.fontWeight.medium,
  },
  features: {
    gap: tokens.space.sm,
    paddingBottom: tokens.space.xl,
  },
  featureRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.sm,
  },
  featureIcon: { fontSize: 18 },
  featureText: { fontSize: tokens.fontSize.sm },
});
