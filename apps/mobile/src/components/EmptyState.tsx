/**
 * Empty State — consistent empty state display across screens
 */
import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { tokens } from '@keepassex/ui';

interface EmptyStateProps {
  icon: string;
  title: string;
  description?: string;
  actionLabel?: string;
  onAction?: () => void;
  theme?: {
    text: string;
    textSecondary: string;
    primary: string;
  };
}

export function EmptyState({
  icon,
  title,
  description,
  actionLabel,
  onAction,
  theme,
}: EmptyStateProps) {
  return (
    <View style={styles.container}>
      <Text style={styles.icon} accessibilityHidden>{icon}</Text>
      <Text style={[styles.title, { color: theme?.text ?? tokens.color.gray900 }]}>
        {title}
      </Text>
      {description && (
        <Text style={[styles.description, { color: theme?.textSecondary ?? tokens.color.gray500 }]}>
          {description}
        </Text>
      )}
      {actionLabel && onAction && (
        <TouchableOpacity
          style={[styles.actionBtn, { backgroundColor: theme?.primary ?? tokens.color.primary }]}
          onPress={onAction}
          accessibilityRole="button"
          accessibilityLabel={actionLabel}
        >
          <Text style={styles.actionBtnText}>{actionLabel}</Text>
        </TouchableOpacity>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: tokens.space['2xl'],
    gap: tokens.space.md,
  },
  icon: {
    fontSize: 56,
    marginBottom: tokens.space.sm,
  },
  title: {
    fontSize: tokens.fontSize.xl,
    fontWeight: tokens.fontWeight.semibold,
    textAlign: 'center',
  },
  description: {
    fontSize: tokens.fontSize.md,
    textAlign: 'center',
    lineHeight: 22,
    maxWidth: 280,
  },
  actionBtn: {
    marginTop: tokens.space.sm,
    paddingHorizontal: tokens.space.xl,
    paddingVertical: tokens.space.md,
    borderRadius: tokens.radius.md,
  },
  actionBtnText: {
    color: 'white',
    fontSize: tokens.fontSize.md,
    fontWeight: tokens.fontWeight.semibold,
  },
});
