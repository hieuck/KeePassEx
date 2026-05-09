/**
 * KeePassEx Mobile App
 */
import React, { useEffect } from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { GestureHandlerRootView } from 'react-native-gesture-handler';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AppState, StatusBar, Text } from 'react-native';
import { useVaultStore } from './store/vault';
import { useThemeStore } from './store/theme';
import { useI18nStore } from './store/i18n';
import { useMobileSettingsStore } from './store/settings';

// Screens
import { WelcomeScreen } from './screens/WelcomeScreen';
import { UnlockScreen } from './screens/UnlockScreen';
import { VaultScreen } from './screens/VaultScreen';
import { EntryDetailScreen } from './screens/EntryDetailScreen';
import { EntryEditScreen } from './screens/EntryEditScreen';
import { GeneratorScreen } from './screens/GeneratorScreen';
import { HealthScreen } from './screens/HealthScreen';
import { BreachScreen } from './screens/BreachScreen';
import { SettingsScreen } from './screens/SettingsScreen';
import { OtpScreen } from './screens/OtpScreen';
import { SyncScreen } from './screens/SyncScreen';
import { ImportExportScreen } from './screens/ImportExportScreen';
import { EmergencyAccessScreen } from './screens/EmergencyAccessScreen';
import { PluginsScreen } from './screens/PluginsScreen';
import { AnalyticsScreen } from './screens/AnalyticsScreen';
import { ShardingScreen } from './screens/ShardingScreen';
import { GroupsScreen } from './screens/GroupsScreen';
import { ChangePasswordScreen } from './screens/ChangePasswordScreen';
import { EntryHistoryScreen } from './screens/EntryHistoryScreen';
import { SecureNoteScreen } from './screens/SecureNoteScreen';

export type RootStackParamList = {
  Welcome: undefined;
  Unlock: { vaultPath: string };
  Main: undefined;
  EntryDetail: { uuid: string };
  EntryEdit: { uuid?: string; groupUuid?: string };
  Generator: { onSelect?: (password: string) => void };
  OtpDetail: { entryUuid: string };
  Sync: undefined;
  ImportExport: undefined;
  Breach: undefined;
  EmergencyAccess: undefined;
  Plugins: undefined;
  Analytics: undefined;
  Sharding: undefined;
  Groups: undefined;
  ChangePassword: undefined;
  EntryHistory: { entryUuid: string; entryTitle: string };
  SecureNote: { entryUuid?: string };
};

export type TabParamList = {
  Vault: undefined;
  Health: undefined;
  Generator: undefined;
  Settings: undefined;
};

const Stack = createNativeStackNavigator<RootStackParamList>();
const Tab = createBottomTabNavigator<TabParamList>();

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { staleTime: 30_000, retry: 1 },
  },
});

function TabIcon({ icon }: { icon: string }) {
  return <Text style={{ fontSize: 20 }}>{icon}</Text>;
}

function MainTabs() {
  const { theme } = useThemeStore();
  const { t } = useI18nStore();

  return (
    <Tab.Navigator
      screenOptions={{
        tabBarStyle: {
          backgroundColor: theme.tabBar,
          borderTopColor: theme.tabBarBorder,
        },
        tabBarActiveTintColor: theme.primary,
        tabBarInactiveTintColor: theme.textTertiary,
        headerShown: false,
      }}
    >
      <Tab.Screen
        name="Vault"
        component={VaultScreen}
        options={{
          tabBarLabel: t('vault.open'),
          tabBarIcon: () => <TabIcon icon="🔑" />,
          tabBarAccessibilityLabel: t('vault.open'),
        }}
      />
      <Tab.Screen
        name="Health"
        component={HealthScreen}
        options={{
          tabBarLabel: t('health.title'),
          tabBarIcon: () => <TabIcon icon="🛡️" />,
          tabBarAccessibilityLabel: t('health.title'),
        }}
      />
      <Tab.Screen
        name="Generator"
        component={GeneratorScreen}
        options={{
          tabBarLabel: t('generator.title'),
          tabBarIcon: () => <TabIcon icon="⚡" />,
          tabBarAccessibilityLabel: t('generator.title'),
        }}
      />
      <Tab.Screen
        name="Settings"
        component={SettingsScreen}
        options={{
          tabBarLabel: t('settings.title'),
          tabBarIcon: () => <TabIcon icon="⚙️" />,
          tabBarAccessibilityLabel: t('settings.title'),
        }}
      />
    </Tab.Navigator>
  );
}

