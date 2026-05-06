/**
 * Password Policy page — define and enforce password requirements
 * Unique feature: no competitor has this level of policy management
 */
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../store/settings';

interface PolicyEvaluation {
  policy_id: string;
  policy_name: string;
  passed: boolean;
  violations: Array<{ rule: string; message: string; message_vi: string }>;
  warnings: Array<{ rule: string; message: string; message_vi: string }>;
}

interface PasswordPolicy {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  scope: 'Global' | { Groups: string[] } | { Tags: string[] };
}

export function PasswordPolicyPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { settings } = useSettingsStore();
  const isVi = settings.language === 'vi';

  const [testPassword, setTestPassword] = useState('');
  const [testResults, setTestResults] = useState<PolicyEvaluation[]>([]);
  const [showTestPassword, setShowTestPassword] = useState(false);

  const { data: policies = [], isLoading } = useQuery({
    queryKey: ['password-policies'],
    queryFn: () => invoke<PasswordPolicy[]>('get_password_policies'),
  });

  const toggleMutation = useMutation({
    mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) =>
      invoke('set_policy_enabled', { id, enabled }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['password-policies'] }),
  });

  const testMutation = useMutation({
    mutationFn: (password: string) =>
      invoke<PolicyEvaluation[]>('evaluate_password_policies', { password }),
    onSuccess: setTestResults,
  });

  const enabledCount = policies.filter(p => p.enabled).length;

  return (
    <div className="policy-page">
      {/* Header */}
      <div className="policy-header">
        <button className="btn-back" onClick={() => navigate('/settings')}>
          ← {isVi ? 'Cài đặt' : 'Settings'}
        </button>
        <h2>{isVi ? '🛡️ Chính sách mật khẩu' : '🛡️ Password Policies'}</h2>
        <p className="policy-subtitle">
          {isVi
            ? 'Định nghĩa và thực thi yêu cầu mật khẩu cho kho'
            : 'Define and enforce password requirements for your vault'}
        </p>
      </div>

      <div className="policy-content">
        {/* Status banner */}
        <div
          className={`policy-status ${enabledCount > 0 ? 'policy-status--active' : 'policy-status--inactive'}`}
        >
          <span>{enabledCount > 0 ? '✅' : '⚠️'}</span>
          <p>
            {enabledCount > 0
              ? isVi
                ? `${enabledCount} chính sách đang hoạt động`
                : `${enabledCount} policy(ies) active`
              : isVi
                ? 'Không có chính sách nào được bật'
                : 'No policies enabled'}
          </p>
        </div>

        {/* Policy list */}
        <div className="policy-section">
          <h3>{isVi ? 'Chính sách tích hợp sẵn' : 'Built-in Policies'}</h3>
          {isLoading ? (
            <p className="policy-loading">⏳ {isVi ? 'Đang tải...' : 'Loading...'}</p>
          ) : (
            <div className="policy-list">
              {policies.map(policy => (
                <PolicyCard
                  key={policy.id}
                  policy={policy}
                  isVi={isVi}
                  onToggle={enabled => toggleMutation.mutate({ id: policy.id, enabled })}
                />
              ))}
            </div>
          )}
        </div>

        {/* Password tester */}
        <div className="policy-section">
          <h3>{isVi ? 'Kiểm tra mật khẩu' : 'Test a Password'}</h3>
          <p className="policy-section-desc">
            {isVi
              ? 'Kiểm tra mật khẩu với tất cả chính sách đang bật'
              : 'Test a password against all enabled policies'}
          </p>
          <div className="test-input-row">
            <input
              type={showTestPassword ? 'text' : 'password'}
              className="form-input test-input"
              value={testPassword}
              onChange={e => setTestPassword(e.target.value)}
              placeholder={isVi ? 'Nhập mật khẩu để kiểm tra...' : 'Enter password to test...'}
              autoComplete="new-password"
            />
            <button
              className="btn-icon"
              onClick={() => setShowTestPassword(v => !v)}
              aria-label={showTestPassword ? 'Hide' : 'Show'}
            >
              {showTestPassword ? '🙈' : '👁'}
            </button>
            <button
              className="btn btn-primary"
              onClick={() => testMutation.mutate(testPassword)}
              disabled={!testPassword || testMutation.isPending || enabledCount === 0}
            >
              {isVi ? 'Kiểm tra' : 'Test'}
            </button>
          </div>

          {enabledCount === 0 && (
            <p className="test-hint">
              {isVi
                ? 'Bật ít nhất một chính sách để kiểm tra.'
                : 'Enable at least one policy to test.'}
            </p>
          )}

          {/* Test results */}
          {testResults.length > 0 && (
            <div className="test-results">
              {testResults.map(result => (
                <div
                  key={result.policy_id}
                  className={`test-result ${result.passed ? 'test-result--pass' : 'test-result--fail'}`}
                >
                  <div className="test-result-header">
                    <span className="test-result-icon">{result.passed ? '✅' : '❌'}</span>
                    <span className="test-result-name">{result.policy_name}</span>
                    <span
                      className={`test-result-badge ${result.passed ? 'badge-pass' : 'badge-fail'}`}
                    >
                      {result.passed
                        ? isVi
                          ? 'Đạt'
                          : 'Pass'
                        : isVi
                          ? `${result.violations.length} vi phạm`
                          : `${result.violations.length} violation(s)`}
                    </span>
                  </div>
                  {result.violations.length > 0 && (
                    <ul className="test-violations">
                      {result.violations.map((v, i) => (
                        <li key={i} className="test-violation">
                          ✗ {isVi ? v.message_vi : v.message}
                        </li>
                      ))}
                    </ul>
                  )}
                  {result.warnings.length > 0 && (
                    <ul className="test-warnings">
                      {result.warnings.map((w, i) => (
                        <li key={i} className="test-warning">
                          ⚠ {isVi ? w.message_vi : w.message}
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      <style>{`
        .policy-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .policy-header {
          padding:var(--space-md) var(--space-xl);
          border-bottom:1px solid var(--color-border); flex-shrink:0;
        }
        .policy-header h2 { font-size:16px; font-weight:700; margin:4px 0; }
        .policy-subtitle { font-size:13px; color:var(--color-text-secondary); }
        .btn-back {
          background:none; border:none; cursor:pointer; color:var(--color-primary);
          font-size:13px; padding:0; margin-bottom:4px;
        }
        .policy-content {
          flex:1; overflow-y:auto; padding:var(--space-xl);
          display:flex; flex-direction:column; gap:var(--space-xl); max-width:640px;
        }
        .policy-status {
          display:flex; align-items:center; gap:var(--space-md);
          padding:var(--space-md); border-radius:var(--radius-md); border:1px solid;
          font-size:13px;
        }
        .policy-status--active { background:rgba(34,197,94,.08); border-color:rgba(34,197,94,.3); color:#16a34a; }
        .policy-status--inactive { background:rgba(234,179,8,.08); border-color:rgba(234,179,8,.3); color:#ca8a04; }
        .policy-section { display:flex; flex-direction:column; gap:var(--space-md); }
        .policy-section h3 { font-size:14px; font-weight:600; }
        .policy-section-desc { font-size:13px; color:var(--color-text-secondary); }
        .policy-loading { font-size:13px; color:var(--color-text-secondary); }
        .policy-list { display:flex; flex-direction:column; gap:var(--space-sm); }
        .policy-card {
          padding:var(--space-md); background:var(--color-bg-secondary);
          border-radius:var(--radius-md); border:1px solid var(--color-border);
          display:flex; flex-direction:column; gap:var(--space-sm);
        }
        .policy-card--enabled { border-color:var(--color-primary); background:rgba(37,99,235,.04); }
        .policy-card-header { display:flex; align-items:center; gap:var(--space-md); }
        .policy-card-info { flex:1; }
        .policy-card-name { font-size:14px; font-weight:600; color:var(--color-text); }
        .policy-card-desc { font-size:12px; color:var(--color-text-secondary); margin-top:2px; }
        .policy-card-scope {
          font-size:11px; padding:2px 8px; background:var(--color-bg-tertiary);
          border-radius:var(--radius-full); color:var(--color-text-secondary);
        }
        .toggle {
          width:44px; height:24px; border-radius:12px; border:none;
          cursor:pointer; position:relative; transition:background .2s; flex-shrink:0;
        }
        .toggle-thumb {
          position:absolute; top:2px; width:20px; height:20px;
          border-radius:50%; background:white; transition:left .2s;
          box-shadow:0 1px 3px rgba(0,0,0,.2);
        }
        .test-input-row { display:flex; gap:var(--space-sm); align-items:center; }
        .test-input { flex:1; }
        .test-hint { font-size:12px; color:var(--color-text-tertiary); }
        .test-results { display:flex; flex-direction:column; gap:var(--space-sm); }
        .test-result {
          padding:var(--space-md); border-radius:var(--radius-md); border:1px solid;
        }
        .test-result--pass { background:rgba(34,197,94,.06); border-color:rgba(34,197,94,.2); }
        .test-result--fail { background:rgba(239,68,68,.06); border-color:rgba(239,68,68,.2); }
        .test-result-header { display:flex; align-items:center; gap:var(--space-sm); }
        .test-result-icon { font-size:16px; }
        .test-result-name { flex:1; font-size:13px; font-weight:500; }
        .test-result-badge {
          font-size:11px; padding:2px 8px; border-radius:var(--radius-full); font-weight:600;
        }
        .badge-pass { background:#dcfce7; color:#16a34a; }
        .badge-fail { background:#fee2e2; color:#ef4444; }
        .test-violations, .test-warnings { margin:var(--space-sm) 0 0 var(--space-xl); padding:0; list-style:none; display:flex; flex-direction:column; gap:2px; }
        .test-violation { font-size:12px; color:#ef4444; }
        .test-warning { font-size:12px; color:#ca8a04; }
      `}</style>
    </div>
  );
}

function PolicyCard({
  policy,
  isVi,
  onToggle,
}: {
  policy: PasswordPolicy;
  isVi: boolean;
  onToggle: (enabled: boolean) => void;
}) {
  const scopeLabel =
    typeof policy.scope === 'string'
      ? isVi
        ? 'Tất cả mục'
        : 'All entries'
      : 'Groups' in policy.scope
        ? isVi
          ? 'Nhóm cụ thể'
          : 'Specific groups'
        : isVi
          ? 'Thẻ cụ thể'
          : 'Specific tags';

  return (
    <div className={`policy-card${policy.enabled ? ' policy-card--enabled' : ''}`}>
      <div className="policy-card-header">
        <div className="policy-card-info">
          <p className="policy-card-name">{policy.name}</p>
          <p className="policy-card-desc">{policy.description}</p>
        </div>
        <span className="policy-card-scope">{scopeLabel}</span>
        <button
          role="switch"
          aria-checked={policy.enabled}
          aria-label={`${policy.enabled ? 'Disable' : 'Enable'} ${policy.name}`}
          className="toggle"
          style={{ background: policy.enabled ? 'var(--color-primary)' : 'var(--color-border)' }}
          onClick={() => onToggle(!policy.enabled)}
        >
          <div className="toggle-thumb" style={{ left: policy.enabled ? 22 : 2 }} />
        </button>
      </div>
    </div>
  );
}
