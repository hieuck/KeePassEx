/**
 * Import/Export page — desktop
 * EN/VI i18n throughout
 */
import React, { useState } from 'react';
import { open, save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';

import { useTranslation } from 'react-i18next';

type ImportFormat = 'auto' | 'bitwarden' | 'lastpass' | 'chrome' | 'firefox' | '1password' | 'csv';
type ExportFormat = 'csv' | 'json';

interface ImportResult {
  entriesImported: number;
  groupsCreated: number;
  entriesSkipped: number;
  warnings: string[];
}

const IMPORT_FORMATS: { value: ImportFormat; label: string; ext: string[] }[] = [
  { value: 'auto', label: 'Auto-detect', ext: ['json', 'csv'] },
  { value: 'bitwarden', label: 'Bitwarden JSON', ext: ['json'] },
  { value: 'lastpass', label: 'LastPass CSV', ext: ['csv'] },
  { value: 'chrome', label: 'Chrome / Edge CSV', ext: ['csv'] },
  { value: 'firefox', label: 'Firefox CSV', ext: ['csv'] },
  { value: '1password', label: '1Password 1PUX', ext: ['1pux', 'json'] },
  { value: 'csv', label: 'Generic CSV', ext: ['csv'] },
];

export function ImportExportPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();

  // Import state
  const [importFormat, setImportFormat] = useState<ImportFormat>('auto');
  const [importing, setImporting] = useState(false);
  const [importResult, setImportResult] = useState<ImportResult | null>(null);
  const [importError, setImportError] = useState<string | null>(null);

  // Export state
  const [exportFormat, setExportFormat] = useState<ExportFormat>('csv');
  const [exporting, setExporting] = useState(false);
  const [exportSuccess, setExportSuccess] = useState<string | null>(null);
  const [exportError, setExportError] = useState<string | null>(null);

  const handleImport = async () => {
    setImportError(null);
    setImportResult(null);

    const fmt = IMPORT_FORMATS.find(f => f.value === importFormat)!;
    const selected = await open({
      filters: [{ name: 'Import file', extensions: fmt.ext }],
      multiple: false,
    });
    if (!selected || typeof selected !== 'string') return;

    setImporting(true);
    try {
      const result = await invoke<ImportResult>('import_vault', {
        args: {
          file_path: selected,
          format: importFormat === 'auto' ? null : importFormat,
          target_group_uuid: null,
        },
      });
      setImportResult(result);
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      queryClient.invalidateQueries({ queryKey: ['groups'] });
    } catch (e: unknown) {
      setImportError(e instanceof Error ? e.message : String(e));
    } finally {
      setImporting(false);
    }
  };

  const handleExport = async () => {
    setExportError(null);
    setExportSuccess(null);

    const ext = exportFormat === 'csv' ? 'csv' : 'json';
    const filePath = await save({
      filters: [{ name: `${ext.toUpperCase()} file`, extensions: [ext] }],
      defaultPath: `keepassex-export.${ext}`,
    });
    if (!filePath) return;

    setExporting(true);
    try {
      const bytes = await invoke<number>('export_vault_cmd', {
        args: { file_path: filePath, format: exportFormat },
      });
      setExportSuccess(
        `${t('importExport.exportSuccess', { path: filePath })} (${bytes.toLocaleString()} bytes)`
      );
    } catch (e: unknown) {
      setExportError(e instanceof Error ? e.message : String(e));
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="ie-page">
      <div className="ie-header">
        <h2>
          📥 {t('importExport.import')} / {t('importExport.export')}
        </h2>
      </div>

      <div className="ie-content">
        <section className="ie-section">
          <h3 className="ie-section-title">📥 {t('importExport.importFrom')}</h3>
          <p className="ie-section-desc">{t('importExport.importWarning')}</p>

          <div className="ie-form">
            <label className="ie-label" htmlFor="import-format">
              {t('common.info')}
            </label>
            <select
              id="import-format"
              className="ie-select"
              value={importFormat}
              onChange={e => setImportFormat(e.target.value as ImportFormat)}
            >
              {IMPORT_FORMATS.map(f => (
                <option key={f.value} value={f.value}>
                  {f.label}
                </option>
              ))}
            </select>
            <button className="btn btn-primary" onClick={handleImport} disabled={importing}>
              {importing ? t('common.loading') : `📂 ${t('importExport.selectFile')}`}
            </button>
          </div>

          {importResult && (
            <div className="ie-result ie-result-success" role="status">
              <p className="ie-result-title">
                ✅ {t('importExport.importSuccess', { count: importResult.entriesImported })}
              </p>
              <ul className="ie-result-list">
                <li>
                  {importResult.entriesImported}{' '}
                  {t('importExport.importSuccess', { count: '' }).split(' ')[0]}
                </li>
                {importResult.groupsCreated > 0 && (
                  <li>{importResult.groupsCreated} groups created</li>
                )}
                {importResult.entriesSkipped > 0 && (
                  <li>{t('importExport.importSkipped', { count: importResult.entriesSkipped })}</li>
                )}
              </ul>
              {importResult.warnings.length > 0 && (
                <details className="ie-warnings">
                  <summary>
                    {importResult.warnings.length} {t('common.warning')}
                  </summary>
                  <ul>
                    {importResult.warnings.map((w, i) => (
                      <li key={i}>{w}</li>
                    ))}
                  </ul>
                </details>
              )}
            </div>
          )}
          {importError && (
            <div className="ie-result ie-result-error" role="alert">
              ⚠️ {importError}
            </div>
          )}
        </section>

        <div className="ie-divider" />

        <section className="ie-section">
          <h3 className="ie-section-title">📤 {t('importExport.exportTo')}</h3>

          <div className="ie-warning-box" role="note">
            ⚠️ {t('importExport.exportWarning')}
          </div>

          <div className="ie-form">
            <label className="ie-label" htmlFor="export-format">
              {t('common.info')}
            </label>
            <select
              id="export-format"
              className="ie-select"
              value={exportFormat}
              onChange={e => setExportFormat(e.target.value as ExportFormat)}
            >
              <option value="csv">CSV</option>
              <option value="json">JSON</option>
            </select>{' '}
            <button className="btn btn-secondary" onClick={handleExport} disabled={exporting}>
              {exporting ? t('common.loading') : `💾 ${t('importExport.export')}`}
            </button>
          </div>

          {exportSuccess && (
            <div className="ie-result ie-result-success" role="status">
              ✅ {exportSuccess}
            </div>
          )}
          {exportError && (
            <div className="ie-result ie-result-error" role="alert">
              ⚠️ {exportError}
            </div>
          )}
        </section>
      </div>

      <style>{`
        .ie-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .ie-header {
          padding: var(--space-md) var(--space-xl);
          border-bottom: 1px solid var(--color-border);
          flex-shrink: 0;
        }
        .ie-header h2 { font-size:16px; font-weight:600; }
        .ie-content {
          flex:1; overflow-y:auto; padding:var(--space-xl);
          display:flex; flex-direction:column; gap:var(--space-xl);
          max-width:600px;
        }
        .ie-section { display:flex; flex-direction:column; gap:var(--space-md); }
        .ie-section-title { font-size:15px; font-weight:600; }
        .ie-section-desc { font-size:13px; color:var(--color-text-secondary); }
        .ie-form { display:flex; flex-direction:column; gap:var(--space-sm); }
        .ie-label { font-size:12px; font-weight:500; color:var(--color-text-secondary); text-transform:uppercase; letter-spacing:.05em; }
        .ie-select {
          border:1px solid var(--color-border); border-radius:var(--radius-md);
          padding:var(--space-sm) var(--space-md); font-size:14px;
          background:var(--color-bg); color:var(--color-text); cursor:pointer;
        }
        .ie-divider { height:1px; background:var(--color-border); }
        .ie-warning-box {
          background:#FFFBEB; border:1px solid #FCD34D; border-radius:var(--radius-md);
          padding:var(--space-md); font-size:13px; color:#92400E;
        }
        .ie-result { padding:var(--space-md); border-radius:var(--radius-md); font-size:13px; }
        .ie-result-success { background:#F0FDF4; border:1px solid #86EFAC; color:#166534; }
        .ie-result-error   { background:#FEF2F2; border:1px solid #FECACA; color:#991B1B; }
        .ie-result-title { font-weight:600; margin-bottom:var(--space-xs); }
        .ie-result-list { padding-left:var(--space-lg); }
        .ie-result-list li { margin-top:2px; }
        .ie-warnings { margin-top:var(--space-sm); font-size:12px; }
        .ie-warnings ul { padding-left:var(--space-lg); margin-top:var(--space-xs); }
      `}</style>
    </div>
  );
}
