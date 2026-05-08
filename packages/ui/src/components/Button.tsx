
import { TouchableOpacity, Text, StyleSheet, ActivityIndicator } from 'react-native';
import { tokens } from '../tokens';

interface ButtonProps {
  label: string;
  onPress: () => void;
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  loading?: boolean;
  icon?: string;
  accessibilityLabel?: string;
}

export function Button({
  label, onPress, variant = 'primary', size = 'md',
  disabled, loading, icon, accessibilityLabel,
}: ButtonProps) {
  const bg = {
    primary: tokens.color.primary,
    secondary: tokens.color.gray100,
    danger: tokens.color.danger,
    ghost: 'transparent',
  }[variant];

  const textColor = variant === 'secondary' ? tokens.color.gray900
    : variant === 'ghost' ? tokens.color.primary
    : 'white';

  const padding = { sm: tokens.space.sm, md: tokens.space.md, lg: tokens.space.lg }[size];

  return (
    <TouchableOpacity
      style={[styles.button, { backgroundColor: bg, paddingVertical: padding, paddingHorizontal: padding * 1.5 }, disabled && styles.disabled]}
      onPress={onPress}
      disabled={disabled || loading}
      accessibilityRole="button"
      accessibilityLabel={accessibilityLabel ?? label}
      accessibilityState={{ disabled: disabled || loading }}
    >
      {loading ? (
        <ActivityIndicator color={textColor} size="small" />
      ) : (
        <Text style={[styles.label, { color: textColor, fontSize: { sm: tokens.fontSize.sm, md: tokens.fontSize.md, lg: tokens.fontSize.lg }[size] }]}>
          {icon ? `${icon} ${label}` : label}
        </Text>
      )}
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  button: {
    borderRadius: tokens.radius.md,
    alignItems: 'center',
    justifyContent: 'center',
    flexDirection: 'row',
  },
  disabled: { opacity: 0.5 },
  label: { fontWeight: tokens.fontWeight.semibold },
});
