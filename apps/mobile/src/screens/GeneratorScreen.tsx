/**
 * Password generator screen (mobile) — full i18n (10 languages)
 * Supports random, passphrase, and pronounceable modes.
 */
import React, { useState, useCallback } from 'react';
import { View, Text, TouchableOpacity, StyleSheet, ScrollView, Switch } from 'react-native';
import Slider from '@react-native-community/slider';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation, useRoute } from '@react-navigation/native';
import type { NativeStackNavigationProp, RouteProp } from '@react-navigation/native-stack';
import { useMutation } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import Clipboard from '@react-native-clipboard/clipboard';
import ReactNativeHapticFeedback from 'react-native-haptic-feedback';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import type { RootStackParamList } from '../App';

const { KeePassExCore } = NativeModules;
type Nav = NativeStackNavigationProp<RootStackParamList>;
type Route = RouteProp<RootStackParamList, 'Generator'>;

type GeneratorMode = 'random' | 'passphrase' | 'pronounceable';

interface GenerateResult {
  password: string;
  strengthScore: 0 | 1 | 2 | 3 | 4;
  entropy: number;
  strengthLabel: string;
}

const STRENGTH_COLORS = ['#DC2626', '#EA580C', '#D97706', '#16A34A', '#059669'];

export function GeneratorScreen() {
  const navigation = useNavigation<Nav>();
  const route = useRoute<Route>();
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const onSelect = route.params?.onSelect;

  const [mode, setMode] = useState<GeneratorMode>('random');
  const [length, setLength] = useState(20);
  const [useUppercase, setUseUppercase] = useState(true);
  const [useLowercase, setUseLowercase] = useState(true);
  const [useDigits, setUseDigits] = useState(true);
  const [useSymbols, setUseSymbols] = useState(true);
  const [excludeAmbiguous, setExcludeAmbiguous] = useState(false);
  const [wordCount, setWordCount] = useState(6);
  const [result, setResult] = useState<GenerateResult | null>(null);
  const [copied, setCopied] = useState(false);

  const generateMutation = useMutation({
    mutationFn: (): Promise<GenerateResult> =>
      KeePassExCore.generatePassword({
        mode,
        length,
        use_uppercase: useUppercase,
        use_lowercase: useLowercase,
        use_digits: useDigits,
        use_symbols: useSymbols,
        exclude_ambiguous: excludeAmbiguous,
        exclude_chars: '',
        min_uppercase: 1,
        min_lowercase: 1,
        min_digits: 1,
        min_symbols: 1,
        word_count: wordCount,
        word_separator: '-',
        capitalize_words: false,
        include_number: true,
      }),
    onSuccess: data => {
      setResult(data);
      ReactNativeHapticFeedback.trigger('impactLight');
    },
  });

  const handleCopy = useCallback(() => {
    if (!result) return;
    Clipboard.setString(result.password);
    ReactNativeHapticFeedback.trigger('impactMedium');
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
    setTimeout(() => Clipboard.setString(''), 10_000);
  }, [result]);

  const handleUse = useCallback(() => {
    if (!result) return;
    onSelect?.(result.password);
    navigation.goBack();
  }, [result, onSelect, navigation]);

  const strengthColor = result ? STRENGTH_COLORS[result.strengthScore] : theme.border;
  const strengthLabel = result
    ? [
        t('generator.strengthVeryWeak'),
        t('generator.strengthWeak'),
        t('generator.strengthFair'),
        t('generator.strengthStrong'),
        t('generator.strengthVeryStrong'),
      ][result.strengthScore]
    : '';

  const MODES: { id: GeneratorMode; label: string }[] = [
    { id: 'random', label: t('generator.modeRandom') },
    { id: 'passphrase', label: t('generator.modePassphrase') },
    { id: 'pronounceable', label: t('generator.modePronounce') },
  ];

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Header */}
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <TouchableOpacity
          onPress={() => navigation.goBack()}
          accessibilityRole="button"
          accessibilityLabel={t('common.close')}
          hitSlop={{ top: 8, bottom: 8, left: 8, right: 8 }}
        >
          <Text style={[styles.headerAction, { color: theme.primary }]}>{t('common.close')}</Text>
        </TouchableOpacity>
        <Text style={[styles.headerTitle, { color: theme.text }]}>{t('generator.title')}</Text>
        <View style={{ width: 50 }} />
      </View>

      <ScrollView style={styles.scroll} contentContainerStyle={styles.scrollContent}>
        {/* Output card */}
        <View
          style={[styles.outputCard, { backgroundColor: theme.surface, borderColor: theme.border }]}
        >
          <Text
            style={[styles.passwordText, { color: result ? theme.text : theme.textTertiary }]}
            numberOfLines={3}
            selectable
            accessibilityLabel={result ? result.password : t('generator.generate')}
          >
            {result?.password ?? t('generator.generate') + '...'}
          </Text>

          {result && (
            <View style={styles.strengthRow}>
              <View style={[styles.strengthBar, { backgroundColor: theme.border }]}>
                <View
                  style={[
                    styles.strengthFill,
                    {
                      width: `${(result.strengthScore + 1) * 20}%` as `${number}%`,
                      backgroundColor: strengthColor,
                    },
                  ]}
                />
              </View>
              <Text style={[styles.strengthLabel, { color: strengthColor }]}>{strengthLabel}</Text>
              <Text style={[styles.entropyText, { color: theme.textTertiary }]}>
                {t('generator.entropy', { bits: result.entropy.toFixed(0) })}
              </Text>
            </View>
          )}

          <View style={styles.outputActions}>
            <TouchableOpacity
              style={[styles.generateButton, { backgroundColor: theme.primary }]}
              onPress={() => generateMutation.mutate()}
              disabled={generateMutation.isPending}
              accessibilityRole="button"
              accessibilityLabel={t('generator.generate')}
            >
              <Text style={styles.generateButtonText}>
                {generateMutation.isPending ? '...' : `🎲 ${t('generator.generate')}`}
              </Text>
            </TouchableOpacity>

            {result && (
              <>
                <TouchableOpacity
                  style={[
                    styles.actionButton,
                    { borderColor: copied ? tokens.color.success : theme.border },
                  ]}
                  onPress={handleCopy}
                  accessibilityRole="button"
                  accessibilityLabel={t('common.copy')}
                >
                  <Text
                    style={[
                      styles.actionButtonText,
                      { color: copied ? tokens.color.success : theme.text },
                    ]}
                  >
                    {copied ? `✓ ${t('common.copied')}` : `⎘ ${t('common.copy')}`}
                  </Text>
                </TouchableOpacity>

                {onSelect && (
                  <TouchableOpacity
                    style={[styles.actionButton, { borderColor: theme.primary }]}
                    onPress={handleUse}
                    accessibilityRole="button"
                    accessibilityLabel={t('generator.usePassword')}
                  >
                    <Text style={[styles.actionButtonText, { color: theme.primary }]}>
                      {t('generator.usePassword')}
                    </Text>
                  </TouchableOpacity>
                )}
              </>
            )}
          </View>
        </View>

        {/* Mode selector */}
        <View style={styles.modeRow} accessibilityRole="radiogroup">
          {MODES.map(m => (
            <TouchableOpacity
              key={m.id}
              style={[
                styles.modeButton,
                { borderColor: theme.border },
                mode === m.id && { backgroundColor: theme.primary, borderColor: theme.primary },
              ]}
              onPress={() => setMode(m.id)}
              accessibilityRole="radio"
              accessibilityState={{ checked: mode === m.id }}
              accessibilityLabel={m.label}
            >
              <Text
                style={[styles.modeButtonText, { color: mode === m.id ? 'white' : theme.text }]}
              >
                {m.label}
              </Text>
            </TouchableOpacity>
          ))}
        </View>

        {/* Options */}
        <View
          style={[
            styles.optionsCard,
            { backgroundColor: theme.surface, borderColor: theme.border },
          ]}
        >
          {mode === 'random' || mode === 'pronounceable' ? (
            <>
              {/* Length slider */}
              <View style={styles.sliderRow}>
                <Text style={[styles.optionLabel, { color: theme.text }]}>
                  {t('generator.length')}: {length}
                </Text>
                <Slider
                  style={styles.slider}
                  minimumValue={8}
                  maximumValue={128}
                  step={1}
                  value={length}
                  onValueChange={v => setLength(Math.round(v))}
                  minimumTrackTintColor={theme.primary}
                  maximumTrackTintColor={theme.border}
                  accessibilityLabel={`${t('generator.length')}: ${length}`}
                />
              </View>

              <ToggleOption
                label={t('generator.uppercase')}
                value={useUppercase}
                onChange={setUseUppercase}
                theme={theme}
              />
              <ToggleOption
                label={t('generator.lowercase')}
                value={useLowercase}
                onChange={setUseLowercase}
                theme={theme}
              />
              <ToggleOption
                label={t('generator.digits')}
                value={useDigits}
                onChange={setUseDigits}
                theme={theme}
              />
              {mode === 'random' && (
                <ToggleOption
                  label={t('generator.symbols')}
                  value={useSymbols}
                  onChange={setUseSymbols}
                  theme={theme}
                />
              )}
              <ToggleOption
                label={t('generator.excludeAmbiguous')}
                value={excludeAmbiguous}
                onChange={setExcludeAmbiguous}
                theme={theme}
              />
            </>
          ) : (
            <>
              {/* Word count slider */}
              <View style={styles.sliderRow}>
                <Text style={[styles.optionLabel, { color: theme.text }]}>
                  {t('generator.wordCount')}: {wordCount}
                </Text>
                <Slider
                  style={styles.slider}
                  minimumValue={3}
                  maximumValue={12}
                  step={1}
                  value={wordCount}
                  onValueChange={v => setWordCount(Math.round(v))}
                  minimumTrackTintColor={theme.primary}
                  maximumTrackTintColor={theme.border}
                  accessibilityLabel={`${t('generator.wordCount')}: ${wordCount}`}
                />
              </View>
            </>
          )}
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function ToggleOption({
  label,
  value,
  onChange,
  theme,
}: {
  label: string;
  value: boolean;
  onChange: (v: boolean) => void;
  theme: ReturnType<typeof useThemeStore>['theme'];
}) {
  return (
    <View style={styles.toggleRow}>
      <Text style={[styles.optionLabel, { color: theme.text }]}>{label}</Text>
      <Switch
        value={value}
        onValueChange={onChange}
        trackColor={{ false: theme.border, true: theme.primary }}
        thumbColor="white"
        accessibilityRole="switch"
        accessibilityLabel={label}
        accessibilityState={{ checked: value }}
      />
    </View>
  );
}

