import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { tokens } from '../tokens';
import { Button } from './Button';
import { PasswordInput } from './PasswordInput';

interface VaultLockScreenProps {
  vaultName?: string;
  onUnlock: (password: string) => void;
  onBiometric?: () => void;
  biometricAvailable?: boolean;
  loading?: boolean;
  error?: string;
}

export function VaultLockScreen({
  vaultName, onUnlock, onBiometric, biometricAvailable, loading, error,
}: VaultLockScreenProps) {
  const [password, setPassword] = React.useState('');

  return (
    <View style={styles.container}>
      <Text style={styles.logo}>🔐</Text>
      <Text style={styles.title}>KeePassEx</Text>
      {vaultName && <Text style={styles.vaultName}>{vaultName}</Text>}

      {biometricAvailable && onBiometric && (
        <Button
          label="Unlock with Biometrics"
          onPress={onBiometric}
          variant="secondary"
          icon="👤"
          disabled={loading}
        />
      )}

      <PasswordInput
        label="Master Password"
        value={password}
        onChangeText={setPassword}
        error={error}
        accessibilityLabel="Master password"
      />

      <Button
        label="Unlock"
        onPress={() => onUnlock(password)}
        disabled={!password.trim() || loading}
        loading={loading}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: tokens.space.xl,
    gap: tokens.space.lg,
  },
  logo: { fontSize: 64 },
  title: { fontSize: tokens.fontSize['2xl'], fontWeight: tokens.fontWeight.bold, color: tokens.color.gray900 },
  vaultName: { fontSize: tokens.fontSize.md, color: tokens.color.gray500 },
});
