/**
 * Command Palette — Cmd/Ctrl+K
 */
import React, { useState, useEffect, useRef, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useVaultStore, type EntryDto } from '../store/vault';

interface Command {
  id: string;
  icon: string;
  label: string;
  action: () => void;
  keywords: string[];
}

interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
}

export function CommandPalette({ open, onClose }: CommandPaletteProps) {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();
  const { t } = useTranslation();
  const { lockVault } = useVaultStore();

  const { data: entries = [] } = useQuery({
    queryKey: ['palette-search', query],
    queryFn: () =>
      query.length >= 2 ? invoke<EntryDto[]>('search_entries', { query }) : Promise.resolve([]),
    enabled: query.length >= 2,
  });

  const staticCommands: Command[] = [
    {
      id: 'new-entry',
      icon: '➕',
      label: t('entry.new'),
      action: () => {
        navigate('/vault/entry/new');
        onClose();
      },
      keywords: ['new', 'add', 'create', 'entry'],
    },
    {
      id: 'generator',
      icon: '⚡',
      label: t('generator.title'),
      action: () => {
        navigate('/generator');
        onClose();
      },
      keywords: ['generate', 'password', 'generator'],
    },
    {
      id: 'health',
      icon: '🛡️',
      label: t('health.title'),
      action: () => {
        navigate('/health');
        onClose();
      },
      keywords: ['health', 'audit', 'weak'],
    },
    {
      id: 'breach',
      icon: '🔍',
      label: t('breach.title'),
      action: () => {
        navigate('/breach');
        onClose();
      },
      keywords: ['breach', 'hibp', 'pwned'],
    },
    {
      id: 'import',
      icon: '📥',
      label: `${t('common.import')} / ${t('common.export')}`,
      action: () => {
        navigate('/import-export');
        onClose();
      },
      keywords: ['import', 'export', 'bitwarden', 'lastpass'],
    },
    {
      id: 'sync',
      icon: '🔄',
      label: t('sync.title'),
      action: () => {
        navigate('/sync');
        onClose();
      },
      keywords: ['sync', 'webdav', 'cloud'],
    },
    {
      id: 'emergency-access',
      icon: '🆘',
      label: t('emergencyAccess.title'),
      action: () => {
        navigate('/emergency-access');
        onClose();
      },
      keywords: ['emergency', 'access', 'trusted'],
    },
    {
      id: 'plugins',
      icon: '🔧',
      label: t('plugins.title'),
      action: () => {
        navigate('/plugins');
        onClose();
      },
      keywords: ['plugin', 'extension', 'wasm'],
    },
    {
      id: 'settings',
      icon: '⚙️',
      label: t('settings.title'),
      action: () => {
        navigate('/settings');
        onClose();
      },
      keywords: ['settings', 'preferences'],
    },
    {
      id: 'lock',
      icon: '🔒',
      label: t('vault.lock'),
      action: () => {
        lockVault();
        onClose();
      },
      keywords: ['lock'],
    },
  ];

  const filteredCommands =
    query.length < 2
      ? staticCommands
      : staticCommands.filter(cmd => {
          const q = query.toLowerCase();
          return cmd.label.toLowerCase().includes(q) || cmd.keywords.some(k => k.includes(q));
        });

  const entryResults = entries.slice(0, 5);
  const totalItems = filteredCommands.length + entryResults.length;

  useEffect(() => {
    if (open) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedIndex(i => Math.min(i + 1, totalItems - 1));
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedIndex(i => Math.max(i - 1, 0));
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (selectedIndex < filteredCommands.length) filteredCommands[selectedIndex].action();
        else {
          const entry = entryResults[selectedIndex - filteredCommands.length];
          if (entry) {
            navigate(`/vault/entry/${entry.uuid}`);
            onClose();
          }
        }
      } else if (e.key === 'Escape') onClose();
    },
    [selectedIndex, filteredCommands, entryResults, navigate, onClose]
  );

  if (!open) return null;

  return (
    <div
      className="palette-overlay"
      onClick={e => {
        if (e.target === e.currentTarget) onClose();
      }}
      role="dialog"
      aria-modal="true"
      aria-label="Command palette"
    >
      <div className="palette-container">
        <div className="palette-search">
          <span className="palette-search-icon" aria-hidden="true">
            ⌘
          </span>
          <input
            ref={inputRef}
            type="text"
            className="palette-input"
            placeholder={t('common.search')}
            value={query}
            onChange={e => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            aria-label={t('common.search')}
            aria-autocomplete="list"
            aria-controls="palette-results"
          />
          <kbd className="palette-esc">ESC</kbd>
        </div>

        <div id="palette-results" className="palette-results" role="listbox">
          {filteredCommands.length > 0 && (
            <div className="palette-group">
              <p className="palette-group-label">Commands</p>
              {filteredCommands.map((cmd, i) => (
                <button
                  key={cmd.id}
                  role="option"
                  aria-selected={selectedIndex === i}
                  className={`palette-item ${selectedIndex === i ? 'selected' : ''}`}
                  onClick={cmd.action}
                  onMouseEnter={() => setSelectedIndex(i)}
                >
                  <span className="palette-item-icon">{cmd.icon}</span>
                  <span className="palette-item-label">{cmd.label}</span>
                </button>
              ))}
            </div>
          )}
          {entryResults.length > 0 && (
            <div className="palette-group">
              <p className="palette-group-label">
                {t('vault.statistics_entries', { count: '' }).trim()}
              </p>
              {entryResults.map((entry, i) => {
                const idx = filteredCommands.length + i;
                return (
                  <button
                    key={entry.uuid}
                    role="option"
                    aria-selected={selectedIndex === idx}
                    className={`palette-item ${selectedIndex === idx ? 'selected' : ''}`}
                    onClick={() => {
                      navigate(`/vault/entry/${entry.uuid}`);
                      onClose();
                    }}
                    onMouseEnter={() => setSelectedIndex(idx)}
                  >
                    <span className="palette-item-icon">🔑</span>
                    <div className="palette-item-entry">
                      <span className="palette-item-label">{entry.title}</span>
                      <span className="palette-item-sub">{entry.username}</span>
                    </div>
                  </button>
                );
              })}
            </div>
          )}
          {totalItems === 0 && <p className="palette-empty">{t('entry.noEntries')}</p>}
        </div>

        <div className="palette-footer">
          <span>
            <kbd>↑↓</kbd> navigate
          </span>
          <span>
            <kbd>↵</kbd> select
          </span>
          <span>
            <kbd>ESC</kbd> close
          </span>
        </div>
      </div>

      <style>{`
        .palette-overlay { position:fixed; inset:0; background:rgba(0,0,0,.5); display:flex; align-items:flex-start; justify-content:center; padding-top:15vh; z-index:9999; }
        .palette-container { width:100%; max-width:560px; background:var(--color-surface); border:1px solid var(--color-border); border-radius:var(--radius-lg); box-shadow:0 24px 64px rgba(0,0,0,.2); overflow:hidden; }
        .palette-search { display:flex; align-items:center; gap:var(--space-md); padding:var(--space-md) var(--space-lg); border-bottom:1px solid var(--color-border); }
        .palette-search-icon { font-size:16px; color:var(--color-text-secondary); flex-shrink:0; }
        .palette-input { flex:1; border:none; background:none; outline:none; font-size:16px; color:var(--color-text); }
        .palette-input::placeholder { color:var(--color-text-tertiary); }
        .palette-esc { background:var(--color-bg-tertiary); border:1px solid var(--color-border); border-radius:4px; padding:2px 6px; font-size:11px; color:var(--color-text-secondary); flex-shrink:0; }
        .palette-results { max-height:360px; overflow-y:auto; }
        .palette-group { padding:var(--space-xs) 0; }
        .palette-group-label { font-size:11px; font-weight:600; text-transform:uppercase; letter-spacing:.08em; color:var(--color-text-tertiary); padding:var(--space-xs) var(--space-lg); }
        .palette-item { display:flex; align-items:center; gap:var(--space-md); width:100%; padding:var(--space-sm) var(--space-lg); background:none; border:none; cursor:pointer; text-align:left; transition:background .1s; }
        .palette-item:hover, .palette-item.selected { background:var(--color-bg-secondary); }
        .palette-item-icon { font-size:16px; flex-shrink:0; width:20px; text-align:center; }
        .palette-item-label { font-size:14px; color:var(--color-text); }
        .palette-item-entry { display:flex; flex-direction:column; gap:1px; }
        .palette-item-sub { font-size:12px; color:var(--color-text-secondary); }
        .palette-empty { padding:var(--space-xl); text-align:center; font-size:14px; color:var(--color-text-secondary); }
        .palette-footer { display:flex; gap:var(--space-lg); padding:var(--space-sm) var(--space-lg); border-top:1px solid var(--color-border); font-size:12px; color:var(--color-text-tertiary); }
        .palette-footer kbd { background:var(--color-bg-tertiary); border:1px solid var(--color-border); border-radius:3px; padding:1px 4px; font-size:10px; }
      `}</style>
    </div>
  );
}
