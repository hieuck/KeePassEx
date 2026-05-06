/**
 * Emergency Access page — desktop
 * Manage trusted contacts who can access your vault in an emergency
 */
import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../store/settings';

interface EmergencyGrant {
  id: string;
  granteeName: string;
  granteeEmail: string;
  accessLevel: 'view' | 'takeover';
  waitTimeDays: number;
  status: 'invited' | 'confirmed' | 'recovery_initiated' | 'recovery_approved' | 'recovery_granted' | 'revoked';
  requestInitiatedAt?: string;
  daysRemaining?: number;
  createdAt: string;
}

const STATUS_LABELS: Record<string, { en: string; vi: string; color: string }> = {
  invited:            { en: 'Invited',           vi: 'Đã mời',          color: '#D97706' },
  confirmed:          { en: 'Confirmed',          vi: 'Đã xác nhận',     color: '#16A34A' },
  recovery_initiated: { en: 'Request pending',    vi: 'Đang chờ duyệt',  color: '#DC2626' },
  recovery_approved:  { en: 'Waiting period',     vi: 'Đang chờ',        color: '#D97706' },
  recovery_granted:   { en: 'Access granted',     vi: 'Đã cấp quyền',    color: '#DC2626' },
  revoked:            { en: 'Revoked',            vi: 'Đã thu hồi',      color: '#9CA3AF' },
};

