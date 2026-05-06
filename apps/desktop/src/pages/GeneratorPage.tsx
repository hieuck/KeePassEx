/**
 * Password generator page
 */
import React, { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useSettingsStore } from '../store/settings';
import { PasswordStrengthBar } from '../components/PasswordStrengthBar';

interface GenerateResult {
  password: string;
  entropy: number;
  strengthScore: number;
  strengthLabel: string;
}

const STRENGTH_COLORS = ['#DC2626', '#EA580C', '#D97706', '#16A34A', '#059669'];

export function GeneratorPage() {
  const { t } = useTranslation();
  const { settings } = useSettingsStore();

  const [mode, setMode] = useState<'random' | 'passphrase' | 'pronounceable'>('random');
  const [length, setLength] = useState(20);
  const [useUppercase, setUseUppercase] = useState(true);
  const [useLowercase, setUseLowercase] = useState(true);
  const [useDigits, setUseDigits] = useState(true);
  const [useSymbols, setUseSymbols] = useState(true);
  const [excludeAmbiguous, setExcludeAmbiguous] = useState(false);
  const [wordCount, setWordCount] = useState(6);
  const [wordSeparator, setWordSeparator] = useState('-');
  const [result, setResult] = useState<GenerateResult | null>(null);
  const [copied, setCopied] = useState(false);
  const [history, setHistory] = useState<string[]>([]);

  const generateMutation = useMutation({
    mutationFn: () =>
      invoke<GenerateResult>('generate_password', {
        args: {
          mode,
          length,
          use_uppercase: useUppercase,
          use_lowercase: useLowercase,
          use_digits: useDigits,
          use_symbols: useSymbols,
          custom_symbols: null,
          exclude_ambiguous: excludeAmbiguous,
          exclude_chars: '',
          min_uppercase: 1,
          min_lowercase: 1,
          min_digits: 1,
          min_symbols: 1,
          word_count: wordCount,
          word_separator: wordSeparator,
          capitalize_words: false,
          include_number: true,
        },
      }),
    onSuccess: data => {
      setResult(data);
      setHistory(prev => [data.password, ...prev.slice(0, 9)]);
    },
  });

  const handleCopy = async () => {
    if (!result) return;
    await invoke('copy_to_clipboard', {
      text: result.password,
      clearAfterSeconds: settings.clipboardClearSeconds ?? 10,
    });
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const MODES = {
    random: t('generator.modeRandom'),
    passphrase: t('generator.modePassphrase'),
    pronounceable: t('generator.modePronounce'),
  } as const;

  return (
    <div className="generator-page">
      <div className="generator-header">
        <h2>⚡ {t('generator.title')}</h2>
      </div>

      <div className="generator-content">
        {/* Output */}
        <div className="generator-output">
          <div className="password-display" aria-live="polite" aria-label="Generated password">
            {result ? (
              <span className="password-text">{result.password}</span>
            ) : (
              <span className="password-placeholder">{t('generator.generate')}</span>
            )}
          </div>

          {result && (
            <div className="password-meta">
              <PasswordStrengthBar password={result.password} showLabel showEntropy />
            </div>
          )}

          <div className="output-actions">
            <button
              className="btn btn-primary"
              onClick={() => generateMutation.mutate()}
              disabled={generateMutation.isPending}
            >
              {generateMutation.isPending ? '...' : `🎲 ${t('generator.generate')}`}
            </button>
            {result && (
              <button
                className={`btn btn-secondary ${copied ? 'btn-success' : ''}`}
                onClick={handleCopy}
                aria-label="Copy password"
              >
                {copied ? `✓ ${t('common.copied')}` : `⎘ ${t('common.copy')}`}
              </button>
            )}
          </div>
        </div>

        {/* Options */}
        <div className="generator-options">
          <div className="option-group">
            <label className="option-label">{t('generator.mode')}</label>
            <div className="mode-tabs" role="tablist">
              {(['random', 'passphrase', 'pronounceable'] as const).map(m => (
                <button
                  key={m}
                  role="tab"
                  aria-selected={mode === m}
                  className={`mode-tab ${mode === m ? 'active' : ''}`}
                  onClick={() => setMode(m)}
                >
                  {MODES[m]}
                </button>
              ))}
            </div>
          </div>

          {mode === 'random' && (
            <>
              <div className="option-group">
                <label className="option-label" htmlFor="pw-length">
                  {t('generator.length')}: <strong>{length}</strong>
                </label>
                <input
                  id="pw-length"
                  type="range"
                  min={8}
                  max={128}
                  value={length}
                  onChange={e => setLength(Number(e.target.value))}
                  className="range-input"
                  aria-valuemin={8}
                  aria-valuemax={128}
                  aria-valuenow={length}
                />
              </div>
              <div className="option-group">
                <label className="option-label">{t('generator.characters')}</label>
                <div className="checkboxes">
                  <CheckOption
                    id="use-uppercase"
                    label={t('generator.uppercase')}
                    checked={useUppercase}
                    onChange={setUseUppercase}
                  />
                  <CheckOption
                    id="use-lowercase"
                    label={t('generator.lowercase')}
                    checked={useLowercase}
                    onChange={setUseLowercase}
                  />
                  <CheckOption
                    id="use-digits"
                    label={t('generator.digits')}
                    checked={useDigits}
                    onChange={setUseDigits}
                  />
                  <CheckOption
                    id="use-symbols"
                    label={t('generator.symbols')}
                    checked={useSymbols}
                    onChange={setUseSymbols}
                  />
                  <CheckOption
                    id="exclude-ambiguous"
                    label={t('generator.excludeAmbiguous')}
                    checked={excludeAmbiguous}
                    onChange={setExcludeAmbiguous}
                  />
                </div>
              </div>
            </>
          )}

          {mode === 'passphrase' && (
            <>
              <div className="option-group">
                <label className="option-label" htmlFor="word-count">
                  {t('generator.wordCount')}: <strong>{wordCount}</strong>
                </label>
                <input
                  id="word-count"
                  type="range"
                  min={3}
                  max={12}
                  value={wordCount}
                  onChange={e => setWordCount(Number(e.target.value))}
                  className="range-input"
                />
              </div>
              <div className="option-group">
                <label className="option-label" htmlFor="word-sep">
                  {t('generator.wordSeparator')}
                </label>
                <input
                  id="word-sep"
                  type="text"
                  className="form-input"
                  value={wordSeparator}
                  onChange={e => setWordSeparator(e.target.value)}
                  maxLength={3}
                  style={{ width: 60 }}
                />
              </div>
            </>
          )}
        </div>

        {/* History */}
        {history.length > 0 && (
          <div className="generator-history">
            <h3 className="history-title">{t('entry.history')}</h3>
            {history.map((pw, i) => (
              <div key={i} className="history-item">
                <span className="history-password">{pw}</span>
                <button
                  className="btn-icon"
                  onClick={() => invoke('copy_to_clipboard', { text: pw, clearAfterSeconds: 10 })}
                  aria-label="Copy"
                >
                  ⎘
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      <style>{`
        .generator-page { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
        .generator-header { padding: var(--space-md) var(--space-xl); border-bottom: 1px solid var(--color-border); flex-shrink: 0; }
        .generator-header h2 { font-size: 16px; font-weight: 600; }
        .generator-content { flex: 1; overflow-y: auto; padding: var(--space-xl); display: flex; flex-direction: column; gap: var(--space-xl); max-width: 560px; }
        .generator-output { background: var(--color-bg-secondary); border: 1px solid var(--color-border); border-radius: var(--radius-lg); padding: var(--space-xl); display: flex; flex-direction: column; gap: var(--space-md); }
        .password-display { min-height: 48px; display: flex; align-items: center; }
        .password-text { font-family: 'SF Mono', 'Consolas', monospace; font-size: 18px; font-weight: 600; color: var(--color-text); word-break: break-all; letter-spacing: 1px; }
        .password-placeholder { color: var(--color-text-tertiary); font-size: 14px; }
        .password-meta { display: flex; align-items: center; gap: var(--space-md); }
        .output-actions { display: flex; gap: var(--space-sm); }
        .btn-success { background: var(--color-success) !important; color: white !important; }
        .generator-options { display: flex; flex-direction: column; gap: var(--space-lg); }
        .option-group { display: flex; flex-direction: column; gap: var(--space-sm); }
        .option-label { font-size: 13px; font-weight: 500; color: var(--color-text-secondary); }
        .mode-tabs { display: flex; gap: 4px; background: var(--color-bg-tertiary); padding: 3px; border-radius: var(--radius-md); }
        .mode-tab { flex: 1; padding: var(--space-xs) var(--space-sm); border: none; background: none; border-radius: calc(var(--radius-md) - 2px); font-size: 13px; cursor: pointer; color: var(--color-text-secondary); transition: background 0.15s, color 0.15s; }
        .mode-tab.active { background: var(--color-surface); color: var(--color-text); font-weight: 500; }
        .checkboxes { display: flex; flex-direction: column; gap: var(--space-sm); }
        .range-input { width: 100%; accent-color: var(--color-primary); }
        .generator-history { display: flex; flex-direction: column; gap: var(--space-sm); }
        .history-title { font-size: 13px; font-weight: 500; color: var(--color-text-secondary); }
        .history-item { display: flex; align-items: center; gap: var(--space-sm); padding: var(--space-sm) var(--space-md); background: var(--color-bg-secondary); border-radius: var(--radius-sm); }
        .history-password { flex: 1; font-family: 'SF Mono', 'Consolas', monospace; font-size: 13px; color: var(--color-text-secondary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
      `}</style>
    </div>
  );
}

function CheckOption({
  id,
  label,
  checked,
  onChange,
}: {
  id: string;
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label htmlFor={id} className="check-option">
      <input
        id={id}
        type="checkbox"
        checked={checked}
        onChange={e => onChange(e.target.checked)}
        style={{ accentColor: 'var(--color-primary)' }}
      />
      <span>{label}</span>
      <style>{`.check-option { display: flex; align-items: center; gap: var(--space-sm); font-size: 13px; cursor: pointer; color: var(--color-text); }`}</style>
    </label>
  );
}
