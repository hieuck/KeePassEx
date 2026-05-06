/**
 * EntryHistoryViewer — Web component for viewing entry history
 * Shows a timeline of changes with diff highlighting
 *
 * Used in desktop EntryDetailPage
 */
import React, { useState } from 'react';

export interface HistoryEntry {
  uuid: string;
  modifiedAt: string;
  title: string;
  username: string;
  url: string;
  notes: string;
  hasPassword: boolean;
}

interface EntryHistoryViewerProps {
  history: HistoryEntry[];
  onRestore?: (historyEntry: HistoryEntry) => void;
  onClearHistory?: () => void;
  /** i18n labels */
  historyLabel?: string;
  noHistoryLabel?: string;
  restoreLabel?: string;
  clearLabel?: string;
  confirmClearLabel?: string;
  modifiedLabel?: string;
  changedFieldsLabel?: string;
}

export function EntryHistoryViewer({
  history,
  onRestore,
  onClearHistory,
  historyLabel = 'History',
  noHistoryLabel = 'No history entries',
  restoreLabel = 'Restore',
  clearLabel = 'Clear History',
  confirmClearLabel = 'Clear all history entries?',
  modifiedLabel = 'Modified',
  changedFieldsLabel = 'Changed',
}: EntryHistoryViewerProps) {
  const [expanded, setExpanded] = useState<string | null>(null);

  if (history.length === 0) {
    return (
      <div className="history-empty">
        <span>📋</span>
        <p>{noHistoryLabel}</p>
      </div>
    );
  }

  const getDiff = (current: HistoryEntry, previous: HistoryEntry | undefined): string[] => {
    if (!previous) return [];
    const changed: string[] = [];
    if (current.title !== previous.title) changed.push('title');
    if (current.username !== previous.username) changed.push('username');
    if (current.url !== previous.url) changed.push('url');
    if (current.notes !== previous.notes) changed.push('notes');
    if (current.hasPassword !== previous.hasPassword) changed.push('password');
    return changed;
  };

  return (
    <div className="history-viewer">
      <div className="history-header">
        <span className="history-title">{historyLabel} ({history.length})</span>
        {onClearHistory && (
          <button
            className="history-clear-btn"
            onClick={() => {
              if (confirm(confirmClearLabel)) onClearHistory();
            }}
          >
            {clearLabel}
          </button>
        )}
      </div>

      <div className="history-list">
        {history.map((entry, idx) => {
          const previous = history[idx + 1];
          const changed = getDiff(entry, previous);
          const isExpanded = expanded === entry.uuid;

          return (
            <div key={entry.uuid} className="history-item">
              <button
                className="history-item-header"
                onClick={() => setExpanded(isExpanded ? null : entry.uuid)}
                aria-expanded={isExpanded}
              >
                <span className="history-item-date">
                  {new Date(entry.modifiedAt).toLocaleString()}
                </span>
                {changed.length > 0 && (
                  <span className="history-item-changed">
                    {changedFieldsLabel}: {changed.join(', ')}
                  </span>
                )}
                <span className="history-item-chevron">{isExpanded ? '▲' : '▼'}</span>
              </button>

              {isExpanded && (
                <div className="history-item-detail">
                  <HistoryField label="Title" value={entry.title} changed={changed.includes('title')} />
                  <HistoryField label="Username" value={entry.username} changed={changed.includes('username')} />
                  <HistoryField label="URL" value={entry.url} changed={changed.includes('url')} />
                  {entry.notes && (
                    <HistoryField label="Notes" value={entry.notes} changed={changed.includes('notes')} multiline />
                  )}
                  <HistoryField
                    label="Password"
                    value={entry.hasPassword ? '••••••••' : '(none)'}
                    changed={changed.includes('password')}
                  />

                  {onRestore && (
                    <button
                      className="history-restore-btn"
                      onClick={() => onRestore(entry)}
                    >
                      ↩ {restoreLabel}
                    </button>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>

      <style>{`
        .history-viewer { display: flex; flex-direction: column; gap: 4px; }
        .history-header {
          display: flex; align-items: center; justify-content: space-between;
          margin-bottom: 4px;
        }
        .history-title { font-size: 13px; font-weight: 500; color: var(--color-text-secondary); }
        .history-clear-btn {
          background: none; border: none; cursor: pointer;
          color: var(--color-danger, #ef4444); font-size: 12px;
          padding: 2px 6px; border-radius: 4px;
        }
        .history-clear-btn:hover { background: rgba(239,68,68,0.08); }
        .history-list { display: flex; flex-direction: column; gap: 2px; }
        .history-item {
          border: 1px solid var(--color-border, #e5e7eb);
          border-radius: var(--radius-sm, 6px);
          overflow: hidden;
        }
        .history-item-header {
          display: flex; align-items: center; gap: 8px;
          width: 100%; padding: 8px 12px;
          background: var(--color-bg-secondary, #f9fafb);
          border: none; cursor: pointer; text-align: left;
          font-size: 13px;
        }
        .history-item-header:hover { background: var(--color-bg-tertiary, #f3f4f6); }
        .history-item-date { color: var(--color-text, #111827); font-weight: 500; }
        .history-item-changed {
          flex: 1; font-size: 11px; color: var(--color-text-secondary, #6b7280);
        }
        .history-item-chevron { color: var(--color-text-tertiary, #9ca3af); font-size: 10px; }
        .history-item-detail {
          padding: 12px; display: flex; flex-direction: column; gap: 8px;
          background: var(--color-bg, #fff);
        }
        .history-field { display: flex; flex-direction: column; gap: 2px; }
        .history-field-label {
          font-size: 11px; font-weight: 500; color: var(--color-text-secondary);
          text-transform: uppercase; letter-spacing: 0.05em;
        }
        .history-field-value {
          font-size: 13px; color: var(--color-text);
          padding: 4px 8px; border-radius: 4px;
        }
        .history-field-value--changed {
          background: rgba(234,179,8,0.1);
          border-left: 2px solid #eab308;
        }
        .history-field-value--multiline { white-space: pre-wrap; }
        .history-restore-btn {
          align-self: flex-start;
          background: var(--color-primary, #2563eb); color: white;
          border: none; border-radius: var(--radius-sm, 6px);
          padding: 6px 12px; font-size: 12px; cursor: pointer;
          margin-top: 4px;
        }
        .history-restore-btn:hover { background: var(--color-primary-dark, #1d4ed8); }
        .history-empty {
          display: flex; align-items: center; gap: 8px;
          color: var(--color-text-tertiary, #9ca3af); font-size: 13px;
          padding: 12px;
        }
      `}</style>
    </div>
  );
}

function HistoryField({
  label, value, changed, multiline = false,
}: {
  label: string; value: string; changed: boolean; multiline?: boolean;
}) {
  return (
    <div className="history-field">
      <span className="history-field-label">{label}</span>
      <span className={`history-field-value${changed ? ' history-field-value--changed' : ''}${multiline ? ' history-field-value--multiline' : ''}`}>
        {value || '—'}
      </span>
    </div>
  );
}
