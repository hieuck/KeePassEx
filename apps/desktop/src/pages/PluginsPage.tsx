/**
 * Plugins page — desktop
 */
import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { open } from '@tauri-apps/plugin-dialog';
import { useTranslation } from 'react-i18next';

interface Plugin {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  capabilities: string[];
  enabled: boolean;
  installedAt: string;
}

const CAPABILITY_LABELS: Record<string, { label: string; icon: string }> = {
  password_generator: { label: 'Password Generator', icon: '⚡' },
  importer: { label: 'Importer', icon: '📥' },
  exporter: { label: 'Exporter', icon: '📤' },
  health_check: { label: 'Health Check', icon: '🛡️' },
  field_validator: { label: 'Field Validator', icon: '✅' },
  icon_set: { label: 'Icon Set', icon: '🎨' },
  ui_extension: { label: 'UI Extension', icon: '🔧' },
};

const PLUGIN_CATALOG = [
  {
    id: 'com.keepassex.importer.dashlane',
    name: 'Dashlane Importer',
    version: '1.0.0',
    description: 'Import passwords from Dashlane CSV export',
    author: 'KeePassEx Team',
    capabilities: ['importer'],
  },
  {
    id: 'com.keepassex.importer.enpass',
    name: 'Enpass Importer',
    version: '1.0.0',
    description: 'Import passwords from Enpass JSON export',
    author: 'KeePassEx Team',
    capabilities: ['importer'],
  },
  {
    id: 'com.keepassex.importer.roboform',
    name: 'RoboForm Importer',
    version: '1.0.0',
    description: 'Import passwords from RoboForm export',
    author: 'KeePassEx Team',
    capabilities: ['importer'],
  },
  {
    id: 'com.keepassex.generator.diceware',
    name: 'Diceware Generator',
    version: '1.0.0',
    description: 'Generate passphrases using the Diceware wordlist',
    author: 'KeePassEx Team',
    capabilities: ['password_generator'],
  },
  {
    id: 'com.keepassex.health.hibp-offline',
    name: 'HIBP Offline Database',
    version: '1.0.0',
    description: 'Full offline HIBP database (500MB) for breach checking without internet',
    author: 'KeePassEx Team',
    capabilities: ['health_check'],
  },
];

export function PluginsPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = useState<'installed' | 'catalog'>('installed');

  const { data: installedPlugins = [] } = useQuery<Plugin[]>({
    queryKey: ['plugins'],
    queryFn: () => Promise.resolve([]),
  });

  const toggleMutation = useMutation({
    mutationFn: (_args: { id: string; enabled: boolean }) => Promise.resolve(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['plugins'] }),
  });

  const uninstallMutation = useMutation({
    mutationFn: (_id: string) => Promise.resolve(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['plugins'] }),
  });

  const installFromFile = async () => {
    const file = await open({
      filters: [{ name: 'KeePassEx Plugin', extensions: ['kpxplugin', 'wasm', 'zip'] }],
    });
    if (!file || typeof file !== 'string') return;
    queryClient.invalidateQueries({ queryKey: ['plugins'] });
  };

  return (
    <div className="plugins-page">
      <div className="plugins-header">
        <div>
          <h2>🔧 {t('plugins.title')}</h2>
          <p className="plugins-subtitle">{t('plugins.subtitle')}</p>
        </div>
        <button className="btn btn-secondary" onClick={installFromFile}>
          📦 {t('plugins.installFromFile')}
        </button>
      </div>

      <div className="plugins-tabs">
        <button
          className={`plugins-tab ${activeTab === 'installed' ? 'active' : ''}`}
          onClick={() => setActiveTab('installed')}
        >
          {t('plugins.installed')} ({installedPlugins.length})
        </button>
        <button
          className={`plugins-tab ${activeTab === 'catalog' ? 'active' : ''}`}
          onClick={() => setActiveTab('catalog')}
        >
          {t('plugins.catalog')} ({PLUGIN_CATALOG.length})
        </button>
      </div>

      <div className="plugins-content">
        {activeTab === 'installed' ? (
          installedPlugins.length === 0 ? (
            <div className="empty-state">
              <span className="empty-state-icon">🔧</span>
              <p className="empty-state-title">{t('plugins.noPlugins')}</p>
              <p className="empty-state-desc">{t('plugins.noPluginsDesc')}</p>
            </div>
          ) : (
            <div className="plugins-list">
              {installedPlugins.map(plugin => (
                <PluginCard
                  key={plugin.id}
                  plugin={plugin}
                  onToggle={enabled => toggleMutation.mutate({ id: plugin.id, enabled })}
                  onUninstall={() => {
                    if (confirm(t('plugins.confirmUninstall', { name: plugin.name })))
                      uninstallMutation.mutate(plugin.id);
                  }}
                  uninstallLabel={t('plugins.uninstall')}
                />
              ))}
            </div>
          )
        ) : (
          <div className="plugins-list">
            {PLUGIN_CATALOG.map(plugin => (
              <CatalogCard
                key={plugin.id}
                plugin={plugin}
                installLabel={`⬇ ${t('plugins.install')}`}
                onInstall={() => queryClient.invalidateQueries({ queryKey: ['plugins'] })}
              />
            ))}
          </div>
        )}
      </div>

      <style>{`
        .plugins-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .plugins-header { display:flex; align-items:flex-start; justify-content:space-between; padding:var(--space-md) var(--space-xl); border-bottom:1px solid var(--color-border); flex-shrink:0; gap:var(--space-lg); }
        .plugins-header h2 { font-size:16px; font-weight:600; }
        .plugins-subtitle { font-size:13px; color:var(--color-text-secondary); margin-top:2px; }
        .plugins-tabs { display:flex; gap:0; border-bottom:1px solid var(--color-border); padding:0 var(--space-xl); flex-shrink:0; }
        .plugins-tab { padding:var(--space-sm) var(--space-lg); background:none; border:none; border-bottom:2px solid transparent; cursor:pointer; font-size:14px; color:var(--color-text-secondary); transition:color .15s, border-color .15s; }
        .plugins-tab:hover { color:var(--color-text); }
        .plugins-tab.active { color:var(--color-primary); border-bottom-color:var(--color-primary); font-weight:500; }
        .plugins-content { flex:1; overflow-y:auto; padding:var(--space-xl); }
        .plugins-list { display:flex; flex-direction:column; gap:var(--space-md); max-width:640px; }
        .plugin-card { background:var(--color-surface); border:1px solid var(--color-border); border-radius:var(--radius-lg); padding:var(--space-lg); display:flex; flex-direction:column; gap:var(--space-sm); }
        .plugin-card-header { display:flex; align-items:flex-start; justify-content:space-between; gap:var(--space-md); }
        .plugin-card-info { flex:1; }
        .plugin-name { font-size:15px; font-weight:600; }
        .plugin-version { font-size:12px; color:var(--color-text-tertiary); margin-left:var(--space-sm); }
        .plugin-author { font-size:12px; color:var(--color-text-secondary); margin-top:2px; }
        .plugin-desc { font-size:13px; color:var(--color-text-secondary); line-height:1.5; }
        .plugin-caps { display:flex; flex-wrap:wrap; gap:var(--space-xs); }
        .plugin-cap { display:flex; align-items:center; gap:4px; background:var(--color-bg-tertiary); border-radius:var(--radius-full); padding:2px 8px; font-size:11px; color:var(--color-text-secondary); }
        .plugin-actions { display:flex; gap:var(--space-sm); align-items:center; }
        .toggle { width:44px; height:24px; border-radius:12px; border:none; cursor:pointer; position:relative; transition:background .2s; flex-shrink:0; }
        .toggle-thumb { position:absolute; top:2px; width:20px; height:20px; border-radius:50%; background:white; transition:left .2s; box-shadow:0 1px 3px rgba(0,0,0,.2); }
      `}</style>
    </div>
  );
}

