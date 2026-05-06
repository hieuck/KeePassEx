/**
 * Emergency Access page — desktop
 */
import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';

interface EmergencyGrant {
  id: string;
  granteeName: string;
  granteeEmail: string;
  accessLevel: 'view' | 'takeover';
  waitTimeDays: number;
  status:
    | 'invited'
    | 'confirmed'
    | 'recovery_initiated'
    | 'recovery_approved'
    | 'recovery_granted'
    | 'revoked';
  daysRemaining?: number;
  createdAt: string;
}

const STATUS_COLORS: Record<string, string> = {
  invited: '#D97706',
  confirmed: '#16A34A',
  recovery_initiated: '#DC2626',
  recovery_approved: '#D97706',
  recovery_granted: '#DC2626',
  revoked: '#9CA3AF',
};

export function EmergencyAccessPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();

  const [showAddForm, setShowAddForm] = useState(false);
  const [form, setForm] = useState({
    name: '',
    email: '',
    accessLevel: 'view' as 'view' | 'takeover',
    waitDays: 7,
  });

  const { data: grants = [] } = useQuery<EmergencyGrant[]>({
    queryKey: ['emergency-access'],
    queryFn: () => Promise.resolve([]),
  });

  const addMutation = useMutation({
    mutationFn: () => Promise.resolve(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['emergency-access'] });
      setShowAddForm(false);
      setForm({ name: '', email: '', accessLevel: 'view', waitDays: 7 });
    },
  });

  const revokeMutation = useMutation({
    mutationFn: (_id: string) => Promise.resolve(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['emergency-access'] }),
  });

  const approveMutation = useMutation({
    mutationFn: (_id: string) => Promise.resolve(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['emergency-access'] }),
  });

  const STATUS_KEYS: Record<string, string> = {
    invited: 'emergencyAccess.statusInvited',
    confirmed: 'emergencyAccess.statusConfirmed',
    recovery_initiated: 'emergencyAccess.statusRequestPending',
    recovery_approved: 'emergencyAccess.statusWaitingPeriod',
    recovery_granted: 'emergencyAccess.statusAccessGranted',
    revoked: 'emergencyAccess.statusRevoked',
  };

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
                  <option value={1}>1 day</option>
                  <option value={3}>3 days</option>
                  <option value={7}>7 days</option>
                  <option value={14}>14 days</option>
                  <option value={30}>30 days</option>
                </select>
              </div>
              <div className="ea-form-actions">
                <button
                  className="btn btn-primary"
                  onClick={() => addMutation.mutate()}
                  disabled={!form.name.trim() || !form.email.trim() || addMutation.isPending}
                >
                  {t('emergencyAccess.sendInvitation')}
                </button>
                <button className="btn btn-secondary" onClick={() => setShowAddForm(false)}>
                  {t('common.cancel')}
                </button>
              </div>
            </div>
          </div>
        )}

        {grants.length === 0 && !showAddForm ? (
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
                    {t('emergencyAccess.waitPeriod')}: <strong>{grant.waitTimeDays} days</strong>
                  </span>
                  {grant.daysRemaining !== undefined && (
                    <span style={{ color: '#D97706' }}>
                      ⏳ {t('emergencyAccess.daysRemaining', { days: grant.daysRemaining })}
                    </span>
                  )}
                </div>
                <div className="ea-grant-actions">
                  {grant.status === 'recovery_initiated' && (
                    <button
                      className="btn btn-success"
                      onClick={() => approveMutation.mutate(grant.id)}
                      style={{ fontSize: 12 }}
                    >
                      ✓ {t('emergencyAccess.approve')}
                    </button>
                  )}
                  {grant.status !== 'revoked' && (
                    <button
                      className="btn btn-danger"
                      onClick={() => {
                        if (
                          confirm(t('emergencyAccess.confirmRevoke', { name: grant.granteeName }))
                        )
                          revokeMutation.mutate(grant.id);
                      }}
                      style={{ fontSize: 12 }}
                    >
                      {t('emergencyAccess.revoke')}
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <style>{`
        .ea-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .ea-header { display:flex; align-items:flex-start; justify-content:space-between; padding:var(--space-md) var(--space-xl); border-bottom:1px solid var(--color-border); flex-shrink:0; gap:var(--space-lg); }
        .ea-header h2 { font-size:16px; font-weight:600; }
        .ea-subtitle { font-size:13px; color:var(--color-text-secondary); margin-top:2px; }
        .ea-content { flex:1; overflow-y:auto; padding:var(--space-xl); display:flex; flex-direction:column; gap:var(--space-xl); max-width:640px; }
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
        .ea-grant-meta { display:flex; gap:var(--space-lg); font-size:12px; color:var(--color-text-secondary); }
        .ea-grant-actions { display:flex; gap:var(--space-sm); }
        .btn-success { background:var(--color-success); color:white; border:none; padding:var(--space-xs) var(--space-md); border-radius:var(--radius-sm); cursor:pointer; font-size:13px; font-weight:500; }
      `}</style>
    </div>
  );
}
