/**
 * Password Strength Meter component
 * Shows visual strength indicator with label
 */

import { View, Text, StyleSheet } from 'react-native';
import { tokens } from '../tokens';

export type StrengthLevel = 0 | 1 | 2 | 3 | 4;

interface StrengthMeterProps {
  score: StrengthLevel;
  label?: string;
  showLabel?: boolean;
  compact?: boolean;
}

const STRENGTH_COLORS: Record<StrengthLevel, string> = {
  0: tokens.color.strengthVeryWeak,
  1: tokens.color.strengthWeak,
  2: tokens.color.strengthFair,
  3: tokens.color.strengthStrong,
  4: tokens.color.strengthVeryStrong,
};

export function StrengthMeter({
  score,
  label,
  showLabel = true,
  compact = false,
}: StrengthMeterProps) {
  const color = STRENGTH_COLORS[score];
  const segments = 4;

  return (
    <View style={styles.container}>
      <View style={[styles.bars, compact && styles.barsCompact]}>
        {Array.from({ length: segments }, (_, i) => (
          <View
            key={i}
            style={[
              styles.bar,
              compact && styles.barCompact,
              {
                backgroundColor:
                  i < score
                    ? color
                    : tokens.color.gray200,
              },
            ]}
          />
        ))}
      </View>
      {showLabel && label && (
        <Text style={[styles.label, { color }]}>{label}</Text>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.sm,
  },
  bars: {
    flexDirection: 'row',
    gap: 3,
    flex: 1,
  },
  barsCompact: {
    flex: 0,
    width: 80,
  },
  bar: {
    flex: 1,
    height: 4,
    borderRadius: tokens.radius.full,
  },
  barCompact: {
    width: 16,
    flex: 0,
  },
  label: {
    fontSize: tokens.fontSize.sm,
    fontWeight: tokens.fontWeight.medium,
  },
});
