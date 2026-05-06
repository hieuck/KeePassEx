/**
 * ShardingScreen — Vault Key Sharding (Shamir's Secret Sharing) — Mobile
 * Exclusive KeePassEx feature — no competitor has this
 */
import React, { useState, useCallback } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  TextInput,
  Alert,
  Share,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { NativeModules } from 'react-native';
import ReactNativeHapticFeedback from 'react-native-haptic-feedback';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';

const { KeePassExCore } = NativeModules;

type Mode = 'split' | 'combine';

interface ShardInfo {
  index: number;
  total: number;
  threshold: number;
  data_b64: string;
  label?: string;
}

export function ShardingScreen() {
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const [mode, setMode] = useState<Mode>('split');

  // Split mode
  const [threshold, setThreshold] = useState(3);
  const [total, setTotal] = useState(5);
  const [generatedShards, setGeneratedShards] = useState<ShardInfo[]>([]);
  const [splitting, setSplitting] = useState(false);

  // Combine mode
  const [loadedShards, setLoadedShards] = useState<ShardInfo[]>([]);
  const [combining, setCombining] = useState(false);
  const [combineSuccess, setCombineSuccess] = useState(false);

  const handleSplit = useCallback(async () => {
    if (threshold < 2 || total < threshold) {
      Alert.alert('Invalid', t('sharding.thresholdExample'));
      return;
    }
    setSplitting(true);
    try {
      const shards: ShardInfo[] = await KeePassExCore.splitVaultKey(threshold, total);
      setGeneratedShards(shards);
      ReactNativeHapticFeedback.trigger('notificationSuccess');
    } catch (err) {
      Alert.alert(t('common.error'), t('sharding.generateFailed', { error: String(err) }));
    } finally {
      setSplitting(false);
    }
  }, [threshold, total, t]);

  const handleShareShard = useCallback(async (shard: ShardInfo) => {
    try {
      await Share.share({
        title: `KeePassEx Shard ${shard.index}/${shard.total}`,
        message: JSON.stringify(shard),
      });
    } catch {
      /* user cancelled */
    }
  }, []);

  const handleCombine = useCallback(async () => {
    if (loadedShards.length === 0) return;
    const threshold = loadedShards[0]?.threshold ?? 3;
    if (loadedShards.length < threshold) {
      Alert.alert(t('sharding.shardsInsufficient', { more: threshold - loadedShards.length }), '');
      return;
    }
    setCombining(true);
    try {
      await KeePassExCore.combineVaultShards(loadedShards);
      setCombineSuccess(true);
      ReactNativeHapticFeedback.trigger('notificationSuccess');
    } catch (err) {
      Alert.alert(t('common.error'), t('sharding.combineFailed', { error: String(err) }));
    } finally {
      setCombining(false);
    }
  }, [loadedShards, t]);

  const handleAddShard = useCallback(async () => {
    // In production: use document picker to load .kpxshard file
    Alert.alert('Add Shard', 'Select a .kpxshard file from Files app');
  }, []);

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <ScrollView contentContainerStyle={styles.content}>
        {/* Header */}
        <View style={styles.header}>
          <Text style={[styles.title, { color: theme.text }]}>🔐 {t('sharding.title')}</Text>
          <Text style={[styles.subtitle, { color: theme.textSecondary }]}>
            {t('sharding.subtitle')}
          </Text>
          <View style={[styles.badge, { backgroundColor: theme.primary + '20' }]}>
            <Text style={[styles.badgeText, { color: theme.primary }]}>
              ✨ {t('sharding.uniqueFeature')}
            </Text>
          </View>
        </View>

        {/* Mode Tabs */}
        <View style={[styles.tabs, { backgroundColor: theme.backgroundSecondary }]}>
          <TouchableOpacity
            style={[styles.tab, mode === 'split' && { backgroundColor: theme.primary }]}
            onPress={() => setMode('split')}
            accessibilityRole="tab"
            accessibilityState={{ selected: mode === 'split' }}
          >
            <Text
              style={[styles.tabText, { color: mode === 'split' ? '#fff' : theme.textSecondary }]}
            >
              ✂️ {t('sharding.generateShards')}
            </Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.tab, mode === 'combine' && { backgroundColor: theme.primary }]}
            onPress={() => setMode('combine')}
            accessibilityRole="tab"
            accessibilityState={{ selected: mode === 'combine' }}
          >
            <Text
              style={[styles.tabText, { color: mode === 'combine' ? '#fff' : theme.textSecondary }]}
            >
              🔗 {t('sharding.combineShards')}
            </Text>
          </TouchableOpacity>
        </View>

        {/* SPLIT MODE */}
        {mode === 'split' && (
          <View style={styles.section}>
            {/* Threshold */}
            <View style={[styles.card, { backgroundColor: theme.surface }]}>
              <Text style={[styles.label, { color: theme.text }]}>
                {t('sharding.threshold')} (M)
              </Text>
              <Text style={[styles.hint, { color: theme.textSecondary }]}>
                {t('sharding.thresholdDesc')}
              </Text>
              <View style={styles.stepper}>
                <TouchableOpacity
                  style={[styles.stepBtn, { backgroundColor: theme.backgroundSecondary }]}
                  onPress={() => setThreshold(v => Math.max(2, v - 1))}
                  accessibilityLabel="Decrease threshold"
                >
                  <Text style={[styles.stepBtnText, { color: theme.text }]}>−</Text>
                </TouchableOpacity>
                <Text style={[styles.stepValue, { color: theme.text }]}>{threshold}</Text>
                <TouchableOpacity
                  style={[styles.stepBtn, { backgroundColor: theme.backgroundSecondary }]}
                  onPress={() => setThreshold(v => Math.min(total, v + 1))}
                  accessibilityLabel="Increase threshold"
                >
                  <Text style={[styles.stepBtnText, { color: theme.text }]}>+</Text>
                </TouchableOpacity>
              </View>
            </View>

            {/* Total */}
            <View style={[styles.card, { backgroundColor: theme.surface }]}>
              <Text style={[styles.label, { color: theme.text }]}>{t('sharding.total')} (N)</Text>
              <Text style={[styles.hint, { color: theme.textSecondary }]}>
                {t('sharding.totalDesc')}
              </Text>
              <View style={styles.stepper}>
                <TouchableOpacity
                  style={[styles.stepBtn, { backgroundColor: theme.backgroundSecondary }]}
                  onPress={() => setTotal(v => Math.max(threshold, v - 1))}
                  accessibilityLabel="Decrease total"
                >
                  <Text style={[styles.stepBtnText, { color: theme.text }]}>−</Text>
                </TouchableOpacity>
                <Text style={[styles.stepValue, { color: theme.text }]}>{total}</Text>
                <TouchableOpacity
                  style={[styles.stepBtn, { backgroundColor: theme.backgroundSecondary }]}
                  onPress={() => setTotal(v => Math.min(20, v + 1))}
                  accessibilityLabel="Increase total"
                >
                  <Text style={[styles.stepBtnText, { color: theme.text }]}>+</Text>
                </TouchableOpacity>
              </View>
            </View>

            {/* Example */}
            <View style={[styles.infoBox, { backgroundColor: theme.primary + '15' }]}>
              <Text style={[styles.infoText, { color: theme.primary }]}>
                ℹ️ {t('sharding.thresholdExample').replace('e.g. ', '')}
                {'\n'}Current: {threshold}-of-{total}
              </Text>
            </View>

            {/* Generate Button */}
            <TouchableOpacity
              style={[styles.primaryBtn, { backgroundColor: theme.primary }]}
              onPress={handleSplit}
              disabled={splitting}
              accessibilityRole="button"
              accessibilityLabel={t('sharding.generateShards')}
            >
              <Text style={styles.primaryBtnText}>
                {splitting ? t('sharding.generating') : `🔐 ${t('sharding.generateShards')}`}
              </Text>
            </TouchableOpacity>

            {/* Generated Shards */}
            {generatedShards.length > 0 && (
              <View style={styles.shardsSection}>
                <Text style={[styles.sectionTitle, { color: theme.text }]}>
                  ✅ {t('sharding.generateSuccess', { total })}
                </Text>
                {generatedShards.map(shard => (
                  <View
                    key={shard.index}
                    style={[styles.shardCard, { backgroundColor: theme.surface }]}
                  >
                    <View style={styles.shardHeader}>
                      <Text style={[styles.shardTitle, { color: theme.text }]}>
                        {t('sharding.shardLabel', { index: shard.index, total: shard.total })}
                      </Text>
                      <TouchableOpacity
                        style={[styles.shareBtn, { backgroundColor: theme.primary }]}
                        onPress={() => handleShareShard(shard)}
                        accessibilityRole="button"
                        accessibilityLabel={`Share shard ${shard.index}`}
                      >
                        <Text style={styles.shareBtnText}>📤 Share</Text>
                      </TouchableOpacity>
                    </View>
                    <Text
                      style={[styles.shardData, { color: theme.textTertiary }]}
                      numberOfLines={2}
                    >
                      {shard.data_b64.slice(0, 40)}...
                    </Text>
                  </View>
                ))}

                {/* Warning */}
                <View style={[styles.warningBox, { backgroundColor: '#FEF3C7' }]}>
                  <Text style={styles.warningText}>⚠️ {t('sharding.warningDesc')}</Text>
                </View>

                {/* Distribution suggestions */}
                <Text style={[styles.sectionTitle, { color: theme.text }]}>
                  {t('sharding.distributionSuggestions')}
                </Text>
                {[
                  t('sharding.suggestion1'),
                  t('sharding.suggestion2'),
                  t('sharding.suggestion3'),
                  t('sharding.suggestion4'),
                  t('sharding.suggestion5'),
                ].map((s, i) => (
                  <Text key={i} style={[styles.suggestion, { color: theme.textSecondary }]}>
                    {i + 1}. {s}
                  </Text>
                ))}
              </View>
            )}
          </View>
        )}

        {/* COMBINE MODE */}
        {mode === 'combine' && (
          <View style={styles.section}>
            <Text style={[styles.hint, { color: theme.textSecondary }]}>
              {t('sharding.combineShards')}
            </Text>

            {/* Loaded shards */}
            {loadedShards.map((shard, i) => (
              <View key={i} style={[styles.shardCard, { backgroundColor: theme.surface }]}>
                <View style={styles.shardHeader}>
                  <Text style={[styles.shardTitle, { color: theme.text }]}>
                    {t('sharding.shardLabel', { index: shard.index, total: shard.total })}
                  </Text>
                  <TouchableOpacity
                    onPress={() => setLoadedShards(prev => prev.filter((_, j) => j !== i))}
                    accessibilityRole="button"
                    accessibilityLabel={`Remove shard ${i + 1}`}
                  >
                    <Text style={{ color: '#ef4444', fontSize: 18 }}>✕</Text>
                  </TouchableOpacity>
                </View>
              </View>
            ))}

            {/* Status */}
            {loadedShards.length > 0 && (
              <View
                style={[
                  styles.infoBox,
                  {
                    backgroundColor:
                      loadedShards.length >= (loadedShards[0]?.threshold ?? 3)
                        ? '#dcfce7'
                        : '#fef9c3',
                  },
                ]}
              >
                <Text
                  style={{
                    color:
                      loadedShards.length >= (loadedShards[0]?.threshold ?? 3)
                        ? '#16a34a'
                        : '#d97706',
                  }}
                >
                  {loadedShards.length >= (loadedShards[0]?.threshold ?? 3)
                    ? `✅ ${t('sharding.shardsReady')}`
                    : `⏳ ${t('sharding.shardsLoaded', { count: loadedShards.length, threshold: loadedShards[0]?.threshold ?? 3 })}`}
                </Text>
              </View>
            )}

            <TouchableOpacity
              style={[styles.secondaryBtn, { borderColor: theme.primary }]}
              onPress={handleAddShard}
              accessibilityRole="button"
              accessibilityLabel={t('sharding.addShard')}
            >
              <Text style={[styles.secondaryBtnText, { color: theme.primary }]}>
                ➕ {t('sharding.addShard')}
              </Text>
            </TouchableOpacity>

            {combineSuccess ? (
              <View style={[styles.successBox, { backgroundColor: '#dcfce7' }]}>
                <Text style={styles.successText}>✅ {t('sharding.combineSuccess')}</Text>
              </View>
            ) : (
              <TouchableOpacity
                style={[
                  styles.primaryBtn,
                  { backgroundColor: theme.primary, opacity: loadedShards.length === 0 ? 0.5 : 1 },
                ]}
                onPress={handleCombine}
                disabled={combining || loadedShards.length === 0}
                accessibilityRole="button"
                accessibilityLabel={t('sharding.combineShards')}
              >
                <Text style={styles.primaryBtnText}>
                  {combining ? t('sharding.combining') : `🔓 ${t('sharding.combineShards')}`}
                </Text>
              </TouchableOpacity>
            )}
          </View>
        )}
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  content: { padding: tokens.space.lg, gap: tokens.space.md },
  header: { gap: tokens.space.xs },
  title: { fontSize: tokens.fontSize['2xl'], fontWeight: tokens.fontWeight.bold },
  subtitle: { fontSize: tokens.fontSize.sm },
  badge: {
    alignSelf: 'flex-start',
    paddingHorizontal: tokens.space.sm,
    paddingVertical: 2,
    borderRadius: tokens.radius.full,
  },
  badgeText: { fontSize: tokens.fontSize.xs, fontWeight: tokens.fontWeight.semibold },
  tabs: { flexDirection: 'row', borderRadius: tokens.radius.lg, padding: 4, gap: 4 },
  tab: {
    flex: 1,
    paddingVertical: tokens.space.sm,
    borderRadius: tokens.radius.md,
    alignItems: 'center',
  },
  tabText: { fontSize: tokens.fontSize.sm, fontWeight: tokens.fontWeight.medium },
  section: { gap: tokens.space.md },
  card: { borderRadius: tokens.radius.lg, padding: tokens.space.lg, gap: tokens.space.sm },
  label: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  hint: { fontSize: tokens.fontSize.sm },
  stepper: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: tokens.space.lg,
    justifyContent: 'center',
    marginTop: tokens.space.sm,
  },
  stepBtn: {
    width: 40,
    height: 40,
    borderRadius: 20,
    alignItems: 'center',
    justifyContent: 'center',
  },
  stepBtnText: { fontSize: 22, fontWeight: tokens.fontWeight.bold },
  stepValue: {
    fontSize: tokens.fontSize['2xl'],
    fontWeight: tokens.fontWeight.bold,
    minWidth: 40,
    textAlign: 'center',
  },
  infoBox: { borderRadius: tokens.radius.md, padding: tokens.space.md },
  infoText: { fontSize: tokens.fontSize.sm, lineHeight: 20 },
  primaryBtn: {
    borderRadius: tokens.radius.lg,
    padding: tokens.space.lg,
    alignItems: 'center',
    marginTop: tokens.space.sm,
  },
  primaryBtnText: {
    color: '#fff',
    fontSize: tokens.fontSize.md,
    fontWeight: tokens.fontWeight.semibold,
  },
  secondaryBtn: {
    borderRadius: tokens.radius.lg,
    padding: tokens.space.lg,
    alignItems: 'center',
    borderWidth: 1.5,
  },
  secondaryBtnText: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  shardsSection: { gap: tokens.space.md },
  sectionTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.semibold },
  shardCard: { borderRadius: tokens.radius.md, padding: tokens.space.md, gap: tokens.space.xs },
  shardHeader: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center' },
  shardTitle: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  shardData: { fontSize: 11, fontFamily: 'monospace' },
  shareBtn: {
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.xs,
    borderRadius: tokens.radius.md,
  },
  shareBtnText: {
    color: '#fff',
    fontSize: tokens.fontSize.xs,
    fontWeight: tokens.fontWeight.medium,
  },
  warningBox: { borderRadius: tokens.radius.md, padding: tokens.space.md },
  warningText: { fontSize: tokens.fontSize.sm, color: '#92400e', lineHeight: 20 },
  suggestion: { fontSize: tokens.fontSize.sm, paddingLeft: tokens.space.sm },
  successBox: { borderRadius: tokens.radius.md, padding: tokens.space.lg, alignItems: 'center' },
  successText: {
    fontSize: tokens.fontSize.lg,
    fontWeight: tokens.fontWeight.semibold,
    color: '#16a34a',
  },
});