function PluginCard({
  plugin,
  onToggle,
  onUninstall,
  uninstallLabel,
}: {
  plugin: Plugin;
  onToggle: (e: boolean) => void;
  onUninstall: () => void;
  uninstallLabel: string;
}) {
  return (
    <div className="plugin-card">
      <div className="plugin-card-header">
        <div className="plugin-card-info">
          <div>
            <span className="plugin-name">{plugin.name}</span>
            <span className="plugin-version">v{plugin.version}</span>
          </div>
          <div className="plugin-author">by {plugin.author}</div>
        </div>
        <div className="plugin-actions">
          <button
            role="switch"
            aria-checked={plugin.enabled}
            className="toggle"
            style={{ background: plugin.enabled ? 'var(--color-primary)' : 'var(--color-border)' }}
            onClick={() => onToggle(!plugin.enabled)}
          >
            <div className="toggle-thumb" style={{ left: plugin.enabled ? 22 : 2 }} />
          </button>
        </div>
      </div>
      <p className="plugin-desc">{plugin.description}</p>
      <div className="plugin-caps">
        {plugin.capabilities.map(cap => {
          const info = CAPABILITY_LABELS[cap];
          return info ? (
            <span key={cap} className="plugin-cap">
              {info.icon} {info.label}
            </span>
          ) : null;
        })}
      </div>
      <div className="plugin-actions">
        <button className="btn btn-danger" onClick={onUninstall} style={{ fontSize: 12 }}>
          {uninstallLabel}
        </button>
      </div>
    </div>
  );
}

function CatalogCard({
  plugin,
  installLabel,
  onInstall,
}: {
  plugin: (typeof PLUGIN_CATALOG)[0];
  installLabel: string;
  onInstall: () => void;
}) {
  return (
    <div className="plugin-card">
      <div className="plugin-card-header">
        <div className="plugin-card-info">
          <div>
            <span className="plugin-name">{plugin.name}</span>
            <span className="plugin-version">v{plugin.version}</span>
          </div>
          <div className="plugin-author">by {plugin.author}</div>
        </div>
        <button className="btn btn-primary" onClick={onInstall} style={{ fontSize: 12 }}>
          {installLabel}
        </button>
      </div>
      <p className="plugin-desc">{plugin.description}</p>
      <div className="plugin-caps">
        {plugin.capabilities.map(cap => {
          const info = CAPABILITY_LABELS[cap];
          return info ? (
            <span key={cap} className="plugin-cap">
              {info.icon} {info.label}
            </span>
          ) : null;
        })}
      </div>
    </div>
  );
}
