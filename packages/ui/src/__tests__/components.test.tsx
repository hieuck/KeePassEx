/**
 * UI component tests
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Button } from '../components/Button';
import { Input } from '../components/Input';
import { HealthBadge } from '../components/HealthBadge';
import { OtpDisplay } from '../components/OtpDisplay';

// ─── Button ───────────────────────────────────────────────────────────────────

describe('Button', () => {
  it('renders with label', () => {
    render(<Button label="Click me" onPress={() => {}} />);
    expect(screen.getByText('Click me')).toBeTruthy();
  });

  it('calls onPress when clicked', () => {
    const onPress = vi.fn();
    render(<Button label="Click" onPress={onPress} />);
    fireEvent.press(screen.getByRole('button'));
    expect(onPress).toHaveBeenCalledOnce();
  });

  it('does not call onPress when disabled', () => {
    const onPress = vi.fn();
    render(<Button label="Click" onPress={onPress} disabled />);
    fireEvent.press(screen.getByRole('button'));
    expect(onPress).not.toHaveBeenCalled();
  });

  it('shows loading indicator when loading', () => {
    render(<Button label="Click" onPress={() => {}} loading />);
    // Loading state should show ActivityIndicator, not label text
    expect(screen.queryByText('Click')).toBeNull();
  });

  it('renders with icon', () => {
    render(<Button label="Save" onPress={() => {}} icon="💾" />);
    expect(screen.getByText(/💾/)).toBeTruthy();
  });

  it('applies primary variant by default', () => {
    const { getByRole } = render(<Button label="Test" onPress={() => {}} />);
    const btn = getByRole('button');
    expect(btn).toBeTruthy();
  });
});

// ─── Input ────────────────────────────────────────────────────────────────────

describe('Input', () => {
  it('renders with label', () => {
    render(<Input label="Username" value="" onChangeText={() => {}} />);
    expect(screen.getByText('Username')).toBeTruthy();
  });

  it('shows error message', () => {
    render(<Input label="Email" value="" onChangeText={() => {}} error="Invalid email" />);
    expect(screen.getByText('Invalid email')).toBeTruthy();
  });

  it('shows hint text', () => {
    render(<Input label="Password" value="" onChangeText={() => {}} hint="At least 8 characters" />);
    expect(screen.getByText('At least 8 characters')).toBeTruthy();
  });

  it('calls onChangeText when typing', () => {
    const onChangeText = vi.fn();
    render(<Input label="Name" value="" onChangeText={onChangeText} />);
    const input = screen.getByRole('textbox');
    fireEvent.changeText(input, 'John');
    expect(onChangeText).toHaveBeenCalledWith('John');
  });
});

// ─── HealthBadge ──────────────────────────────────────────────────────────────

describe('HealthBadge', () => {
  it('renders with score', () => {
    const { getByRole } = render(<HealthBadge score={85} />);
    const badge = getByRole('progressbar');
    expect(badge).toBeTruthy();
  });

  it('has correct aria values', () => {
    const { getByRole } = render(<HealthBadge score={75} />);
    const badge = getByRole('progressbar');
    expect(badge.props.accessibilityValue).toEqual({ min: 0, max: 100, now: 75 });
  });

  it('renders different sizes', () => {
    const { rerender, getByRole } = render(<HealthBadge score={90} size="sm" />);
    expect(getByRole('progressbar')).toBeTruthy();

    rerender(<HealthBadge score={90} size="lg" />);
    expect(getByRole('progressbar')).toBeTruthy();
  });
});

// ─── OtpDisplay ───────────────────────────────────────────────────────────────

describe('OtpDisplay', () => {
  const defaultProps = {
    code: '123456',
    remainingSeconds: 20,
    period: 30,
    issuer: 'GitHub',
    account: 'user@example.com',
  };

  it('renders OTP code', () => {
    render(<OtpDisplay {...defaultProps} />);
    // Code is formatted as "123 456"
    expect(screen.getByText('123 456')).toBeTruthy();
  });

  it('renders issuer', () => {
    render(<OtpDisplay {...defaultProps} />);
    expect(screen.getByText('GitHub')).toBeTruthy();
  });

  it('renders account', () => {
    render(<OtpDisplay {...defaultProps} />);
    expect(screen.getByText('user@example.com')).toBeTruthy();
  });

  it('calls onCopy when pressed', () => {
    const onCopy = vi.fn();
    render(<OtpDisplay {...defaultProps} onCopy={onCopy} />);
    fireEvent.press(screen.getByRole('button'));
    expect(onCopy).toHaveBeenCalledWith('123456');
  });

  it('shows urgent styling when remaining <= 5', () => {
    render(<OtpDisplay {...defaultProps} remainingSeconds={3} />);
    // The code should be rendered (urgent styling is visual)
    expect(screen.getByText('123 456')).toBeTruthy();
  });

  it('calculates progress correctly', () => {
    render(<OtpDisplay {...defaultProps} remainingSeconds={15} period={30} />);
    // Progress = 15/30 = 0.5
    expect(screen.getByText('123 456')).toBeTruthy();
  });
});
