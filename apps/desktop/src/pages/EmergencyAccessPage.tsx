/**
 * Emergency Access page — desktop
 * KeePassEx exclusive: no competitor has emergency access built into KDBX vault.
 */
import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useVaultStore } from '../store/vault';

interface EmergencyGrant {
  id: string;
  granteeName: string;
  granteeEmail: string;
  accessLevel: 'view' | 'takeover';
  waitTimeDays: number;
  status:
    | 'invited'
    | 'confirmed'
    | 'recoveryinitiated'
    | 'recoveryapproved'
    | 'recoverygranted'
    | 'revoked';
  daysRemaining?: number;
  createdAt: string;
}

const STATUS_COLORS: Record<string, string> = {
  invited: '#D97706',
  confirmed: '#16A34A',
  recoveryinitiated: '#DC2626',
  recoveryapproved: '#D97706',
  recoverygranted: '#DC2626',
  revoked: '#9CA3AF',
};

const STATUS_KEYS: Record<string, string> = {
  invited: 'emergencyAccess.statusInvited',
  confirmed: 'emergencyAccess.statusConfirmed',
  recoveryinitiated: 'emergencyAccess.statusRequestPending',
  recoveryapproved: 'emergencyAccess.statusWaitingPeriod',
  recoverygranted: 'emergencyAccess.statusAccessGranted',
  revoked: 'emergencyAccess.statusRevoked',
};

