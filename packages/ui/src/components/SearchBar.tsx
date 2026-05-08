import { useState, useRef } from 'react';
import { View, TextInput, TouchableOpacity, Text, StyleSheet } from 'react-native';
import { tokens } from '../tokens';

interface SearchBarProps {
  placeholder?: string;
  value?: string;
  onChangeText: (text: string) => void;
  onSubmit?: (text: string) => void;
}

export function SearchBar({
  placeholder = 'Search...',
  value,
  onChangeText,
  onSubmit,
}: SearchBarProps) {
  const [focused, setFocused] = useState(false);
  const inputRef = useRef<TextInput>(null);

  return (
    <View style={[styles.container, focused && styles.containerFocused]} accessibilityRole="none">
      <Text style={styles.icon} accessibilityElementsHidden>
        🔍
      </Text>
      <TextInput
        ref={inputRef}
        style={styles.input}
        value={value}
        onChangeText={onChangeText}
        placeholder={placeholder}
        placeholderTextColor={tokens.color.gray400}
        onFocus={() => setFocused(true)}
        onBlur={() => setFocused(false)}
        onSubmitEditing={e => onSubmit?.(e.nativeEvent.text)}
        returnKeyType="search"
        clearButtonMode="while-editing"
        accessibilityLabel={placeholder}
      />
      {value ? (
        <TouchableOpacity
          onPress={() => onChangeText('')}
          accessibilityRole="button"
          accessibilityLabel="Clear search"
        >
          <Text style={styles.clearIcon}>✕</Text>
        </TouchableOpacity>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: tokens.color.gray100,
    borderRadius: tokens.radius.full,
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.sm,
    gap: tokens.space.sm,
    borderWidth: 1,
    borderColor: 'transparent',
  },
  containerFocused: { borderColor: tokens.color.primary, backgroundColor: tokens.color.white },
  icon: { fontSize: 14 },
  input: { flex: 1, fontSize: tokens.fontSize.md, color: tokens.color.gray900, padding: 0 },
  clearIcon: { fontSize: 12, color: tokens.color.gray400 },
});
