import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet, FlatList } from 'react-native';
import { tokens } from '../tokens';

const ICONS: Array<{ id: number; emoji: string; label: string }> = [
  { id: 0, emoji: '🔑', label: 'Key' },
  { id: 1, emoji: '🌐', label: 'Web' },
  { id: 2, emoji: '⚠️', label: 'Warning' },
  { id: 3, emoji: '🖥️', label: 'Server' },
  { id: 4, emoji: '🔧', label: 'Tool' },
  { id: 5, emoji: '💻', label: 'Computer' },
  { id: 6, emoji: '📁', label: 'Folder' },
  { id: 7, emoji: '🔒', label: 'Lock' },
  { id: 8, emoji: '📧', label: 'Email' },
  { id: 9, emoji: '💳', label: 'Card' },
  { id: 10, emoji: '🏦', label: 'Bank' },
  { id: 11, emoji: '📱', label: 'Mobile' },
  { id: 12, emoji: '🛡️', label: 'Shield' },
  { id: 13, emoji: '👤', label: 'Person' },
  { id: 14, emoji: '🏠', label: 'Home' },
  { id: 15, emoji: '💼', label: 'Work' },
];

interface IconPickerProps {
  selectedId: number;
  onSelect: (id: number) => void;
}

export function IconPicker({ selectedId, onSelect }: IconPickerProps) {
  return (
    <FlatList
      data={ICONS}
      numColumns={6}
      keyExtractor={item => String(item.id)}
      renderItem={({ item }) => (
        <TouchableOpacity
          style={[styles.iconButton, item.id === selectedId && styles.iconButtonSelected]}
          onPress={() => onSelect(item.id)}
          accessibilityRole="radio"
          accessibilityLabel={item.label}
          accessibilityState={{ checked: item.id === selectedId }}
        >
          <Text style={styles.iconEmoji}>{item.emoji}</Text>
        </TouchableOpacity>
      )}
      contentContainerStyle={styles.grid}
    />
  );
}

const styles = StyleSheet.create({
  grid: { gap: tokens.space.xs },
  iconButton: {
    flex: 1,
    aspectRatio: 1,
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: tokens.radius.sm,
    borderWidth: 2,
    borderColor: 'transparent',
    margin: 2,
  },
  iconButtonSelected: { borderColor: tokens.color.primary, backgroundColor: '#EFF6FF' },
  iconEmoji: { fontSize: 24 },
});
