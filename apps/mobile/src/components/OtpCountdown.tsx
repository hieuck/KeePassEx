/**
 * OTP Countdown — animated countdown ring for TOTP codes
 */
import React, { useEffect, useState } from 'react';
import { View, Text, StyleSheet, Animated } from 'react-native';
import { tokens } from '@keepassex/ui';

interface OtpCountdownProps {
  code: string;
  remainingSeconds: number;
  period: number;
  issuer?: string;
  account?: string;
  onCopy?: (code: string) => void;
  size?: 'sm' | 'md' | 'lg';
}

export function OtpCountdown({
  code,
  remainingSeconds,
  period,
  issuer,
  account,
  onCopy,
  size = 'md',
}: OtpCountdownProps) {
  const isUrgent = remainingSeconds <= 5;
  const progress = remainingSeconds / period;

  const codeColor = isUrgent ? tokens.color.danger : tokens.color.primary;
  const formattedCode = code.length === 6
    ? `${code.slice(0, 3)} ${code.slice(3)}`
    : code;

  const fontSize = { sm: 18, md: 24, lg: 32 }[size];

  return (
    <View style={styles.container}>
      {(issuer || account) && (
        <View style={styles.meta}>
          {issuer && <Text style={styles.issuer} numberOfLines={1}>{issuer}</Text>}
          {account && <Text style={styles.account} numberOfLines={1}>{account}</Text>}
        </View>
      )}

      <View style={styles.codeRow}>
        <Text
          style={[styles.code, { color: codeColor, fontSize }]}
          accessibilityLabel={`OTP code: ${code}`}
          selectable
        >
          {formattedCode}
        </Text>

        <View style={[styles.timerRing, { borderColor: isUrgent ? tokens.color.danger : tokens.color.primary }]}>
          <Text style={[styles.timerText, { color: isUrgent ? tokens.color.danger : tokens.color.gray500 }]}>
            {remainingSeconds}
          </Text>
        </View>
      </View>

      {/* Progress bar */}
      <View style={styles.progressTrack}>
        <View
          style={[
            styles.progressFill,
            {
              width: `${progress * 100}%`,
              backgroundColor: isUrgent ? tokens.color.danger : tokens.color.primary,
            },
          ]}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    gap: tokens.space.xs,
  },
  meta: {
    gap: 2,
  },
  issuer: {
    fontSize: tokens.fontSize.sm,
    fontWeight: tokens.fontWeight.semibold,
    color: tokens.color.gray700,
  },
  account: {
    fontSize: tokens.fontSize.xs,
    color: tokens.color.gray500,
  },
  codeRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  code: {
    fontFamily: 'Menlo',
    fontWeight: tokens.fontWeight.bold,
    letterSpacing: 4,
    flex: 1,
  },
  timerRing: {
    width: 36,
    height: 36,
    borderRadius: 18,
    borderWidth: 2,
    alignItems: 'center',
    justifyContent: 'center',
  },
  timerText: {
    fontSize: tokens.fontSize.xs,
    fontWeight: tokens.fontWeight.bold,
  },
  progressTrack: {
    height: 3,
    backgroundColor: tokens.color.gray200,
    borderRadius: tokens.radius.full,
    overflow: 'hidden',
  },
  progressFill: {
    height: '100%',
    borderRadius: tokens.radius.full,
  },
});
