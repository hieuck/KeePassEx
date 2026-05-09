/**
 * ExpiryBanner — shows a dismissible banner when entries are expiring soon
 * Appears at the top of the vault page when there are expiring entries
 * KeePassEx exclusive: proactive expiry notifications in the UI
 */
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useVaultStore } from '../store/vault';

interface ExpiryInfo {
  expiredCount: number;
  expiringSoonCount: number;
  expiringSoon: Array<{ entryUuid: string; entryTitle: string; daysRemaining: number }>;
}

export function ExpiryBanner() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { isOpen, isLocked } = useVaultStore();
  const [dismissed, setDismissed] = useState(false);

  const { data } = useQuery<ExpiryInfo>({
    queryKey: ['expiry-banner'],
    queryFn: async () => {
      const report = await invoke<{
        expiredCount: number;
        expiringSoonCount: number;
        expiringSoon: Array<{ entryUuid: string; entryTitle: string; daysRemaining: number }>;
      }>('audit_vault');
      return {
        expiredCount: report.expiredCount,
        expiringSoonCount: report.expiringSoonCount,
        expiringSoon: report.expiringSoon ?? [],
      };
    },
    enabled: isOpen && !isLocked,
    staleTime: 300_000, // 5 minutes
    retry: false,
  });

  if (dismissed || !data) return null;
  if (data.expiredCount === 0 && data.expiringSoonCount === 0) return null;

  const isUrgent = data.expiredCount > 0;
  const total = data.expiredCount + data.expiringSoonCount;

  return (
    <div
      className={`expiry-banner ${isUrgent ? 'expiry-banner--urgent' : 'expiry-banner--warning'}`}
      role="alert"
      aria-live="polite"
    >
      <span className="expiry-banner-icon" aria-hidden="true">
        {isUrgent ? '⏰' : '📅'}
      </span>
      <div className="expiry-banner-content">
        <span className="expiry-banner-text">
          {isUrgent
            ? `${data.expiredCount} ${t('health.expiredEntries')}${data.expiringSoonCount > 0 ? `, ${data.expiringSoonCount} ${t('health.expiringSoon')}` : ''}`
            : `${data.expiringSoonCount} ${t('health.expiringSoon')}`}
        </span>
        {data.expiringSoon.slice(0, 2).map(e => (
          <button
            key={e.entryUuid}
            className="expiry-entry-link"
            onClick={() => navigate(`/vault/entry/${e.entryUuid}`)}
          >
            {e.entryTitle} ({e.daysRemaining}d)
          </button>
        ))}
        {total > 2 && (
          <button className="expiry-entry-link" onClick={() => navigate('/health')}>
            +{total - 2} more →
          </button>
        )}
      </div>
      <button
        className="expiry-banner-action"
        onClick={() => navigate('/health')}
        aria-label={t('health.title')}
      >
        {t('health.title')} →
      </button>
      <button
        className="expiry-banner-dismiss"
        onClick={() => setDismissed(true)}
        aria-label={t('common.close')}
      >
        ✕
      </button>

      <style>{`
        .expiry-banner {
          display: flex; align-items: center; gap: var(--space-sm);
          padding: var(--space-sm) var(--space-lg); flex-shrink: 0;
          font-size: 13px; animation: slideDown 0.2s ease;
        }
        @keyframes slideDown { from { transform: translateY(-100%); opacity: 0; } to { transform: translateY(0); opacity: 1; } }
        .expiry-banner--urgent { background: rgba(239,68,68,0.1); border-bottom: 1px solid rgba(239,68,68,0.2); }
        .expiry-banner--warning { background: rgba(245,158,11,0.1); border-bottom: 1px solid rgba(245,158,11,0.2); }
        .expiry-banner-icon { font-size: 16px; flex-shrink: 0; }
        .expiry-banner-content { flex: 1; display: flex; align-items: center; gap: var(--space-sm); flex-wrap: wrap; }
        .expiry-banner-text { font-weight: 500; color: var(--color-text); }
        .expiry-entry-link {
          background: none; border: none; cursor: pointer; font-size: 12px;
          color: var(--color-primary); text-decoration: underline; padding: 0;
        }
        .expiry-banner-action {
          background: none; border: 1px solid currentColor; border-radius: var(--radius-sm);
          padding: 2px 8px; cursor: pointer; font-size: 12px; font-weight: 600;
          color: var(--color-text-secondary); flex-shrink: 0;
        }
        .expiry-banner-action:hover { background: rgba(0,0,0,0.05); }
        .expiry-banner-dismiss {
          background: none; border: none; cursor: pointer; font-size: 14px;
          color: var(--color-text-tertiary); padding: 2px 6px; border-radius: var(--radius-sm);
          flex-shrink: 0;
        }
        .expiry-banner-dismiss:hover { background: rgba(0,0,0,0.05); }
      `}</style>
    </div>
  );
}
