/**
 * Audit Log page — view vault access and modification events
 * Unique feature: no competitor has this
 */
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../store/settings';

interface AuditEvent {
  id: string;
  event_type: string;
  timestamp: string;
  platform: string;
  entry_uuid?: string;
  entry_title?: string;
  details?: string;
}

type FilterType = 'all' | 'security' | 'access' | 'modifications';

const EVENT_ICONS: Record<string, string> = {
  vault_opened: '🔓',
  vault_locked: '🔒',
  vault_saved: '💾',
  entry_viewed: '👁',
  entry_created: '✨',
  entry_modified: '✏️',
  entry_deleted: '🗑',
  entry_moved: '📦',
  password_copied: '⎘',
  otp_generated: '⏱',
  emergency_access_requested: '🆘',
  emergency_access_granted: '✅',
  emergency_access_revoked: '❌',
  sync_completed: '🔄',
  sync_failed: '⚠️',
  import_completed: '📥',
  export_completed: '📤',
  master_password_changed: '🔑',
  hardware_key_added: '🔐',
  hardware_key_removed: '🔓',
  biometric_unlock: '👆',
  failed_unlock_attempt: '🚫',
  plugin_installed: '🔧',
  plugin_uninstalled: '🗑',
};

const SECURITY_EVENTS = new Set([
  'vault_opened',
  'vault_locked',
  'master_password_changed',
  'hardware_key_added',
  'hardware_key_removed',
  'biometric_unlock',
  'failed_unlock_attempt',
  'emergency_access_requested',
  'emergency_access_granted',
  'emergency_access_revoked',
]);

const ACCESS_EVENTS = new Set(['entry_viewed', 'password_copied', 'otp_generated']);

const MODIFICATION_EVENTS = new Set([
  'entry_created',
  'entry_modified',
  'entry_deleted',
  'entry_moved',
  'vault_saved',
  'import_completed',
  'export_completed',
  'plugin_installed',
  'plugin_uninstalled',
]);

