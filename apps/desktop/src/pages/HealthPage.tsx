/**
 * Vault health audit page — with password rotation engine and duplicate detection
 * KeePassEx exclusive: rotation engine + duplicate detection + bulk rotation built-in
 */

import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useBreachStore } from '../store/breach';
import { useVaultStore } from '../store/vault';

interface HealthReport {
  totalEntries: number;
  score: number;
  weakCount: number;
  reusedCount: number;
  expiredCount: number;
  expiringSoonCount: number;
  noPasswordCount: number;
  oldPasswordCount: number;
  weakPasswords: Array<{
    entryUuid: string;
    entryTitle: string;
    strengthScore: number;
    strengthLabel: string;
  }>;
  reusedPasswords: Array<{ entries: Array<{ uuid: string; title: string }> }>;
  expiredEntries: Array<{ entryUuid: string; entryTitle: string; expiredAt: string }>;
  expiringSoon: Array<{
    entryUuid: string;
    entryTitle: string;
    expiresAt: string;
    daysRemaining: number;
  }>;
}

interface RotationRec {
  entryUuid: string;
  entryTitle: string;
  urgency: 'aging' | 'soon' | 'overdue' | 'expired';
  urgencyColor: string;
  ageDays: number;
  recommendedMaxDays: number;
  daysUntilOverdue: number;
  messageEn: string;
  messageVi: string;
}

interface DuplicateGroup {
  reason: 'same_password' | 'same_url_username' | 'same_title';
  entries: Array<{ uuid: string; title: string }>;
}

