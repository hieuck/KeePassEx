/**
 * Entry list item component
 * Used in vault entry lists across all platforms
 */
import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { tokens } from '../tokens';

export interface EntryListItemData {
  uuid: string;
  title: string;
  username: string;
  url?: string;
  iconId?: number;
  hasOtp?: boolean;
  hasPasskey?: boolean;
  hasSshKey?: boolean;
  isExpired?: boolean;
  expiresInDays?: number;
  tags?: string[];
}

interface EntryListItemProps {
  entry: EntryListItemData;
  onPress?: (uuid: string) => void;
  onLongPress?: (uuid: string) => void;
  onCopyPassword?: (uuid: string) => void;
  showUrl?: boolean;
  compact?: boolean;
}

export function EntryListItem({
  entry,
  onPress,
  onLongPress,
  onCopyPassword,
  showUrl = true,
  compact = false,
}: EntryListItemProps) {
  const isExpiringSoon = entry.expiresInDays !== undefined && entry.expiresInDays <= 30;

  return (
    <TouchableOpacity
      style={[styles.container, compact && styles.containerCompact]}
      onPress={() => onPress?.(entry.uuid)}
      onLongPress={() => onLongPress?.(entry.uuid)}
      accessibilityRole="button"
      accessibilityLabel={`${entry.title}, ${entry.username}`}
      accessibilityHint="Double tap to open entry"
    >
      {/* Icon */}
      <View style={[styles.icon, entry.isExpired && styles.iconExpired]}>
        <Text style={styles.iconText}>
          {getEntryEmoji(entry.iconId)}
        </Text>
      </View>

      {/* Content */}
      <View style={styles.content}>
        <View style={styles.titleRow}>
          <Text
            style={[styles.title, entry.isExpired && styles.titleExpired]}
            numberOfLines={1}
          >
            {entry.title}
          </Text>

          {/* Badges */}
          <View style={styles.badges}>
            {entry.hasOtp && (
              <View style={[styles.badge, styles.badgeOtp]}>
                <Text style={styles.badgeText}>OTP</Text>
              </View>
            )}
            {entry.hasPasskey && (
              <View style={[styles.badge, styles.badgePasskey]}>
                <Text style={styles.badgeText}>🔑</Text>
              </View>
            )}
            {entry.hasSshKey && (
              <View style={[styles.badge, styles.badgeSsh]}>
                <Text style={styles.badgeText}>SSH</Text>
              </View>
            )}
            {entry.isExpired && (
              <View style={[styles.badge, styles.badgeExpired]}>
                <Text style={styles.badgeText}>Expired</Text>
              </View>
            )}
            {isExpiringSoon && !entry.isExpired && (
              <View style={[styles.badge, styles.badgeWarning]}>
                <Text style={styles.badgeText}>{entry.expiresInDays}d</Text>
              </View>
            )}
          </View>
        </View>

        <Text style={styles.username} numberOfLines={1}>
          {entry.username || '—'}
        </Text>

        {showUrl && entry.url && !compact && (
          <Text style={styles.url} numberOfLines={1}>
            {formatUrl(entry.url)}
          </Text>
        )}
      </View>

      {/* Quick copy button */}
      {onCopyPassword && (
        <TouchableOpacity
          style={styles.copyButton}
          onPress={() => onCopyPassword(entry.uuid)}
          accessibilityLabel="Copy password"
          accessibilityRole="button"
          hitSlop={{ top: 8, bottom: 8, left: 8, right: 8 }}
        >
          <Text style={styles.copyIcon}>⎘</Text>
        </TouchableOpacity>
      )}
    </TouchableOpacity>
  );
}

function getEntryEmoji(iconId?: number): string {
  const icons: Record<number, string> = {
    0: '🔑',
    1: '🌐',
    2: '⚠️',
    3: '📋',
    4: '🔧',
    5: '💻',
    6: '📁',
    7: '🔒',
    8: '📧',
    9: '💳',
    10: '🏦',
    11: '📱',
    12: '🛡️',
    13: '👤',
    14: '🏠',
    15: '💼',
  };
  return icons[iconId ?? 0] ?? '🔑';
}

function formatUrl(url: string): string {
  try {
    const u = new URL(url);
    return u.hostname;
  } catch {
    return url;
  }
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.md,
    gap: tokens.space.md,
    backgroundColor: tokens.color.white,
  },
  containerCompact: {
    paddingVertical: tokens.space.sm,
  },
  icon: {
    width: 40,
    height: 40,
    borderRadius: tokens.radius.md,
    backgroundColor: tokens.color.gray100,
    alignItems: 'center',
    justifyContent: 'center',
  },
  iconExpired: {
    opacity: 0.5,
  },
  iconText: {
    fontSize: 20,
  },
  content: {
    flex: 1,
    gap: 2,
  },
  titleRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.xs,
  },
  title: {
    fontSize: tokens.fontSize.md,
    fontWeight: tokens.fontWeight.semibold,
    color: tokens.color.gray900,
    flex: 1,
  },
  titleExpired: {
    color: tokens.color.gray400,
    textDecorationLine: 'line-through',
  },
  username: {
    fontSize: tokens.fontSize.sm,
    color: tokens.color.gray500,
  },
  url: {
    fontSize: tokens.fontSize.xs,
    color: tokens.color.gray400,
  },
  badges: {
    flexDirection: 'row',
    gap: 4,
  },
  badge: {
    paddingHorizontal: 5,
    paddingVertical: 2,
    borderRadius: tokens.radius.xs,
  },
  badgeOtp: {
    backgroundColor: '#EFF6FF',
  },
  badgePasskey: {
    backgroundColor: '#F0FDF4',
  },
  badgeSsh: {
    backgroundColor: '#FFF7ED',
  },
  badgeExpired: {
    backgroundColor: '#FEF2F2',
  },
  badgeWarning: {
    backgroundColor: '#FFFBEB',
  },
  badgeText: {
    fontSize: 10,
    fontWeight: tokens.fontWeight.semibold,
    color: tokens.color.gray600,
  },
  copyButton: {
    padding: tokens.space.xs,
  },
  copyIcon: {
    fontSize: 18,
    color: tokens.color.gray400,
  },
});
