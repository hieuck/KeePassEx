/**
 * VaultTabBar — Multi-vault tab bar component
 *
 * Displays open vault tabs at the top of the app, allowing users to switch
 * between multiple open vaults. Supports drag-to-reorder, close, and lock indicators.
 *
 * This feature surpasses KeePass/KeePassXC which only support one vault at a time.
 */
import { useCallback, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useTabsStore, type VaultTab } from '../store/tabs';

interface VaultTabBarProps {
  onOpenNewVault: () => void;
}

export function VaultTabBar({ onOpenNewVault }: VaultTabBarProps) {
  const { t } = useTranslation();
  const { tabs, activeTabId, switchTab, closeTab, lockTab } = useTabsStore();
  const dragTabId = useRef<string | null>(null);
  const dragOverTabId = useRef<string | null>(null);

  const handleTabClick = useCallback(
    (tabId: string) => {
      switchTab(tabId);
    },
    [switchTab]
  );

  const handleCloseTab = useCallback(
    (e: React.MouseEvent, tabId: string) => {
      e.stopPropagation();
      closeTab(tabId);
    },
    [closeTab]
  );

  const handleLockTab = useCallback(
    (e: React.MouseEvent, tabId: string) => {
      e.stopPropagation();
      lockTab(tabId);
    },
    [lockTab]
  );

  const handleDragStart = useCallback((tabId: string) => {
    dragTabId.current = tabId;
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent, tabId: string) => {
    e.preventDefault();
    dragOverTabId.current = tabId;
  }, []);

  const handleDrop = useCallback(() => {
    if (!dragTabId.current || !dragOverTabId.current) return;
    if (dragTabId.current === dragOverTabId.current) return;

    const { reorderTab, tabs: currentTabs } = useTabsStore.getState();
    const newIndex = currentTabs.findIndex(t => t.id === dragOverTabId.current);
    reorderTab(dragTabId.current, newIndex);

    dragTabId.current = null;
    dragOverTabId.current = null;
  }, []);

  if (tabs.length === 0) return null;

  return (
    <div
      role="tablist"
      aria-label={t('tabs.tabList')}
      style={{
        display: 'flex',
        alignItems: 'center',
        height: 36,
        background: 'var(--color-surface-2)',
        borderBottom: '1px solid var(--color-border)',
        overflowX: 'auto',
        overflowY: 'hidden',
        scrollbarWidth: 'none',
        userSelect: 'none',
      }}
    >
      {tabs.map(tab => (
        <VaultTabItem
          key={tab.id}
          tab={tab}
          isActive={tab.id === activeTabId}
          onClick={() => handleTabClick(tab.id)}
          onClose={e => handleCloseTab(e, tab.id)}
          onLock={e => handleLockTab(e, tab.id)}
          onDragStart={() => handleDragStart(tab.id)}
          onDragOver={e => handleDragOver(e, tab.id)}
          onDrop={handleDrop}
        />
      ))}

      {/* New tab button */}
      <button
        onClick={onOpenNewVault}
        aria-label={t('tabs.openNewVault')}
        title={t('tabs.openNewVault')}
        style={{
          flexShrink: 0,
          width: 28,
          height: 28,
          margin: '0 4px',
          border: 'none',
          borderRadius: 4,
          background: 'transparent',
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: 18,
          color: 'var(--color-text-secondary)',
          opacity: 0.7,
        }}
      >
        +
      </button>
    </div>
  );
}

interface VaultTabItemProps {
  tab: VaultTab;
  isActive: boolean;
  onClick: () => void;
  onClose: (e: React.MouseEvent) => void;
  onLock: (e: React.MouseEvent) => void;
  onDragStart: () => void;
  onDragOver: (e: React.DragEvent) => void;
  onDrop: () => void;
}

function VaultTabItem({
  tab,
  isActive,
  onClick,
  onClose,
  onLock,
  onDragStart,
  onDragOver,
  onDrop,
}: VaultTabItemProps) {
  const { t } = useTranslation();

  return (
    <div
      role="tab"
      aria-selected={isActive}
      aria-label={`${tab.meta.name}${tab.isLocked ? ` (${t('tabs.locked')})` : ''}${tab.isDirty ? ' *' : ''}`}
      draggable
      onClick={onClick}
      onDragStart={onDragStart}
      onDragOver={onDragOver}
      onDrop={onDrop}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 6,
        height: '100%',
        padding: '0 10px',
        minWidth: 120,
        maxWidth: 200,
        cursor: 'pointer',
        borderRight: '1px solid var(--color-border)',
        background: isActive ? 'var(--color-surface)' : 'transparent',
        borderBottom: isActive ? '2px solid var(--color-accent)' : '2px solid transparent',
        flexShrink: 0,
        position: 'relative',
      }}
    >
      {/* Lock icon */}
      {tab.isLocked && (
        <span
          aria-hidden="true"
          style={{ fontSize: 11, opacity: 0.6, flexShrink: 0 }}
          title={t('tabs.locked')}
        >
          🔒
        </span>
      )}

      {/* Vault name */}
      <span
        style={{
          flex: 1,
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
          fontSize: 13,
          color: isActive ? 'var(--color-text)' : 'var(--color-text-secondary)',
          fontWeight: isActive ? 500 : 400,
        }}
      >
        {tab.isDirty ? `${tab.meta.name} *` : tab.meta.name}
      </span>

      {/* Lock button (only for unlocked tabs) */}
      {!tab.isLocked && (
        <button
          onClick={onLock}
          aria-label={t('tabs.lockThisVault', { name: tab.meta.name })}
          title={t('tabs.lockThisVault', { name: tab.meta.name })}
          style={{
            flexShrink: 0,
            width: 16,
            height: 16,
            border: 'none',
            background: 'transparent',
            cursor: 'pointer',
            padding: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 10,
            opacity: 0,
            color: 'var(--color-text-secondary)',
          }}
          className="tab-lock-btn"
        >
          🔒
        </button>
      )}

      {/* Close button */}
      <button
        onClick={onClose}
        aria-label={t('tabs.closeTab', { name: tab.meta.name })}
        title={t('tabs.closeTab', { name: tab.meta.name })}
        style={{
          flexShrink: 0,
          width: 16,
          height: 16,
          border: 'none',
          background: 'transparent',
          cursor: 'pointer',
          padding: 0,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: 12,
          opacity: 0,
          color: 'var(--color-text-secondary)',
          borderRadius: 3,
        }}
        className="tab-close-btn"
      >
        ×
      </button>
    </div>
  );
}