export function HealthPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { checkVault: checkBreaches, report: breachReport } = useBreachStore();
  const { isOpen, isLocked } = useVaultStore();

  const {
    data: report,
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ['health'],
    queryFn: () => invoke<HealthReport>('audit_vault'),
    enabled: isOpen && !isLocked,
    staleTime: 60_000,
  });

  const { data: rotations = [] } = useQuery({
    queryKey: ['rotation-recommendations'],
    queryFn: () => invoke<RotationRec[]>('get_rotation_recommendations'),
    enabled: isOpen && !isLocked,
    staleTime: 60_000,
  });

  const { data: duplicates = [] } = useQuery({
    queryKey: ['duplicate-entries'],
    queryFn: () => invoke<DuplicateGroup[]>('find_duplicate_entries'),
    enabled: isOpen && !isLocked,
    staleTime: 60_000,
  });

  const [selectedForRotation, setSelectedForRotation] = useState<Set<string>>(new Set());
  const [showRotationSelect, setShowRotationSelect] = useState(false);

  const bulkRotateMutation = useMutation({
    mutationFn: (uuids: string[]) =>
      invoke<string[]>('bulk_rotate_passwords', {
        args: { entry_uuids: uuids, length: 20, use_symbols: true },
      }),
    onSuccess: rotated => {
      queryClient.invalidateQueries({ queryKey: ['rotation-recommendations'] });
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      invoke('save_vault').catch(console.error);
      setSelectedForRotation(new Set());
      setShowRotationSelect(false);
      alert(`✅ ${rotated.length} ${t('health.passwordsRotated')}`);
    },
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
        <p>{t('vault.openToView')}</p>
      </div>
    );
  }

  const scoreColor = !report
    ? '#9CA3AF'
    : report.score >= 90
      ? '#059669'
      : report.score >= 70
        ? '#16A34A'
        : report.score >= 50
          ? '#D97706'
          : '#DC2626';

  return (
    <div className="health-page">
      <div className="health-header">
        <h2>🛡️ {t('health.title')}</h2>
        <div style={{ display: 'flex', gap: 'var(--space-sm)' }}>
          <button
            className="btn btn-secondary"
            onClick={() => checkBreaches(false)}
            title={t('health.breachCheck')}
          >
            🔍 {t('health.breachCheck')}
          </button>
          <button className="btn btn-secondary" onClick={() => refetch()}>
            🔄 {t('health.runCheck')}
          </button>
        </div>
      </div>

      {isLoading ? (
        <div className="health-loading">
          <p>{t('health.checking')}</p>
        </div>
      ) : report ? (
        <div className="health-content">
          {/* Score card */}
          <div className="score-card">
            <div className="score-circle" style={{ borderColor: scoreColor }}>
              <span className="score-number" style={{ color: scoreColor }}>
                {report.score}
              </span>
              <span className="score-label">/100</span>
            </div>
            <div className="score-info">
              <h3 className="score-title">
                {report.score >= 90
                  ? `✅ ${t('health.score')} — ${t('health.noIssues').split('!')[0]}`
                  : report.score >= 70
                    ? `👍 ${t('health.score')}: ${report.score}/100`
                    : report.score >= 50
                      ? `⚠️ ${t('health.score')}: ${report.score}/100`
                      : `🚨 ${t('health.score')}: ${report.score}/100`}
              </h3>
              <p className="score-desc">
                {t('vault.statistics_entries', { count: report.totalEntries })}
              </p>
            </div>
          </div>

          {/* Summary grid */}
          <div className="health-grid">
            <HealthCard
              icon="🔓"
              count={report.weakCount}
              label={t('health.weakPasswords')}
              severity={report.weakCount > 0 ? 'danger' : 'ok'}
            />
            <HealthCard
              icon="♻️"
              count={report.reusedCount}
              label={t('health.reusedPasswords')}
              severity={report.reusedCount > 0 ? 'warning' : 'ok'}
            />
            <HealthCard
              icon="⏰"
              count={report.expiredCount}
              label={t('health.expiredEntries')}
              severity={report.expiredCount > 0 ? 'danger' : 'ok'}
            />
            <HealthCard
              icon="📅"
              count={report.expiringSoonCount}
              label={t('health.expiringSoon')}
              severity={report.expiringSoonCount > 0 ? 'warning' : 'ok'}
            />
          </div>

          {/* Issue lists */}
          {(report.weakPasswords ?? []).length > 0 && (
            <IssueSection
              title={`🔓 ${t('health.weakPasswords')}`}
              items={(report.weakPasswords ?? []).map(w => ({
                uuid: w.entryUuid,
                title: w.entryTitle,
                detail: w.strengthLabel,
                detailColor: '#DC2626',
              }))}
              onItemClick={uuid => navigate(`/vault/entry/${uuid}`)}
            />
          )}

          {(report.reusedPasswords ?? []).length > 0 && (
            <div className="issue-section">
              <h3 className="issue-title">♻️ {t('health.reusedPasswords')}</h3>
              {(report.reusedPasswords ?? []).map((group, i) => (
                <div key={i} className="reused-group">
                  {(group.entries ?? []).map(e => (
                    <button
                      key={e.uuid}
                      className="issue-item"
                      onClick={() => navigate(`/vault/entry/${e.uuid}`)}
                    >
                      <span className="issue-item-title">{e.title}</span>
                    </button>
                  ))}
                </div>
              ))}
            </div>
          )}

          {(report.expiredEntries ?? []).length > 0 && (
            <IssueSection
              title={`⏰ ${t('health.expiredEntries')}`}
              items={(report.expiredEntries ?? []).map(e => ({
                uuid: e.entryUuid,
                title: e.entryTitle,
                detail: e.expiredAt,
                detailColor: '#DC2626',
              }))}
              onItemClick={uuid => navigate(`/vault/entry/${uuid}`)}
            />
          )}

          {(report.expiringSoon ?? []).length > 0 && (
            <IssueSection
              title={`📅 ${t('health.expiringSoon')}`}
              items={(report.expiringSoon ?? []).map(e => ({
                uuid: e.entryUuid,
                title: e.entryTitle,
                detail: `${e.daysRemaining} days`,
                detailColor: '#D97706',
              }))}
              onItemClick={uuid => navigate(`/vault/entry/${uuid}`)}
            />
          )}

          {/* ── Password Rotation Engine — KeePassEx exclusive ── */}
          {rotations.length > 0 && (
            <div className="issue-section">
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  marginBottom: 'var(--space-sm)',
                }}
              >
                <h3 className="issue-title">🔄 {t('health.passwordRotation')}</h3>
                <div style={{ display: 'flex', gap: 'var(--space-xs)' }}>
                  {showRotationSelect && selectedForRotation.size > 0 && (
                    <button
                      className="btn btn-primary btn-sm"
                      onClick={() => bulkRotateMutation.mutate(Array.from(selectedForRotation))}
                      disabled={bulkRotateMutation.isPending}
                    >
                      {bulkRotateMutation.isPending
                        ? '⏳'
                        : `🔄 ${t('health.rotateSelected', { count: selectedForRotation.size })}`}
                    </button>
                  )}
                  <button
                    className="btn btn-secondary btn-sm"
                    onClick={() => {
                      setShowRotationSelect(v => !v);
                      setSelectedForRotation(new Set());
                    }}
                  >
                    {showRotationSelect ? t('common.cancel') : t('health.selectToRotate')}
                  </button>
                </div>
              </div>
              <p className="issue-subtitle">{t('health.passwordRotationDesc')}</p>
              {rotations.map(rec => (
                <button
                  key={rec.entryUuid}
                  className={`issue-item rotation-item${showRotationSelect && selectedForRotation.has(rec.entryUuid) ? ' rotation-selected' : ''}`}
                  onClick={() => {
                    if (showRotationSelect) {
                      setSelectedForRotation(prev => {
                        const next = new Set(prev);
                        if (next.has(rec.entryUuid)) next.delete(rec.entryUuid);
                        else next.add(rec.entryUuid);
                        return next;
                      });
                    } else {
                      navigate(`/vault/entry/${rec.entryUuid}`);
                    }
                  }}
                >
                  <div className="rotation-item-left">
                    {showRotationSelect && (
                      <span className="rotation-checkbox">
                        {selectedForRotation.has(rec.entryUuid) ? '☑' : '☐'}
                      </span>
                    )}
                    <span className="issue-item-title">{rec.entryTitle}</span>
                    <span className="rotation-age">{rec.ageDays}d old</span>
                  </div>
                  <span
                    className="rotation-badge"
                    style={{ background: rec.urgencyColor + '20', color: rec.urgencyColor }}
                  >
                    {rec.urgency}
                  </span>
                </button>
              ))}
            </div>
          )}

          {/* ── Duplicate Detection — KeePassEx exclusive ── */}
          {duplicates.length > 0 && (
            <div className="issue-section">
              <h3 className="issue-title">🔁 {t('health.duplicateEntries')}</h3>
              <p className="issue-subtitle">{t('health.duplicateEntriesDesc')}</p>
              {duplicates.map((group, i) => (
                <div key={i} className="reused-group">
                  <div className="duplicate-reason-badge">
                    {group.reason === 'same_password'
                      ? `🔑 ${t('health.duplicateSamePassword')}`
                      : group.reason === 'same_url_username'
                        ? `🌐 ${t('health.duplicateSameUrlUser')}`
                        : `📝 ${t('health.duplicateSameTitle')}`}
                  </div>
                  {group.entries.map(e => (
                    <button
                      key={e.uuid}
                      className="issue-item"
                      onClick={() => navigate(`/vault/entry/${e.uuid}`)}
                    >
                      <span className="issue-item-title">{e.title}</span>
                    </button>
                  ))}
                </div>
              ))}
            </div>
          )}

          {report.weakCount === 0 &&
            report.reusedCount === 0 &&
            report.expiredCount === 0 &&
            report.expiringSoonCount === 0 &&
            rotations.length === 0 &&
            duplicates.length === 0 && (
              <div className="health-all-good">
                <span style={{ fontSize: 48 }}>🎉</span>
                <p>{t('health.noIssues')}</p>
              </div>
            )}
        </div>
      ) : null}

      <style>{`
        .health-page { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
        .health-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: var(--space-md) var(--space-xl);
          border-bottom: 1px solid var(--color-border);
          flex-shrink: 0;
        }
        .health-header h2 { font-size: 16px; font-weight: 600; }
        .health-loading, .health-all-good {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          gap: var(--space-lg);
          height: 100%;
          color: var(--color-text-secondary);
        }
        .health-content {
          flex: 1;
          overflow-y: auto;
          padding: var(--space-xl);
          display: flex;
          flex-direction: column;
          gap: var(--space-xl);
          max-width: 640px;
        }
        .score-card {
          display: flex;
          align-items: center;
          gap: var(--space-xl);
          padding: var(--space-xl);
          background: var(--color-bg-secondary);
          border-radius: var(--radius-lg);
          border: 1px solid var(--color-border);
        }
        .score-circle {
          width: 80px;
          height: 80px;
          border-radius: 50%;
          border: 4px solid;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          flex-shrink: 0;
        }
        .score-number { font-size: 28px; font-weight: 700; line-height: 1; }
        .score-label { font-size: 11px; color: var(--color-text-secondary); }
        .score-title { font-size: 16px; font-weight: 600; }
        .score-desc { font-size: 13px; color: var(--color-text-secondary); margin-top: 4px; }
        .health-grid {
          display: grid;
          grid-template-columns: repeat(2, 1fr);
          gap: var(--space-md);
        }
        .issue-section { display: flex; flex-direction: column; gap: var(--space-sm); }
        .issue-title { font-size: 14px; font-weight: 600; }
        .issue-item {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: var(--space-sm) var(--space-md);
          background: var(--color-bg-secondary);
          border: 1px solid var(--color-border);
          border-radius: var(--radius-sm);
          cursor: pointer;
          text-align: left;
          width: 100%;
          transition: background 0.1s;
        }
        .issue-item:hover { background: var(--color-bg-tertiary); }
        .issue-item-title { font-size: 13px; color: var(--color-text); }
        .reused-group {
          border: 1px solid var(--color-border);
          border-radius: var(--radius-md);
          overflow: hidden;
        }
        .reused-group .issue-item { border: none; border-bottom: 1px solid var(--color-border); border-radius: 0; }
        .reused-group .issue-item:last-child { border-bottom: none; }
        .rotation-item { justify-content: space-between; }
        .rotation-item-left { display: flex; flex-direction: column; gap: 2px; align-items: flex-start; }
        .rotation-age { font-size: 11px; color: var(--color-text-tertiary); }
        .rotation-badge { font-size: 11px; font-weight: 700; padding: 2px 8px; border-radius: var(--radius-full); text-transform: capitalize; flex-shrink: 0; }
        .rotation-selected { background: rgba(37,99,235,0.08) !important; border-color: var(--color-primary) !important; }
        .rotation-checkbox { font-size: 16px; margin-right: 4px; }
        .issue-subtitle { font-size: 12px; color: var(--color-text-secondary); margin-bottom: var(--space-sm); }
        .duplicate-reason-badge { font-size: 11px; font-weight: 600; color: var(--color-text-secondary); padding: var(--space-xs) var(--space-md); background: var(--color-bg-tertiary); border-bottom: 1px solid var(--color-border); }
      `}</style>
    </div>
  );
}

