/**
 * Vault Statistics page — detailed metrics about the vault
 */

import { useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import type { VaultStatistics } from '@keepassex/types';
import { useVaultStore } from '../store/vault';

export function StatisticsPage() {
  const navigate = useNavigate();
  const { t } = useTranslation();
  const { isOpen, isLocked } = useVaultStore();

  const { data: stats, isLoading } = useQuery({
    queryKey: ['vault-statistics'],
    queryFn: () => invoke<VaultStatistics>('get_vault_statistics'),
    enabled: isOpen && !isLocked,
    staleTime: 30_000,
  });

  if (!isOpen) {
    return (
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100%',
          gap: 16,
          color: 'var(--color-text-secondary)',
        }}
      >
        <span style={{ fontSize: 48 }}>🔐</span>
        {t('vault.openToView')}
      </div>
    );
  }

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const strengthLabel = (score: number) => {
    if (score >= 4) return t('generator.strengthVeryStrong');
    if (score >= 3) return t('generator.strengthStrong');
    if (score >= 2) return t('generator.strengthFair');
    if (score >= 1) return t('generator.strengthWeak');
    return t('generator.strengthVeryWeak');
  };

  const strengthColor = (score: number) => {
    if (score >= 4) return '#22c55e';
    if (score >= 3) return '#84cc16';
    if (score >= 2) return '#f59e0b';
    if (score >= 1) return '#f97316';
    return '#ef4444';
  };

  return (
    <div className="stats-page">
      <div className="stats-header">
        <button className="btn-back" onClick={() => navigate('/settings')}>
          ← {t('settings.title')}
        </button>
        <h2>📊 {t('statistics.title')}</h2>
      </div>

      <div className="stats-content">
        {isLoading ? (
          <div className="stats-loading">
            <span className="animate-pulse">⏳</span>
            <p>{t('common.loading')}</p>
          </div>
        ) : stats ? (
          <>
            {/* Overview grid */}
            <div className="stats-grid">
              <StatCard
                icon="🔑"
                value={stats.totalEntries}
                label={t('statistics.totalEntries')}
                color="var(--color-primary)"
              />
              <StatCard
                icon="📁"
                value={stats.totalGroups}
                label={t('statistics.totalGroups')}
                color="#8b5cf6"
              />
              <StatCard
                icon="📎"
                value={stats.totalAttachments}
                label={t('statistics.totalAttachments')}
                color="#f59e0b"
                subtitle={formatBytes(stats.totalAttachmentSize)}
              />
              <StatCard
                icon="⏱"
                value={stats.entriesWithOtp}
                label={t('statistics.entriesWithOtp')}
                color="#06b6d4"
              />
              <StatCard
                icon="🛡️"
                value={stats.entriesWithPasskey}
                label={t('statistics.entriesWithPasskey')}
                color="#10b981"
              />
              <StatCard
                icon="🔐"
                value={stats.entriesWithSshKey}
                label={t('statistics.entriesWithSshKey')}
                color="#6366f1"
              />
            </div>

            {/* Password strength */}
            <div className="stats-section">
              <h3>{t('statistics.averageStrength')}</h3>
              <div className="stats-strength">
                <div className="stats-strength-bar-wrap">
                  <div
                    className="stats-strength-bar"
                    style={{
                      width: `${(stats.averagePasswordStrength / 4) * 100}%`,
                      background: strengthColor(stats.averagePasswordStrength),
                    }}
                  />
                </div>
                <span
                  className="stats-strength-label"
                  style={{ color: strengthColor(stats.averagePasswordStrength) }}
                >
                  {strengthLabel(stats.averagePasswordStrength)} (
                  {stats.averagePasswordStrength.toFixed(1)}/4)
                </span>
              </div>
            </div>

            {/* Expiry */}
            <div className="stats-section">
              <h3>{t('entry.expiry')}</h3>
              <div className="stats-row-list">
                <StatsRow
                  label={t('entry.expires')}
                  value={stats.entriesWithExpiry}
                  total={stats.totalEntries}
                />
                <StatsRow
                  label={t('entry.expired')}
                  value={stats.entriesExpired}
                  total={stats.totalEntries}
                  danger={stats.entriesExpired > 0}
                />
                <StatsRow
                  label={t('health.expiringSoon')}
                  value={stats.entriesExpiringSoon}
                  total={stats.totalEntries}
                  warn={stats.entriesExpiringSoon > 0}
                />
              </div>
            </div>

            {/* Notable entries */}
            {(stats.mostUsedEntry || stats.oldestEntry || stats.newestEntry) && (
              <div className="stats-section">
                <h3>{t('statistics.mostUsed')}</h3>
                <div className="stats-notable">
                  {stats.mostUsedEntry && (
                    <div className="stats-notable-row">
                      <span className="stats-notable-icon">🏆</span>
                      <div>
                        <p className="stats-notable-label">{t('statistics.mostUsed')}</p>
                        <p className="stats-notable-value">{stats.mostUsedEntry.title}</p>
                      </div>
                    </div>
                  )}
                  {stats.oldestEntry && (
                    <div className="stats-notable-row">
                      <span className="stats-notable-icon">📅</span>
                      <div>
                        <p className="stats-notable-label">{t('statistics.oldestEntry')}</p>
                        <p className="stats-notable-value">
                          {new Date(stats.oldestEntry).toLocaleDateString()}
                        </p>
                      </div>
                    </div>
                  )}
                  {stats.newestEntry && (
                    <div className="stats-notable-row">
                      <span className="stats-notable-icon">🆕</span>
                      <div>
                        <p className="stats-notable-label">{t('statistics.newestEntry')}</p>
                        <p className="stats-notable-value">
                          {new Date(stats.newestEntry).toLocaleDateString()}
                        </p>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            )}
          </>
        ) : (
          <div className="stats-loading">
            <span>📊</span>
            <p>{t('common.none')}</p>
          </div>
        )}
      </div>

      <style>{`
        .stats-page { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
        .stats-header {
          display: flex; align-items: center; gap: var(--space-md);
          padding: var(--space-md) var(--space-xl);
          border-bottom: 1px solid var(--color-border); flex-shrink: 0;
        }
        .stats-header h2 { font-size: 16px; font-weight: 600; }
        .btn-back {
          background: none; border: none; cursor: pointer; font-size: 13px;
          color: var(--color-primary); padding: var(--space-xs) var(--space-sm);
          border-radius: var(--radius-sm);
        }
        .btn-back:hover { background: var(--color-bg-secondary); }
        .stats-content {
          flex: 1; overflow-y: auto; padding: var(--space-xl);
          display: flex; flex-direction: column; gap: var(--space-xl);
          max-width: 600px;
        }
        .stats-loading {
          display: flex; flex-direction: column; align-items: center;
          gap: var(--space-md); padding: var(--space-2xl);
          color: var(--color-text-secondary); font-size: 13px;
        }
        .stats-loading span { font-size: 32px; }
        .stats-grid {
          display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--space-md);
        }
        .stat-card {
          display: flex; flex-direction: column; align-items: center; gap: 4px;
          padding: var(--space-md); background: var(--color-bg-secondary);
          border-radius: var(--radius-md); border: 1px solid var(--color-border);
          text-align: center;
        }
        .stat-card-icon { font-size: 20px; }
        .stat-card-value { font-size: 24px; font-weight: 700; }
        .stat-card-label { font-size: 11px; color: var(--color-text-secondary); }
        .stat-card-subtitle { font-size: 10px; color: var(--color-text-tertiary); }
        .stats-section {
          background: var(--color-bg-secondary); border-radius: var(--radius-md);
          padding: var(--space-lg); display: flex; flex-direction: column; gap: var(--space-md);
        }
        .stats-section h3 { font-size: 14px; font-weight: 600; }
        .stats-strength { display: flex; flex-direction: column; gap: var(--space-sm); }
        .stats-strength-bar-wrap {
          height: 8px; background: var(--color-border); border-radius: 4px; overflow: hidden;
        }
        .stats-strength-bar { height: 100%; border-radius: 4px; transition: width 0.5s; }
        .stats-strength-label { font-size: 13px; font-weight: 600; }
        .stats-row-list { display: flex; flex-direction: column; gap: var(--space-sm); }
        .stats-row {
          display: flex; align-items: center; justify-content: space-between;
          padding: var(--space-sm) var(--space-md);
          background: var(--color-bg); border-radius: var(--radius-sm);
        }
        .stats-row-label { font-size: 13px; color: var(--color-text); }
        .stats-row-value { font-size: 13px; font-weight: 600; }
        .stats-row-bar-wrap {
          flex: 1; height: 4px; background: var(--color-border);
          border-radius: 2px; overflow: hidden; margin: 0 var(--space-md);
        }
        .stats-row-bar { height: 100%; border-radius: 2px; }
        .stats-notable { display: flex; flex-direction: column; gap: var(--space-sm); }
        .stats-notable-row {
          display: flex; align-items: center; gap: var(--space-md);
          padding: var(--space-sm) var(--space-md);
          background: var(--color-bg); border-radius: var(--radius-sm);
        }
        .stats-notable-icon { font-size: 20px; }
        .stats-notable-label { font-size: 11px; color: var(--color-text-secondary); }
        .stats-notable-value { font-size: 13px; font-weight: 500; }
      `}</style>
    </div>
  );
}

function StatCard({
  icon,
  value,
  label,
  color,
  subtitle,
}: {
  icon: string;
  value: number;
  label: string;
  color: string;
  subtitle?: string;
}) {
  return (
    <div className="stat-card">
      <span className="stat-card-icon">{icon}</span>
      <span className="stat-card-value" style={{ color }}>
        {value.toLocaleString()}
      </span>
      <span className="stat-card-label">{label}</span>
      {subtitle && <span className="stat-card-subtitle">{subtitle}</span>}
    </div>
  );
}

function StatsRow({
  label,
  value,
  total,
  danger,
  warn,
}: {
  label: string;
  value: number;
  total: number;
  danger?: boolean;
  warn?: boolean;
}) {
  const pct = total > 0 ? (value / total) * 100 : 0;
  const color = danger ? '#ef4444' : warn ? '#f59e0b' : 'var(--color-primary)';

  return (
    <div className="stats-row">
      <span className="stats-row-label">{label}</span>
      <div className="stats-row-bar-wrap">
        <div className="stats-row-bar" style={{ width: `${pct}%`, background: color }} />
      </div>
      <span className="stats-row-value" style={{ color }}>
        {value}
      </span>
    </div>
  );
}
