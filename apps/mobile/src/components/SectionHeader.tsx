/**
 * Section Header — consistent section headers across screens
 */
import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { tokens } from '@keepassex/ui';

interface SectionHeaderProps {
  title: string;
  subtitle?: string;
  rightElement?: React.ReactNode;
  theme?: {
    textSecondary: string;
    text: string;
  };
}

export function SectionHeader({ title, subtitle, rightElement, theme }: SectionHeaderProps) {
  return (
    <View style={styles.container}>
      <View style={styles.textContainer}>
        <Text style={[styles.title, { color: theme?.textSecondary ?? tokens.color.gray500 }]}>
          {title.toUpperCase()}
        </Text>
        {subtitle && (
          <Text style={[styles.subtitle, { color: theme?.textSecondary ?? tokens.color.gray400 }]}>
            {subtitle}
          </Text>
        )}
      </View>
      {rightElement}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: tokens.space.xs,
    paddingTop: tokens.space.md,
    paddingBottom: tokens.space.xs,
  },
  textContainer: { flex: 1 },
  title: {
    fontSize: 11,
    fontWeight: '600',
    letterSpacing: 0.8,
  },
  subtitle: {
    fontSize: tokens.fontSize.xs,
    marginTop: 1,
  },
});
