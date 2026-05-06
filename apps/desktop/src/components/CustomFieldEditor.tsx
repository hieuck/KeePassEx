/**
 * Custom Field Editor — add/edit/delete custom fields on an entry
 */
import React, { useState } from 'react';
import { useSettingsStore } from '../store/settings';

export interface CustomField {
  key: string;
  value: string;
  protected: boolean;
}

interface CustomFieldEditorProps {
  fields: CustomField[];
  onChange: (fields: CustomField[]) => void;
  readOnly?: boolean;
}

export function CustomFieldEditor({ fields, onChange, readOnly = false }: CustomFieldEditorProps) {
  const { settings } = useSettingsStore();
  const isVi = settings.language === 'vi';
  const [showValues, setShowValues] = useState<Record<string, boolean>>({});
  const [copiedKey, setCopiedKey] = useState<string | null>(null);

  const addField = () => {
    const newKey = `${isVi ? 'Trường' : 'Field'} ${fields.length + 1}`;
    onChange([...fields, { key: newKey, value: '', protected: false }]);
  };

  const updateField = (index: number, updates: Partial<CustomField>) => {
    const updated = fields.map((f, i) => i === index ? { ...f, ...updates } : f);
    onChange(updated);
  };

  const removeField = (index: number) => {
    onChange(fields.filter((_, i) => i !== index));
  };

  const copyValue = async (field: CustomField) => {
    await navigator.clipboard.writeText(field.value);
    setCopiedKey(field.key);
    setTimeout(() => setCopiedKey(null), 2000);
    // Auto-clear clipboard
    setTimeout(() => navigator.clipboard.writeText(''), 10_000);
  };

  const toggleShow = (key: string) => {
    setShowValues(prev => ({ ...prev, [key]: !prev[key] }));
  };

  if (fields.length === 0 && readOnly) {
    return null;
  }

  return (
    <div className="cfe-container">
      {fields.length > 0 && (
        <div className="cfe-list">
          {fields.map((field, index) => (
            <div key={index} className="cfe-field">
              {readOnly ? (
                // Read-only view
                <>
                  <div className="cfe-field-header">
                    <span className="cfe-key">{field.key}</span>
                    <div className="cfe-actions">
                      {field.protected && (
                        <button
                          className="btn-icon"
                          onClick={() => toggleShow(field.key)}
                          aria-label={showValues[field.key] ? 'Hide' : 'Show'}
                          title={showValues[field.key] ? (isVi ? 'Ẩn' : 'Hide') : (isVi ? 'Hiện' : 'Show')}
                        >
                          {showValues[field.key] ? '🙈' : '👁'}
                        </button>
                      )}
                      <button
                        className={`btn-icon ${copiedKey === field.key ? 'copied' : ''}`}
                        onClick={() => copyValue(field)}
                        aria-label={`Copy ${field.key}`}
                        title={isVi ? 'Sao chép' : 'Copy'}
                      >
                        {copiedKey === field.key ? '✓' : '⎘'}
                      </button>
                    </div>
                  </div>
                  <span className="cfe-value">
                    {field.protected && !showValues[field.key]
                      ? '••••••••'
                      : field.value || '—'}
                  </span>
                </>
              ) : (
                // Edit mode
                <div className="cfe-edit-row">
                  <input
                    type="text"
                    className="form-input cfe-key-input"
                    value={field.key}
                    onChange={e => updateField(index, { key: e.target.value })}
                    placeholder={isVi ? 'Tên trường' : 'Field name'}
                    aria-label="Field name"
                  />
                  <input
                    type={field.protected ? 'password' : 'text'}
                    className="form-input cfe-value-input"
                    value={field.value}
                    onChange={e => updateField(index, { value: e.target.value })}
                    placeholder={isVi ? 'Giá trị' : 'Value'}
                    aria-label="Field value"
                  />
                  <button
                    className={`btn-icon ${field.protected ? 'protected-active' : ''}`}
                    onClick={() => updateField(index, { protected: !field.protected })}
                    title={field.protected
                      ? (isVi ? 'Bỏ bảo vệ' : 'Unprotect')
                      : (isVi ? 'Bảo vệ' : 'Protect')}
                    aria-label={field.protected ? 'Unprotect field' : 'Protect field'}
                    aria-pressed={field.protected}
                  >
                    {field.protected ? '🔒' : '🔓'}
                  </button>
                  <button
                    className="btn-icon"
                    onClick={() => removeField(index)}
                    title={isVi ? 'Xóa trường' : 'Remove field'}
                    aria-label={`Remove ${field.key}`}
                  >
                    🗑
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {!readOnly && (
        <button className="btn btn-secondary cfe-add-btn" onClick={addField}>
          + {isVi ? 'Thêm trường tùy chỉnh' : 'Add Custom Field'}
        </button>
      )}

      <style>{`
        .cfe-container { display: flex; flex-direction: column; gap: var(--space-sm); }
        .cfe-list { display: flex; flex-direction: column; gap: var(--space-sm); }
        .cfe-field {
          background: var(--color-bg-secondary); border: 1px solid var(--color-border);
          border-radius: var(--radius-md); padding: var(--space-sm) var(--space-md);
          display: flex; flex-direction: column; gap: 4px;
        }
        .cfe-field-header { display: flex; align-items: center; justify-content: space-between; }
        .cfe-key { font-size: 11px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: .05em; }
        .cfe-value { font-size: 14px; color: var(--color-text); font-family: 'SF Mono', 'Consolas', monospace; word-break: break-all; }
        .cfe-actions { display: flex; gap: 2px; }
        .cfe-edit-row { display: flex; gap: var(--space-sm); align-items: center; }
        .cfe-key-input { width: 140px; flex-shrink: 0; font-size: 13px; }
        .cfe-value-input { flex: 1; font-size: 13px; }
        .cfe-add-btn { align-self: flex-start; font-size: 13px; }
        .btn-icon.copied { color: var(--color-success); }
        .btn-icon.protected-active { color: var(--color-primary); }
      `}</style>
    </div>
  );
}