// ─── Styles ───────────────────────────────────────────────────────────────────

const styles = StyleSheet.create({
  container: { flex: 1 },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.md,
    borderBottomWidth: StyleSheet.hairlineWidth,
  },
  headerAction: { fontSize: tokens.fontSize.md, width: 50 },
  headerTitle: {
    flex: 1,
    fontSize: tokens.fontSize.lg,
    fontWeight: tokens.fontWeight.semibold,
    textAlign: 'center',
  },
  scroll: { flex: 1 },
  scrollContent: { padding: tokens.space.lg, gap: tokens.space.lg },
  outputCard: {
    borderRadius: tokens.radius.lg,
    borderWidth: 1,
    padding: tokens.space.lg,
    gap: tokens.space.md,
  },
  passwordText: {
    fontSize: tokens.fontSize.xl,
    fontFamily: 'Menlo',
    fontWeight: tokens.fontWeight.semibold,
    letterSpacing: 1,
    minHeight: 60,
  },
  strengthRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.sm,
    flexWrap: 'wrap',
  },
  strengthBar: {
    flex: 1,
    height: 4,
    borderRadius: tokens.radius.full,
    overflow: 'hidden',
    minWidth: 60,
  },
  strengthFill: { height: '100%', borderRadius: tokens.radius.full },
  strengthLabel: { fontSize: tokens.fontSize.xs, fontWeight: '600' },
  entropyText: { fontSize: tokens.fontSize.xs },
  outputActions: { flexDirection: 'row', gap: tokens.space.sm, flexWrap: 'wrap' },
  generateButton: {
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.sm,
    borderRadius: tokens.radius.md,
  },
  generateButtonText: {
    color: 'white',
    fontWeight: tokens.fontWeight.semibold,
    fontSize: tokens.fontSize.md,
  },
  actionButton: {
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.sm,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
  },
  actionButtonText: { fontWeight: tokens.fontWeight.medium, fontSize: tokens.fontSize.sm },
  modeRow: { flexDirection: 'row', gap: tokens.space.sm },
  modeButton: {
    flex: 1,
    paddingVertical: tokens.space.sm,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
    alignItems: 'center',
  },
  modeButtonText: { fontSize: tokens.fontSize.xs, fontWeight: tokens.fontWeight.medium },
  optionsCard: {
    borderRadius: tokens.radius.lg,
    borderWidth: 1,
    padding: tokens.space.md,
    gap: tokens.space.md,
  },
  sliderRow: { gap: tokens.space.xs },
  slider: { width: '100%' },
  toggleRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 2,
  },
  optionLabel: { fontSize: tokens.fontSize.md, flex: 1 },
});
