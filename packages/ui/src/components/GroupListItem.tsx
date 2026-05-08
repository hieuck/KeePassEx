
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { tokens } from '../tokens';

interface GroupListItemProps {
  uuid: string;
  name: string;
  iconId?: number;
  entryCount: number;
  selected?: boolean;
  onPress: (uuid: string) => void;
  depth?: number;
}

export function GroupListItem({ uuid, name, iconId, entryCount, selected, onPress, depth = 0 }: GroupListItemProps) {
  return (
    <TouchableOpacity
      style={[styles.container, { paddingLeft: 16 + depth * 16 }, selected && styles.selected]}
      onPress={() => onPress(uuid)}
      accessibilityRole="button"
      accessibilityLabel={`${name}, ${entryCount} entries`}
      accessibilityState={{ selected }}
    >
      <Text style={styles.icon}>{iconId === 43 ? '🗑️' : '📁'}</Text>
      <Text style={[styles.name, selected && styles.nameSelected]} numberOfLines={1}>{name}</Text>
      {entryCount > 0 && (
        <Text style={[styles.count, selected && styles.countSelected]}>{entryCount}</Text>
      )}
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: tokens.space.sm,
    paddingRight: tokens.space.md,
    gap: tokens.space.sm,
    borderRadius: tokens.radius.sm,
    marginHorizontal: tokens.space.sm,
  },
  selected: { backgroundColor: tokens.color.primary },
  icon: { fontSize: 16 },
  name: { flex: 1, fontSize: tokens.fontSize.sm, color: tokens.color.gray700 },
  nameSelected: { color: 'white', fontWeight: tokens.fontWeight.medium },
  count: {
    fontSize: 11,
    color: tokens.color.gray400,
    backgroundColor: tokens.color.gray100,
    paddingHorizontal: 5,
    paddingVertical: 1,
    borderRadius: 10,
  },
  countSelected: { backgroundColor: 'rgba(255,255,255,0.2)', color: 'white' },
});
