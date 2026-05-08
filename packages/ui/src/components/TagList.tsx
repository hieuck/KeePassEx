/**
 * TagList — shared tag display component (React Native)
 */

import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { tokens } from '../tokens';

interface TagListProps {
  tags: string[];
  onRemove?: (tag: string) => void;
  onPress?: (tag: string) => void;
  editable?: boolean;
  maxVisible?: number;
  size?: 'sm' | 'md';
}

export function TagList({
  tags,
  onRemove,
  onPress,
  editable,
  maxVisible,
  size = 'sm',
}: TagListProps) {
  const visibleTags = maxVisible ? tags.slice(0, maxVisible) : tags;
  const hiddenCount = maxVisible ? Math.max(0, tags.length - maxVisible) : 0;

  if (tags.length === 0) return null;

  const fontSize = size === 'sm' ? tokens.fontSize.xs : tokens.fontSize.sm;
  const paddingH = size === 'sm' ? 6 : 8;
  const paddingV = size === 'sm' ? 2 : 4;

  return (
    <View style={styles.container} accessibilityRole="list">
      {visibleTags.map(tag => (
        <TouchableOpacity
          key={tag}
          style={[styles.tag, { paddingHorizontal: paddingH, paddingVertical: paddingV }]}
          onPress={() => onPress?.(tag)}
          disabled={!onPress && !editable}
          accessibilityRole="button"
          accessibilityLabel={tag}
        >
          <Text style={[styles.tagText, { fontSize }]}>{tag}</Text>
          {editable && onRemove && (
            <TouchableOpacity
              onPress={() => onRemove(tag)}
              hitSlop={{ top: 4, bottom: 4, left: 4, right: 4 }}
              accessibilityRole="button"
              accessibilityLabel={`Remove ${tag}`}
            >
              <Text style={[styles.removeIcon, { fontSize: fontSize - 1 }]}>✕</Text>
            </TouchableOpacity>
          )}
        </TouchableOpacity>
      ))}
      {hiddenCount > 0 && (
        <View style={[styles.tag, styles.moreTag]}>
          <Text style={[styles.tagText, { fontSize }]}>+{hiddenCount}</Text>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 4,
  },
  tag: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 3,
    backgroundColor: tokens.color.gray100,
    borderRadius: tokens.radius.full,
  },
  moreTag: {
    backgroundColor: tokens.color.gray200,
  },
  tagText: {
    color: tokens.color.gray600,
    fontWeight: tokens.fontWeight.medium,
  },
  removeIcon: {
    color: tokens.color.gray400,
    fontWeight: tokens.fontWeight.bold,
  },
});
