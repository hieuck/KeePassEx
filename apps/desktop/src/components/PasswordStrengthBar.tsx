/**
 * Password Strength Bar — inline strength indicator for password inputs
 * Shows real-time strength as user types
 */
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../store/settings';

interface PasswordStrengthBarProps {
  password: string;
  showLabel?: boolean;
  showEntropy?: boolean;
}

const STRENGTH_COLORS = ['#DC2626', '#EA580C', '#D97706', '#16A34A', '#059669'];
const STRENGTH_LABELS_EN = ['Very Weak', 'Weak', 'Fair', 'Strong', 'Very Strong'];
const STRENGTH_LABELS_VI = ['Rất yếu', 'Yếu', 'Trung bình', 'Mạnh', 'Rất mạnh'];

export function PasswordStrengthBar({
  password,
  showLabel = true,
  showEntropy = false,
}: PasswordStrengthBarProps) {
  const { settings } = useSettingsStore();
  const isVi = settings.language === 'vi';
  const [score, setScore] = useState(0);
  const [entropy, setEntropy] = useState(0);

  useEffect(() => {
    if (!password) {
      setScore(0);
      setEntropy(0);
      return;
    }

    // Use Tauri command for accurate scoring
    invoke<number>('score_strength', { password })
      .then(setScore)
      .catch(() => {
        // Fallback: simple client-side estimate
        setScore(estimateScore(password));
      });

    invoke<number>('estimate_entropy', { password })
      .then(setEntropy)
      .catch(() => {
        setEntropy(estimateEntropy(password));
      });
  }, [password]);

  if (!password) return null;

  const color = STRENGTH_COLORS[score] ?? STRENGTH_COLORS[0];
  const label = isVi ? STRENGTH_LABELS_VI[score] : STRENGTH_LABELS_EN[score];
  const width = `${(score + 1) * 20}%`;

  return (
    <div className="psb-container" aria-label={`Password strength: ${label}`}>
      <div className="psb-track" role="progressbar" aria-valuenow={score} aria-valuemin={0} aria-valuemax={4}>
        <div
          className="psb-fill"
          style={{ width, backgroundColor: color, transition: 'width 0.3s, background-color 0.3s' }}
        />
      </div>
      {(showLabel || showEntropy) && (
        <div className="psb-meta">
          {showLabel && (
            <span className="psb-label" style={{ color }}>{label}</span>
          )}
          {showEntropy && entropy > 0 && (
            <span className="psb-entropy">{entropy.toFixed(0)} bits</span>
          )}
        </div>
      )}

      <style>{`
        .psb-container { display: flex; flex-direction: column; gap: 4px; }
        .psb-track {
          height: 4px; background: var(--color-border);
          border-radius: var(--radius-full); overflow: hidden;
        }
        .psb-fill { height: 100%; border-radius: var(--radius-full); }
        .psb-meta { display: flex; justify-content: space-between; align-items: center; }
        .psb-label { font-size: 12px; font-weight: 600; }
        .psb-entropy { font-size: 11px; color: var(--color-text-tertiary); }
      `}</style>
    </div>
  );
}

// Client-side fallback estimators
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
