import React, { useState } from 'react';
import { View, TouchableOpacity, Text, StyleSheet } from 'react-native';
import { Input } from './Input';
import { tokens } from '../tokens';

interface PasswordInputProps {
  label?: string;
  value: string;
  onChangeText: (v: string) => void;
  placeholder?: string;
  error?: string;
  accessibilityLabel?: string;
}

export function PasswordInput({ label, value, onChangeText, placeholder, error, accessibilityLabel }: PasswordInputProps) {
  const [visible, setVisible] = useState(false);

  return (
    <View style={styles.container}>
      <Input
        label={label}
        value={value}
        onChangeText={onChangeText}
        placeholder={placeholder ?? 'Password'}
        secureTextEntry={!visible}
        autoCapitalize="none"
        autoCorrect={false}
        error={error}
        accessibilityLabel={accessibilityLabel ?? label ?? 'Password'}
      />
      <TouchableOpacity
        style={styles.toggle}
        onPress={() => setVisible(v => !v)}
        accessibilityRole="button"
        accessibilityLabel={visible ? 'Hide password' : 'Show password'}
      >
        <Text style={styles.toggleIcon}>{visible ? '🙈' : '👁'}</Text>
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { position: 'relative' },
  toggle: {
    position: 'absolute',
    right: tokens.space.md,
    bottom: tokens.space.sm,
    padding: tokens.space.xs,
  },
  toggleIcon: { fontSize: 18 },
});
