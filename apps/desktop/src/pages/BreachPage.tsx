/**
 * Breach monitor page — desktop
 * Checks vault passwords against HaveIBeenPwned (k-anonymity)
 */
import React, { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useNavigate } from 'react-router-dom';
import { useSettingsStore } from '../store/settings';
import { useBreachStore } from '../store/breach';
import type { VaultBreachReport } from '@keepassex/types';

export function BreachPage() {
  const { settings } = useSettingsStore();
  const navigate = useNavigate();
  const isVi = settings.language === 'vi';
  const [useOnline, setUseOnline] = useState(false);
  const { report, loading, error, lastChecked, checkVault } = useBreachStore();

  return (
    <div className="breach-page">
      <div className="breach-header">
        <h2>{isVi ? '🛡️ Kiểm tra rò rỉ dữ liệu' : '🛡️ Breach Monitor'}</h2>
      </div>

      <div className="breach-content">
        {/* Explanation */}
        <div className="breach-info-card">
          <h3>{isVi ? 'Cách hoạt động' : 'How it works'}</h3>
          <p>
            {isVi
              ? 'KeePassEx kiểm tra mật khẩu của bạn với cơ sở dữ liệu HaveIBeenPwned bằng mô hình k-anonymity. Chỉ 5 ký tự đầu của hash SHA-1 được gửi đi — mật khẩu thực sự KHÔNG BAO GIỜ rời khỏi thiết bị của bạn.'
              : 'KeePassEx checks your passwords against the HaveIBeenPwned database using k-anonymity. Only the first 5 characters of the SHA-1 hash are sent — your actual passwords NEVER leave your device.'}
          </p>
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
                <p className="breach-mode-title">{isVi ? 'Ngoại tuyến' : 'Offline'}</p>
                <p className="breach-mode-desc">
                  {isVi ? 'Kiểm tra với danh sách mật khẩu phổ biến nhất (nhanh, không cần mạng)' : 'Check against top common passwords (fast, no network)'}
                </p>
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
                <p className="breach-mode-title">{isVi ? 'Trực tuyến (HIBP)' : 'Online (HIBP)'}</p>
                <p className="breach-mode-desc">
                  {isVi ? 'Kiểm tra với 14 tỷ mật khẩu bị rò rỉ (cần kết nối internet)' : 'Check against 14 billion breached passwords (requires internet)'}
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Run check */}
        <button
          className="btn btn-primary btn-lg"
          onClick={() => checkVault(useOnline)}
          disabled={loading}
          style={{ alignSelf: 'flex-start' }}
        >
          {loading
            ? (isVi ? '🔍 Đang kiểm tra...' : '🔍 Checking...')
            : (isVi ? '🔍 Kiểm tra ngay' : '🔍 Run Check')}
        </button>

        {/* Results */}
        {report && (
          <div className="breach-report">
            {/* Summary */}
            <div className={`breach-summary ${report.breachedCount === 0 ? 'safe' : 'danger'}`}>
              {report.breachedCount === 0 ? (
                <>
                  <span className="breach-summary-icon">✅</span>
                  <div>
                    <p className="breach-summary-title">
                      {isVi ? 'Không tìm thấy rò rỉ!' : 'No breaches found!'}
                    </p>
                    <p className="breach-summary-desc">
                      {isVi
                        ? `Đã kiểm tra ${report.totalChecked} mật khẩu — tất cả an toàn.`
                        : `Checked ${report.totalChecked} passwords — all clear.`}
                    </p>
                  </div>
                </>
              ) : (
                <>
                  <span className="breach-summary-icon">⚠️</span>
                  <div>
                    <p className="breach-summary-title">
                      {isVi
                        ? `${report.breachedCount} mật khẩu bị rò rỉ!`
                        : `${report.breachedCount} breached password${report.breachedCount !== 1 ? 's' : ''} found!`}
                    </p>
                    <p className="breach-summary-desc">
                      {isVi
                        ? `Đã kiểm tra ${report.totalChecked} mật khẩu. Hãy thay đổi ngay các mật khẩu bị rò rỉ.`
                        : `Checked ${report.totalChecked} passwords. Change breached passwords immediately.`}
                    </p>
                  </div>
                </>
              )}
            </div>

            {/* Breached entries */}
            {report.results.length > 0 && (
              <div className="breach-list">
                <h3 className="breach-list-title">
                  {isVi ? 'Mật khẩu bị rò rỉ' : 'Breached passwords'}
                </h3>
                {report.results.map(r => (
                  <div key={r.entryUuid} className="breach-item">
                    <div className="breach-item-info">
                      <span className="breach-item-icon">⚠️</span>
                      <div>
                        <p className="breach-item-title">{r.entryTitle}</p>
                        {r.breachCount > 0 && (
                          <p className="breach-item-count">
                            {isVi
                              ? `Xuất hiện ${r.breachCount.toLocaleString()} lần trong các vụ rò rỉ`
                              : `Found ${r.breachCount.toLocaleString()} times in data breaches`}
                          </p>
                        )}
                      </div>
                    </div>
                    <button
                      className="btn btn-secondary"
                      onClick={() => navigate(`/vault/entry/${r.entryUuid}`)}
                      aria-label={`Change password for ${r.entryTitle}`}
                    >
                      {isVi ? 'Đổi mật khẩu' : 'Change password'}
                    </button>
                  </div>
                ))}
              </div>
            )}

            <p className="breach-footer">
              {report.usedOnline
                ? (isVi ? '🌐 Đã kiểm tra với HIBP (trực tuyến)' : '🌐 Checked against HIBP (online)')
                : (isVi ? '📴 Đã kiểm tra ngoại tuyến' : '📴 Checked offline')}
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
        .breach-header {
          padding:var(--space-md) var(--space-xl);
          border-bottom:1px solid var(--color-border); flex-shrink:0;
        }
        .breach-header h2 { font-size:16px; font-weight:600; }
        .breach-content {
          flex:1; overflow-y:auto; padding:var(--space-xl);
          display:flex; flex-direction:column; gap:var(--space-xl); max-width:640px;
        }
        .breach-info-card {
          background:var(--color-bg-secondary); border:1px solid var(--color-border);
          border-radius:var(--radius-lg); padding:var(--space-xl);
          display:flex; flex-direction:column; gap:var(--space-md);
        }
        .breach-info-card h3 { font-size:14px; font-weight:600; }
        .breach-info-card p { font-size:13px; color:var(--color-text-secondary); line-height:1.6; }
        .breach-modes { display:flex; flex-direction:column; gap:var(--space-sm); }
        .breach-mode-btn {
          display:flex; align-items:flex-start; gap:var(--space-md);
          padding:var(--space-md); border:2px solid var(--color-border);
          border-radius:var(--radius-md); cursor:pointer; transition:border-color .15s;
        }
        .breach-mode-btn:hover { border-color:var(--color-primary); }
        .breach-mode-btn.selected { border-color:var(--color-primary); background:#EFF6FF; }
        .breach-mode-icon { font-size:24px; flex-shrink:0; }
        .breach-mode-title { font-size:14px; font-weight:600; }
        .breach-mode-desc { font-size:12px; color:var(--color-text-secondary); margin-top:2px; }
        .breach-report { display:flex; flex-direction:column; gap:var(--space-lg); }
        .breach-summary {
          display:flex; align-items:center; gap:var(--space-lg);
          padding:var(--space-xl); border-radius:var(--radius-lg); border:1px solid;
        }
        .breach-summary.safe { background:#F0FDF4; border-color:#86EFAC; }
        .breach-summary.danger { background:#FEF2F2; border-color:#FECACA; }
        .breach-summary-icon { font-size:36px; flex-shrink:0; }
        .breach-summary-title { font-size:16px; font-weight:700; }
        .breach-summary-desc { font-size:13px; margin-top:4px; }
        .breach-list { display:flex; flex-direction:column; gap:var(--space-sm); }
        .breach-list-title { font-size:14px; font-weight:600; }
        .breach-item {
          display:flex; align-items:center; justify-content:space-between;
          padding:var(--space-md); background:var(--color-bg-secondary);
          border:1px solid var(--color-border); border-radius:var(--radius-md); gap:var(--space-md);
        }
        .breach-item-info { display:flex; align-items:center; gap:var(--space-md); flex:1; min-width:0; }
        .breach-item-icon { font-size:20px; flex-shrink:0; }
        .breach-item-title { font-size:14px; font-weight:500; }
        .breach-item-count { font-size:12px; color:var(--color-danger); margin-top:2px; }
        .breach-footer { font-size:12px; color:var(--color-text-tertiary); }
        .breach-error {
          background:#FEF2F2; border:1px solid #FECACA; border-radius:var(--radius-md);
          padding:var(--space-md); font-size:13px; color:#991B1B;
        }
      `}</style>
    </div>
  );
}
