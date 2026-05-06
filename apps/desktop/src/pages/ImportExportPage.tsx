/**
 * Import/Export page — desktop
 * EN/VI i18n throughout
 */
import React, { useState } from 'react';
import { open, save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient } from '@tanstack/react-query';
import { useSettingsStore } from '../store/settings';

type ImportFormat = 'auto' | 'bitwarden' | 'lastpass' | 'chrome' | 'firefox' | '1password' | 'csv';
type ExportFormat = 'csv' | 'json';

interface ImportResult {
  entriesImported: number;
  groupsCreated: number;
  entriesSkipped: number;
  warnings: string[];
}

const IMPORT_FORMATS: { value: ImportFormat; labelEn: string; labelVi: string; ext: string[] }[] = [
  { value: 'auto',        labelEn: 'Auto-detect',          labelVi: 'Tự động nhận dạng',    ext: ['json','csv'] },
  { value: 'bitwarden',   labelEn: 'Bitwarden JSON',       labelVi: 'Bitwarden JSON',        ext: ['json'] },
  { value: 'lastpass',    labelEn: 'LastPass CSV',         labelVi: 'LastPass CSV',          ext: ['csv'] },
  { value: 'chrome',      labelEn: 'Chrome / Edge CSV',    labelVi: 'Chrome / Edge CSV',     ext: ['csv'] },
  { value: 'firefox',     labelEn: 'Firefox CSV',          labelVi: 'Firefox CSV',           ext: ['csv'] },
  { value: '1password',   labelEn: '1Password 1PUX',       labelVi: '1Password 1PUX',        ext: ['1pux','json'] },
  { value: 'csv',         labelEn: 'Generic CSV',          labelVi: 'CSV chung',             ext: ['csv'] },
];

export function ImportExportPage() {
  const { settings } = useSettingsStore();
  const queryClient = useQueryClient();
  const isVi = settings.language === 'vi';

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
      setExportSuccess(isVi
        ? `Đã xuất ${bytes.toLocaleString()} bytes sang ${filePath}`
        : `Exported ${bytes.toLocaleString()} bytes to ${filePath}`);
    } catch (e: unknown) {
      setExportError(e instanceof Error ? e.message : String(e));
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="ie-page">
      <div className="ie-header">
        <h2>{isVi ? '📥 Nhập / Xuất' : '📥 Import / Export'}</h2>
      </div>

      <div className="ie-content">
        {/* ── Import ── */}
        <section className="ie-section">
          <h3 className="ie-section-title">
            {isVi ? '📥 Nhập từ trình quản lý mật khẩu khác' : '📥 Import from another password manager'}
          </h3>
          <p className="ie-section-desc">
            {isVi
              ? 'Nhập mục từ Bitwarden, LastPass, Chrome, Firefox, 1Password hoặc CSV.'
              : 'Import entries from Bitwarden, LastPass, Chrome, Firefox, 1Password, or CSV.'}
          </p>

          <div className="ie-form">
            <label className="ie-label" htmlFor="import-format">
              {isVi ? 'Định dạng' : 'Format'}
            </label>
            <select
              id="import-format"
              className="ie-select"
              value={importFormat}
              onChange={e => setImportFormat(e.target.value as ImportFormat)}
            >
              {IMPORT_FORMATS.map(f => (
                <option key={f.value} value={f.value}>
                  {isVi ? f.labelVi : f.labelEn}
                </option>
              ))}
            </select>

            <button
              className="btn btn-primary"
              onClick={handleImport}
              disabled={importing}
            >
              {importing
                ? (isVi ? 'Đang nhập...' : 'Importing...')
                : (isVi ? '📂 Chọn tệp để nhập' : '📂 Choose file to import')}
            </button>
          </div>

          {importResult && (
            <div className="ie-result ie-result-success" role="status">
              <p className="ie-result-title">
                {isVi ? '✅ Nhập thành công!' : '✅ Import successful!'}
              </p>
              <ul className="ie-result-list">
                <li>{isVi ? `${importResult.entriesImported} mục đã nhập` : `${importResult.entriesImported} entries imported`}</li>
                {importResult.groupsCreated > 0 && (
                  <li>{isVi ? `${importResult.groupsCreated} nhóm đã tạo` : `${importResult.groupsCreated} groups created`}</li>
                )}
                {importResult.entriesSkipped > 0 && (
                  <li>{isVi ? `${importResult.entriesSkipped} mục bỏ qua` : `${importResult.entriesSkipped} entries skipped`}</li>
                )}
              </ul>
              {importResult.warnings.length > 0 && (
                <details className="ie-warnings">
                  <summary>{isVi ? `${importResult.warnings.length} cảnh báo` : `${importResult.warnings.length} warnings`}</summary>
                  <ul>
                    {importResult.warnings.map((w, i) => <li key={i}>{w}</li>)}
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

        {/* ── Export ── */}
        <section className="ie-section">
          <h3 className="ie-section-title">
            {isVi ? '📤 Xuất kho mật khẩu' : '📤 Export vault'}
          </h3>

          <div className="ie-warning-box" role="note">
            ⚠️ {isVi
              ? 'Tệp xuất chứa mật khẩu KHÔNG được mã hóa. Hãy bảo mật tệp này cẩn thận!'
              : 'Exported file contains UNENCRYPTED passwords. Keep this file secure!'}
          </div>

          <div className="ie-form">
            <label className="ie-label" htmlFor="export-format">
              {isVi ? 'Định dạng' : 'Format'}
            </label>
            <select
              id="export-format"
              className="ie-select"
              value={exportFormat}
              onChange={e => setExportFormat(e.target.value as ExportFormat)}
            >
              <option value="csv">CSV</option>
              <option value="json">JSON</option>
            </select>

            <button
              className="btn btn-secondary"
              onClick={handleExport}
              disabled={exporting}
            >
              {exporting
                ? (isVi ? 'Đang xuất...' : 'Exporting...')
                : (isVi ? '💾 Xuất kho' : '💾 Export vault')}
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
