/**
 * OtpDisplay — Web component for showing a live OTP code with countdown ring
 * Used in desktop EntryDetailPage and browser extension popup
 *
 * Features:
 * - Live countdown (1s interval)
 * - Color change when ≤5s remaining
 * - SVG countdown ring
 * - Click to copy
 * - Accessible (ARIA live region)
 */
import type {} from 'react';

export interface OtpCode {
  code: string;
  remainingSeconds: number;
  period: number;
}

interface OtpDisplayProps {
  /** Current OTP state — caller is responsible for refreshing */
  otp: OtpCode | null;
  /** Called when user clicks copy */
  onCopy?: (code: string) => void;
  /** Whether the code was just copied (shows checkmark) */
  copied?: boolean;
  /** Compact mode — smaller size for list views */
  compact?: boolean;
  /** i18n label */
  label?: string;
  /** i18n "expires in" text */
  expiresInLabel?: string;
  /** i18n "copy" label */
  copyLabel?: string;
}

export function OtpDisplay({
  otp,
  onCopy,
  copied = false,
  compact = false,
  label = 'OTP',
  expiresInLabel = 'expires in',
  copyLabel = 'Copy OTP',
}: OtpDisplayProps) {
  if (!otp) return null;

  const { code, remainingSeconds, period } = otp;
  const isUrgent = remainingSeconds <= 5;
  const progress = remainingSeconds / Math.max(period, 1);
  const color = isUrgent ? 'var(--color-danger, #ef4444)' : 'var(--color-primary, #2563eb)';

  // SVG ring parameters
  const size = compact ? 28 : 36;
  const strokeWidth = compact ? 2.5 : 3;
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const dashOffset = circumference * (1 - progress);

  const formattedCode = code.length === 6 ? `${code.slice(0, 3)} ${code.slice(3)}` : code;

  return (
    <div
      className={`otp-display${compact ? ' otp-display--compact' : ''}`}
      role="region"
      aria-label={`${label}: ${formattedCode}, ${expiresInLabel} ${remainingSeconds}s`}
      aria-live="polite"
      aria-atomic="true"
    >
      {/* Code */}
      <span
        className="otp-display__code"
        style={{ color, fontFamily: "'SF Mono', 'Consolas', 'Courier New', monospace" }}
      >
        {formattedCode}
      </span>

      {/* Countdown ring */}
      <svg
        width={size}
        height={size}
        viewBox={`0 0 ${size} ${size}`}
        aria-hidden="true"
        className="otp-display__ring"
      >
        {/* Background track */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="var(--color-border, #e5e7eb)"
          strokeWidth={strokeWidth}
        />
        {/* Progress arc */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke={color}
          strokeWidth={strokeWidth}
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={dashOffset}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
          style={{ transition: 'stroke-dashoffset 1s linear, stroke 0.3s' }}
        />
        {/* Seconds text */}
        <text
          x={size / 2}
          y={size / 2}
          textAnchor="middle"
          dominantBaseline="central"
          fontSize={compact ? 9 : 11}
          fontWeight="600"
          fill={color}
        >
          {remainingSeconds}
        </text>
      </svg>

      {/* Copy button */}
      {onCopy && (
        <button
          className={`otp-display__copy${copied ? ' otp-display__copy--copied' : ''}`}
          onClick={() => onCopy(code)}
          aria-label={copyLabel}
          title={copyLabel}
        >
          {copied ? '✓' : '⎘'}
        </button>
      )}

      <style>{`
        .otp-display {
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 8px 12px;
          background: var(--color-bg-secondary, #f9fafb);
          border-radius: var(--radius-md, 8px);
          border: 1px solid var(--color-border, #e5e7eb);
        }
        .otp-display--compact {
          padding: 4px 8px;
          gap: 6px;
        }
        .otp-display__code {
          font-size: 22px;
          font-weight: 700;
          letter-spacing: 4px;
          flex: 1;
          user-select: all;
          cursor: text;
        }
        .otp-display--compact .otp-display__code {
          font-size: 15px;
          letter-spacing: 2px;
        }
        .otp-display__ring {
          flex-shrink: 0;
        }
        .otp-display__copy {
          background: none;
          border: none;
          cursor: pointer;
          color: var(--color-text-secondary, #6b7280);
          font-size: 16px;
          padding: 4px 6px;
          border-radius: var(--radius-sm, 4px);
          transition: background 0.1s, color 0.1s;
          flex-shrink: 0;
        }
        .otp-display__copy:hover {
          background: var(--color-bg-tertiary, #f3f4f6);
          color: var(--color-text, #111827);
        }
        .otp-display__copy--copied {
          color: var(--color-success, #16a34a);
        }
      `}</style>
    </div>
  );
}
