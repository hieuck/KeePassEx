/**
 * Attachment Viewer — view and manage file attachments on an entry
 */
import React, { useState } from 'react';
import { open, save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../store/settings';

export interface Attachment {
  name: string;
  dataRef: string;
  size?: number;
  mimeType?: string;
}

interface AttachmentViewerProps {
  attachments: Attachment[];
  entryUuid: string;
  readOnly?: boolean;
  onAdd?: (name: string, data: number[]) => void;
  onRemove?: (name: string) => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

function getFileIcon(name: string): string {
  const ext = name.split('.').pop()?.toLowerCase() ?? '';
  const icons: Record<string, string> = {
    pdf: '📄', png: '🖼️', jpg: '🖼️', jpeg: '🖼️', gif: '🖼️', svg: '🖼️',
    txt: '📝', md: '📝', doc: '📝', docx: '📝',
    xls: '📊', xlsx: '📊', csv: '📊',
    zip: '📦', tar: '📦', gz: '📦',
    key: '🔑', pem: '🔑', p12: '🔑', pfx: '🔑',
    mp3: '🎵', mp4: '🎬', mov: '🎬',
  };
  return icons[ext] ?? '📎';
}

export function AttachmentViewer({
  attachments,
  entryUuid,
  readOnly = false,
  onAdd,
  onRemove,
}: AttachmentViewerProps) {
  const { settings } = useSettingsStore();
  const isVi = settings.language === 'vi';
  const [saving, setSaving] = useState<string | null>(null);

  const handleAdd = async () => {
    const selected = await open({ multiple: false });
    if (!selected || typeof selected !== 'string') return;

    try {
      // Read file via Tauri fs plugin
      const data = await invoke<number[]>('read_file_bytes', { path: selected });
      const name = selected.split('/').pop() ?? selected.split('\\').pop() ?? 'attachment';
      onAdd?.(name, data);
    } catch (e: unknown) {
      console.error('Failed to read attachment:', e);
    }
  };

  const handleSave = async (attachment: Attachment) => {
    setSaving(attachment.name);
    try {
      const savePath = await save({
        defaultPath: attachment.name,
      });
      if (!savePath) return;

      await invoke('save_attachment', {
        entryUuid,
        attachmentName: attachment.name,
        outputPath: savePath,
      });
    } catch (e: unknown) {
      console.error('Failed to save attachment:', e);
    } finally {
      setSaving(null);
    }
  };

  if (attachments.length === 0 && readOnly) {
    return null;
  }

  return (
    <div className="att-container">
      {attachments.length > 0 && (
        <div className="att-list">
          {attachments.map(att => (
            <div key={att.name} className="att-item">
              <span className="att-icon" aria-hidden="true">{getFileIcon(att.name)}</span>
              <div className="att-info">
                <span className="att-name">{att.name}</span>
                {att.size !== undefined && (
                  <span className="att-size">{formatBytes(att.size)}</span>
                )}
              </div>
              <div className="att-actions">
                <button
                  className="btn-icon"
                  onClick={() => handleSave(att)}
                  disabled={saving === att.name}
                  title={isVi ? 'Lưu tệp' : 'Save file'}
                  aria-label={`Save ${att.name}`}
                >
                  {saving === att.name ? '⏳' : '💾'}
                </button>
                {!readOnly && onRemove && (
                  <button
                    className="btn-icon"
                    onClick={() => {
                      if (confirm(isVi
                        ? `Xóa tệp đính kèm "${att.name}"?`
                        : `Remove attachment "${att.name}"?`
                      )) {
                        onRemove(att.name);
                      }
                    }}
                    title={isVi ? 'Xóa tệp đính kèm' : 'Remove attachment'}
                    aria-label={`Remove ${att.name}`}
                  >
                    🗑
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {!readOnly && (
        <button className="btn btn-secondary att-add-btn" onClick={handleAdd}>
          📎 {isVi ? 'Thêm tệp đính kèm' : 'Add Attachment'}
        </button>
      )}

      {attachments.length === 0 && !readOnly && (
        <p className="att-empty">
          {isVi ? 'Chưa có tệp đính kèm' : 'No attachments'}
        </p>
      )}

      <style>{`
        .att-container { display: flex; flex-direction: column; gap: var(--space-sm); }
        .att-list { display: flex; flex-direction: column; gap: 4px; }
        .att-item {
          display: flex; align-items: center; gap: var(--space-sm);
          padding: var(--space-sm) var(--space-md);
          background: var(--color-bg-secondary); border: 1px solid var(--color-border);
          border-radius: var(--radius-sm);
        }
        .att-icon { font-size: 18px; flex-shrink: 0; }
        .att-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
        .att-name { font-size: 13px; font-weight: 500; color: var(--color-text); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
        .att-size { font-size: 11px; color: var(--color-text-tertiary); }
        .att-actions { display: flex; gap: 2px; flex-shrink: 0; }
        .att-add-btn { align-self: flex-start; font-size: 13px; }
        .att-empty { font-size: 13px; color: var(--color-text-tertiary); }
      `}</style>
    </div>
  );
}
