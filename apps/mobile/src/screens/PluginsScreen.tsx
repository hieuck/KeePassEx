/**
 * Plugins screen — mobile (with full i18n EN/VI)
 */
import React, { useState } from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, Switch, Alert } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { NativeModules } from 'react-native';
import { useThemeStore } from '../store/theme';
import { useTranslation } from '../store/i18n';
import { tokens } from '@keepassex/ui';
import type { InstalledPlugin } from '@keepassex/types';

const { KeePassExCore } = NativeModules;

const CATALOG = [
  {
    id: 'com.keepassex.importer.dashlane',
    name: 'Dashlane Importer',
    desc: 'Import from Dashlane CSV',
    cap: '📥 Importer',
  },
  {
    id: 'com.keepassex.importer.enpass',
    name: 'Enpass Importer',
    desc: 'Import from Enpass JSON',
    cap: '📥 Importer',
  },
  {
    id: 'com.keepassex.generator.diceware',
    name: 'Diceware Generator',
    desc: 'Diceware passphrase generator',
    cap: '⚡ Generator',
  },
];

export function PluginsScreen() {
  const { theme } = useThemeStore();
  const { t } = useTranslation();
  const navigation = useNavigation();
  const queryClient = useQueryClient();
  const [tab, setTab] = useState<'installed' | 'catalog'>('installed');

  const { data: plugins = [], isLoading } = useQuery<InstalledPlugin[]>({
    queryKey: ['plugins'],
    queryFn: () => KeePassExCore.listPlugins(),
    staleTime: 30_000,
  });

  const toggleMutation = useMutation({
    mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) =>
      KeePassExCore.togglePlugin(id, enabled),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['plugins'] }),
  });

  const uninstallMutation = useMutation({
    mutationFn: (id: string) => KeePassExCore.uninstallPlugin(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['plugins'] }),
  });

  return (
    <SafeAreaView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <TouchableOpacity
          onPress={() => navigation.goBack()}
          accessibilityRole="button"
          accessibilityLabel={t('common.back')}
        >
          <Text style={[styles.backBtn, { color: theme.primary }]}>← {t('common.back')}</Text>
        </TouchableOpacity>
        <Text style={[styles.headerTitle, { color: theme.text }]}>🔧 {t('plugins.title')}</Text>
        <View style={{ width: 50 }} />
      </View>

      <View style={[styles.tabs, { borderBottomColor: theme.border }]}>
        {(['installed', 'catalog'] as const).map(tabId => (
          <TouchableOpacity
            key={tabId}
            style={[
              styles.tab,
              tab === tabId && { borderBottomColor: theme.primary, borderBottomWidth: 2 },
            ]}
            onPress={() => setTab(tabId)}
            accessibilityRole="tab"
            accessibilityState={{ selected: tab === tabId }}
          >
            <Text
              style={[
                styles.tabText,
                { color: tab === tabId ? theme.primary : theme.textSecondary },
              ]}
            >
              {tabId === 'installed'
                ? `${t('plugins.installed')} (${plugins.length})`
                : `${t('plugins.catalog')} (${CATALOG.length})`}
            </Text>
          </TouchableOpacity>
        ))}
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.contentInner}>
        {tab === 'installed' ? (
          plugins.length === 0 ? (
            <View style={styles.emptyState}>
              <Text style={styles.emptyIcon}>🔧</Text>
              <Text style={[styles.emptyTitle, { color: theme.text }]}>
                {t('plugins.noPlugins')}
              </Text>
              <Text style={[styles.emptyDesc, { color: theme.textSecondary }]}>
                {t('plugins.noPluginsDesc')}
              </Text>
            </View>
          ) : (
            plugins.map(plugin => (
              <View
                key={plugin.manifest.id}
                style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}
              >
                <View style={styles.cardHeader}>
                  <View style={styles.cardInfo}>
                    <Text style={[styles.cardName, { color: theme.text }]}>
                      {plugin.manifest.name}
                    </Text>
                    <Text style={[styles.cardAuthor, { color: theme.textSecondary }]}>
                      v{plugin.manifest.version} by {plugin.manifest.author}
                    </Text>
                  </View>
                  <Switch
                    value={plugin.enabled}
                    onValueChange={v =>
                      toggleMutation.mutate({ id: plugin.manifest.id, enabled: v })
                    }
                    trackColor={{ false: theme.border, true: theme.primary }}
                    thumbColor="white"
                    accessibilityLabel={`Toggle ${plugin.manifest.name}`}
                  />
                </View>
                <Text style={[styles.cardDesc, { color: theme.textSecondary }]}>
                  {plugin.manifest.description}
                </Text>
                <TouchableOpacity
                  style={[styles.uninstallBtn, { borderColor: tokens.color.danger }]}
                  onPress={() =>
                    Alert.alert(
                      t('plugins.uninstall'),
                      t('plugins.confirmUninstall', { name: plugin.manifest.name }),
                      [
                        { text: t('common.cancel'), style: 'cancel' },
                        {
                          text: t('plugins.uninstall'),
                          style: 'destructive',
                          onPress: () => uninstallMutation.mutate(plugin.manifest.id),
                        },
                      ]
                    )
                  }
                  accessibilityRole="button"
                  accessibilityLabel={t('plugins.uninstall')}
                >
                  <Text style={[styles.uninstallBtnText, { color: tokens.color.danger }]}>
                    {t('plugins.uninstall')}
                  </Text>
                </TouchableOpacity>
              </View>
            ))
          )
        ) : (
          CATALOG.map(item => (
            <View
              key={item.id}
              style={[styles.card, { backgroundColor: theme.surface, borderColor: theme.border }]}
            >
              <View style={styles.cardHeader}>
                <View style={styles.cardInfo}>
                  <Text style={[styles.cardName, { color: theme.text }]}>{item.name}</Text>
                  <Text style={[styles.cardCap, { color: theme.textSecondary }]}>{item.cap}</Text>
                </View>
                <TouchableOpacity
                  style={[styles.installBtn, { backgroundColor: theme.primary }]}
                  onPress={() =>
                    Alert.alert(t('plugins.install'), `${t('plugins.install')}: "${item.name}"?`, [
                      { text: t('common.cancel'), style: 'cancel' },
                      { text: t('plugins.install'), onPress: () => {} },
                    ])
                  }
                  accessibilityRole="button"
                  accessibilityLabel={`${t('plugins.install')} ${item.name}`}
                >
                  <Text style={styles.installBtnText}>{t('plugins.install')}</Text>
                </TouchableOpacity>
              </View>
              <Text style={[styles.cardDesc, { color: theme.textSecondary }]}>{item.desc}</Text>
            </View>
          ))
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
  headerTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.semibold },
  tabs: {
    flexDirection: 'row',
    borderBottomWidth: StyleSheet.hairlineWidth,
  },
  tab: {
    flex: 1,
    paddingVertical: tokens.space.sm,
    alignItems: 'center',
    borderBottomWidth: 2,
    borderBottomColor: 'transparent',
  },
  tabText: { fontSize: tokens.fontSize.sm, fontWeight: tokens.fontWeight.medium },
  content: { flex: 1 },
  contentInner: { padding: tokens.space.lg, gap: tokens.space.md },
  emptyState: { alignItems: 'center', paddingVertical: tokens.space['2xl'], gap: tokens.space.md },
  emptyIcon: { fontSize: 48 },
  emptyTitle: { fontSize: tokens.fontSize.lg, fontWeight: tokens.fontWeight.semibold },
  emptyDesc: { fontSize: tokens.fontSize.sm, textAlign: 'center', lineHeight: 20 },
  card: {
    borderRadius: tokens.radius.lg,
    borderWidth: StyleSheet.hairlineWidth,
    padding: tokens.space.md,
    gap: tokens.space.sm,
  },
  cardHeader: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    justifyContent: 'space-between',
    gap: tokens.space.md,
  },
  cardInfo: { flex: 1 },
  cardName: { fontSize: tokens.fontSize.md, fontWeight: tokens.fontWeight.semibold },
  cardAuthor: { fontSize: tokens.fontSize.xs, marginTop: 2 },
  cardCap: { fontSize: tokens.fontSize.xs, marginTop: 2 },
  cardDesc: { fontSize: tokens.fontSize.sm, lineHeight: 18 },
  uninstallBtn: {
    borderWidth: 1,
    borderRadius: tokens.radius.sm,
    paddingVertical: tokens.space.xs,
    paddingHorizontal: tokens.space.md,
    alignSelf: 'flex-start',
  },
  uninstallBtnText: { fontSize: tokens.fontSize.sm, fontWeight: tokens.fontWeight.medium },
  installBtn: {
    borderRadius: tokens.radius.sm,
    paddingVertical: tokens.space.xs,
    paddingHorizontal: tokens.space.md,
  },
  installBtnText: {
    color: 'white',
    fontSize: tokens.fontSize.sm,
    fontWeight: tokens.fontWeight.semibold,
  },
});
