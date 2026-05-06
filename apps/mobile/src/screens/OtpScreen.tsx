/**
 * OTP detail screen — full-screen countdown (with i18n EN/VI)
 */
import React, { useState, useEffect } from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation, useRoute } from '@react-navigation/native';
import type { NativeStackNavigationProp, RouteProp } from '@react-navigation/native-stack';
import { NativeModules } from 'react-native';
import Clipboard from '@react-native-clipboard/clipboard';
import ReactNativeHapticFeedback from 'react-native-haptic-feedback';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import type { RootStackParamList } from '../App';

const { KeePassExCore } = NativeModules;
type Nav = NativeStackNavigationProp<RootStackParamList>;
type Route = RouteProp<RootStackParamList, 'OtpDetail'>;

export function OtpScreen() {
  const navigation = useNavigation<Nav>();
  const route = useRoute<Route>();
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const { entryUuid } = route.params;

  const [otpData, setOtpData] = useState<{
    code: string;
    remainingSeconds: number;
    period: number;
    issuer?: string;
    account?: string;
  } | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const refresh = () => {
      KeePassExCore.generateTotp(entryUuid)
        .then(setOtpData)
        .catch(() => {});
    };
    refresh();
    const interval = setInterval(refresh, 1000);
    return () => clearInterval(interval);
  }, [entryUuid]);

  const handleCopy = () => {
    if (!otpData) return;
    Clipboard.setString(otpData.code);
    ReactNativeHapticFeedback.trigger('impactMedium');
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const isUrgent = (otpData?.remainingSeconds ?? 30) <= 5;
  const codeColor = isUrgent ? tokens.color.danger : theme.primary;
  const progress = otpData ? otpData.remainingSeconds / otpData.period : 1;

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <TouchableOpacity
          onPress={() => navigation.goBack()}
          accessibilityRole="button"
          accessibilityLabel={t('common.back')}
        >
          <Text style={[styles.backButton, { color: theme.primary }]}>← {t('common.back')}</Text>
        </TouchableOpacity>
        <Text style={[styles.headerTitle, { color: theme.text }]}>{t('otp.title')}</Text>
        <View style={{ width: 60 }} />
      </View>

      <View style={styles.content}>
        {otpData && (
          <>
            {otpData.issuer && (
              <Text style={[styles.issuer, { color: theme.text }]}>{otpData.issuer}</Text>
            )}
            {otpData.account && (
              <Text style={[styles.account, { color: theme.textSecondary }]}>
                {otpData.account}
              </Text>
            )}

            <TouchableOpacity
              onPress={handleCopy}
              style={styles.codeContainer}
              accessibilityRole="button"
              accessibilityLabel={`${t('otp.code')}: ${otpData.code}. ${t('common.copy')}.`}
            >
              <Text style={[styles.code, { color: codeColor }]}>
                {otpData.code.slice(0, 3)} {otpData.code.slice(3)}
              </Text>
              <Text style={[styles.copyHint, { color: theme.textTertiary }]}>
                {copied ? `✓ ${t('common.copied')}` : t('common.copy')}
              </Text>
            </TouchableOpacity>

            <View style={[styles.progressTrack, { backgroundColor: theme.border }]}>
              <View
                style={[
                  styles.progressFill,
                  {
                    width: `${progress * 100}%`,
                    backgroundColor: isUrgent ? tokens.color.danger : theme.primary,
                  },
                ]}
              />
            </View>

            <Text
              style={[
                styles.timer,
                { color: isUrgent ? tokens.color.danger : theme.textSecondary },
              ]}
            >
              {t('otp.refreshIn', { seconds: otpData.remainingSeconds })}
            </Text>
          </>
        )}
      </View>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.md,
    borderBottomWidth: StyleSheet.hairlineWidth,
  },
  backButton: { fontSize: tokens.fontSize.md, width: 60 },
  headerTitle: {
    flex: 1,
    fontSize: tokens.fontSize.lg,
    fontWeight: tokens.fontWeight.semibold,
    textAlign: 'center',
  },
  content: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: tokens.space['2xl'],
    gap: tokens.space.lg,
  },
  issuer: {
    fontSize: tokens.fontSize.xl,
    fontWeight: tokens.fontWeight.bold,
    textAlign: 'center',
  },
  account: {
    fontSize: tokens.fontSize.md,
    textAlign: 'center',
  },
  codeContainer: {
    alignItems: 'center',
    gap: tokens.space.sm,
    padding: tokens.space.xl,
  },
  code: {
    fontSize: 52,
    fontWeight: tokens.fontWeight.bold,
    fontFamily: 'Menlo',
    letterSpacing: 8,
  },
  copyHint: {
    fontSize: tokens.fontSize.sm,
  },
  progressTrack: {
    width: '80%',
    height: 6,
    borderRadius: tokens.radius.full,
    overflow: 'hidden',
  },
  progressFill: {
    height: '100%',
    borderRadius: tokens.radius.full,
    transition: 'width 1s linear',
  },
  timer: {
    fontSize: tokens.fontSize.md,
  },
});
