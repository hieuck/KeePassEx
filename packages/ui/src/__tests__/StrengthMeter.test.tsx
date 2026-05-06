/**
 * StrengthMeter component tests
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { StrengthMeter } from '../components/StrengthMeter';

describe('StrengthMeter', () => {
  it('renders with score 0 (very weak)', () => {
    render(<StrengthMeter score={0} label="Very Weak" showLabel />);
    expect(screen.getByText('Very Weak')).toBeTruthy();
  });

  it('renders with score 4 (very strong)', () => {
    render(<StrengthMeter score={4} label="Very Strong" showLabel />);
    expect(screen.getByText('Very Strong')).toBeTruthy();
  });

  it('renders without label when showLabel is false', () => {
    render(<StrengthMeter score={3} label="Strong" showLabel={false} />);
    expect(screen.queryByText('Strong')).toBeNull();
  });

  it('renders compact variant', () => {
    const { container } = render(<StrengthMeter score={2} compact />);
    expect(container).toBeTruthy();
  });
});
