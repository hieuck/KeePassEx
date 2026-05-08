/**
 * Breach monitor page — desktop
 */
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useBreachStore } from '../store/breach';

export function BreachPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [useOnline, setUseOnline] = useState(false);
  const { report, loading, error, checkVault } = useBreachStore();

  return (
    <div className="breach-page">
      <div className="breach-header">
        <h2>🛡️ {t('breach.title')}</h2>
      </div>

      <div className="breach-content">
        <div className="breach-info-card">
          <h3>{t('breach.howItWorks')}</h3>
          <p>{t('breach.howItWorksDesc')}</p>
          <div className="breach-modes">
            <div
              className={`breach-mode-btn ${!useOnline ? 'selected' : ''}`}
              onClick={() => setUseOnline(false)}
              role="radio"
              aria-checked={!useOnline}
              tabIndex={0}
              onKeyDown={e => e.key === 'Enter' && setUseOnline(false)}
            >
              <span className="breach-mode-icon">📴</span>
              <div>
                <p className="breach-mode-title">{t('breach.offlineMode')}</p>
                <p className="breach-mode-desc">{t('breach.offlineModeDesc')}</p>
              </div>
            </div>
            <div
              className={`breach-mode-btn ${useOnline ? 'selected' : ''}`}
              onClick={() => setUseOnline(true)}
              role="radio"
              aria-checked={useOnline}
              tabIndex={0}
              onKeyDown={e => e.key === 'Enter' && setUseOnline(true)}
            >
              <span className="breach-mode-icon">🌐</span>
              <div>
                <p className="breach-mode-title">{t('breach.onlineMode')}</p>
                <p className="breach-mode-desc">{t('breach.onlineModeDesc')}</p>
              </div>
            </div>
          </div>
        </div>

        <button
          className="btn btn-primary btn-lg"
          onClick={() => checkVault(useOnline)}
          disabled={loading}
          style={{ alignSelf: 'flex-start' }}
        >
          {loading ? `🔍 ${t('breach.checking')}` : `🔍 ${t('breach.runCheck')}`}
        </button>

        {report && (
          <div className="breach-report">
            <div className={`breach-summary ${report.breachedCount === 0 ? 'safe' : 'danger'}`}>
              {report.breachedCount === 0 ? (
                <>
                  <span className="breach-summary-icon">✅</span>
                  <div>
                    <p className="breach-summary-title">{t('breach.noBreaches')}</p>
                    <p className="breach-summary-desc">
                      {t('breach.noBreachesDesc', { count: report.totalChecked })}
                    </p>
                  </div>
                </>
              ) : (
                <>
                  <span className="breach-summary-icon">⚠️</span>
                  <div>
                    <p className="breach-summary-title">
                      {t('breach.breachesFound', {
                        count: report.breachedCount,
                        plural: report.breachedCount !== 1 ? 's' : '',
                      })}
                    </p>
                    <p className="breach-summary-desc">
                      {t('breach.breachesFoundDesc', { count: report.totalChecked })}
                    </p>
                  </div>
                </>
              )}
            </div>

            {report.results.length > 0 && (
              <div className="breach-list">
                <h3 className="breach-list-title">{t('breach.breachedPasswords')}</h3>
                {report.results.map(r => (
                  <div key={r.entryUuid} className="breach-item">
                    <div className="breach-item-info">
                      <span className="breach-item-icon">⚠️</span>
                      <div>
                        <p className="breach-item-title">{r.entryTitle}</p>
                        {r.breachCount > 0 && (
                          <p className="breach-item-count">
                            {t('breach.foundInBreaches', { count: r.breachCount })}
                          </p>
                        )}
                      </div>
                    </div>
                    <button
                      className="btn btn-secondary"
                      onClick={() => navigate(`/vault/entry/${r.entryUuid}`)}
                      aria-label={`Change password for ${r.entryTitle}`}
                    >
                      {t('breach.changePassword')}
                    </button>
                  </div>
                ))}
              </div>
            )}

            <p className="breach-footer">
              {report.usedOnline
                ? `🌐 ${t('breach.checkedOnline')}`
                : `📴 ${t('breach.checkedOffline')}`}
            </p>
          </div>
        )}

        {error && (
          <div className="breach-error" role="alert">
            ⚠️ {error}
          </div>
        )}
      </div>

      <style>{`
        .breach-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .breach-header { padding:var(--space-md) var(--space-xl); border-bottom:1px solid var(--color-border); flex-shrink:0; }
        .breach-header h2 { font-size:16px; font-weight:600; }
        .breach-content { flex:1; overflow-y:auto; padding:var(--space-xl); display:flex; flex-direction:column; gap:var(--space-xl); max-width:640px; }
        .breach-info-card { background:var(--color-bg-secondary); border:1px solid var(--color-border); border-radius:var(--radius-lg); padding:var(--space-xl); display:flex; flex-direction:column; gap:var(--space-md); }
        .breach-info-card h3 { font-size:14px; font-weight:600; }
        .breach-info-card p { font-size:13px; color:var(--color-text-secondary); line-height:1.6; }
        .breach-modes { display:flex; flex-direction:column; gap:var(--space-sm); }
        .breach-mode-btn { display:flex; align-items:flex-start; gap:var(--space-md); padding:var(--space-md); border:2px solid var(--color-border); border-radius:var(--radius-md); cursor:pointer; transition:border-color .15s; }
        .breach-mode-btn:hover { border-color:var(--color-primary); }
        .breach-mode-btn.selected { border-color:var(--color-primary); background:#EFF6FF; }
        .breach-mode-icon { font-size:24px; flex-shrink:0; }
        .breach-mode-title { font-size:14px; font-weight:600; }
        .breach-mode-desc { font-size:12px; color:var(--color-text-secondary); margin-top:2px; }
        .breach-report { display:flex; flex-direction:column; gap:var(--space-lg); }
        .breach-summary { display:flex; align-items:center; gap:var(--space-lg); padding:var(--space-xl); border-radius:var(--radius-lg); border:1px solid; }
        .breach-summary.safe { background:#F0FDF4; border-color:#86EFAC; }
        .breach-summary.danger { background:#FEF2F2; border-color:#FECACA; }
        .breach-summary-icon { font-size:36px; flex-shrink:0; }
        .breach-summary-title { font-size:16px; font-weight:700; }
        .breach-summary-desc { font-size:13px; margin-top:4px; }
        .breach-list { display:flex; flex-direction:column; gap:var(--space-sm); }
        .breach-list-title { font-size:14px; font-weight:600; }
        .breach-item { display:flex; align-items:center; justify-content:space-between; padding:var(--space-md); background:var(--color-bg-secondary); border:1px solid var(--color-border); border-radius:var(--radius-md); gap:var(--space-md); }
        .breach-item-info { display:flex; align-items:center; gap:var(--space-md); flex:1; min-width:0; }
        .breach-item-icon { font-size:20px; flex-shrink:0; }
        .breach-item-title { font-size:14px; font-weight:500; }
        .breach-item-count { font-size:12px; color:var(--color-danger); margin-top:2px; }
        .breach-footer { font-size:12px; color:var(--color-text-tertiary); }
        .breach-error { background:#FEF2F2; border:1px solid #FECACA; border-radius:var(--radius-md); padding:var(--space-md); font-size:13px; color:#991B1B; }
      `}</style>
    </div>
  );
}
