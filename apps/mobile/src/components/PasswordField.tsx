/**
 * Password Field — secure text input with show/hide toggle and strength indicator
 */
import React, { useState } from 'react';
import {
  View, Text, TextInput, TouchableOpacity,
  StyleSheet, type TextInputProps,
} from 'react-native';
import { tokens } from '@keepassex/ui';

interface PasswordFieldProps extends Omit<TextInputProps, 'secureTextEntry'> {
  label?: string;
  error?: string;
  showStrength?: boolean;
  strengthScore?: 0 | 1 | 2 | 3 | 4;
  strengthLabel?: string;
  onGeneratePress?: () => void;
  theme?: {
    surface: string;
    border: string;
    text: string;
    textSecondary: string;
    textTertiary: string;
    primary: string;
    danger: string;
  };
}

const STRENGTH_COLORS = [
  tokens.color.strengthVeryWeak,
  tokens.color.strengthWeak,
  tokens.color.strengthFair,
  tokens.color.strengthStrong,
  tokens.color.strengthVeryStrong,
];

export function PasswordField({
  label,
  error,
  showStrength,
  strengthScore,
  strengthLabel,
  onGeneratePress,
  theme,
  value,
  onChangeText,
  placeholder,
  ...props
}: PasswordFieldProps) {
  const [visible, setVisible] = useState(false);

  const borderColor = error ? tokens.color.danger : (theme?.border ?? tokens.color.gray300);
  const strengthColor = strengthScore !== undefined ? STRENGTH_COLORS[strengthScore] : tokens.color.gray300;

  return (
    <View style={styles.container}>
      {label && (
        <Text style={[styles.label, { color: theme?.textSecondary ?? tokens.color.gray600 }]}>
          {label}
        </Text>
      )}

      <View style={[styles.inputRow, { borderColor, backgroundColor: theme?.surface ?? tokens.color.white }]}>
        <TextInput
          style={[styles.input, { color: theme?.text ?? tokens.color.gray900 }]}
          value={value}
          onChangeText={onChangeText}
          placeholder={placeholder}
          placeholderTextColor={theme?.textTertiary ?? tokens.color.gray400}
          secureTextEntry={!visible}
          autoCapitalize="none"
          autoCorrect={false}
          accessibilityLabel={label ?? 'Password'}
          {...props}
        />

        <View style={styles.actions}>
          <TouchableOpacity
            onPress={() => setVisible(v => !v)}
            style={styles.actionBtn}
            accessibilityRole="button"
            accessibilityLabel={visible ? 'Hide password' : 'Show password'}
          >
            <Text style={{ fontSize: 18 }}>{visible ? '🙈' : '👁'}</Text>
          </TouchableOpacity>

          {onGeneratePress && (
            <TouchableOpacity
              onPress={onGeneratePress}
              style={[styles.actionBtn, styles.generateBtn, { backgroundColor: theme?.primary ?? tokens.color.primary }]}
              accessibilityRole="button"
              accessibilityLabel="Generate password"
            >
              <Text style={{ fontSize: 14 }}>⚡</Text>
            </TouchableOpacity>
          )}
        </View>
      </View>

      {error && (
        <Text style={styles.error} accessibilityRole="alert">{error}</Text>
      )}

      {showStrength && value && strengthScore !== undefined && (
        <View style={styles.strengthContainer}>
          <View style={styles.strengthTrack}>
            <View
              style={[
                styles.strengthFill,
                {
                  width: `${(strengthScore + 1) * 20}%`,
                  backgroundColor: strengthColor,
                },
              ]}
            />
          </View>
          {strengthLabel && (
            <Text style={[styles.strengthLabel, { color: strengthColor }]}>
              {strengthLabel}
            </Text>
          )}
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { gap: tokens.space.xs },
  label: {
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
    paddingLeft: tokens.space.md,
  },
  input: {
    flex: 1,
    fontSize: tokens.fontSize.md,
    paddingVertical: tokens.space.sm,
    fontFamily: 'Menlo',
  },
  actions: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingRight: tokens.space.xs,
    gap: 4,
  },
  actionBtn: {
    padding: tokens.space.sm,
    borderRadius: tokens.radius.sm,
  },
  generateBtn: {
    borderRadius: tokens.radius.sm,
  },
  error: {
    fontSize: tokens.fontSize.xs,
    color: tokens.color.danger,
  },
  strengthContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.sm,
  },
  strengthTrack: {
    flex: 1,
    height: 3,
    backgroundColor: tokens.color.gray200,
    borderRadius: tokens.radius.full,
    overflow: 'hidden',
  },
  strengthFill: {
    height: '100%',
    borderRadius: tokens.radius.full,
  },
  strengthLabel: {
    fontSize: tokens.fontSize.xs,
    fontWeight: tokens.fontWeight.semibold,
    minWidth: 60,
  },
});
