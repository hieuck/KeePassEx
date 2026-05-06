/**
 * Tag List — display and manage entry tags
 */
import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { tokens } from '@keepassex/ui';

interface TagListProps {
  tags: string[];
  onRemove?: (tag: string) => void;
  onPress?: (tag: string) => void;
  editable?: boolean;
  theme?: {
    backgroundTertiary: string;
    textSecondary: string;
    danger: string;
  };
}

export function TagList({ tags, onRemove, onPress, editable, theme }: TagListProps) {
  if (tags.length === 0) return null;

  return (
    <View style={styles.container} accessibilityRole="list">
      {tags.map(tag => (
        <TouchableOpacity
          key={tag}
          style={[styles.tag, { backgroundColor: theme?.backgroundTertiary ?? tokens.color.gray100 }]}
          onPress={() => onPress?.(tag)}
          disabled={!onPress && !editable}
          accessibilityRole="button"
          accessibilityLabel={tag}
        >
          <Text style={[styles.tagText, { color: theme?.textSecondary ?? tokens.color.gray600 }]}>
            {tag}
          </Text>
          {editable && onRemove && (
            <TouchableOpacity
              onPress={() => onRemove(tag)}
              style={styles.removeBtn}
              hitSlop={{ top: 4, bottom: 4, left: 4, right: 4 }}
              accessibilityRole="button"
              accessibilityLabel={`Remove tag ${tag}`}
            >
              <Text style={[styles.removeIcon, { color: theme?.danger ?? tokens.color.danger }]}>✕</Text>
            </TouchableOpacity>
          )}
        </TouchableOpacity>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: tokens.space.xs,
  },
  tag: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: tokens.space.sm,
    paddingVertical: 3,
    borderRadius: tokens.radius.full,
    gap: 4,
  },
  tagText: {
    fontSize: tokens.fontSize.xs,
    fontWeight: tokens.fontWeight.medium,
  },
  removeBtn: {
    padding: 1,
  },
  removeIcon: {
    fontSize: 10,
    fontWeight: tokens.fontWeight.bold,
  },
});
