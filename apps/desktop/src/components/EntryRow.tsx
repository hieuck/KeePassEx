/**
 * Entry row for the vault list
 */
import React, { useState } from 'react';
import type { EntryDto } from '../store/vault';

interface EntryRowProps {
  entry: EntryDto;
  onPress: (uuid: string, e?: React.MouseEvent) => void;
  onCopyPassword: (uuid: string) => void;
}

export function EntryRow({ entry, onPress, onCopyPassword }: EntryRowProps) {
  const [copyFeedback, setCopyFeedback] = useState(false);

  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation();
    onCopyPassword(entry.uuid);
    setCopyFeedback(true);
    setTimeout(() => setCopyFeedback(false), 1500);
  };

  const handleClick = (e: React.MouseEvent) => {
    onPress(entry.uuid, e);
  };

  const favicon = entry.url
    ? `https://www.google.com/s2/favicons?domain=${encodeURIComponent(entry.url)}&sz=32`
    : null;

  return (
    <div
      className="entry-row"
      role="listitem"
      onClick={handleClick}
      onKeyDown={e => e.key === 'Enter' && onPress(entry.uuid)}
      tabIndex={0}
      aria-label={`${entry.title}, ${entry.username}`}
    >
      {/* Icon */}
      <div className="entry-row-icon" aria-hidden="true">
        {favicon ? (
          <img
            src={favicon}
            alt=""
            width={20}
            height={20}
            onError={e => { (e.target as HTMLImageElement).style.display = 'none'; }}
          />
        ) : (
          <span>{getIconEmoji(entry.iconId)}</span>
        )}
      </div>

      {/* Content */}
      <div className="entry-row-content">
        <div className="entry-row-title-line">
          <span className={`entry-row-title ${entry.isExpired ? 'expired' : ''}`}>
            {entry.title || '(no title)'}
          </span>
          <div className="entry-row-badges" aria-label="Entry features">
            {entry.hasOtp && <span className="badge badge-otp" title="Has OTP">OTP</span>}
            {entry.hasPasskey && <span className="badge badge-passkey" title="Has Passkey">🔑</span>}
            {entry.hasSshKey && <span className="badge badge-ssh" title="Has SSH key">SSH</span>}
            {entry.isExpired && <span className="badge badge-expired" title="Expired">Expired</span>}
          </div>
        </div>
        <span className="entry-row-username">{entry.username || '—'}</span>
      </div>

      {/* Copy button */}
      {entry.hasPassword && (
        <button
          className={`entry-row-copy ${copyFeedback ? 'copied' : ''}`}
          onClick={handleCopy}
          aria-label={`Copy password for ${entry.title}`}
          title="Copy password"
          tabIndex={-1}
        >
          {copyFeedback ? '✓' : '⎘'}
        </button>
      )}

      <style>{`
        .entry-row {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 10px 20px;
          cursor: pointer;
          border-bottom: 1px solid var(--color-border);
          transition: background 0.1s;
          outline: none;
        }
        .entry-row:hover, .entry-row:focus-visible {
          background: var(--color-bg-secondary);
        }
        .entry-row:focus-visible {
          box-shadow: inset 0 0 0 2px var(--color-primary);
        }
        .entry-row-icon {
          width: 32px;
          height: 32px;
          border-radius: 8px;
          background: var(--color-bg-tertiary);
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: 16px;
          flex-shrink: 0;
          overflow: hidden;
        }
        .entry-row-content { flex: 1; min-width: 0; }
        .entry-row-title-line {
          display: flex;
          align-items: center;
          gap: 6px;
        }
        .entry-row-title {
          font-size: 14px;
          font-weight: 500;
          color: var(--color-text);
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }
        .entry-row-title.expired {
          color: var(--color-text-tertiary);
          text-decoration: line-through;
        }
        .entry-row-username {
          font-size: 12px;
          color: var(--color-text-secondary);
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
          display: block;
          margin-top: 1px;
        }
        .entry-row-badges { display: flex; gap: 3px; flex-shrink: 0; }
        .badge {
          font-size: 10px;
          font-weight: 600;
          padding: 1px 5px;
          border-radius: 4px;
        }
        .badge-otp { background: #EFF6FF; color: #2563EB; }
        .badge-passkey { background: #F0FDF4; color: #16A34A; }
        .badge-ssh { background: #FFF7ED; color: #EA580C; }
        .badge-expired { background: #FEF2F2; color: #DC2626; }
        .entry-row-copy {
          width: 28px;
          height: 28px;
          border: none;
          background: none;
          cursor: pointer;
          border-radius: 6px;
          font-size: 14px;
          color: var(--color-text-tertiary);
          display: flex;
          align-items: center;
          justify-content: center;
          flex-shrink: 0;
          transition: background 0.1s, color 0.1s;
          opacity: 0;
        }
        .entry-row:hover .entry-row-copy { opacity: 1; }
        .entry-row-copy:hover { background: var(--color-bg-tertiary); color: var(--color-text); }
        .entry-row-copy.copied { color: var(--color-success); opacity: 1; }
      `}</style>
    </div>
  );
}

function getIconEmoji(iconId: number): string {
  const map: Record<number, string> = {
    0: '🔑', 1: '🌐', 2: '⚠️', 3: '🖥️', 4: '🔧',
    5: '💻', 6: '📁', 7: '🔒', 8: '📧', 9: '💳',
    10: '🏦', 11: '📱', 12: '🛡️', 13: '👤', 14: '🏠',
    15: '💼', 48: '📂',
  };
  return map[iconId] ?? '🔑';
}
