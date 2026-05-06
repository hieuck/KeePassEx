/**
 * Welcome page — shown when no vault is open
 * Shows recent vaults and open/create actions
 */
import React, { useState } from 'react';
import { open, save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useVaultStore } from '../store/vault';
import { useSettingsStore } from '../store/settings';
import type { RecentVault } from '@keepassex/types';

export function WelcomePage() {
  const { openVault, createVault } = useVaultStore();
  const { settings } = useSettingsStore();
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [recentVaults, setRecentVaults] = useState<RecentVault[]>(
    settings.recentVaults ?? []
  );
  const isVi = settings.language === 'vi';

  const handleOpenVault = async (filePath?: string) => {
    setError(null);
    let path = filePath;

    if (!path) {
      const selected = await open({
        filters: [{ name: 'KeePass Database', extensions: ['kdbx', 'kdb'] }],
        multiple: false,
      });
      if (!selected || typeof selected !== 'string') return;
      path = selected;
    }

    const password = window.prompt(
      isVi ? 'Nhập mật khẩu chính:' : 'Enter master password:'
    );
    if (password === null) return;

    setLoading(true);
    try {
      await openVault(path, password);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleCreateVault = async () => {
    setError(null);

    const name = window.prompt(
      isVi ? 'Tên kho mật khẩu:' : 'Vault name:',
      isVi ? 'Kho của tôi' : 'My Vault'
    );
    if (!name?.trim()) return;

    const password = window.prompt(
      isVi ? 'Mật khẩu chính (hãy chọn mật khẩu mạnh):' : 'Master password (choose a strong one):'
    );
    if (!password) return;

    const confirm = window.prompt(
      isVi ? 'Xác nhận mật khẩu chính:' : 'Confirm master password:'
    );
    if (confirm !== password) {
      setError(isVi ? 'Mật khẩu không khớp' : 'Passwords do not match');
      return;
    }

    const filePath = await save({
      filters: [{ name: 'KeePass Database', extensions: ['kdbx'] }],
      defaultPath: `${name.trim()}.kdbx`,
    });
    if (!filePath) return;

    setLoading(true);
    try {
      await createVault(filePath, name.trim(), password);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  const formatRelativeTime = (dateStr: string): string => {
    const date = new Date(dateStr);
    const now = new Date();
    const diffDays = Math.floor((now.getTime() - date.getTime()) / 86_400_000);
    if (diffDays === 0) return isVi ? 'Hôm nay' : 'Today';
    if (diffDays === 1) return isVi ? 'Hôm qua' : 'Yesterday';
    if (diffDays < 7) return isVi ? `${diffDays} ngày trước` : `${diffDays} days ago`;
    return date.toLocaleDateString(isVi ? 'vi-VN' : 'en-US', { month: 'short', day: 'numeric' });
  };

  return (
    <div className="welcome-page">
      {/* Hero */}
      <div className="welcome-hero">
        <div className="app-logo" aria-hidden="true">🔐</div>
        <h1 className="app-name">KeePassEx</h1>
        <p className="app-tagline">
          {isVi ? 'Mật khẩu của bạn, quyền kiểm soát của bạn' : 'Your passwords, your control'}
        </p>
      </div>

      {/* Main actions */}
      <div className="welcome-actions">
        <button
          className="btn btn-primary btn-lg"
          onClick={() => handleOpenVault()}
          disabled={loading}
          aria-label={isVi ? 'Mở kho mật khẩu' : 'Open vault'}
        >
          {loading ? '⏳' : '📂'} {isVi ? 'Mở kho mật khẩu' : 'Open Vault'}
        </button>

        <button
          className="btn btn-secondary btn-lg"
          onClick={handleCreateVault}
          disabled={loading}
          aria-label={isVi ? 'Tạo kho mới' : 'Create new vault'}
        >
          ✨ {isVi ? 'Tạo kho mới' : 'Create New Vault'}
        </button>
      </div>

      {/* Error */}
      {error && (
        <div className="error-banner" role="alert">
          ⚠️ {error}
        </div>
      )}

      {/* Recent vaults */}
      {recentVaults.length > 0 && (
        <div className="recent-vaults">
          <h2 className="recent-title">
            {isVi ? '🕐 Kho gần đây' : '🕐 Recent Vaults'}
          </h2>
          <div className="recent-list" role="list">
            {recentVaults.slice(0, 5).map(vault => (
              <button
                key={vault.path}
                className="recent-item"
                onClick={() => handleOpenVault(vault.path)}
                disabled={loading}
                role="listitem"
                aria-label={`Open ${vault.name}`}
              >
                <span className="recent-icon" aria-hidden="true">🔐</span>
                <div className="recent-info">
                  <span className="recent-name">{vault.name}</span>
                  <span className="recent-path" title={vault.path}>
                    {vault.path.length > 50
                      ? '...' + vault.path.slice(-47)
                      : vault.path}
                  </span>
                </div>
                <span className="recent-time">
                  {formatRelativeTime(vault.lastOpened)}
                </span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Feature highlights */}
      <div className="welcome-features" aria-label="Key features">
        {[
          { icon: '🔒', en: 'Argon2id + ChaCha20 encryption', vi: 'Mã hóa Argon2id + ChaCha20' },
          { icon: '📱', en: 'Native on all platforms', vi: 'Native trên mọi nền tảng' },
          { icon: '🔑', en: 'Passkey & SSH support', vi: 'Hỗ trợ Passkey & SSH' },
          { icon: '🛡️', en: 'Breach monitoring', vi: 'Kiểm tra rò rỉ dữ liệu' },
          { icon: '🔄', en: 'Multi-provider sync', vi: 'Đồng bộ đa nhà cung cấp' },
          { icon: '🌐', en: 'Browser extension', vi: 'Tiện ích trình duyệt' },
        ].map(f => (
          <div key={f.en} className="feature-item">
            <span aria-hidden="true">{f.icon}</span>
            <span>{isVi ? f.vi : f.en}</span>
          </div>
        ))}
      </div>

      <style>{`
        .welcome-page {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          min-height: 100vh;
          gap: var(--space-xl);
          padding: var(--space-2xl);
          background: var(--color-bg);
          overflow-y: auto;
        }
        .welcome-hero { text-align: center; }
        .app-logo { font-size: 72px; line-height: 1; margin-bottom: var(--space-md); }
        .app-name { font-size: 36px; font-weight: 700; color: var(--color-text); }
        .app-tagline { font-size: 16px; color: var(--color-text-secondary); margin-top: var(--space-sm); }
        .welcome-actions {
          display: flex;
          flex-direction: column;
          gap: var(--space-md);
          width: 100%;
          max-width: 340px;
        }
        .recent-vaults {
          width: 100%;
          max-width: 480px;
        }
        .recent-title {
          font-size: 13px;
          font-weight: 600;
          color: var(--color-text-secondary);
          text-transform: uppercase;
          letter-spacing: 0.08em;
          margin-bottom: var(--space-sm);
          padding: 0 var(--space-xs);
        }
        .recent-list {
          display: flex;
          flex-direction: column;
          gap: 2px;
        }
        .recent-item {
          display: flex;
          align-items: center;
          gap: var(--space-md);
          padding: var(--space-sm) var(--space-md);
          background: var(--color-bg-secondary);
          border: 1px solid var(--color-border);
          border-radius: var(--radius-md);
          cursor: pointer;
          text-align: left;
          transition: background 0.1s, border-color 0.1s;
          width: 100%;
        }
        .recent-item:hover:not(:disabled) {
          background: var(--color-bg-tertiary);
          border-color: var(--color-primary);
        }
        .recent-item:disabled { opacity: 0.5; cursor: not-allowed; }
        .recent-icon { font-size: 20px; flex-shrink: 0; }
        .recent-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
        .recent-name { font-size: 14px; font-weight: 500; color: var(--color-text); }
        .recent-path {
          font-size: 11px;
          color: var(--color-text-tertiary);
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }
        .recent-time { font-size: 12px; color: var(--color-text-tertiary); flex-shrink: 0; }
        .welcome-features {
          display: grid;
          grid-template-columns: repeat(3, 1fr);
          gap: var(--space-sm);
          max-width: 480px;
          width: 100%;
        }
        .feature-item {
          display: flex;
          align-items: center;
          gap: var(--space-sm);
          font-size: 12px;
          color: var(--color-text-secondary);
          padding: var(--space-sm);
          background: var(--color-bg-secondary);
          border-radius: var(--radius-sm);
        }
        @media (max-width: 600px) {
          .welcome-features { grid-template-columns: repeat(2, 1fr); }
        }
      `}</style>
    </div>
  );
}