export function EmergencyAccessPage() {
  const { settings } = useSettingsStore();
  const queryClient = useQueryClient();
  const isVi = settings.language === 'vi';

  const [showAddForm, setShowAddForm] = useState(false);
  const [form, setForm] = useState({
    name: '',
    email: '',
    accessLevel: 'view' as 'view' | 'takeover',
    waitDays: 7,
  });

  // In production: these would be Tauri commands
  const { data: grants = [] } = useQuery<EmergencyGrant[]>({
    queryKey: ['emergency-access'],
    queryFn: () => Promise.resolve([]), // invoke('get_emergency_grants')
  });

  const addMutation = useMutation({
    mutationFn: () => Promise.resolve(), // invoke('add_emergency_grant', { args: form })
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['emergency-access'] });
      setShowAddForm(false);
      setForm({ name: '', email: '', accessLevel: 'view', waitDays: 7 });
    },
  });

  const revokeMutation = useMutation({
    mutationFn: (id: string) => Promise.resolve(), // invoke('revoke_emergency_grant', { id })
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['emergency-access'] }),
  });

  const approveMutation = useMutation({
    mutationFn: (id: string) => Promise.resolve(), // invoke('approve_emergency_request', { id })
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['emergency-access'] }),
  });

  return (
    <div className="ea-page">
      <div className="ea-header">
        <div>
          <h2>{isVi ? '🆘 Truy cập khẩn cấp' : '🆘 Emergency Access'}</h2>
          <p className="ea-subtitle">
            {isVi
              ? 'Cho phép người tin cậy truy cập kho của bạn trong trường hợp khẩn cấp'
              : 'Allow trusted contacts to access your vault in an emergency'}
          </p>
        </div>
        <button
          className="btn btn-primary"
          onClick={() => setShowAddForm(true)}
        >
          + {isVi ? 'Thêm liên hệ' : 'Add Contact'}
        </button>
      </div>

      <div className="ea-content">
        {/* How it works */}
        <div className="ea-info-card">
          <h3>{isVi ? 'Cách hoạt động' : 'How it works'}</h3>
          <ol className="ea-steps">
            <li>{isVi ? 'Bạn mời một người tin cậy (ví dụ: thành viên gia đình)' : 'You invite a trusted contact (e.g., family member)'}</li>
            <li>{isVi ? 'Họ chấp nhận lời mời và thiết lập khóa của họ' : 'They accept the invitation and set up their key'}</li>
            <li>{isVi ? 'Trong trường hợp khẩn cấp, họ yêu cầu truy cập' : 'In an emergency, they request access'}</li>
            <li>{isVi ? 'Bạn có thể từ chối trong thời gian chờ (mặc định 7 ngày)' : 'You can deny during the waiting period (default 7 days)'}</li>
            <li>{isVi ? 'Sau thời gian chờ, họ nhận được quyền truy cập' : 'After the waiting period, they receive access'}</li>
          </ol>
        </div>

        {/* Add form */}
        {showAddForm && (
          <div className="ea-form-card">
            <h3>{isVi ? 'Thêm liên hệ tin cậy' : 'Add Trusted Contact'}</h3>

            <div className="ea-form">
              <div className="ea-field">
                <label className="ea-label" htmlFor="ea-name">
                  {isVi ? 'Tên' : 'Name'}
                </label>
                <input
                  id="ea-name"
                  type="text"
                  className="form-input"
                  value={form.name}
                  onChange={e => setForm(f => ({ ...f, name: e.target.value }))}
                  placeholder={isVi ? 'Nguyễn Văn A' : 'John Doe'}
                />
              </div>

              <div className="ea-field">
                <label className="ea-label" htmlFor="ea-email">
                  Email
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
                  {isVi ? 'Mức độ truy cập' : 'Access Level'}
                </label>
                <select
                  id="ea-level"
                  className="form-input"
                  value={form.accessLevel}
                  onChange={e => setForm(f => ({ ...f, accessLevel: e.target.value as 'view' | 'takeover' }))}
                >
                  <option value="view">{isVi ? 'Chỉ xem' : 'View only'}</option>
                  <option value="takeover">{isVi ? 'Toàn quyền' : 'Full takeover'}</option>
                </select>
              </div>

              <div className="ea-field">
                <label className="ea-label" htmlFor="ea-wait">
                  {isVi ? 'Thời gian chờ (ngày)' : 'Waiting period (days)'}
                </label>
                <select
                  id="ea-wait"
                  className="form-input"
                  value={form.waitDays}
                  onChange={e => setForm(f => ({ ...f, waitDays: Number(e.target.value) }))}
                >
                  <option value={1}>1 {isVi ? 'ngày' : 'day'}</option>
                  <option value={3}>3 {isVi ? 'ngày' : 'days'}</option>
                  <option value={7}>7 {isVi ? 'ngày' : 'days'} ({isVi ? 'khuyến nghị' : 'recommended'})</option>
                  <option value={14}>14 {isVi ? 'ngày' : 'days'}</option>
                  <option value={30}>30 {isVi ? 'ngày' : 'days'}</option>
                </select>
              </div>

              <div className="ea-form-actions">
                <button
                  className="btn btn-primary"
                  onClick={() => addMutation.mutate()}
                  disabled={!form.name.trim() || !form.email.trim() || addMutation.isPending}
                >
                  {isVi ? 'Gửi lời mời' : 'Send Invitation'}
                </button>
                <button
                  className="btn btn-secondary"
                  onClick={() => setShowAddForm(false)}
                >
                  {isVi ? 'Hủy' : 'Cancel'}
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Grants list */}
        {grants.length === 0 && !showAddForm ? (
          <div className="empty-state">
            <span className="empty-state-icon">🆘</span>
            <p className="empty-state-title">
              {isVi ? 'Chưa có liên hệ khẩn cấp' : 'No emergency contacts'}
            </p>
            <p className="empty-state-desc">
              {isVi
                ? 'Thêm người tin cậy để họ có thể truy cập kho của bạn trong trường hợp khẩn cấp.'
                : 'Add trusted contacts who can access your vault in an emergency.'}
            </p>
          </div>
        ) : (
          <div className="ea-grants">
            {grants.map(grant => {
              const statusInfo = STATUS_LABELS[grant.status] ?? STATUS_LABELS.invited;
              return (
                <div key={grant.id} className="ea-grant-card">
                  <div className="ea-grant-header">
                    <div className="ea-grant-info">
                      <span className="ea-grant-name">{grant.granteeName}</span>
                      <span className="ea-grant-email">{grant.granteeEmail}</span>
                    </div>
                    <span
                      className="ea-grant-status"
                      style={{ color: statusInfo.color }}
                    >
                      {isVi ? statusInfo.vi : statusInfo.en}
                    </span>
                  </div>

                  <div className="ea-grant-meta">
                    <span>
                      {isVi ? 'Mức độ' : 'Level'}:{' '}
                      <strong>
                        {grant.accessLevel === 'view'
                          ? (isVi ? 'Chỉ xem' : 'View only')
                          : (isVi ? 'Toàn quyền' : 'Full takeover')}
                      </strong>
                    </span>
                    <span>
                      {isVi ? 'Thời gian chờ' : 'Wait'}:{' '}
                      <strong>{grant.waitTimeDays} {isVi ? 'ngày' : 'days'}</strong>
                    </span>
                    {grant.daysRemaining !== undefined && (
                      <span style={{ color: '#D97706' }}>
                        ⏳ {grant.daysRemaining} {isVi ? 'ngày còn lại' : 'days remaining'}
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
                        {isVi ? '✓ Duyệt' : '✓ Approve'}
                      </button>
                    )}
                    {grant.status !== 'revoked' && (
                      <button
                        className="btn btn-danger"
                        onClick={() => {
                          if (confirm(isVi
                            ? `Thu hồi quyền truy cập của ${grant.granteeName}?`
                            : `Revoke access for ${grant.granteeName}?`
                          )) {
                            revokeMutation.mutate(grant.id);
                          }
                        }}
                        style={{ fontSize: 12 }}
                      >
                        {isVi ? 'Thu hồi' : 'Revoke'}
                      </button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      <style>{`
        .ea-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .ea-header {
          display:flex; align-items:flex-start; justify-content:space-between;
          padding:var(--space-md) var(--space-xl);
          border-bottom:1px solid var(--color-border); flex-shrink:0; gap:var(--space-lg);
        }
        .ea-header h2 { font-size:16px; font-weight:600; }
        .ea-subtitle { font-size:13px; color:var(--color-text-secondary); margin-top:2px; }
        .ea-content {
          flex:1; overflow-y:auto; padding:var(--space-xl);
          display:flex; flex-direction:column; gap:var(--space-xl); max-width:640px;
        }
        .ea-info-card {
          background:var(--color-bg-secondary); border:1px solid var(--color-border);
          border-radius:var(--radius-lg); padding:var(--space-lg);
        }
        .ea-info-card h3 { font-size:14px; font-weight:600; margin-bottom:var(--space-sm); }
        .ea-steps {
          padding-left:var(--space-xl); display:flex; flex-direction:column;
          gap:var(--space-xs); font-size:13px; color:var(--color-text-secondary);
        }
        .ea-steps li { line-height:1.6; }
        .ea-form-card {
          background:var(--color-surface); border:1px solid var(--color-border);
          border-radius:var(--radius-lg); padding:var(--space-xl);
        }
        .ea-form-card h3 { font-size:14px; font-weight:600; margin-bottom:var(--space-lg); }
        .ea-form { display:flex; flex-direction:column; gap:var(--space-md); }
        .ea-field { display:flex; flex-direction:column; gap:var(--space-xs); }
        .ea-label {
          font-size:12px; font-weight:500; color:var(--color-text-secondary);
          text-transform:uppercase; letter-spacing:.05em;
        }
        .ea-form-actions { display:flex; gap:var(--space-sm); padding-top:var(--space-sm); }
        .ea-grants { display:flex; flex-direction:column; gap:var(--space-md); }
        .ea-grant-card {
          background:var(--color-surface); border:1px solid var(--color-border);
          border-radius:var(--radius-lg); padding:var(--space-lg);
          display:flex; flex-direction:column; gap:var(--space-sm);
        }
        .ea-grant-header { display:flex; align-items:flex-start; justify-content:space-between; gap:var(--space-md); }
        .ea-grant-info { display:flex; flex-direction:column; gap:2px; }
        .ea-grant-name { font-size:15px; font-weight:600; }
        .ea-grant-email { font-size:13px; color:var(--color-text-secondary); }
        .ea-grant-status { font-size:12px; font-weight:600; flex-shrink:0; }
        .ea-grant-meta {
          display:flex; gap:var(--space-lg); font-size:12px;
          color:var(--color-text-secondary);
        }
        .ea-grant-actions { display:flex; gap:var(--space-sm); }
        .btn-success {
          background:var(--color-success); color:white; border:none;
          padding:var(--space-xs) var(--space-md); border-radius:var(--radius-sm);
          cursor:pointer; font-size:13px; font-weight:500;
        }
      `}</style>
    </div>
  );
}
