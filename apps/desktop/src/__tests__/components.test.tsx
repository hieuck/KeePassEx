/**
 * Desktop component tests
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { CustomFieldEditor } from '../components/CustomFieldEditor';
import { SearchBar } from '../components/SearchBar';
import { EntryRow } from '../components/EntryRow';

// Mock settings store
vi.mock('../store/settings', () => ({
  useSettingsStore: () => ({
    settings: { language: 'en', clipboardClearSeconds: 10 },
  }),
}));

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

// ─── CustomFieldEditor ────────────────────────────────────────────────────────

describe('CustomFieldEditor', () => {
  it('renders empty state in read-only mode', () => {
    const { container } = render(
      <CustomFieldEditor fields={[]} onChange={() => {}} readOnly />
    );
    // Empty read-only should render nothing
    expect(container.firstChild).toBeNull();
  });

  it('shows add button in edit mode', () => {
    render(<CustomFieldEditor fields={[]} onChange={() => {}} />);
    expect(screen.getByText(/Add Custom Field/i)).toBeTruthy();
  });

  it('renders existing fields', () => {
    const fields = [
      { key: 'API Key', value: 'secret123', protected: false },
      { key: 'Token', value: 'tok_abc', protected: true },
    ];
    render(<CustomFieldEditor fields={fields} onChange={() => {}} readOnly />);
    expect(screen.getByText('API Key')).toBeTruthy();
    expect(screen.getByText('Token')).toBeTruthy();
  });

  it('calls onChange when adding a field', () => {
    const onChange = vi.fn();
    render(<CustomFieldEditor fields={[]} onChange={onChange} />);

    fireEvent.click(screen.getByText(/Add Custom Field/i));
    expect(onChange).toHaveBeenCalledWith(
      expect.arrayContaining([
        expect.objectContaining({ key: expect.stringContaining('Field') }),
      ])
    );
  });

  it('shows protected field as masked in read-only', () => {
    const fields = [{ key: 'Secret', value: 'hidden_value', protected: true }];
    render(<CustomFieldEditor fields={fields} onChange={() => {}} readOnly />);
    expect(screen.getByText('••••••••')).toBeTruthy();
  });
});

// ─── SearchBar ────────────────────────────────────────────────────────────────

describe('SearchBar', () => {
  it('renders with placeholder', () => {
    render(<SearchBar placeholder="Search..." onChange={() => {}} />);
    expect(screen.getByPlaceholderText('Search...')).toBeTruthy();
  });

  it('calls onChange when typing', async () => {
    const onChange = vi.fn();
    render(<SearchBar placeholder="Search..." onChange={onChange} debounceMs={0} />);

    const input = screen.getByRole('searchbox');
    fireEvent.change(input, { target: { value: 'github' } });

    // Wait for debounce
    await new Promise(r => setTimeout(r, 10));
    expect(onChange).toHaveBeenCalledWith('github');
  });

  it('shows clear button when has value', () => {
    render(<SearchBar placeholder="Search..." onChange={() => {}} />);
    const input = screen.getByRole('searchbox');
    fireEvent.change(input, { target: { value: 'test' } });
    expect(screen.getByLabelText('Clear search')).toBeTruthy();
  });

  it('clears value when clear button clicked', () => {
    const onChange = vi.fn();
    render(<SearchBar placeholder="Search..." onChange={onChange} debounceMs={0} />);
    const input = screen.getByRole('searchbox');
    fireEvent.change(input, { target: { value: 'test' } });

    const clearBtn = screen.getByLabelText('Clear search');
    fireEvent.click(clearBtn);

    expect(onChange).toHaveBeenLastCalledWith('');
  });
});

// ─── EntryRow ─────────────────────────────────────────────────────────────────

describe('EntryRow', () => {
  const mockEntry = {
    uuid: '550e8400-e29b-41d4-a716-446655440000',
    groupUuid: 'root',
    title: 'GitHub',
    username: 'user@example.com',
    url: 'https://github.com',
    notes: '',
    iconId: 0,
    tags: [],
    hasPassword: true,
    hasOtp: false,
    hasPasskey: false,
    hasSshKey: false,
    hasAttachments: false,
    isExpired: false,
    createdAt: new Date().toISOString(),
    modifiedAt: new Date().toISOString(),
    customFields: [],
  };

  it('renders entry title and username', () => {
    render(
      <EntryRow
        entry={mockEntry}
        onPress={() => {}}
        onCopyPassword={() => {}}
      />
    );
    expect(screen.getByText('GitHub')).toBeTruthy();
    expect(screen.getByText('user@example.com')).toBeTruthy();
  });

  it('calls onPress when clicked', () => {
    const onPress = vi.fn();
    render(
      <EntryRow
        entry={mockEntry}
        onPress={onPress}
        onCopyPassword={() => {}}
      />
    );
    fireEvent.click(screen.getByRole('button', { name: /GitHub/i }));
    expect(onPress).toHaveBeenCalledWith(mockEntry.uuid);
  });

  it('shows OTP badge when entry has OTP', () => {
    const entryWithOtp = { ...mockEntry, hasOtp: true };
    render(
      <EntryRow
        entry={entryWithOtp}
        onPress={() => {}}
        onCopyPassword={() => {}}
      />
    );
    expect(screen.getByText('OTP')).toBeTruthy();
  });

  it('shows expired styling for expired entries', () => {
    const expiredEntry = { ...mockEntry, isExpired: true };
    render(
      <EntryRow
        entry={expiredEntry}
        onPress={() => {}}
        onCopyPassword={() => {}}
      />
    );
    const title = screen.getByText('GitHub');
    expect(title.className).toContain('expired');
  });
});
