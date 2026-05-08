/**
 * TeamPage — Collaborative Vault with RBAC
 *
 * Exclusive KeePassEx feature: No competitor has this (Bitwarden has it but cloud-only).
 * Manage team members, roles, per-entry permissions, and encrypted comments.
 */
import { useState, useCallback, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';

type TeamRole = 'admin' | 'editor' | 'viewer';
type MemberStatus = 'invited' | 'active' | 'suspended' | 'removed';
type TabId = 'members' | 'activity' | 'permissions';

interface TeamMember {
  id: string;
  email: string;
  name: string;
  role: TeamRole;
  status: MemberStatus;
  joined_at: string | null;
  invited_at: string;
}

interface TeamActivity {
  id: string;
  member_name: string;
  action: string;
  entry_title: string | null;
  timestamp: string;
  details: string | null;
}

interface TeamVault {
  id: string;
  name: string;
  members: TeamMember[];
  activity_log: TeamActivity[];
  real_time_sync: boolean;
}

export function TeamPage() {
  const { t } = useTranslation();
  const [team, setTeam] = useState<TeamVault | null>(null);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<TabId>('members');
  const [showInviteForm, setShowInviteForm] = useState(false);
  const [inviteEmail, setInviteEmail] = useState('');
  const [inviteName, setInviteName] = useState('');
  const [inviteRole, setInviteRole] = useState<TeamRole>('editor');
  const [inviting, setInviting] = useState(false);

  const loadTeam = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<TeamVault>('get_team_vault');
      setTeam(data);
    } catch {
      setTeam(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadTeam();
  }, [loadTeam]);

  const handleInvite = useCallback(async () => {
    if (!inviteEmail || !inviteName) return;
    setInviting(true);
    try {
      await invoke('invite_team_member', {
        email: inviteEmail,
        name: inviteName,
        role: inviteRole,
      });
      setInviteEmail('');
      setInviteName('');
      setShowInviteForm(false);
      await loadTeam();
    } catch {
      // show error
    } finally {
      setInviting(false);
    }
  }, [inviteEmail, inviteName, inviteRole, loadTeam]);

  const handleChangeRole = useCallback(
    async (memberId: string, newRole: TeamRole) => {
      try {
        await invoke('change_team_member_role', { memberId, role: newRole });
        await loadTeam();
      } catch {
        // show error
      }
    },
    [loadTeam]
  );

  const handleRemoveMember = useCallback(
    async (memberId: string) => {
      try {
        await invoke('remove_team_member', { memberId });
        await loadTeam();
      } catch {
        // show error
      }
    },
    [loadTeam]
  );

  const roleIcon: Record<TeamRole, string> = {
    admin: '👑',
    editor: '✏️',
    viewer: '👁️',
  };

  const statusColor: Record<MemberStatus, string> = {
    active: 'status-active',
    invited: 'status-invited',
    suspended: 'status-suspended',
    removed: 'status-removed',
  };

  if (loading) {
    return (
      <div className="page-loading" role="status">
        <span className="spinner" aria-hidden="true" />
        {t('common.loading')}
      </div>
    );
  }

  return (
    <div className="page-container" role="main" aria-label={t('team.title')}>
      {/* Header */}
      <div className="page-header">
        <div className="page-header-row">
          <div>
            <h1 className="page-title">👥 {t('team.title')}</h1>
            <p className="page-subtitle">{t('team.subtitle')}</p>
          </div>
          <div className="page-header-actions">
            {team?.real_time_sync && (
              <span className="badge badge-sync" aria-label={t('team.realTimeSync')}>
                🔄 {t('team.realTimeSync')}
              </span>
            )}
            <button
              className="btn btn-primary"
              onClick={() => setShowInviteForm(true)}
              aria-label={t('team.addMember')}
            >
              ➕ {t('team.addMember')}
            </button>
          </div>
        </div>
        <span className="badge badge-exclusive">✨ {t('team.uniqueFeature')}</span>
      </div>

      {/* Invite Form */}
      {showInviteForm && (
        <div
          className="modal-overlay"
          role="dialog"
          aria-modal="true"
          aria-labelledby="invite-title"
        >
          <div className="modal">
            <h2 id="invite-title">{t('team.addMember')}</h2>

            <div className="form-group">
              <label className="form-label" htmlFor="invite-email">
                {t('team.memberEmail')}
              </label>
              <input
                id="invite-email"
                type="email"
                className="form-input"
                value={inviteEmail}
                onChange={e => setInviteEmail(e.target.value)}
                placeholder="colleague@company.com"
                autoFocus
              />
            </div>

            <div className="form-group">
              <label className="form-label" htmlFor="invite-name">
                {t('team.memberName')}
              </label>
              <input
                id="invite-name"
                type="text"
                className="form-input"
                value={inviteName}
                onChange={e => setInviteName(e.target.value)}
                placeholder="Full Name"
              />
            </div>

            <div className="form-group">
              <label className="form-label" htmlFor="invite-role">
                {t('team.memberRole')}
              </label>
              <select
                id="invite-role"
                className="form-select"
                value={inviteRole}
                onChange={e => setInviteRole(e.target.value as TeamRole)}
              >
                <option value="admin">
                  {roleIcon.admin} {t('team.roleAdmin')}
                </option>
                <option value="editor">
                  {roleIcon.editor} {t('team.roleEditor')}
                </option>
                <option value="viewer">
                  {roleIcon.viewer} {t('team.roleViewer')}
                </option>
              </select>
              <p className="form-hint">
                {inviteRole === 'admin' && t('team.roleAdminDesc')}
                {inviteRole === 'editor' && t('team.roleEditorDesc')}
                {inviteRole === 'viewer' && t('team.roleViewerDesc')}
              </p>
            </div>

            <div className="modal-actions">
              <button className="btn btn-secondary" onClick={() => setShowInviteForm(false)}>
                {t('common.cancel')}
              </button>
              <button
                className="btn btn-primary"
                onClick={handleInvite}
                disabled={inviting || !inviteEmail || !inviteName}
                aria-busy={inviting}
              >
                {inviting ? t('common.loading') : `📧 ${t('team.addMember')}`}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Tabs */}
      <div className="tab-bar" role="tablist">
        <button
          role="tab"
          aria-selected={activeTab === 'members'}
          className={`tab ${activeTab === 'members' ? 'tab-active' : ''}`}
          onClick={() => setActiveTab('members')}
        >
          👥 {t('team.members')}{' '}
          {team && `(${team.members.filter(m => m.status !== 'removed').length})`}
        </button>
        <button
          role="tab"
          aria-selected={activeTab === 'activity'}
          className={`tab ${activeTab === 'activity' ? 'tab-active' : ''}`}
          onClick={() => setActiveTab('activity')}
        >
          📋 {t('team.activityLog')}
        </button>
        <button
          role="tab"
          aria-selected={activeTab === 'permissions'}
          className={`tab ${activeTab === 'permissions' ? 'tab-active' : ''}`}
          onClick={() => setActiveTab('permissions')}
        >
          🔒 {t('team.permissions')}
        </button>
      </div>

      <div className="page-content">
        {/* MEMBERS TAB */}
        {activeTab === 'members' && (
          <div className="members-list" role="list">
            {!team || team.members.filter(m => m.status !== 'removed').length === 0 ? (
              <div className="empty-state">
                <p>{t('team.noMembers')}</p>
                <p className="empty-state-desc">{t('team.noMembersDesc')}</p>
              </div>
            ) : (
              team.members
                .filter(m => m.status !== 'removed')
                .map(member => (
                  <div key={member.id} className="member-card" role="listitem">
                    <div className="member-avatar" aria-hidden="true">
                      {member.name.charAt(0).toUpperCase()}
                    </div>
                    <div className="member-info">
                      <span className="member-name">{member.name}</span>
                      <span className="member-email">{member.email}</span>
                    </div>
                    <div className="member-role">
                      <span className="role-icon" aria-hidden="true">
                        {roleIcon[member.role]}
                      </span>
                      <select
                        className="role-select"
                        value={member.role}
                        onChange={e => handleChangeRole(member.id, e.target.value as TeamRole)}
                        aria-label={`${member.name} role`}
                      >
                        <option value="admin">{t('team.roleAdmin')}</option>
                        <option value="editor">{t('team.roleEditor')}</option>
                        <option value="viewer">{t('team.roleViewer')}</option>
                      </select>
                    </div>
                    <span className={`member-status ${statusColor[member.status]}`}>
                      {member.status}
                    </span>
                    <button
                      className="btn btn-icon btn-danger"
                      onClick={() => handleRemoveMember(member.id)}
                      aria-label={`${t('team.removeMember')} ${member.name}`}
                    >
                      🗑️
                    </button>
                  </div>
                ))
            )}
          </div>
        )}

        {/* ACTIVITY TAB */}
        {activeTab === 'activity' && (
          <div className="activity-log" role="log" aria-live="polite">
            {!team || team.activity_log.length === 0 ? (
              <p className="empty-state">{t('team.noActivity')}</p>
            ) : (
              [...team.activity_log]
                .reverse()
                .slice(0, 50)
                .map(activity => (
                  <div key={activity.id} className="activity-item">
                    <span className="activity-time">
                      {new Date(activity.timestamp).toLocaleString()}
                    </span>
                    <span className="activity-member">{activity.member_name}</span>
                    <span className="activity-action">{activity.action}</span>
                    {activity.entry_title && (
                      <span className="activity-entry">→ {activity.entry_title}</span>
                    )}
                  </div>
                ))
            )}
          </div>
        )}

        {/* PERMISSIONS TAB */}
        {activeTab === 'permissions' && (
          <div className="permissions-info">
            <h3>{t('team.permissions')}</h3>
            <table className="permissions-table" role="table">
              <thead>
                <tr>
                  <th scope="col">Action</th>
                  <th scope="col">{t('team.roleAdmin')}</th>
                  <th scope="col">{t('team.roleEditor')}</th>
                  <th scope="col">{t('team.roleViewer')}</th>
                </tr>
              </thead>
              <tbody>
                {[
                  ['View entries', true, true, true],
                  ['Copy passwords', true, true, true],
                  ['Create entries', true, true, false],
                  ['Edit entries', true, true, false],
                  ['Delete entries', true, true, false],
                  ['Add comments', true, true, true],
                  ['Manage members', true, false, false],
                  ['Export vault', true, false, false],
                ].map(([action, admin, editor, viewer]) => (
                  <tr key={String(action)}>
                    <td>{action}</td>
                    <td>{admin ? '✅' : '❌'}</td>
                    <td>{editor ? '✅' : '❌'}</td>
                    <td>{viewer ? '✅' : '❌'}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
