/**
 * PasswordField — Web component for password input with strength meter,
 * show/hide toggle, copy button, and generate button
 *
 * Used in desktop EntryDetailPage and generator
 */
import { useState } from 'react';

export type StrengthScore = 0 | 1 | 2 | 3 | 4;

interface PasswordFieldProps {
  value: string;
  onChange?: (value: string) => void;
  onCopy?: () => void;
  onGenerate?: () => void;
  /** Whether the field is in read-only (view) mode */
  readOnly?: boolean;
  /** Whether the password was just copied */
  copied?: boolean;
  /** Password strength score (0–4) */
  strengthScore?: StrengthScore;
  /** Strength label text */
  strengthLabel?: string;
  /** Entropy in bits */
  entropy?: number;
  placeholder?: string;
  /** i18n labels */
  showLabel?: string;
  hideLabel?: string;
  copyLabel?: string;
  generateLabel?: string;
  strengthText?: string;
  entropyText?: string;
}

const STRENGTH_COLORS: Record<StrengthScore, string> = {
  0: '#dc2626',
  1: '#ea580c',
  2: '#d97706',
  3: '#16a34a',
  4: '#059669',
};

const STRENGTH_LABELS: Record<StrengthScore, string> = {
  0: 'Very Weak',
  1: 'Weak',
  2: 'Fair',
  3: 'Strong',
  4: 'Very Strong',
};

export function PasswordField({
  value,
  onChange,
  onCopy,
  onGenerate,
  readOnly = false,
  copied = false,
  strengthScore,
  strengthLabel,
  entropy,
  placeholder = 'Password',
  showLabel = 'Show password',
  hideLabel = 'Hide password',
  copyLabel = 'Copy password',
  generateLabel = 'Generate password',
  strengthText = 'Strength',
  entropyText = 'Entropy',
}: PasswordFieldProps) {
  const [visible, setVisible] = useState(false);

  const displayValue = readOnly ? (visible ? value : '••••••••••••') : value;

  const color = strengthScore !== undefined ? STRENGTH_COLORS[strengthScore] : undefined;
  const label =
    strengthLabel ?? (strengthScore !== undefined ? STRENGTH_LABELS[strengthScore] : undefined);

  return (
    <div className="pw-field">
      {/* Input row */}
      <div className="pw-field__row">
        {readOnly ? (
          <span
            className="pw-field__value"
            style={{ fontFamily: visible ? "'SF Mono', 'Consolas', monospace" : 'inherit' }}
            aria-label={visible ? 'Password visible' : 'Password hidden'}
          >
            {displayValue}
          </span>
        ) : (
          <input
            type={visible ? 'text' : 'password'}
            className="pw-field__input form-input"
            value={value}
            onChange={e => onChange?.(e.target.value)}
            placeholder={placeholder}
            autoComplete="new-password"
            spellCheck={false}
            aria-label={placeholder}
          />
        )}

        <div className="pw-field__actions">
          {/* Show/hide toggle */}
          <button
            type="button"
            className="pw-field__btn"
            onClick={() => setVisible(v => !v)}
            aria-label={visible ? hideLabel : showLabel}
            title={visible ? hideLabel : showLabel}
          >
            {visible ? '🙈' : '👁'}
          </button>

          {/* Copy */}
          {onCopy && (
            <button
              type="button"
              className={`pw-field__btn${copied ? ' pw-field__btn--copied' : ''}`}
              onClick={onCopy}
              aria-label={copyLabel}
              title={copyLabel}
            >
              {copied ? '✓' : '⎘'}
            </button>
          )}

          {/* Generate */}
          {onGenerate && !readOnly && (
            <button
              type="button"
              className="pw-field__btn"
              onClick={onGenerate}
              aria-label={generateLabel}
              title={generateLabel}
            >
              ⚡
            </button>
          )}
        </div>
      </div>

      {/* Strength meter */}
      {strengthScore !== undefined && (
        <div className="pw-field__strength" aria-label={`${strengthText}: ${label}`}>
          <div className="pw-field__bars">
            {[0, 1, 2, 3].map(i => (
              <div
                key={i}
                className="pw-field__bar"
                style={{
                  background: i < strengthScore ? color : 'var(--color-border, #e5e7eb)',
                  transition: 'background 0.2s',
                }}
              />
            ))}
          </div>
          <span className="pw-field__strength-label" style={{ color }}>
            {label}
          </span>
          {entropy !== undefined && (
            <span className="pw-field__entropy">
              {entropyText}: {Math.round(entropy)} bits
            </span>
          )}
        </div>
      )}

      <style>{`
        .pw-field { display: flex; flex-direction: column; gap: 6px; }
        .pw-field__row {
          display: flex; align-items: center; gap: 6px;
          background: var(--color-bg, #fff);
          border: 1px solid var(--color-border, #e5e7eb);
          border-radius: var(--radius-sm, 6px);
          padding: 0 6px 0 0;
          overflow: hidden;
        }
        .pw-field__row:focus-within {
          border-color: var(--color-primary, #2563eb);
          box-shadow: 0 0 0 2px rgba(37,99,235,0.15);
        }
        .pw-field__input {
          flex: 1; border: none; outline: none;
          padding: var(--space-sm, 8px) var(--space-md, 12px);
          font-size: 14px; background: transparent;
          font-family: 'SF Mono', 'Consolas', monospace;
          color: var(--color-text, #111827);
        }
        .pw-field__value {
          flex: 1; padding: var(--space-sm, 8px) var(--space-md, 12px);
          font-size: 14px; color: var(--color-text, #111827);
          word-break: break-all;
        }
        .pw-field__actions { display: flex; gap: 2px; flex-shrink: 0; }
        .pw-field__btn {
          background: none; border: none; cursor: pointer;
          color: var(--color-text-secondary, #6b7280);
          font-size: 14px; padding: 4px 6px;
          border-radius: var(--radius-sm, 4px);
          transition: background 0.1s, color 0.1s;
        }
        .pw-field__btn:hover {
          background: var(--color-bg-secondary, #f9fafb);
          color: var(--color-text, #111827);
        }
        .pw-field__btn--copied { color: var(--color-success, #16a34a); }
        .pw-field__strength {
          display: flex; align-items: center; gap: 8px;
        }
        .pw-field__bars {
          display: flex; gap: 3px; flex: 1; max-width: 120px;
        }
        .pw-field__bar {
          flex: 1; height: 4px; border-radius: 2px;
        }
        .pw-field__strength-label {
          font-size: 12px; font-weight: 500; white-space: nowrap;
        }
        .pw-field__entropy {
          font-size: 11px; color: var(--color-text-tertiary, #9ca3af);
          margin-left: auto;
        }
      `}</style>
    </div>
  );
}
