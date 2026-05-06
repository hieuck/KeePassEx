/**
 * Password generator screen (mobile) — with full i18n EN/VI
 */
import React, { useState } from 'react';
import { View, Text, TouchableOpacity, StyleSheet, ScrollView, Switch, Slider } from 'react-native';
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

const STRENGTH_COLORS = [
  tokens.color.strengthVeryWeak,
  tokens.color.strengthWeak,
  tokens.color.strengthFair,
  tokens.color.strengthStrong,
  tokens.color.strengthVeryStrong,
];

export function GeneratorScreen() {
  const navigation = useNavigation<Nav>();
  const route = useRoute<Route>();
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const onSelect = route.params?.onSelect;

  const [mode, setMode] = useState<'random' | 'passphrase'>('random');
  const [length, setLength] = useState(20);
  const [useUppercase, setUseUppercase] = useState(true);
  const [useLowercase, setUseLowercase] = useState(true);
  const [useDigits, setUseDigits] = useState(true);
  const [useSymbols, setUseSymbols] = useState(true);
  const [wordCount, setWordCount] = useState(6);
  const [result, setResult] = useState<{
    password: string;
    strengthScore: number;
    entropy: number;
  } | null>(null);
  const [copied, setCopied] = useState(false);

  const generateMutation = useMutation({
    mutationFn: () =>
      KeePassExCore.generatePassword({
        mode,
        length,
        use_uppercase: useUppercase,
        use_lowercase: useLowercase,
        use_digits: useDigits,
        use_symbols: useSymbols,
        exclude_ambiguous: false,
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
    onSuccess: (data: { password: string; strengthScore: number; entropy: number }) => {
      setResult(data);
      ReactNativeHapticFeedback.trigger('impactLight');
    },
  });

  const handleCopy = () => {
    if (!result) return;
    Clipboard.setString(result.password);
    ReactNativeHapticFeedback.trigger('impactMedium');
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
    setTimeout(() => Clipboard.setString(''), 10_000);
  };

  const handleUse = () => {
    if (!result) return;
    onSelect?.(result.password);
    navigation.goBack();
  };

  const strengthColor = result ? STRENGTH_COLORS[result.strengthScore] : theme.border;

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Header */}
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <TouchableOpacity
          onPress={() => navigation.goBack()}
          accessibilityRole="button"
          accessibilityLabel="Close"
        >
          <Text style={[styles.closeButton, { color: theme.primary }]}>Close</Text>
        </TouchableOpacity>
        <Text style={[styles.headerTitle, { color: theme.text }]}>Generator</Text>
        <View style={{ width: 50 }} />
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.contentInner}>
        {/* Output */}
        <View
          style={[styles.outputCard, { backgroundColor: theme.surface, borderColor: theme.border }]}
        >
          <Text
            style={[styles.passwordText, { color: result ? theme.text : theme.textTertiary }]}
            numberOfLines={3}
            selectable
          >
            {result?.password ?? 'Tap Generate'}
          </Text>

          {result && (
            <View style={styles.strengthRow}>
              <View style={[styles.strengthBar, { backgroundColor: theme.border }]}>
                <View
                  style={[
                    styles.strengthFill,
                    {
                      width: `${(result.strengthScore + 1) * 20}%`,
                      backgroundColor: strengthColor,
                    },
                  ]}
                />
              </View>
              <Text style={[styles.entropyText, { color: theme.textSecondary }]}>
                {result.entropy.toFixed(0)} bits
              </Text>
            </View>
          )}

          <View style={styles.outputActions}>
            <TouchableOpacity
              style={[styles.generateButton, { backgroundColor: theme.primary }]}
              onPress={() => generateMutation.mutate()}
              disabled={generateMutation.isPending}
              accessibilityRole="button"
              accessibilityLabel="Generate password"
            >
              <Text style={styles.generateButtonText}>
                {generateMutation.isPending ? '...' : '🎲 Generate'}
              </Text>
            </TouchableOpacity>

            {result && (
              <>
                <TouchableOpacity
                  style={[styles.actionButton, { borderColor: theme.border }]}
                  onPress={handleCopy}
                  accessibilityRole="button"
                  accessibilityLabel="Copy password"
                >
                  <Text
                    style={[
                      styles.actionButtonText,
                      { color: copied ? tokens.color.success : theme.text },
                    ]}
                  >
                    {copied ? '✓ Copied' : '⎘ Copy'}
                  </Text>
                </TouchableOpacity>

                {onSelect && (
                  <TouchableOpacity
                    style={[styles.actionButton, { borderColor: theme.primary }]}
                    onPress={handleUse}
                    accessibilityRole="button"
                    accessibilityLabel="Use this password"
                  >
                    <Text style={[styles.actionButtonText, { color: theme.primary }]}>
                      Use This
                    </Text>
                  </TouchableOpacity>
                )}
              </>
            )}
          </View>
        </View>

        {/* Mode selector */}
        <View style={styles.modeRow}>
          {(['random', 'passphrase'] as const).map(m => (
            <TouchableOpacity
              key={m}
              style={[
                styles.modeButton,
                { borderColor: theme.border },
                mode === m && { backgroundColor: theme.primary, borderColor: theme.primary },
              ]}
              onPress={() => setMode(m)}
              accessibilityRole="radio"
              accessibilityState={{ checked: mode === m }}
            >
              <Text style={[styles.modeButtonText, { color: mode === m ? 'white' : theme.text }]}>
                {m === 'random' ? 'Random' : 'Passphrase'}
              </Text>
            </TouchableOpacity>
          ))}
        </View>

        {/* Options */}
        {mode === 'random' ? (
          <View
            style={[
              styles.optionsCard,
              { backgroundColor: theme.surface, borderColor: theme.border },
            ]}
          >
            <View style={styles.sliderRow}>
              <Text style={[styles.optionLabel, { color: theme.text }]}>Length: {length}</Text>
              <Slider
                style={styles.slider}
                minimumValue={8}
                maximumValue={128}
                step={1}
                value={length}
                onValueChange={setLength}
                minimumTrackTintColor={theme.primary}
                maximumTrackTintColor={theme.border}
                accessibilityLabel={`Password length: ${length}`}
              />
            </View>

            <ToggleOption
              label="Uppercase (A-Z)"
              value={useUppercase}
              onChange={setUseUppercase}
              theme={theme}
            />
            <ToggleOption
              label="Lowercase (a-z)"
              value={useLowercase}
              onChange={setUseLowercase}
              theme={theme}
            />
            <ToggleOption
              label="Digits (0-9)"
              value={useDigits}
              onChange={setUseDigits}
              theme={theme}
            />
            <ToggleOption
              label="Symbols (!@#...)"
              value={useSymbols}
              onChange={setUseSymbols}
              theme={theme}
            />
          </View>
        ) : (
          <View
            style={[
              styles.optionsCard,
              { backgroundColor: theme.surface, borderColor: theme.border },
            ]}
          >
            <View style={styles.sliderRow}>
              <Text style={[styles.optionLabel, { color: theme.text }]}>Words: {wordCount}</Text>
              <Slider
                style={styles.slider}
                minimumValue={3}
                maximumValue={12}
                step={1}
                value={wordCount}
                onValueChange={setWordCount}
                minimumTrackTintColor={theme.primary}
                maximumTrackTintColor={theme.border}
                accessibilityLabel={`Word count: ${wordCount}`}
              />
            </View>
          </View>
        )}
      </ScrollView>
    </SafeAreaView>
  );
}

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

