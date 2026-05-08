/**
 * Password Strength Bar — inline strength indicator
 */
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';

interface PasswordStrengthBarProps {
  password: string;
  showLabel?: boolean;
  showEntropy?: boolean;
}

const STRENGTH_COLORS = ['#DC2626', '#EA580C', '#D97706', '#16A34A', '#059669'];
const STRENGTH_KEYS = [
  'generator.strengthVeryWeak',
  'generator.strengthWeak',
  'generator.strengthFair',
  'generator.strengthStrong',
  'generator.strengthVeryStrong',
];

export function PasswordStrengthBar({
  password,
  showLabel = true,
  showEntropy = false,
}: PasswordStrengthBarProps) {
  const { t } = useTranslation();
  const [score, setScore] = useState(0);
  const [entropy, setEntropy] = useState(0);

  useEffect(() => {
    if (!password) {
      setScore(0);
      setEntropy(0);
      return;
    }
    invoke<number>('score_strength', { password })
      .then(setScore)
      .catch(() => setScore(estimateScore(password)));
    invoke<number>('estimate_entropy', { password })
      .then(setEntropy)
      .catch(() => setEntropy(estimateEntropy(password)));
  }, [password]);

  if (!password) return null;

  const color = STRENGTH_COLORS[score] ?? STRENGTH_COLORS[0];
  const label = t(STRENGTH_KEYS[score] ?? STRENGTH_KEYS[0]);
  const width = `${(score + 1) * 20}%`;

  return (
    <div className="psb-container" aria-label={`Password strength: ${label}`}>
      <div
        className="psb-track"
        role="progressbar"
        aria-valuenow={score}
        aria-valuemin={0}
        aria-valuemax={4}
      >
        <div
          className="psb-fill"
          style={{ width, backgroundColor: color, transition: 'width 0.3s, background-color 0.3s' }}
        />
      </div>
      {(showLabel || showEntropy) && (
        <div className="psb-meta">
          {showLabel && (
            <span className="psb-label" style={{ color }}>
              {label}
            </span>
          )}
          {showEntropy && entropy > 0 && (
            <span className="psb-entropy">{entropy.toFixed(0)} bits</span>
          )}
        </div>
      )}
      <style>{`
        .psb-container { display:flex; flex-direction:column; gap:4px; }
        .psb-track { height:4px; background:var(--color-border); border-radius:var(--radius-full); overflow:hidden; }
        .psb-fill { height:100%; border-radius:var(--radius-full); }
        .psb-meta { display:flex; justify-content:space-between; align-items:center; }
        .psb-label { font-size:12px; font-weight:600; }
        .psb-entropy { font-size:11px; color:var(--color-text-tertiary); }
      `}</style>
    </div>
  );
}

function estimateScore(password: string): number {
  const entropy = estimateEntropy(password);
  if (entropy < 28) return 0;
  if (entropy < 36) return 1;
  if (entropy < 60) return 2;
  if (entropy < 80) return 3;
  return 4;
}

function estimateEntropy(password: string): number {
  let charsetSize = 0;
  if (/[a-z]/.test(password)) charsetSize += 26;
  if (/[A-Z]/.test(password)) charsetSize += 26;
  if (/[0-9]/.test(password)) charsetSize += 10;
  if (/[^a-zA-Z0-9]/.test(password)) charsetSize += 32;
  if (charsetSize === 0) return 0;
  return password.length * Math.log2(charsetSize);
}