export function AuditLogPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { settings } = useSettingsStore();
  const isVi = settings.language === 'vi';

  const [filter, setFilter] = useState<FilterType>('all');
  const [search, setSearch] = useState('');
  const [limit, setLimit] = useState(100);

  const { data: events = [], isLoading } = useQuery({
    queryKey: ['audit-log', limit],
    queryFn: () => invoke<AuditEvent[]>('get_audit_log', { limit }),
    staleTime: 5_000,
  });

  const clearMutation = useMutation({
    mutationFn: () => invoke('clear_audit_log'),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['audit-log'] }),
  });

  const exportMutation = useMutation({
    mutationFn: () => invoke('export_audit_log'),
  });

  const filteredEvents = events.filter(e => {
    if (filter === 'security' && !SECURITY_EVENTS.has(e.event_type)) return false;
    if (filter === 'access' && !ACCESS_EVENTS.has(e.event_type)) return false;
    if (filter === 'modifications' && !MODIFICATION_EVENTS.has(e.event_type)) return false;
    if (search) {
      const q = search.toLowerCase();
      return (
        e.event_type.includes(q) ||
        (e.entry_title?.toLowerCase().includes(q) ?? false) ||
        (e.details?.toLowerCase().includes(q) ?? false) ||
        e.platform.toLowerCase().includes(q)
      );
    }
    return true;
  });

  const failedAttempts = events.filter(e => e.event_type === 'failed_unlock_attempt').length;

  const formatEventType = (type: string): string => {
    const labels: Record<string, [string, string]> = {
      vault_opened: ['Vault opened', 'Đã mở kho'],
      vault_locked: ['Vault locked', 'Đã khóa kho'],
      vault_saved: ['Vault saved', 'Đã lưu kho'],
      entry_viewed: ['Entry viewed', 'Đã xem mục'],
      entry_created: ['Entry created', 'Đã tạo mục'],
      entry_modified: ['Entry modified', 'Đã sửa mục'],
      entry_deleted: ['Entry deleted', 'Đã xóa mục'],
      entry_moved: ['Entry moved', 'Đã di chuyển mục'],
      password_copied: ['Password copied', 'Đã sao chép mật khẩu'],
      otp_generated: ['OTP generated', 'Đã tạo OTP'],
      emergency_access_requested: ['Emergency access requested', 'Yêu cầu truy cập khẩn cấp'],
      emergency_access_granted: ['Emergency access granted', 'Đã cấp quyền khẩn cấp'],
      emergency_access_revoked: ['Emergency access revoked', 'Đã thu hồi quyền khẩn cấp'],
      sync_completed: ['Sync completed', 'Đồng bộ hoàn tất'],
      sync_failed: ['Sync failed', 'Đồng bộ thất bại'],
      import_completed: ['Import completed', 'Nhập hoàn tất'],
      export_completed: ['Export completed', 'Xuất hoàn tất'],
      master_password_changed: ['Master password changed', 'Đã đổi mật khẩu chính'],
      hardware_key_added: ['Hardware key added', 'Đã thêm khóa phần cứng'],
      hardware_key_removed: ['Hardware key removed', 'Đã xóa khóa phần cứng'],
      biometric_unlock: ['Biometric unlock', 'Mở khóa sinh trắc học'],
      failed_unlock_attempt: ['Failed unlock attempt', 'Thử mở khóa thất bại'],
      plugin_installed: ['Plugin installed', 'Đã cài plugin'],
      plugin_uninstalled: ['Plugin uninstalled', 'Đã gỡ plugin'],
    };
    const pair = labels[type];
    if (!pair) return type.replace(/_/g, ' ');
    return isVi ? pair[1] : pair[0];
  };

  return (
    <div className="audit-page">
      {/* Header */}
      <div className="audit-header">
        <button className="btn-back" onClick={() => navigate('/settings')}>
          ← {isVi ? 'Cài đặt' : 'Settings'}
        </button>
        <div className="audit-header-main">
          <h2>{isVi ? '📋 Nhật ký kiểm tra' : '📋 Audit Log'}</h2>
          <div className="audit-header-actions">
            {failedAttempts > 0 && (
              <div className="audit-alert" role="alert">
                🚫 {failedAttempts} {isVi ? 'lần thử mở khóa thất bại' : 'failed unlock attempt(s)'}
              </div>
            )}
            <button
              className="btn btn-secondary"
              onClick={() => exportMutation.mutate()}
              disabled={exportMutation.isPending}
            >
              {isVi ? '📤 Xuất' : '📤 Export'}
            </button>
            <button
              className="btn btn-danger-outline"
              onClick={() => {
                if (confirm(isVi ? 'Xóa tất cả nhật ký?' : 'Clear all audit log entries?')) {
                  clearMutation.mutate();
                }
              }}
              disabled={clearMutation.isPending}
            >
              {isVi ? '🗑 Xóa nhật ký' : '🗑 Clear Log'}
            </button>
          </div>
        </div>
      </div>

      {/* Filters */}
      <div className="audit-filters">
        <div className="audit-filter-tabs" role="tablist">
          {[
            { id: 'all', label: isVi ? 'Tất cả' : 'All' },
            { id: 'security', label: isVi ? 'Bảo mật' : 'Security' },
            { id: 'access', label: isVi ? 'Truy cập' : 'Access' },
            { id: 'modifications', label: isVi ? 'Thay đổi' : 'Modifications' },
          ].map(f => (
            <button
              key={f.id}
              role="tab"
              aria-selected={filter === f.id}
              className={`audit-tab${filter === f.id ? ' active' : ''}`}
              onClick={() => setFilter(f.id as FilterType)}
            >
              {f.label}
            </button>
          ))}
        </div>
        <input
          type="search"
          className="audit-search form-input"
          placeholder={isVi ? 'Tìm kiếm sự kiện...' : 'Search events...'}
          value={search}
          onChange={e => setSearch(e.target.value)}
          aria-label={isVi ? 'Tìm kiếm nhật ký' : 'Search audit log'}
        />
      </div>

      {/* Stats bar */}
      <div className="audit-stats">
        <span>
          {filteredEvents.length} {isVi ? 'sự kiện' : 'events'}
        </span>
        {events.length > 0 && (
          <span className="audit-stats-range">
            {new Date(events[events.length - 1]?.timestamp).toLocaleDateString()}
            {' — '}
            {new Date(events[0]?.timestamp).toLocaleDateString()}
          </span>
        )}
        <select
          className="audit-limit"
          value={limit}
          onChange={e => setLimit(Number(e.target.value))}
          aria-label={isVi ? 'Số sự kiện hiển thị' : 'Events to show'}
        >
          <option value={50}>50</option>
          <option value={100}>100</option>
          <option value={500}>500</option>
          <option value={1000}>1000</option>
        </select>
      </div>

      {/* Event list */}
      <div className="audit-list" role="log" aria-label={isVi ? 'Nhật ký kiểm tra' : 'Audit log'}>
        {isLoading ? (
          <div className="audit-empty">⏳ {isVi ? 'Đang tải...' : 'Loading...'}</div>
        ) : filteredEvents.length === 0 ? (
          <div className="audit-empty">
            <span>📋</span>
            <p>{isVi ? 'Không có sự kiện nào' : 'No events found'}</p>
          </div>
        ) : (
          filteredEvents.map(event => (
            <div
              key={event.id}
              className={`audit-event${event.event_type === 'failed_unlock_attempt' ? ' audit-event--danger' : ''}`}
            >
              <span className="audit-event-icon" aria-hidden="true">
                {EVENT_ICONS[event.event_type] ?? '📌'}
              </span>
              <div className="audit-event-info">
                <span className="audit-event-type">{formatEventType(event.event_type)}</span>
                {event.entry_title && (
                  <span className="audit-event-entry">"{event.entry_title}"</span>
                )}
                {event.details && <span className="audit-event-details">{event.details}</span>}
              </div>
              <div className="audit-event-meta">
                <span className="audit-event-platform">{event.platform}</span>
                <span className="audit-event-time">
                  {new Date(event.timestamp).toLocaleString()}
                </span>
              </div>
            </div>
          ))
        )}
      </div>

      <style>{`
        .audit-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .audit-header {
          padding:var(--space-md) var(--space-xl);
          border-bottom:1px solid var(--color-border); flex-shrink:0;
        }
        .btn-back {
          background:none; border:none; cursor:pointer; color:var(--color-primary);
          font-size:13px; padding:0; margin-bottom:4px;
        }
        .audit-header-main { display:flex; align-items:center; justify-content:space-between; }
        .audit-header-main h2 { font-size:16px; font-weight:700; }
        .audit-header-actions { display:flex; align-items:center; gap:var(--space-sm); }
        .audit-alert {
          font-size:12px; padding:4px 10px; background:rgba(239,68,68,.1);
          border:1px solid rgba(239,68,68,.3); border-radius:var(--radius-sm);
          color:#ef4444;
        }
        .audit-filters {
          display:flex; align-items:center; gap:var(--space-md);
          padding:var(--space-sm) var(--space-xl);
          border-bottom:1px solid var(--color-border); flex-shrink:0;
        }
        .audit-filter-tabs { display:flex; gap:2px; }
        .audit-tab {
          background:none; border:none; cursor:pointer;
          padding:var(--space-xs) var(--space-md); font-size:13px;
          color:var(--color-text-secondary); border-radius:var(--radius-sm);
          transition:background .1s, color .1s;
        }
        .audit-tab:hover { background:var(--color-bg-secondary); color:var(--color-text); }
        .audit-tab.active { background:var(--color-primary); color:white; }
        .audit-search { flex:1; max-width:280px; }
        .audit-stats {
          display:flex; align-items:center; gap:var(--space-md);
          padding:var(--space-xs) var(--space-xl);
          font-size:12px; color:var(--color-text-secondary);
          border-bottom:1px solid var(--color-border); flex-shrink:0;
        }
        .audit-stats-range { flex:1; }
        .audit-limit {
          background:var(--color-bg); border:1px solid var(--color-border);
          border-radius:var(--radius-sm); padding:2px 6px; font-size:12px;
          color:var(--color-text); cursor:pointer;
        }
        .audit-list { flex:1; overflow-y:auto; }
        .audit-empty {
          display:flex; flex-direction:column; align-items:center; gap:var(--space-md);
          padding:var(--space-2xl); color:var(--color-text-secondary); font-size:13px;
        }
        .audit-empty span { font-size:32px; }
        .audit-event {
          display:flex; align-items:flex-start; gap:var(--space-md);
          padding:var(--space-sm) var(--space-xl);
          border-bottom:1px solid var(--color-border);
          transition:background .1s;
        }
        .audit-event:hover { background:var(--color-bg-secondary); }
        .audit-event--danger { background:rgba(239,68,68,.04); }
        .audit-event--danger:hover { background:rgba(239,68,68,.08); }
        .audit-event-icon { font-size:16px; flex-shrink:0; margin-top:1px; }
        .audit-event-info { flex:1; display:flex; flex-direction:column; gap:2px; }
        .audit-event-type { font-size:13px; font-weight:500; color:var(--color-text); }
        .audit-event-entry { font-size:12px; color:var(--color-primary); }
        .audit-event-details { font-size:11px; color:var(--color-text-secondary); }
        .audit-event-meta { display:flex; flex-direction:column; align-items:flex-end; gap:2px; flex-shrink:0; }
        .audit-event-platform {
          font-size:10px; padding:1px 6px; background:var(--color-bg-tertiary);
          border-radius:var(--radius-full); color:var(--color-text-secondary);
        }
        .audit-event-time { font-size:11px; color:var(--color-text-tertiary); }
        .btn-danger-outline {
          background:none; border:1px solid rgba(239,68,68,.4); color:#ef4444;
          border-radius:var(--radius-sm); padding:var(--space-xs) var(--space-md);
          cursor:pointer; font-size:13px;
        }
        .btn-danger-outline:hover { background:rgba(239,68,68,.08); }
      `}</style>
    </div>
  );
}
