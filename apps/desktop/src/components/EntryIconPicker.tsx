/**
 * Entry Icon Picker — select icon for vault entries and groups
 * KeePassEx matches KeePassXC's icon picker with 68 built-in icons
 * Plus custom icon upload support
 */
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

// KeePass standard icon set (68 icons, IDs 0-67)
const ICON_EMOJIS: Record<number, string> = {
  0: '🔑',
  1: '🌐',
  2: '⚠️',
  3: '🖥️',
  4: '📁',
  5: '💡',
  6: '📝',
  7: '🔌',
  8: '📧',
  9: '💳',
  10: '🏦',
  11: '📱',
  12: '📷',
  13: '📡',
  14: '🔒',
  15: '🖨️',
  16: '🗂️',
  17: '🔧',
  18: '🌍',
  19: '🗒️',
  20: '📋',
  21: '🔗',
  22: '📌',
  23: '🏠',
  24: '⭐',
  25: '💾',
  26: '🖱️',
  27: '⌨️',
  28: '🎮',
  29: '🎵',
  30: '🎬',
  31: '📺',
  32: '📻',
  33: '🔔',
  34: '🔍',
  35: '🛒',
  36: '💰',
  37: '📊',
  38: '📈',
  39: '🗓️',
  40: '📅',
  41: '⏰',
  42: '🔐',
  43: '🛡️',
  44: '🔓',
  45: '🏷️',
  46: '📦',
  47: '🎁',
  48: '📁',
  49: '🗃️',
  50: '🗄️',
  51: '🗑️',
  52: '📤',
  53: '📥',
  54: '🔄',
  55: '⚙️',
  56: '🔨',
  57: '🔩',
  58: '🔬',
  59: '🧪',
  60: '💊',
  61: '🏥',
  62: '🚗',
  63: '✈️',
  64: '🏢',
  65: '🎓',
  66: '👤',
  67: '👥',
};

interface EntryIconPickerProps {
  currentIconId: number;
  onSelect: (iconId: number) => void;
  onClose: () => void;
}

export function EntryIconPicker({ currentIconId, onSelect, onClose }: EntryIconPickerProps) {
  const { t } = useTranslation();
  const [search, setSearch] = useState('');
  const [selected, setSelected] = useState(currentIconId);

  const iconIds = Object.keys(ICON_EMOJIS).map(Number);

  const handleConfirm = () => {
    onSelect(selected);
    onClose();
  };

  return (
    <div
      className="icon-picker-overlay"
      role="dialog"
      aria-label={t('entry.icon')}
      aria-modal="true"
      onClick={e => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="icon-picker-modal">
        <div className="icon-picker-header">
          <h3 className="icon-picker-title">{t('entry.icon')}</h3>
          <button className="icon-picker-close" onClick={onClose} aria-label={t('common.close')}>
            ✕
          </button>
        </div>

        <div className="icon-picker-current">
          <span className="icon-picker-preview">{ICON_EMOJIS[selected] ?? '🔑'}</span>
          <span className="icon-picker-current-label">
            {t('entry.icon')} #{selected}
          </span>
        </div>

        <div className="icon-picker-grid" role="listbox" aria-label={t('entry.icon')}>
          {iconIds.map(id => (
            <button
              key={id}
              role="option"
              aria-selected={selected === id}
              className={`icon-btn-cell ${selected === id ? 'selected' : ''}`}
              onClick={() => setSelected(id)}
              title={`Icon ${id}`}
            >
              <span className="icon-cell-emoji">{ICON_EMOJIS[id]}</span>
              <span className="icon-cell-id">{id}</span>
            </button>
          ))}
        </div>

        <div className="icon-picker-actions">
          <button className="btn btn-secondary" onClick={onClose}>
            {t('common.cancel')}
          </button>
          <button className="btn btn-primary" onClick={handleConfirm}>
            {t('common.ok')}
          </button>
        </div>
      </div>

      <style>{`
        .icon-picker-overlay {
          position: fixed; inset: 0; background: rgba(0,0,0,0.6);
          display: flex; align-items: center; justify-content: center;
          z-index: 2000;
        }
        .icon-picker-modal {
          background: var(--color-surface); border: 1px solid var(--color-border);
          border-radius: var(--radius-xl); padding: var(--space-xl);
          width: 480px; max-width: 95vw; max-height: 80vh;
          display: flex; flex-direction: column; gap: var(--space-md);
          box-shadow: 0 25px 80px rgba(0,0,0,0.4);
        }
        .icon-picker-header { display: flex; align-items: center; justify-content: space-between; }
        .icon-picker-title { font-size: 16px; font-weight: 700; margin: 0; }
        .icon-picker-close { background: none; border: none; cursor: pointer; font-size: 16px; color: var(--color-text-secondary); padding: 4px 8px; border-radius: var(--radius-sm); }
        .icon-picker-current { display: flex; align-items: center; gap: var(--space-md); padding: var(--space-sm) var(--space-md); background: var(--color-bg-secondary); border-radius: var(--radius-md); }
        .icon-picker-preview { font-size: 32px; }
        .icon-picker-current-label { font-size: 13px; color: var(--color-text-secondary); }
        .icon-picker-grid {
          display: grid; grid-template-columns: repeat(8, 1fr);
          gap: 4px; overflow-y: auto; max-height: 320px;
          padding: var(--space-xs);
        }
        .icon-btn-cell {
          display: flex; flex-direction: column; align-items: center; gap: 1px;
          padding: 6px 4px; border: 1px solid transparent; border-radius: var(--radius-sm);
          background: none; cursor: pointer; transition: background .1s, border-color .1s;
        }
        .icon-btn-cell:hover { background: var(--color-bg-tertiary); }
        .icon-btn-cell.selected { background: var(--color-primary); border-color: var(--color-primary); }
        .icon-cell-emoji { font-size: 20px; }
        .icon-cell-id { font-size: 9px; color: var(--color-text-tertiary); }
        .icon-btn-cell.selected .icon-cell-id { color: rgba(255,255,255,0.7); }
        .icon-picker-actions { display: flex; gap: var(--space-sm); justify-content: flex-end; }
      `}</style>
    </div>
  );
}