export function App() {
  const { isOpen, isLocked, lockVault } = useVaultStore();
  const { theme } = useThemeStore();
  const { init: initI18n } = useI18nStore();
  const { settings } = useMobileSettingsStore();

  // Initialize i18n with saved language preference
  useEffect(() => {
    initI18n(settings.language ?? 'en');
  }, [settings.language]);

  // Lock vault when app goes to background
  useEffect(() => {
    const subscription = AppState.addEventListener('change', nextState => {
      if (nextState === 'background' || nextState === 'inactive') {
        if (settings.lockOnBackground !== false) {
          lockVault();
        }
      }
    });
    return () => subscription.remove();
  }, [lockVault, settings.lockOnBackground]);

  return (
    <GestureHandlerRootView style={{ flex: 1 }}>
      <SafeAreaProvider>
        <QueryClientProvider client={queryClient}>
          <StatusBar barStyle={theme.statusBar} backgroundColor={theme.background} />
          <NavigationContainer
            theme={{
              dark: theme.mode !== 'light',
              colors: {
                primary: theme.primary,
                background: theme.background,
                card: theme.surface,
                text: theme.text,
                border: theme.border,
                notification: theme.danger,
              },
            }}
          >
            <Stack.Navigator screenOptions={{ headerShown: false }}>
              {!isOpen ? (
                <>
                  <Stack.Screen name="Welcome" component={WelcomeScreen} />
                  <Stack.Screen name="Unlock" component={UnlockScreen} />
                </>
              ) : isLocked ? (
                <Stack.Screen name="Unlock" component={UnlockScreen} />
              ) : (
                <>
                  <Stack.Screen name="Main" component={MainTabs} />
                  <Stack.Screen
                    name="EntryDetail"
                    component={EntryDetailScreen}
                    options={{ presentation: 'card' }}
                  />
                  <Stack.Screen
                    name="EntryEdit"
                    component={EntryEditScreen}
                    options={{ presentation: 'modal' }}
                  />
                  <Stack.Screen
                    name="Generator"
                    component={GeneratorScreen}
                    options={{ presentation: 'modal' }}
                  />
                  <Stack.Screen
                    name="OtpDetail"
                    component={OtpScreen}
                    options={{ presentation: 'card' }}
                  />
                  <Stack.Screen
                    name="Sync"
                    component={SyncScreen}
                    options={{ presentation: 'modal' }}
                  />
                  <Stack.Screen
                    name="ImportExport"
                    component={ImportExportScreen}
                    options={{ presentation: 'modal' }}
                  />
                  <Stack.Screen
                    name="Breach"
                    component={BreachScreen}
                    options={{ presentation: 'card' }}
                  />
                  <Stack.Screen
                    name="EmergencyAccess"
                    component={EmergencyAccessScreen}
                    options={{ presentation: 'modal' }}
                  />
                  <Stack.Screen
                    name="Plugins"
                    component={PluginsScreen}
                    options={{ presentation: 'modal' }}
                  />
                  <Stack.Screen
                    name="Analytics"
                    component={AnalyticsScreen}
                    options={{ presentation: 'card' }}
                  />
                  <Stack.Screen
                    name="Sharding"
                    component={ShardingScreen}
                    options={{ presentation: 'modal' }}
                  />
                  <Stack.Screen
                    name="Groups"
                    component={GroupsScreen}
                    options={{ presentation: 'modal' }}
                  />
                  <Stack.Screen
                    name="ChangePassword"
                    component={ChangePasswordScreen}
                    options={{ presentation: 'modal' }}
                  />
                  <Stack.Screen
                    name="EntryHistory"
                    component={EntryHistoryScreen}
                    options={{ presentation: 'card' }}
                  />
                  <Stack.Screen
                    name="SecureNote"
                    component={SecureNoteScreen}
                    options={{ presentation: 'modal' }}
                  />
                </>
              )}
            </Stack.Navigator>
          </NavigationContainer>
        </QueryClientProvider>
      </SafeAreaProvider>
    </GestureHandlerRootView>
  );
}
