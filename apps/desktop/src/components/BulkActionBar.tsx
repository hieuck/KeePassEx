/**
 * Bulk Action Bar — shown when multiple entries are selected
 */
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';

interface BulkActionBarProps {
  selectedUuids: string[];
  onClearSelection: () => void;
}

export function BulkActionBar({ selectedUuids, onClearSelection }: BulkActionBarProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [loading, setLoading] = useState(false);

  if (selectedUuids.length === 0) return null;

  const handleBulkDelete = async () => {
    if (!confirm(t('bulk.confirmDelete', { count: selectedUuids.length }))) return;
    setLoading(true);
    try {
      await Promise.all(
        selectedUuids.map(uuid => invoke('delete_entry', { uuid, permanent: false }))
      );
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      onClearSelection();
    } catch (e) {
      console.error('Bulk delete failed:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleBulkMove = async () => {
    const groupUuid = window.prompt('Target group UUID:');
    if (!groupUuid?.trim()) return;
    setLoading(true);
    try {
      await Promise.all(
        selectedUuids.map(uuid => invoke('move_entry', { uuid, newGroupUuid: groupUuid.trim() }))
      );
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      onClearSelection();
    } catch (e) {
      console.error('Bulk move failed:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleBulkTag = async () => {
    const tag = window.prompt(t('entry.tags'));
    if (!tag?.trim()) return;
    setLoading(true);
    try {
      for (const uuid of selectedUuids) {
        const entry = await invoke<{ tags: string[] }>('get_entry', {
          uuid,
          includePassword: false,
        });
        if (!entry.tags.includes(tag.trim())) {
          await invoke('update_entry', { args: { uuid, tags: [...entry.tags, tag.trim()] } });
        }
      }
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      onClearSelection();
    } catch (e) {
      console.error('Bulk tag failed:', e);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div
      className="bulk-bar"
      role="toolbar"
      aria-label={t('bulk.selected', { count: selectedUuids.length })}
    >
      <span className="bulk-count">{t('bulk.selected', { count: selectedUuids.length })}</span>
      <div className="bulk-actions">
        <button
          className="bulk-btn"
          onClick={handleBulkMove}
          disabled={loading}
          aria-label="Move selected entries"
        >
          📁 {t('bulk.moveSelected')}
        </button>
        <button
          className="bulk-btn"
          onClick={handleBulkTag}
          disabled={loading}
          aria-label="Tag selected entries"
        >
          🏷️ {t('bulk.tagSelected')}
        </button>
        <button
          className="bulk-btn bulk-btn-danger"
          onClick={handleBulkDelete}
          disabled={loading}
          aria-label="Delete selected entries"
        >
          🗑 {t('bulk.deleteSelected')}
        </button>
      </div>
      <button
        className="bulk-clear"
        onClick={onClearSelection}
        aria-label="Clear selection"
        title={t('bulk.deselectAll')}
      >
        ✕
      </button>
      <style>{`
        .bulk-bar { display:flex; align-items:center; gap:var(--space-md); padding:var(--space-sm) var(--space-xl); background:var(--color-primary); color:white; flex-shrink:0; animation:slideIn 0.15s ease; }
        .bulk-count { font-size:13px; font-weight:600; flex-shrink:0; }
        .bulk-actions { display:flex; gap:var(--space-sm); flex:1; }
        .bulk-btn { background:rgba(255,255,255,0.15); border:1px solid rgba(255,255,255,0.3); color:white; padding:var(--space-xs) var(--space-md); border-radius:var(--radius-sm); cursor:pointer; font-size:12px; font-weight:500; transition:background 0.1s; }
        .bulk-btn:hover:not(:disabled) { background:rgba(255,255,255,0.25); }
        .bulk-btn:disabled { opacity:0.5; cursor:not-allowed; }
        .bulk-btn-danger { background:rgba(220,38,38,0.3); border-color:rgba(220,38,38,0.5); }
        .bulk-btn-danger:hover:not(:disabled) { background:rgba(220,38,38,0.5); }
        .bulk-clear { background:none; border:none; color:rgba(255,255,255,0.7); cursor:pointer; font-size:14px; padding:var(--space-xs); border-radius:var(--radius-sm); flex-shrink:0; }
        .bulk-clear:hover { color:white; background:rgba(255,255,255,0.1); }
      `}</style>
    </div>
  );
}