export function EmergencyAccessPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { isOpen, isLocked } = useVaultStore();

  const [showAddForm, setShowAddForm] = useState(false);
  const [form, setForm] = useState({
    name: '',
    email: '',
    accessLevel: 'view' as 'view' | 'takeover',
    waitDays: 7,
  });

  const { data: grants = [], isLoading } = useQuery<EmergencyGrant[]>({
    queryKey: ['emergency-access'],
    queryFn: () => invoke<EmergencyGrant[]>('get_emergency_grants'),
    enabled: isOpen && !isLocked,
    staleTime: 30_000,
  });

  const addMutation = useMutation({
    mutationFn: () =>
      invoke('add_emergency_grant', {
        args: {
          name: form.name,
          email: form.email,
          access_level: form.accessLevel,
          wait_days: form.waitDays,
        },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['emergency-access'] });
      setShowAddForm(false);
      setForm({ name: '', email: '', accessLevel: 'view', waitDays: 7 });
    },
  });

  const revokeMutation = useMutation({
    mutationFn: (id: string) => invoke('revoke_emergency_grant', { grantId: id }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['emergency-access'] }),
  });

  const approveMutation = useMutation({
    mutationFn: (id: string) => invoke('approve_emergency_request', { grantId: id }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['emergency-access'] }),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => invoke('delete_emergency_grant', { grantId: id }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['emergency-access'] }),
  });

  if (!isOpen) {
    return (
      <div className="page-locked">
        <span>🔐</span>
        <p>{t('vault.openToView')}</p>
      </div>
    );
  }

  return (
    <div className="ea-page">
      <div className="ea-header">
        <div>
          <h2>🆘 {t('emergencyAccess.title')}</h2>
          <p className="ea-subtitle">{t('emergencyAccess.subtitle')}</p>
        </div>
        <button className="btn btn-primary" onClick={() => setShowAddForm(true)}>
          + {t('emergencyAccess.addContact')}
        </button>
      </div>

      <div className="ea-content">
        {/* How it works */}
        <div className="ea-info-card">
          <h3>{t('emergencyAccess.howItWorks')}</h3>
          <ol className="ea-steps">
            <li>{t('emergencyAccess.step1')}</li>
            <li>{t('emergencyAccess.step2')}</li>
            <li>{t('emergencyAccess.step3')}</li>
            <li>{t('emergencyAccess.step4')}</li>
            <li>{t('emergencyAccess.step5')}</li>
          </ol>
        </div>

        {/* Add form */}
        {showAddForm && (
          <div className="ea-form-card">
            <h3>{t('emergencyAccess.addContact')}</h3>
            <div className="ea-form">
              <div className="ea-field">
                <label className="ea-label" htmlFor="ea-name">
                  {t('emergencyAccess.granteeName')}
                </label>
                <input
                  id="ea-name"
                  type="text"
                  className="form-input"
                  value={form.name}
                  onChange={e => setForm(f => ({ ...f, name: e.target.value }))}
                  placeholder="John Doe"
                />
              </div>
              <div className="ea-field">
                <label className="ea-label" htmlFor="ea-email">
                  {t('emergencyAccess.granteeEmail')}
                </label>
                <input
                  id="ea-email"
                  type="email"
                  className="form-input"
                  value={form.email}
                  onChange={e => setForm(f => ({ ...f, email: e.target.value }))}
                  placeholder="contact@example.com"
                />
              </div>
              <div className="ea-field">
                <label className="ea-label" htmlFor="ea-level">
                  {t('emergencyAccess.accessLevel')}
                </label>
                <select
                  id="ea-level"
                  className="form-input"
                  value={form.accessLevel}
                  onChange={e =>
                    setForm(f => ({ ...f, accessLevel: e.target.value as 'view' | 'takeover' }))
                  }
                >
                  <option value="view">{t('emergencyAccess.accessLevelView')}</option>
                  <option value="takeover">{t('emergencyAccess.accessLevelTakeover')}</option>
                </select>
              </div>
              <div className="ea-field">
                <label className="ea-label" htmlFor="ea-wait">
                  {t('emergencyAccess.waitPeriod')}
                </label>
                <select
                  id="ea-wait"
                  className="form-input"
                  value={form.waitDays}
                  onChange={e => setForm(f => ({ ...f, waitDays: Number(e.target.value) }))}
                >
                  {[1, 3, 7, 14, 30].map(d => (
                    <option key={d} value={d}>
                      {d} {d === 1 ? t('common.info') : t('common.info')}
                    </option>
                  ))}
                </select>
              </div>
              <div className="ea-form-actions">
                <button
                  className="btn btn-primary"
                  onClick={() => addMutation.mutate()}
                  disabled={!form.name.trim() || !form.email.trim() || addMutation.isPending}
                >
                  {addMutation.isPending ? '⏳' : t('emergencyAccess.sendInvitation')}
                </button>
                <button className="btn btn-secondary" onClick={() => setShowAddForm(false)}>
                  {t('common.cancel')}
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Grants list */}
        {isLoading ? (
          <div className="ea-loading">⏳ {t('common.loading')}</div>
        ) : grants.length === 0 && !showAddForm ? (
          <div className="empty-state">
            <span className="empty-state-icon">🆘</span>
            <p className="empty-state-title">{t('emergencyAccess.noContacts')}</p>
            <p className="empty-state-desc">{t('emergencyAccess.noContactsDesc')}</p>
          </div>
        ) : (
          <div className="ea-grants">
            {grants.map(grant => (
              <div key={grant.id} className="ea-grant-card">
                <div className="ea-grant-header">
                  <div className="ea-grant-info">
                    <span className="ea-grant-name">{grant.granteeName}</span>
                    <span className="ea-grant-email">{grant.granteeEmail}</span>
                  </div>
                  <span
                    className="ea-grant-status"
                    style={{ color: STATUS_COLORS[grant.status] ?? '#9CA3AF' }}
                  >
                    {t(STATUS_KEYS[grant.status] ?? 'emergencyAccess.statusInvited')}
                  </span>
                </div>
                <div className="ea-grant-meta">
                  <span>
                    {t('emergencyAccess.accessLevel')}:{' '}
                    <strong>
                      {grant.accessLevel === 'view'
                        ? t('emergencyAccess.accessLevelView')
                        : t('emergencyAccess.accessLevelTakeover')}
                    </strong>
                  </span>
                  <span>
                    {t('emergencyAccess.waitPeriod')}: <strong>{grant.waitTimeDays}d</strong>
                  </span>
                  {grant.daysRemaining !== undefined && grant.daysRemaining > 0 && (
                    <span style={{ color: '#D97706' }}>
                      ⏳ {t('emergencyAccess.daysRemaining', { days: grant.daysRemaining })}
                    </span>
                  )}
                </div>
                <div className="ea-grant-actions">
                  {grant.status === 'recoveryinitiated' && (
                    <button
                      className="btn btn-success"
                      onClick={() => approveMutation.mutate(grant.id)}
                      disabled={approveMutation.isPending}
                    >
                      ✓ {t('emergencyAccess.approve')}
                    </button>
                  )}
                  {grant.status !== 'revoked' && (
                    <button
                      className="btn btn-secondary"
                      onClick={() => {
                        if (
                          confirm(t('emergencyAccess.confirmRevoke', { name: grant.granteeName }))
                        )
                          revokeMutation.mutate(grant.id);
                      }}
                      disabled={revokeMutation.isPending}
                    >
                      {t('emergencyAccess.revoke')}
                    </button>
                  )}
                  <button
                    className="btn-icon"
                    onClick={() => {
                      if (confirm(t('common.confirm'))) deleteMutation.mutate(grant.id);
                    }}
                    aria-label={t('common.delete')}
                    title={t('common.delete')}
                  >
                    🗑
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <style>{`
        .page-locked { display:flex; flex-direction:column; align-items:center; justify-content:center; height:100%; gap:16px; color:var(--color-text-secondary); }
        .page-locked span { font-size:48px; }
        .ea-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .ea-header { display:flex; align-items:flex-start; justify-content:space-between; padding:var(--space-md) var(--space-xl); border-bottom:1px solid var(--color-border); flex-shrink:0; gap:var(--space-lg); }
        .ea-header h2 { font-size:16px; font-weight:600; }
        .ea-subtitle { font-size:13px; color:var(--color-text-secondary); margin-top:2px; }
        .ea-content { flex:1; overflow-y:auto; padding:var(--space-xl); display:flex; flex-direction:column; gap:var(--space-xl); max-width:640px; }
        .ea-loading { font-size:13px; color:var(--color-text-secondary); }
        .ea-info-card { background:var(--color-bg-secondary); border:1px solid var(--color-border); border-radius:var(--radius-lg); padding:var(--space-lg); }
        .ea-info-card h3 { font-size:14px; font-weight:600; margin-bottom:var(--space-sm); }
        .ea-steps { padding-left:var(--space-xl); display:flex; flex-direction:column; gap:var(--space-xs); font-size:13px; color:var(--color-text-secondary); }
        .ea-steps li { line-height:1.6; }
        .ea-form-card { background:var(--color-surface); border:1px solid var(--color-border); border-radius:var(--radius-lg); padding:var(--space-xl); }
        .ea-form-card h3 { font-size:14px; font-weight:600; margin-bottom:var(--space-lg); }
        .ea-form { display:flex; flex-direction:column; gap:var(--space-md); }
        .ea-field { display:flex; flex-direction:column; gap:var(--space-xs); }
        .ea-label { font-size:12px; font-weight:500; color:var(--color-text-secondary); text-transform:uppercase; letter-spacing:.05em; }
        .ea-form-actions { display:flex; gap:var(--space-sm); padding-top:var(--space-sm); }
        .ea-grants { display:flex; flex-direction:column; gap:var(--space-md); }
        .ea-grant-card { background:var(--color-surface); border:1px solid var(--color-border); border-radius:var(--radius-lg); padding:var(--space-lg); display:flex; flex-direction:column; gap:var(--space-sm); }
        .ea-grant-header { display:flex; align-items:flex-start; justify-content:space-between; gap:var(--space-md); }
        .ea-grant-info { display:flex; flex-direction:column; gap:2px; }
        .ea-grant-name { font-size:15px; font-weight:600; }
        .ea-grant-email { font-size:13px; color:var(--color-text-secondary); }
        .ea-grant-status { font-size:12px; font-weight:600; flex-shrink:0; }
        .ea-grant-meta { display:flex; gap:var(--space-lg); font-size:12px; color:var(--color-text-secondary); flex-wrap:wrap; }
        .ea-grant-actions { display:flex; gap:var(--space-sm); align-items:center; }
        .btn-success { background:#16A34A; color:white; border:none; padding:var(--space-xs) var(--space-md); border-radius:var(--radius-sm); cursor:pointer; font-size:12px; font-weight:500; }
        .btn-success:hover { background:#15803d; }
        .btn-icon { background:none; border:none; cursor:pointer; font-size:16px; color:var(--color-text-secondary); padding:4px 6px; border-radius:var(--radius-sm); }
        .btn-icon:hover { background:var(--color-bg-tertiary); }
        .empty-state { display:flex; flex-direction:column; align-items:center; gap:var(--space-md); padding:var(--space-2xl); color:var(--color-text-secondary); text-align:center; }
        .empty-state-icon { font-size:48px; }
        .empty-state-title { font-size:16px; font-weight:600; color:var(--color-text); }
        .empty-state-desc { font-size:13px; }
      `}</style>
    </div>
  );
}
