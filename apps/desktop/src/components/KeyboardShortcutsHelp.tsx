/**
 * Keyboard Shortcuts Help overlay
 * Shows all available keyboard shortcuts in a modal
 * Triggered by ? key or from Settings
 */
import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';

interface ShortcutGroup {
  title: string;
  shortcuts: Array<{ keys: string[]; description: string }>;
}

interface KeyboardShortcutsHelpProps {
  open: boolean;
  onClose: () => void;
}

export function KeyboardShortcutsHelp({ open, onClose }: KeyboardShortcutsHelpProps) {
  const { t } = useTranslation();

  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' || e.key === '?') onClose();
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [open, onClose]);

  if (!open) return null;

  const isMac = navigator.platform.toUpperCase().includes('MAC');
  const mod = isMac ? '⌘' : 'Ctrl';

  const groups: ShortcutGroup[] = [
    {
      title: t('shortcuts.global'),
      shortcuts: [
        { keys: [mod, 'K'], description: t('shortcuts.commandPalette') },
        { keys: [mod, 'L'], description: t('shortcuts.lockVault') },
        { keys: [mod, 'S'], description: t('shortcuts.saveVault') },
        { keys: [mod, 'N'], description: t('shortcuts.newEntry') },
        { keys: ['?'], description: t('shortcuts.showHelp') },
      ],
    },
    {
      title: t('shortcuts.entryList'),
      shortcuts: [
        { keys: [mod, 'A'], description: t('shortcuts.selectAll') },
        { keys: ['Esc'], description: t('shortcuts.clearSelection') },
        { keys: ['N'], description: t('shortcuts.newEntry') },
        { keys: ['Delete'], description: t('shortcuts.deleteSelected') },
        { keys: ['↑', '↓'], description: t('shortcuts.navigate') },
        { keys: ['Enter'], description: t('shortcuts.openEntry') },
      ],
    },
    {
      title: t('shortcuts.entryDetail'),
      shortcuts: [
        { keys: [mod, 'C'], description: t('shortcuts.copyPassword') },
        { keys: [mod, 'B'], description: t('shortcuts.copyUsername') },
        { keys: [mod, 'U'], description: t('shortcuts.copyUrl') },
        { keys: [mod, 'E'], description: t('shortcuts.editEntry') },
        { keys: ['Esc'], description: t('shortcuts.back') },
      ],
    },
    {
      title: t('shortcuts.navigation'),
      shortcuts: [
        { keys: [mod, '1'], description: t('vault.open') },
        { keys: [mod, '2'], description: t('health.title') },
        { keys: [mod, '3'], description: t('generator.title') },
        { keys: [mod, '4'], description: t('breach.title') },
      ],
    },
  ];

  return (
    <div
      className="shortcuts-overlay"
      role="dialog"
      aria-label={t('shortcuts.title')}
      aria-modal="true"
      onClick={e => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="shortcuts-modal">
        <div className="shortcuts-header">
          <h2 className="shortcuts-title">⌨️ {t('shortcuts.title')}</h2>
          <button className="shortcuts-close" onClick={onClose} aria-label={t('common.close')}>
            ✕
          </button>
        </div>

        <div className="shortcuts-grid">
          {groups.map(group => (
            <div key={group.title} className="shortcuts-group">
              <h3 className="shortcuts-group-title">{group.title}</h3>
              <div className="shortcuts-list">
                {group.shortcuts.map((s, i) => (
                  <div key={i} className="shortcut-row">
                    <div className="shortcut-keys">
                      {s.keys.map((k, ki) => (
                        <span key={ki}>
                          <kbd className="shortcut-key">{k}</kbd>
                          {ki < s.keys.length - 1 && <span className="shortcut-plus">+</span>}
                        </span>
                      ))}
                    </div>
                    <span className="shortcut-desc">{s.description}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        <p className="shortcuts-footer">{t('shortcuts.pressEsc')}</p>
      </div>

      <style>{`
        .shortcuts-overlay {
          position: fixed; inset: 0; background: rgba(0,0,0,0.6);
          display: flex; align-items: center; justify-content: center;
          z-index: 2000; animation: fadeIn 0.15s ease;
        }
        @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
        .shortcuts-modal {
          background: var(--color-surface); border: 1px solid var(--color-border);
          border-radius: var(--radius-xl); padding: var(--space-xl);
          width: 640px; max-width: 95vw; max-height: 80vh; overflow-y: auto;
          box-shadow: 0 25px 80px rgba(0,0,0,0.4);
          animation: slideUp 0.2s ease;
        }
        @keyframes slideUp { from { transform: translateY(16px); opacity: 0; } to { transform: translateY(0); opacity: 1; } }
        .shortcuts-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-xl); }
        .shortcuts-title { font-size: 18px; font-weight: 700; margin: 0; }
        .shortcuts-close { background: none; border: none; cursor: pointer; font-size: 16px; color: var(--color-text-secondary); padding: 4px 8px; border-radius: var(--radius-sm); }
        .shortcuts-close:hover { background: var(--color-bg-tertiary); }
        .shortcuts-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-xl); }
        .shortcuts-group { display: flex; flex-direction: column; gap: var(--space-sm); }
        .shortcuts-group-title { font-size: 12px; font-weight: 700; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: .08em; margin: 0 0 var(--space-xs); }
        .shortcuts-list { display: flex; flex-direction: column; gap: 6px; }
        .shortcut-row { display: flex; align-items: center; gap: var(--space-md); }
        .shortcut-keys { display: flex; align-items: center; gap: 3px; min-width: 100px; }
        .shortcut-key {
          display: inline-flex; align-items: center; justify-content: center;
          background: var(--color-bg-tertiary); border: 1px solid var(--color-border);
          border-radius: 4px; padding: 2px 6px; font-size: 11px; font-weight: 600;
          font-family: 'SF Mono', 'Consolas', monospace; color: var(--color-text);
          box-shadow: 0 1px 0 var(--color-border); min-width: 22px; text-align: center;
        }
        .shortcut-plus { font-size: 10px; color: var(--color-text-tertiary); margin: 0 1px; }
        .shortcut-desc { font-size: 13px; color: var(--color-text-secondary); }
        .shortcuts-footer { font-size: 12px; color: var(--color-text-tertiary); text-align: center; margin-top: var(--space-xl); }
      `}</style>
    </div>
  );
}
