/**
 * Mobile screen tests
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react-native';

// Mock React Navigation
vi.mock('@react-navigation/native', () => ({
  useNavigation: () => ({
    navigate: vi.fn(),
    goBack: vi.fn(),
  }),
  useRoute: () => ({
    params: {},
  }),
}));

// Mock NativeModules
vi.mock('react-native', async () => {
  const actual = await vi.importActual('react-native');
  return {
    ...actual,
    NativeModules: {
      KeePassExCore: {
        getEntries: vi.fn().mockResolvedValue([]),
        getEntry: vi.fn().mockResolvedValue(null),
        searchEntries: vi.fn().mockResolvedValue([]),
        generateTotp: vi.fn().mockResolvedValue({ code: '123456', remainingSeconds: 25, period: 30 }),
        auditVault: vi.fn().mockResolvedValue({ score: 85, totalEntries: 10, weakCount: 0, reusedCount: 0, expiredCount: 0, expiringSoonCount: 0 }),
        generatePassword: vi.fn().mockResolvedValue({ password: 'TestP@ss123!', entropy: 65, strengthScore: 3, strengthLabel: 'Strong' }),
      },
    },
  };
});

// Mock stores
vi.mock('../store/theme', () => ({
  useThemeStore: () => ({
    theme: {
      background: '#FFFFFF',
      surface: '#F9FAFB',
      text: '#111827',
      textSecondary: '#6B7280',
      textTertiary: '#9CA3AF',
      primary: '#2563EB',
      border: '#E5E7EB',
      tabBar: '#FFFFFF',
      tabBarBorder: '#E5E7EB',
      statusBar: 'dark-content',
      mode: 'light',
      backgroundSecondary: '#F9FAFB',
      backgroundTertiary: '#F3F4F6',
      danger: '#DC2626',
      success: '#16A34A',
      warning: '#D97706',
    },
    mode: 'light',
    setMode: vi.fn(),
  }),
}));

vi.mock('../store/vault', () => ({
  useVaultStore: () => ({
    isOpen: true,
    isLocked: false,
    meta: { name: 'Test Vault', entryCount: 5, groupCount: 2, path: '/test.kdbx' },
    selectedGroupUuid: null,
    searchQuery: '',
    lockVault: vi.fn(),
    setSelectedGroup: vi.fn(),
    setSearchQuery: vi.fn(),
  }),
}));

vi.mock('@tanstack/react-query', () => ({
  useQuery: vi.fn(() => ({ data: [], isLoading: false, refetch: vi.fn() })),
  useMutation: vi.fn(() => ({
    mutate: vi.fn(),
    mutateAsync: vi.fn(),
    isPending: false,
    isError: false,
    error: null,
    data: null,
  })),
  useQueryClient: vi.fn(() => ({
    invalidateQueries: vi.fn(),
  })),
}));

vi.mock('@react-native-clipboard/clipboard', () => ({
  default: { setString: vi.fn() },
}));

vi.mock('react-native-haptic-feedback', () => ({
  default: { trigger: vi.fn() },
}));

// ─── VaultScreen ──────────────────────────────────────────────────────────────

import { VaultScreen } from '../screens/VaultScreen';

describe('VaultScreen', () => {
  it('renders without crashing', () => {
    const { getByText } = render(<VaultScreen />);
    expect(getByText('Vault')).toBeTruthy();
  });

  it('shows empty state when no entries', () => {
    const { getByText } = render(<VaultScreen />);
    expect(getByText(/No entries/i)).toBeTruthy();
  });

  it('renders add button', () => {
    const { getByLabelText } = render(<VaultScreen />);
    expect(getByLabelText('Add new entry')).toBeTruthy();
  });

  it('renders search bar', () => {
    const { getByLabelText } = render(<VaultScreen />);
    expect(getByLabelText('Search entries')).toBeTruthy();
  });
});

// ─── HealthScreen ─────────────────────────────────────────────────────────────

import { HealthScreen } from '../screens/HealthScreen';

describe('HealthScreen', () => {
  it('renders without crashing', () => {
    const { getByText } = render(<HealthScreen />);
    expect(getByText(/Health/i)).toBeTruthy();
  });

  it('shows health score when data loaded', async () => {
    const { useQuery } = await import('@tanstack/react-query');
    vi.mocked(useQuery).mockReturnValue({
      data: {
        score: 85,
        totalEntries: 10,
        weakCount: 1,
        reusedCount: 0,
        expiredCount: 0,
        expiringSoonCount: 2,
        weakPasswords: [],
        reusedPasswords: [],
        expiredEntries: [],
        expiringSoon: [],
      },
      isLoading: false,
      refetch: vi.fn(),
    } as ReturnType<typeof useQuery>);

    const { getByText } = render(<HealthScreen />);
    expect(getByText('85')).toBeTruthy();
  });
});

// ─── GeneratorScreen ──────────────────────────────────────────────────────────

import { GeneratorScreen } from '../screens/GeneratorScreen';

describe('GeneratorScreen', () => {
  it('renders without crashing', () => {
    const { getByText } = render(<GeneratorScreen />);
    expect(getByText('Generator')).toBeTruthy();
  });

  it('shows generate button', () => {
    const { getByLabelText } = render(<GeneratorScreen />);
    expect(getByLabelText('Generate password')).toBeTruthy();
  });

  it('shows mode selector', () => {
    const { getByText } = render(<GeneratorScreen />);
    expect(getByText('Random')).toBeTruthy();
    expect(getByText('Passphrase')).toBeTruthy();
  });
});

// ─── UnlockScreen ─────────────────────────────────────────────────────────────

import { UnlockScreen } from '../screens/UnlockScreen';

describe('UnlockScreen', () => {
  it('renders without crashing', () => {
    const { getByText } = render(<UnlockScreen />);
    expect(getByText('KeePassEx')).toBeTruthy();
  });

  it('shows password input', () => {
    const { getByLabelText } = render(<UnlockScreen />);
    expect(getByLabelText('Master password')).toBeTruthy();
  });

  it('shows unlock button', () => {
    const { getByLabelText } = render(<UnlockScreen />);
    expect(getByLabelText('Unlock vault')).toBeTruthy();
  });
});

// ─── SettingsScreen ───────────────────────────────────────────────────────────

import { SettingsScreen } from '../screens/SettingsScreen';

describe('SettingsScreen', () => {
  it('renders without crashing', () => {
    const { getByText } = render(<SettingsScreen />);
    expect(getByText(/Settings/i)).toBeTruthy();
  });

  it('shows theme options', () => {
    const { getByText } = render(<SettingsScreen />);
    expect(getByText('Light')).toBeTruthy();
    expect(getByText('Dark')).toBeTruthy();
  });

  it('shows tools section', () => {
    const { getByText } = render(<SettingsScreen />);
    expect(getByText(/Sync/i)).toBeTruthy();
    expect(getByText(/Import/i)).toBeTruthy();
  });
});
