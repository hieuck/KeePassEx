import React, { useState } from 'react';
import { View, Text, TextInput, StyleSheet, type TextInputProps } from 'react-native';
import { tokens } from '../tokens';

interface InputProps extends TextInputProps {
  label?: string;
  error?: string;
  hint?: string;
}

export function Input({ label, error, hint, style, ...props }: InputProps) {
  const [focused, setFocused] = useState(false);

  return (
    <View style={styles.container}>
      {label && <Text style={styles.label}>{label}</Text>}
      <TextInput
        style={[
          styles.input,
          focused && styles.inputFocused,
          error && styles.inputError,
          style,
        ]}
        onFocus={() => setFocused(true)}
        onBlur={() => setFocused(false)}
        placeholderTextColor={tokens.color.gray400}
        {...props}
      />
      {error && <Text style={styles.error} accessibilityRole="alert">{error}</Text>}
      {hint && !error && <Text style={styles.hint}>{hint}</Text>}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { gap: tokens.space.xs },
  label: { fontSize: tokens.fontSize.sm, fontWeight: tokens.fontWeight.medium, color: tokens.color.gray600 },
  input: {
    borderWidth: 1,
    borderColor: tokens.color.gray300,
    borderRadius: tokens.radius.md,
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.sm,
    fontSize: tokens.fontSize.md,
    color: tokens.color.gray900,
    backgroundColor: tokens.color.white,
  },
  inputFocused: { borderColor: tokens.color.primary },
  inputError: { borderColor: tokens.color.danger },
  error: { fontSize: tokens.fontSize.xs, color: tokens.color.danger },
  hint: { fontSize: tokens.fontSize.xs, color: tokens.color.gray500 },
});