const styles = StyleSheet.create({
  container: { flex: 1 },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.md,
    borderBottomWidth: StyleSheet.hairlineWidth,
  },
  closeButton: { fontSize: tokens.fontSize.md },
  headerTitle: {
    flex: 1,
    fontSize: tokens.fontSize.lg,
    fontWeight: tokens.fontWeight.semibold,
    textAlign: 'center',
  },
  content: { flex: 1 },
  contentInner: {
    padding: tokens.space.lg,
    gap: tokens.space.lg,
  },
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
  },
  strengthBar: {
    flex: 1,
    height: 4,
    borderRadius: tokens.radius.full,
    overflow: 'hidden',
  },
  strengthFill: {
    height: '100%',
    borderRadius: tokens.radius.full,
  },
  entropyText: { fontSize: tokens.fontSize.xs },
  outputActions: {
    flexDirection: 'row',
    gap: tokens.space.sm,
    flexWrap: 'wrap',
  },
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
  actionButtonText: {
    fontWeight: tokens.fontWeight.medium,
    fontSize: tokens.fontSize.sm,
  },
  modeRow: {
    flexDirection: 'row',
    gap: tokens.space.sm,
  },
  modeButton: {
    flex: 1,
    paddingVertical: tokens.space.sm,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
    alignItems: 'center',
  },
  modeButtonText: {
    fontSize: tokens.fontSize.sm,
    fontWeight: tokens.fontWeight.medium,
  },
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
  },
  optionLabel: { fontSize: tokens.fontSize.md },
});