function HealthCard({
  icon,
  count,
  label,
  severity,
}: {
  icon: string;
  count: number;
  label: string;
  severity: 'ok' | 'warning' | 'danger';
}) {
  const colors = { ok: '#16A34A', warning: '#D97706', danger: '#DC2626' };
  const bgs = { ok: '#F0FDF4', warning: '#FFFBEB', danger: '#FEF2F2' };
  const color = count === 0 ? colors.ok : colors[severity];
  const bg = count === 0 ? bgs.ok : bgs[severity];

  return (
    <div className="health-card" style={{ background: bg, borderColor: color + '40' }}>
      <span className="health-card-icon">{icon}</span>
      <span className="health-card-count" style={{ color }}>
        {count}
      </span>
      <span className="health-card-label">{label}</span>
      <style>{`
        .health-card {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 4px;
          padding: var(--space-lg);
          border-radius: var(--radius-md);
          border: 1px solid;
          text-align: center;
        }
        .health-card-icon { font-size: 24px; }
        .health-card-count { font-size: 28px; font-weight: 700; }
        .health-card-label { font-size: 12px; color: var(--color-text-secondary); }
      `}</style>
    </div>
  );
}

function IssueSection({
  title,
  items,
  onItemClick,
}: {
  title: string;
  items: Array<{ uuid: string; title: string; detail: string; detailColor: string }>;
  onItemClick: (uuid: string) => void;
}) {
  return (
    <div className="issue-section">
      <h3 className="issue-title">{title}</h3>
      {items.map(item => (
        <button key={item.uuid} className="issue-item" onClick={() => onItemClick(item.uuid)}>
          <span className="issue-item-title">{item.title}</span>
          <span style={{ fontSize: 12, color: item.detailColor, fontWeight: 500 }}>
            {item.detail}
          </span>
        </button>
      ))}
    </div>
  );
}
