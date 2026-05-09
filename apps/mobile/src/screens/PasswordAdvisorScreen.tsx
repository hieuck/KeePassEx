/**
 * Password Advisor screen — mobile
 * AI-powered password suggestions with context awareness
 * KeePassEx exclusive: on-device AI, no network required
 * No competitor has AI password suggestions on mobile
 */
import React, { useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  TextInput,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import { useQuery } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import Clipboard from '@react-native-clipboard/clipboard';
import ReactNativeHapticFeedback from 'react-native-haptic-feedback';
import { useThemeStore } from '../store/theme';
import { useI18nStore } from '../store/i18n';
import { tokens } from '@keepassex/ui';

const { KeePassExCore } = NativeModules;

interface AiSuggestion {
  password: string;
  entropy: number;
  strengthScore: number;
  rationaleEn: string;
  rationaleVi: string;
  strategy: string;
}

const CATEGORIES = [
  { id: 'general', icon: '🔑', label: 'General' },
  { id: 'banking', icon: '🏦', label: 'Banking' },
  { id: 'email', icon: '📧', label: 'Email' },
  { id: 'social', icon: '💬', label: 'Social' },
  { id: 'development', icon: '💻', label: 'Dev' },
  { id: 'security', icon: '🛡️', label: 'Security' },
  { id: 'shopping', icon: '🛒', label: 'Shopping' },
  { id: 'gaming', icon: '🎮', label: 'Gaming' },
];

const STRENGTH_COLORS = ['#EF4444', '#F97316', '#EAB308', '#22C55E', '#16A34A'];

export function PasswordAdvisorScreen() {
  const navigation = useNavigation();
  const { theme } = useThemeStore();
  const { t, locale } = useI18nStore();

  const [url, setUrl] = useState('');
  const [category, setCategory] = useState('general');
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  const {
    data: suggestions = [],
    isLoading,
    refetch,
  } = useQuery<AiSuggestion[]>({
    queryKey: ['ai-suggestions', url, category],
    queryFn: () =>
      KeePassExCore.suggestPasswords({
        url,
        title: '',
        category,
        count: 5,
      }),
    enabled: false, // Manual trigger
    staleTime: 30_000,
  });

  const handleGenerate = () => {
    refetch();
  };

  const handleCopy = (suggestion: AiSuggestion, index: number) => {
    Clipboard.setString(suggestion.password);
    ReactNativeHapticFeedback.trigger('impactLight');
    setCopiedIndex(index);
    setTimeout(() => setCopiedIndex(null), 2000);
    setTimeout(() => Clipboard.setString(''), 10_000);
  };

  const getRationale = (s: AiSuggestion) => (locale === 'vi' ? s.rationaleVi : s.rationaleEn);

  const getStrengthLabel = (score: number) => {
    const labels = [
      t('generator.strengthVeryWeak'),
      t('generator.strengthWeak'),
      t('generator.strengthFair'),
      t('generator.strengthStrong'),
      t('generator.strengthVeryStrong'),
    ];
    return labels[Math.min(score, 4)] ?? labels[2];
  };

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Header */}
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <TouchableOpacity onPress={() => navigation.goBack()} accessibilityRole="button">
          <Text style={[styles.backBtn, { color: theme.primary }]}>← {t('common.back')}</Text>
        </TouchableOpacity>
        <Text style={[styles.headerTitle, { color: theme.text }]}>
          🤖 {t('passwordAdvisor.title')}
        </Text>
        <View style={{ width: 60 }} />
      </View>

      <ScrollView contentContainerStyle={styles.content}>
        {/* Context inputs */}
        <View
          style={[styles.section, { backgroundColor: theme.surface, borderColor: theme.border }]}
        >
          <Text style={[styles.sectionTitle, { color: theme.text }]}>
            {t('passwordAdvisor.context')}
          </Text>

          <View style={styles.fieldGroup}>
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('entry.url')}
            </Text>
            <TextInput
              style={[
                styles.input,
                { color: theme.text, borderColor: theme.border, backgroundColor: theme.background },
              ]}
              value={url}
              onChangeText={setUrl}
              placeholder="https://example.com"
              placeholderTextColor={theme.textTertiary}
              autoCapitalize="none"
              autoCorrect={false}
              keyboardType="url"
              accessibilityLabel={t('entry.url')}
            />
          </View>

          <View style={styles.fieldGroup}>
            <Text style={[styles.fieldLabel, { color: theme.textSecondary }]}>
              {t('categorizer.title')}
            </Text>
            <ScrollView
              horizontal
              showsHorizontalScrollIndicator={false}
              contentContainerStyle={styles.categoryList}
            >
              {CATEGORIES.map(cat => (
                <TouchableOpacity
                  key={cat.id}
                  style={[
                    styles.categoryChip,
                    { borderColor: theme.border, backgroundColor: theme.backgroundSecondary },
                    category === cat.id && {
                      backgroundColor: theme.primary,
                      borderColor: theme.primary,
                    },
                  ]}
                  onPress={() => setCategory(cat.id)}
                  accessibilityRole="radio"
                  accessibilityState={{ checked: category === cat.id }}
                >
                  <Text style={styles.categoryIcon}>{cat.icon}</Text>
                  <Text
                    style={[
                      styles.categoryLabel,
                      { color: category === cat.id ? 'white' : theme.text },
                    ]}
                  >
                    {cat.label}
                  </Text>
                </TouchableOpacity>
              ))}
            </ScrollView>
          </View>

          <TouchableOpacity
            style={[styles.generateBtn, { backgroundColor: theme.primary }]}
            onPress={handleGenerate}
            disabled={isLoading}
            accessibilityRole="button"
          >
            {isLoading ? (
              <ActivityIndicator color="white" />
            ) : (
              <Text style={styles.generateBtnText}>🤖 {t('generator.generate')}</Text>
            )}
          </TouchableOpacity>
        </View>

        {/* AI info card */}
        <View style={[styles.infoCard, { backgroundColor: '#EFF6FF', borderColor: '#BFDBFE' }]}>
          <Text style={styles.infoIcon}>🔒</Text>
          <Text style={[styles.infoText, { color: '#1E40AF' }]}>
            {t('passwordAdvisor.onDevice')}
          </Text>
        </View>

        {/* Suggestions */}
        {suggestions.length > 0 && (
          <View style={styles.suggestionsSection}>
            <Text style={[styles.suggestionsTitle, { color: theme.text }]}>
              {t('passwordAdvisor.suggestions')}
            </Text>
            {suggestions.map((s, i) => (
              <View
                key={i}
                style={[
                  styles.suggestionCard,
                  { backgroundColor: theme.surface, borderColor: theme.border },
                ]}
              >
                <View style={styles.suggestionHeader}>
                  <Text
                    style={[styles.suggestionPassword, { color: theme.text }]}
                    numberOfLines={1}
                  >
                    {s.password}
                  </Text>
                  <TouchableOpacity
                    style={[
                      styles.copyBtn,
                      { backgroundColor: copiedIndex === i ? '#16A34A' : theme.primary },
                    ]}
                    onPress={() => handleCopy(s, i)}
                    accessibilityRole="button"
                    accessibilityLabel={t('entry.copyPassword')}
                  >
                    <Text style={styles.copyBtnText}>{copiedIndex === i ? '✓' : '⎘'}</Text>
                  </TouchableOpacity>
                </View>

                <View style={styles.suggestionMeta}>
                  <View
                    style={[
                      styles.strengthBadge,
                      { backgroundColor: STRENGTH_COLORS[Math.min(s.strengthScore, 4)] + '20' },
                    ]}
                  >
                    <Text
                      style={[
                        styles.strengthText,
                        { color: STRENGTH_COLORS[Math.min(s.strengthScore, 4)] },
                      ]}
                    >
                      {getStrengthLabel(s.strengthScore)}
                    </Text>
                  </View>
                  <Text style={[styles.entropyText, { color: theme.textTertiary }]}>
                    {s.entropy.toFixed(0)} bits
                  </Text>
                  <View
                    style={[styles.strategyBadge, { backgroundColor: theme.backgroundSecondary }]}
                  >
                    <Text style={[styles.strategyText, { color: theme.textSecondary }]}>
                      {s.strategy}
                    </Text>
                  </View>
                </View>

                <Text style={[styles.rationale, { color: theme.textSecondary }]}>
                  {getRationale(s)}
                </Text>
              </View>
            ))}
          </View>
        )}
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.md,
    borderBottomWidth: StyleSheet.hairlineWidth,
  },
  backBtn: { fontSize: tokens.fontSize.md },
  headerTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.bold },
  content: { padding: tokens.space.lg, gap: tokens.space.lg },
  section: {
    borderRadius: tokens.radius.lg,
    borderWidth: StyleSheet.hairlineWidth,
    padding: tokens.space.lg,
    gap: tokens.space.md,
  },
  sectionTitle: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.bold },
  fieldGroup: { gap: tokens.space.xs },
  fieldLabel: { fontSize: tokens.fontSize.sm, fontWeight: '500' },
  input: {
    borderWidth: 1,
    borderRadius: tokens.radius.md,
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.sm,
    fontSize: tokens.fontSize.md,
  },
  categoryList: { gap: tokens.space.sm, paddingVertical: 2 },
  categoryChip: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.xs,
    borderRadius: tokens.radius.full,
    borderWidth: 1,
  },
  categoryIcon: { fontSize: 16 },
  categoryLabel: { fontSize: tokens.fontSize.xs, fontWeight: '600' },
  generateBtn: {
    paddingVertical: tokens.space.md,
    borderRadius: tokens.radius.md,
    alignItems: 'center',
  },
  generateBtnText: {
    color: 'white',
    fontSize: tokens.fontSize.md,
    fontWeight: tokens.fontWeight.bold,
  },
  infoCard: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.sm,
    padding: tokens.space.md,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
  },
  infoIcon: { fontSize: 20 },
  infoText: { flex: 1, fontSize: tokens.fontSize.sm, lineHeight: 18 },
  suggestionsSection: { gap: tokens.space.md },
  suggestionsTitle: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.bold },
  suggestionCard: {
    borderRadius: tokens.radius.md,
    borderWidth: StyleSheet.hairlineWidth,
    padding: tokens.space.md,
    gap: tokens.space.sm,
  },
  suggestionHeader: { flexDirection: 'row', alignItems: 'center', gap: tokens.space.sm },
  suggestionPassword: {
    flex: 1,
    fontSize: tokens.fontSize.md,
    fontWeight: '600',
    fontFamily: 'Menlo',
  },
  copyBtn: {
    width: 36,
    height: 36,
    borderRadius: tokens.radius.sm,
    alignItems: 'center',
    justifyContent: 'center',
  },
  copyBtnText: { color: 'white', fontSize: 16 },
  suggestionMeta: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.sm,
    flexWrap: 'wrap',
  },
  strengthBadge: { paddingHorizontal: 8, paddingVertical: 2, borderRadius: tokens.radius.full },
  strengthText: { fontSize: 11, fontWeight: '700' },
  entropyText: { fontSize: 11 },
  strategyBadge: { paddingHorizontal: 8, paddingVertical: 2, borderRadius: tokens.radius.full },
  strategyText: { fontSize: 10 },
  rationale: { fontSize: tokens.fontSize.xs, lineHeight: 18 },
});
