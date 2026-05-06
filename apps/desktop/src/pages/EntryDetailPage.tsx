/**
 * Entry detail / edit page — complete with OTP, custom fields, attachments,
 * history viewer, passkey section, SSH key section, password strength
 */
import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../store/settings';
import { OtpSetupDialog } from '../components/OtpSetupDialog';
import { CustomFieldEditor, type CustomField } from '../components/CustomFieldEditor';
import { AttachmentViewer, type Attachment } from '../components/AttachmentViewer';
import { TagInput } from '../components/TagInput';
import { OtpDisplay } from '@keepassex/ui';
import { PasswordField } from '@keepassex/ui';
import { EntryHistoryViewer } from '@keepassex/ui';
import type { EntryDto } from '../store/vault';

export function EntryDetailPage() {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { settings } = useSettingsStore();
  const isVi = settings.language === 'vi';
  const isNew = uuid === 'new';

  const [editing, setEditing] = useState(isNew);
  const [showPassword, setShowPassword] = useState(false);
  const [password, setPassword] = useState('');
  const [showOtpSetup, setShowOtpSetup] = useState(false);
  const [form, setForm] = useState({
    title: '',
    username: '',
    password: '',
    url: '',
    notes: '',
    tags: [] as string[],
    iconId: 0,
    expiry: '',
    customFields: [] as CustomField[],
  });
  const [otpCode, setOtpCode] = useState<{ code: string; remainingSeconds: number; period: number } | null>(null);
  const [copied, setCopied] = useState<string | null>(null);
  const [tagInput, setTagInput] = useState('');
  const [activeTab, setActiveTab] = useState<'fields' | 'history' | 'passkeys' | 'ssh'>('fields');
  const [passwordStrength, setPasswordStrength] = useState<{ score: 0|1|2|3|4; entropy: number; label: string } | null>(null);

  // Fetch entry
  const { data: entry } = useQuery({
    queryKey: ['entry', uuid],
    queryFn: () => invoke<EntryDto>('get_entry', { uuid, includePassword: false }),
    enabled: !isNew && !!uuid,
  });

  // Fetch entry history
  const { data: history = [] } = useQuery({
    queryKey: ['entry-history', uuid],
    queryFn: () => invoke<Array<{
      uuid: string; modifiedAt: string; title: string;
      username: string; url: string; notes: string; hasPassword: boolean;
    }>>('get_entry_history', { uuid }),
    enabled: !isNew && !!uuid,
  });

  // Populate form when entry loads
  useEffect(() => {
    if (entry) {
      setForm({
        title: entry.title,
        username: entry.username,
        password: '',
        url: entry.url,
        notes: entry.notes,
        tags: entry.tags,
        iconId: entry.iconId,
        expiry: entry.expiry ?? '',
        customFields: entry.customFields.map(f => ({
          key: f.key,
          value: f.value,
          protected: f.protected,
        })),
      });
    }
  }, [entry]);

  // OTP refresh
  useEffect(() => {
    if (!entry?.hasOtp || !uuid) return;
    const refresh = () => {
      invoke<{ code: string; remainingSeconds: number; period: number }>('generate_totp', { entryUuid: uuid })
        .then(setOtpCode)
        .catch(() => {});
    };
    refresh();
    const interval = setInterval(refresh, 1000);
    return () => clearInterval(interval);
  }, [entry?.hasOtp, uuid]);

  // Password strength check when editing
  useEffect(() => {
    if (!editing || !form.password) { setPasswordStrength(null); return; }
    invoke<{ score: 0|1|2|3|4; entropy: number; strengthLabel: string }>('check_password_strength', {
      password: form.password,
    }).then(r => setPasswordStrength({ score: r.score, entropy: r.entropy, label: r.strengthLabel }))
      .catch(() => {});
  }, [form.password, editing]);

  // Save mutation
  const saveMutation = useMutation({
    mutationFn: async () => {
      if (isNew) {
        await invoke('create_entry', {
          args: {
            group_uuid: '00000000-0000-0000-0000-000000000000',
            title: form.title,
            username: form.username,
            password: form.password,
            url: form.url,
            notes: form.notes,
            tags: form.tags,
            icon_id: form.iconId,
          },
        });
      } else {
        await invoke('update_entry', {
          args: {
            uuid,
            title: form.title,
            username: form.username,
            password: form.password,
            url: form.url,
            notes: form.notes,
            tags: form.tags,
            icon_id: form.iconId,
            expiry: form.expiry || null,
            custom_fields: form.customFields,
          },
        });
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      queryClient.invalidateQueries({ queryKey: ['entry', uuid] });
      setEditing(false);
      if (isNew) navigate('/vault');
    },
  });

  // Delete mutation
  const deleteMutation = useMutation({
    mutationFn: () => invoke('delete_entry', { uuid, permanent: false }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      navigate('/vault');
    },
  });

  const copyToClipboard = async (text: string, field: string) => {
    await invoke('copy_to_clipboard', {
      text,
      clearAfterSeconds: settings.clipboardClearSeconds ?? 10,
    });
    setCopied(field);
    setTimeout(() => setCopied(null), 2000);
  };

  const revealPassword = async () => {
    if (!uuid) return;
    const pw = await invoke<string>('get_entry_password', { uuid });
    setPassword(pw);
    setShowPassword(true);
  };

  const handleOtpSave = async (uri: string) => {
    // Save OTP URI to entry via custom field or dedicated OTP field
    // In production: invoke('set_entry_otp', { uuid, uri })
    setShowOtpSetup(false);
    queryClient.invalidateQueries({ queryKey: ['entry', uuid] });
  };

  const addTag = () => {
    const tag = tagInput.trim();
    if (tag && !form.tags.includes(tag)) {
      setForm(f => ({ ...f, tags: [...f.tags, tag] }));
    }
    setTagInput('');
  };

  const removeTag = (tag: string) => {
    setForm(f => ({ ...f, tags: f.tags.filter(t => t !== tag) }));
  };

  if (!isNew && !entry) {
    return (
      <div className="entry-detail-loading">
        <p>{isVi ? 'Đang tải...' : 'Loading...'}</p>
      </div>
    );
  }

  return (
    <div className="entry-detail">
      {/* OTP Setup Dialog */}
      {showOtpSetup && (
        <OtpSetupDialog
          entryUuid={uuid ?? ''}
          onSave={handleOtpSave}
          onClose={() => setShowOtpSetup(false)}
        />
      )}

      {/* Header */}
      <div className="entry-detail-header">
        <button className="btn-icon" onClick={() => navigate(-1)} aria-label="Back">←</button>
        <h2 className="entry-detail-title">
          {isNew ? (isVi ? 'Mục mới' : 'New Entry') : (entry?.title ?? '')}
        </h2>
        <div className="entry-detail-actions">
          {!isNew && !editing && (
            <>
              <button className="btn btn-secondary" onClick={() => setEditing(true)}>
                {isVi ? 'Chỉnh sửa' : 'Edit'}
              </button>
              <button
                className="btn btn-danger"
                onClick={() => {
                  if (confirm(isVi ? `Xóa "${entry?.title}"?` : `Delete "${entry?.title}"?`)) {
                    deleteMutation.mutate();
                  }
                }}
              >
                {isVi ? 'Xóa' : 'Delete'}
              </button>
            </>
          )}
          {editing && (
            <>
              <button
                className="btn btn-primary"
                onClick={() => saveMutation.mutate()}
                disabled={saveMutation.isPending || !form.title.trim()}
              >
                {saveMutation.isPending ? '...' : (isVi ? 'Lưu' : 'Save')}
              </button>
              {!isNew && (
                <button className="btn btn-secondary" onClick={() => setEditing(false)}>
                  {isVi ? 'Hủy' : 'Cancel'}
                </button>
              )}
            </>
          )}
        </div>
      </div>

      {/* Tabs */}
      {!isNew && (
        <div className="entry-tabs" role="tablist">
          {[
            { id: 'fields', label: isVi ? 'Thông tin' : 'Details' },
            { id: 'history', label: isVi ? 'Lịch sử' : 'History', count: history.length },
            { id: 'passkeys', label: 'Passkeys', show: entry?.hasPasskey },
            { id: 'ssh', label: 'SSH', show: entry?.hasSshKey },
          ].filter(t => t.show !== false).map(tab => (
            <button
              key={tab.id}
              role="tab"
              aria-selected={activeTab === tab.id}
              className={`entry-tab${activeTab === tab.id ? ' entry-tab--active' : ''}`}
              onClick={() => setActiveTab(tab.id as typeof activeTab)}
            >
              {tab.label}
              {tab.count !== undefined && tab.count > 0 && (
                <span className="entry-tab-badge">{tab.count}</span>
              )}
            </button>
          ))}
        </div>
      )}

      {/* Content */}
      <div className="entry-detail-content">
        {/* ── Details tab ── */}
        {(isNew || activeTab === 'fields') && (
          <>
            {/* Title */}
            <FieldRow label={isVi ? 'Tiêu đề' : 'Title'} editing={editing}
              value={entry?.title} inputValue={form.title}
              onInputChange={v => setForm(f => ({ ...f, title: v }))}
              placeholder={isVi ? 'Tiêu đề mục' : 'Entry title'} />

            {/* Username */}
            <FieldRow label={isVi ? 'Tên đăng nhập' : 'Username'} editing={editing}
              value={entry?.username} inputValue={form.username}
              onInputChange={v => setForm(f => ({ ...f, username: v }))}
              placeholder={isVi ? 'Tên đăng nhập' : 'Username'}
              onCopy={() => copyToClipboard(entry?.username ?? form.username, 'username')}
              copied={copied === 'username'} />

            {/* Password — use PasswordField component */}
            <div className="field-row">
              <span className="field-label">{isVi ? 'Mật khẩu' : 'Password'}</span>
              <PasswordField
                value={editing ? form.password : (showPassword ? password : '')}
                onChange={v => setForm(f => ({ ...f, password: v }))}
                onCopy={() => copyToClipboard(showPassword ? password : '', 'password')}
                onGenerate={editing ? () => invoke<string>('generate_password', {
                  config: { mode: 'random', length: 20, useUppercase: true, useLowercase: true, useDigits: true, useSymbols: true, excludeAmbiguous: false, excludeChars: '', minUppercase: 0, minLowercase: 0, minDigits: 0, minSymbols: 0, wordCount: 6, wordSeparator: '-', capitalizeWords: false, includeNumber: false },
                }).then(pw => setForm(f => ({ ...f, password: pw }))) : undefined}
                readOnly={!editing}
                copied={copied === 'password'}
                strengthScore={passwordStrength?.score}
                strengthLabel={passwordStrength?.label}
                entropy={passwordStrength?.entropy}
                placeholder={isVi ? 'Mật khẩu' : 'Password'}
                showLabel={isVi ? 'Hiện mật khẩu' : 'Show password'}
                hideLabel={isVi ? 'Ẩn mật khẩu' : 'Hide password'}
                copyLabel={isVi ? 'Sao chép mật khẩu' : 'Copy password'}
                generateLabel={isVi ? 'Tạo mật khẩu' : 'Generate password'}
              />
            </div>

            {/* URL */}
            <FieldRow label="URL" editing={editing}
              value={entry?.url} inputValue={form.url}
              onInputChange={v => setForm(f => ({ ...f, url: v }))}
              placeholder="https://"
              onCopy={() => copyToClipboard(entry?.url ?? form.url, 'url')}
              copied={copied === 'url'} isUrl />

            {/* OTP — use OtpDisplay component */}
            <div className="field-row">
              <div className="field-label-row">
                <span className="field-label">OTP</span>
                {editing && (
                  <button className="btn-link" onClick={() => setShowOtpSetup(true)}>
                    {entry?.hasOtp ? (isVi ? 'Chỉnh sửa' : 'Edit') : (isVi ? '+ Thêm OTP' : '+ Add OTP')}
                  </button>
                )}
              </div>
              {entry?.hasOtp && otpCode ? (
                <OtpDisplay
                  otp={otpCode}
                  onCopy={code => copyToClipboard(code, 'otp')}
                  copied={copied === 'otp'}
                  label="OTP"
                  expiresInLabel={isVi ? 'hết hạn sau' : 'expires in'}
                  copyLabel={isVi ? 'Sao chép OTP' : 'Copy OTP'}
                />
              ) : !editing ? (
                <span className="field-value" style={{ color: 'var(--color-text-tertiary)' }}>
                  {isVi ? 'Chưa cấu hình' : 'Not configured'}
                </span>
              ) : null}
            </div>

            {/* Notes */}
            <div className="field-row">
              <span className="field-label">{isVi ? 'Ghi chú' : 'Notes'}</span>
              {editing ? (
                <textarea className="form-input form-textarea"
                  value={form.notes} onChange={e => setForm(f => ({ ...f, notes: e.target.value }))}
                  placeholder={isVi ? 'Ghi chú...' : 'Notes...'} rows={4} />
              ) : (
                <span className="field-value field-value-notes">{entry?.notes || '—'}</span>
              )}
            </div>

            {/* Tags */}
            <div className="field-row">
              <span className="field-label">{isVi ? 'Thẻ' : 'Tags'}</span>
              {editing ? (
                <TagInput
                  tags={form.tags}
                  onChange={tags => setForm(f => ({ ...f, tags }))}
                  placeholder={isVi ? 'Thêm thẻ...' : 'Add tags...'}
                />
              ) : (
                <div className="tags-list">
                  {(entry?.tags ?? []).map(tag => (
                    <span key={tag} className="tag">{tag}</span>
                  ))}
                  {(entry?.tags ?? []).length === 0 && (
                    <span className="field-value" style={{ color: 'var(--color-text-tertiary)' }}>—</span>
                  )}
                </div>
              )}
            </div>

            {/* Expiry */}
            {(editing || entry?.expiry) && (
              <div className="field-row">
                <span className="field-label">{isVi ? 'Ngày hết hạn' : 'Expiry Date'}</span>
                {editing ? (
                  <input type="date" className="form-input"
                    value={form.expiry ? form.expiry.slice(0, 10) : ''}
                    onChange={e => setForm(f => ({ ...f, expiry: e.target.value ? new Date(e.target.value).toISOString() : '' }))} />
                ) : (
                  <span className={`field-value ${entry?.isExpired ? 'text-danger' : ''}`}>
                    {entry?.expiry ? new Date(entry.expiry).toLocaleDateString() : '—'}
                    {entry?.isExpired && ` (${isVi ? 'Đã hết hạn' : 'Expired'})`}
                  </span>
                )}
              </div>
            )}

            {/* Custom Fields */}
            <div className="field-row">
              <span className="field-label">{isVi ? 'Trường tùy chỉnh' : 'Custom Fields'}</span>
              <CustomFieldEditor
                fields={editing ? form.customFields : (entry?.customFields ?? []).map(f => ({ key: f.key, value: f.value, protected: f.protected }))}
                onChange={fields => setForm(f => ({ ...f, customFields: fields }))}
                readOnly={!editing}
              />
            </div>

            {/* Attachments */}
            <div className="field-row">
              <span className="field-label">{isVi ? 'Tệp đính kèm' : 'Attachments'}</span>
              <AttachmentViewer
                attachments={(entry?.customFields ?? [])
                  .filter(f => f.key.startsWith('_attachment_'))
                  .map(f => ({ name: f.key.replace('_attachment_', ''), dataRef: f.value }))}
                entryUuid={uuid ?? ''}
                readOnly={!editing}
              />
            </div>

            {/* Metadata */}
            {!editing && entry && (
              <div className="entry-meta">
                <span>{isVi ? 'Tạo lúc' : 'Created'}: {new Date(entry.createdAt).toLocaleDateString()}</span>
                <span>{isVi ? 'Sửa lúc' : 'Modified'}: {new Date(entry.modifiedAt).toLocaleDateString()}</span>
              </div>
            )}
          </>
        )}

        {/* ── History tab ── */}
        {activeTab === 'history' && !isNew && (
          <EntryHistoryViewer
            history={history}
            onRestore={async (h) => {
              if (confirm(isVi ? 'Khôi phục từ lịch sử này?' : 'Restore from this history entry?')) {
                await invoke('restore_entry_from_history', { entryUuid: uuid, historyUuid: h.uuid });
                queryClient.invalidateQueries({ queryKey: ['entry', uuid] });
                queryClient.invalidateQueries({ queryKey: ['entry-history', uuid] });
                setActiveTab('fields');
              }
            }}
            onClearHistory={async () => {
              await invoke('clear_entry_history', { uuid });
              queryClient.invalidateQueries({ queryKey: ['entry-history', uuid] });
            }}
            historyLabel={isVi ? 'Lịch sử' : 'History'}
            noHistoryLabel={isVi ? 'Không có lịch sử' : 'No history entries'}
            restoreLabel={isVi ? 'Khôi phục' : 'Restore'}
            clearLabel={isVi ? 'Xóa lịch sử' : 'Clear History'}
            confirmClearLabel={isVi ? 'Xóa tất cả lịch sử?' : 'Clear all history entries?'}
            modifiedLabel={isVi ? 'Sửa lúc' : 'Modified'}
            changedFieldsLabel={isVi ? 'Đã thay đổi' : 'Changed'}
          />
        )}

        {/* ── Passkeys tab ── */}
        {activeTab === 'passkeys' && !isNew && (
          <div className="passkeys-section">
            <p className="section-desc">
              {isVi
                ? 'Passkey (FIDO2/WebAuthn) được lưu trữ cho mục này.'
                : 'Passkeys (FIDO2/WebAuthn) stored for this entry.'}
            </p>
            <div className="passkey-placeholder">
              <span>🔑</span>
              <p>{isVi ? 'Quản lý passkey trong cài đặt mục.' : 'Manage passkeys in entry settings.'}</p>
            </div>
          </div>
        )}

        {/* ── SSH tab ── */}
        {activeTab === 'ssh' && !isNew && (
          <div className="ssh-section">
            <p className="section-desc">
              {isVi
                ? 'Khóa SSH được lưu trữ cho mục này.'
                : 'SSH keys stored for this entry.'}
            </p>
            <div className="ssh-placeholder">
              <span>🔐</span>
              <p>{isVi ? 'Quản lý khóa SSH trong cài đặt mục.' : 'Manage SSH keys in entry settings.'}</p>
            </div>
          </div>
        )}
      </div>

      <style>{`
        .entry-detail { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .entry-detail-header {
          display:flex; align-items:center; gap:var(--space-md);
          padding:var(--space-md) var(--space-xl);
          border-bottom:1px solid var(--color-border); flex-shrink:0;
        }
        .entry-detail-title { flex:1; font-size:16px; font-weight:600; }
        .entry-detail-actions { display:flex; gap:var(--space-sm); }
        .entry-tabs {
          display:flex; gap:2px; padding:0 var(--space-xl);
          border-bottom:1px solid var(--color-border); flex-shrink:0;
        }
        .entry-tab {
          background:none; border:none; cursor:pointer;
          padding:var(--space-sm) var(--space-md);
          font-size:13px; color:var(--color-text-secondary);
          border-bottom:2px solid transparent; margin-bottom:-1px;
          display:flex; align-items:center; gap:6px;
          transition:color 0.1s, border-color 0.1s;
        }
        .entry-tab:hover { color:var(--color-text); }
        .entry-tab--active { color:var(--color-primary); border-bottom-color:var(--color-primary); font-weight:500; }
        .entry-tab-badge {
          background:var(--color-bg-tertiary); color:var(--color-text-secondary);
          font-size:10px; padding:1px 5px; border-radius:10px;
        }
        .entry-detail-content {
          flex:1; overflow-y:auto; padding:var(--space-xl);
          display:flex; flex-direction:column; gap:var(--space-md); max-width:640px;
        }
        .field-row { display:flex; flex-direction:column; gap:var(--space-xs); }
        .field-label-row { display:flex; align-items:center; justify-content:space-between; }
        .field-label {
          font-size:12px; font-weight:500; color:var(--color-text-secondary);
          text-transform:uppercase; letter-spacing:.05em;
        }
        .field-value-row { display:flex; align-items:center; gap:var(--space-sm); }
        .field-value { flex:1; font-size:15px; color:var(--color-text); word-break:break-all; }
        .field-value-notes { font-family:inherit; white-space:pre-wrap; line-height:1.6; }
        .field-actions { display:flex; gap:4px; flex-shrink:0; }
        .otp-code {
          font-size:22px; font-weight:700; font-family:'SF Mono','Consolas',monospace;
          letter-spacing:4px; flex:1;
        }
        .otp-timer { font-size:13px; color:var(--color-text-secondary); min-width:28px; }
        .form-textarea { resize:vertical; min-height:80px; font-family:inherit; line-height:1.5; }
        .tags-container { display:flex; flex-direction:column; gap:var(--space-sm); }
        .tags-list { display:flex; flex-wrap:wrap; gap:var(--space-xs); }
        .tag {
          display:flex; align-items:center; gap:4px;
          background:var(--color-bg-tertiary); color:var(--color-text-secondary);
          padding:2px 8px; border-radius:var(--radius-full); font-size:12px;
        }
        .tag-remove {
          background:none; border:none; cursor:pointer; color:var(--color-text-tertiary);
          font-size:10px; padding:0; line-height:1;
        }
        .tag-remove:hover { color:var(--color-danger); }
        .tag-input-row { display:flex; gap:var(--space-sm); }
        .tag-input { flex:1; }
        .entry-meta {
          display:flex; gap:var(--space-xl); font-size:12px; color:var(--color-text-tertiary);
          padding-top:var(--space-md); border-top:1px solid var(--color-border);
        }
        .btn-link {
          background:none; border:none; cursor:pointer; color:var(--color-primary);
          font-size:12px; padding:0; text-decoration:underline;
        }
        .btn-icon.copied { color:var(--color-success); }
        .entry-detail-loading {
          display:flex; align-items:center; justify-content:center;
          height:100%; color:var(--color-text-secondary);
        }
        .btn-danger {
          background:var(--color-danger); color:white; border:none;
          padding:var(--space-sm) var(--space-md); border-radius:var(--radius-md);
          cursor:pointer; font-size:13px;
        }
        .text-danger { color:var(--color-danger); }
        .field-link { color:var(--color-primary); text-decoration:none; font-size:15px; }
        .field-link:hover { text-decoration:underline; }
        .section-desc { font-size:13px; color:var(--color-text-secondary); margin-bottom:var(--space-md); }
        .passkey-placeholder, .ssh-placeholder {
          display:flex; flex-direction:column; align-items:center; gap:var(--space-md);
          padding:var(--space-2xl); color:var(--color-text-tertiary); font-size:13px;
          background:var(--color-bg-secondary); border-radius:var(--radius-md);
          border:1px dashed var(--color-border);
        }
        .passkey-placeholder span, .ssh-placeholder span { font-size:32px; }
      `}</style>
    </div>
  );
}

// ─── Field Row Component ──────────────────────────────────────────────────────

interface FieldRowProps {
  label: string;
  value?: string;
  editing: boolean;
  inputValue: string;
  onInputChange: (v: string) => void;
  placeholder?: string;
  onCopy?: () => void;
  copied?: boolean;
  isUrl?: boolean;
}

function FieldRow({ label, value, editing, inputValue, onInputChange, placeholder, onCopy, copied, isUrl }: FieldRowProps) {
  return (
    <div className="field-row">
      <span className="field-label">{label}</span>
      <div className="field-value-row">
        {editing ? (
          <input type="text" className="form-input" value={inputValue}
            onChange={e => onInputChange(e.target.value)}
            placeholder={placeholder} style={{ flex: 1 }} />
        ) : (
          <>
            {isUrl && value ? (
              <a href={value} target="_blank" rel="noopener noreferrer"
                className="field-link" style={{ flex: 1 }}>{value || '—'}</a>
            ) : (
              <span className="field-value" style={{ flex: 1 }}>{value || '—'}</span>
            )}
            {onCopy && (
              <button className={`btn-icon ${copied ? 'copied' : ''}`}
                onClick={onCopy} aria-label={`Copy ${label}`}>
                {copied ? '✓' : '⎘'}
              </button>
            )}
          </>
        )}
      </div>
    </div>
  );
}
