
import { View, Text, StyleSheet } from 'react-native';
import { tokens } from '../tokens';

interface HealthBadgeProps {
  score: number; // 0-100
  size?: 'sm' | 'md' | 'lg';
}

export function HealthBadge({ score, size = 'md' }: HealthBadgeProps) {
  const color = score >= 90 ? tokens.color.strengthVeryStrong
    : score >= 70 ? tokens.color.strengthStrong
    : score >= 50 ? tokens.color.strengthFair
    : tokens.color.strengthVeryWeak;

  const dim = { sm: 32, md: 48, lg: 64 }[size];
  const fontSize = { sm: 12, md: 18, lg: 24 }[size];

  return (
    <View
      style={[styles.badge, { width: dim, height: dim, borderRadius: dim / 2, borderColor: color }]}
      accessibilityRole="progressbar"
      accessibilityValue={{ min: 0, max: 100, now: score }}
      accessibilityLabel={`Health score: ${score} out of 100`}
    >
      <Text style={[styles.score, { color, fontSize }]}>{score}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  badge: {
    borderWidth: 3,
    alignItems: 'center',
    justifyContent: 'center',
  },
  score: { fontWeight: tokens.fontWeight.bold, lineHeight: undefined },
});
