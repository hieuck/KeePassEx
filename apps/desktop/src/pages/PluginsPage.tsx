/**
 * Plugins page — desktop
 * KeePassEx exclusive: WASM plugin sandbox — no competitor has this.
 */
import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useTranslation } from 'react-i18next';
import { useVaultStore } from '../store/vault';

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

const CAPABILITY_ICONS: Record<string, string> = {
  password_generator: '⚡',
  importer: '📥',
  exporter: '📤',
  health_check: '🛡️',
  field_validator: '✅',
  icon_set: '🎨',
  ui_extension: '🔧',
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
    description: 'Import passwords from RoboForm HTML export',
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
  const { isOpen } = useVaultStore();
  const [activeTab, setActiveTab] = useState<'installed' | 'catalog'>('installed');

  const { data: installedPlugins = [], isLoading } = useQuery<Plugin[]>({
    queryKey: ['plugins'],
    queryFn: () => invoke<Plugin[]>('list_plugins'),
    staleTime: 30_000,
  });

  const toggleMutation = useMutation({
    mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) =>
      invoke('toggle_plugin', { pluginId: id, enabled }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['plugins'] }),
  });

  const uninstallMutation = useMutation({
    mutationFn: (id: string) => invoke('uninstall_plugin', { pluginId: id }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['plugins'] }),
  });

  const installFileMutation = useMutation({
    mutationFn: (filePath: string) => invoke('install_plugin_from_file', { filePath }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plugins'] });
      setActiveTab('installed');
    },
  });

  const installFromFile = async () => {
    const file = await open({
      filters: [{ name: 'KeePassEx Plugin', extensions: ['kpxplugin', 'wasm', 'zip'] }],
    });
    if (!file || typeof file !== 'string') return;
    installFileMutation.mutate(file);
  };

  return (
    <div className="plugins-page">
      <div className="plugins-header">
        <div>
          <h2>🔧 {t('plugins.title')}</h2>
          <p className="plugins-subtitle">{t('plugins.subtitle')}</p>
        </div>
        <button
          className="btn btn-secondary"
          onClick={installFromFile}
          disabled={installFileMutation.isPending}
        >
          {installFileMutation.isPending ? '⏳' : '📦'} {t('plugins.installFromFile')}
        </button>
      </div>

      <div className="plugins-tabs" role="tablist">
        <button
          role="tab"
          aria-selected={activeTab === 'installed'}
          className={`plugins-tab ${activeTab === 'installed' ? 'active' : ''}`}
          onClick={() => setActiveTab('installed')}
        >
          {t('plugins.installed')} ({installedPlugins.length})
        </button>
        <button
          role="tab"
          aria-selected={activeTab === 'catalog'}
          className={`plugins-tab ${activeTab === 'catalog' ? 'active' : ''}`}
          onClick={() => setActiveTab('catalog')}
        >
          {t('plugins.catalog')} ({PLUGIN_CATALOG.length})
        </button>
      </div>

      <div className="plugins-content">
        {activeTab === 'installed' ? (
          isLoading ? (
            <div className="plugins-loading">⏳ {t('common.loading')}</div>
          ) : installedPlugins.length === 0 ? (
            <div className="empty-state">
              <span className="empty-state-icon">🔧</span>
              <p className="empty-state-title">{t('plugins.noPlugins')}</p>
              <p className="empty-state-desc">{t('plugins.noPluginsDesc')}</p>
              <button className="btn btn-primary" onClick={installFromFile}>
                📦 {t('plugins.installFromFile')}
              </button>
            </div>
          ) : (
            <div className="plugins-list">
              {installedPlugins.map(plugin => (
                <div key={plugin.id} className="plugin-card">
                  <div className="plugin-card-header">
                    <div className="plugin-card-info">
                      <div className="plugin-name-row">
                        <span className="plugin-name">{plugin.name}</span>
                        <span className="plugin-version">v{plugin.version}</span>
                      </div>
                      <div className="plugin-author">
                        {t('plugins.by')} {plugin.author}
                      </div>
                    </div>
                    <button
                      role="switch"
                      aria-checked={plugin.enabled}
                      className="toggle"
                      style={{
                        background: plugin.enabled ? 'var(--color-primary)' : 'var(--color-border)',
                      }}
                      onClick={() =>
                        toggleMutation.mutate({ id: plugin.id, enabled: !plugin.enabled })
                      }
                      aria-label={plugin.enabled ? t('plugins.disable') : t('plugins.enable')}
                    >
                      <div className="toggle-thumb" style={{ left: plugin.enabled ? 22 : 2 }} />
                    </button>
                  </div>
                  <p className="plugin-desc">{plugin.description}</p>
                  <div className="plugin-caps">
                    {plugin.capabilities.map(cap => (
                      <span key={cap} className="plugin-cap">
                        {CAPABILITY_ICONS[cap] ?? '🔌'}{' '}
                        {t(
                          `plugins.capabilities.${cap.replace(/_([a-z])/g, (_, l) => l.toUpperCase())}`
                        ) || cap}
                      </span>
                    ))}
                  </div>
                  <div className="plugin-actions">
                    <button
                      className="btn btn-danger-sm"
                      onClick={() => {
                        if (confirm(t('plugins.confirmUninstall', { name: plugin.name })))
                          uninstallMutation.mutate(plugin.id);
                      }}
                      disabled={uninstallMutation.isPending}
                    >
                      {t('plugins.uninstall')}
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )
        ) : (
          <div className="plugins-list">
            {PLUGIN_CATALOG.map(plugin => {
              const isInstalled = installedPlugins.some(p => p.id === plugin.id);
              return (
                <div key={plugin.id} className="plugin-card">
                  <div className="plugin-card-header">
                    <div className="plugin-card-info">
                      <div className="plugin-name-row">
                        <span className="plugin-name">{plugin.name}</span>
                        <span className="plugin-version">v{plugin.version}</span>
                      </div>
                      <div className="plugin-author">
                        {t('plugins.by')} {plugin.author}
                      </div>
                    </div>
                    {isInstalled ? (
                      <span className="plugin-installed-badge">✓ {t('plugins.installed')}</span>
                    ) : (
                      <button
                        className="btn btn-primary btn-sm"
                        onClick={() => {
                          // Catalog install: show info (real install requires file download)
                          alert(
                            `${t('plugins.install')}: ${plugin.name}\n\nDownload the plugin file and use "Install from file".`
                          );
                        }}
                      >
                        ⬇ {t('plugins.install')}
                      </button>
                    )}
                  </div>
                  <p className="plugin-desc">{plugin.description}</p>
                  <div className="plugin-caps">
                    {plugin.capabilities.map(cap => (
                      <span key={cap} className="plugin-cap">
                        {CAPABILITY_ICONS[cap] ?? '🔌'} {cap.replace(/_/g, ' ')}
                      </span>
                    ))}
                  </div>
                </div>
              );
            })}
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
        .plugins-loading { font-size:13px; color:var(--color-text-secondary); }
        .plugins-list { display:flex; flex-direction:column; gap:var(--space-md); max-width:640px; }
        .plugin-card { background:var(--color-surface); border:1px solid var(--color-border); border-radius:var(--radius-lg); padding:var(--space-lg); display:flex; flex-direction:column; gap:var(--space-sm); }
        .plugin-card-header { display:flex; align-items:flex-start; justify-content:space-between; gap:var(--space-md); }
        .plugin-card-info { flex:1; }
        .plugin-name-row { display:flex; align-items:center; gap:var(--space-sm); }
        .plugin-name { font-size:15px; font-weight:600; }
        .plugin-version { font-size:12px; color:var(--color-text-tertiary); }
        .plugin-author { font-size:12px; color:var(--color-text-secondary); margin-top:2px; }
        .plugin-desc { font-size:13px; color:var(--color-text-secondary); line-height:1.5; }
        .plugin-caps { display:flex; flex-wrap:wrap; gap:var(--space-xs); }
        .plugin-cap { display:flex; align-items:center; gap:4px; background:var(--color-bg-tertiary); border-radius:var(--radius-full); padding:2px 8px; font-size:11px; color:var(--color-text-secondary); }
        .plugin-actions { display:flex; gap:var(--space-sm); }
        .plugin-installed-badge { font-size:12px; color:#16A34A; font-weight:600; padding:4px 8px; background:#f0fdf4; border-radius:var(--radius-sm); }
        .toggle { width:44px; height:24px; border-radius:12px; border:none; cursor:pointer; position:relative; transition:background .2s; flex-shrink:0; }
        .toggle-thumb { position:absolute; top:2px; width:20px; height:20px; border-radius:50%; background:white; transition:left .2s; box-shadow:0 1px 3px rgba(0,0,0,.2); }
        .btn-danger-sm { background:none; border:1px solid rgba(239,68,68,.4); color:#ef4444; border-radius:var(--radius-sm); padding:3px 10px; cursor:pointer; font-size:12px; }
        .btn-danger-sm:hover { background:rgba(239,68,68,.08); }
        .btn-sm { font-size:12px; padding:3px 10px; }
        .empty-state { display:flex; flex-direction:column; align-items:center; gap:var(--space-md); padding:var(--space-2xl); color:var(--color-text-secondary); text-align:center; }
        .empty-state-icon { font-size:48px; }
        .empty-state-title { font-size:16px; font-weight:600; color:var(--color-text); }
        .empty-state-desc { font-size:13px; }
      `}</style>
    </div>
  );
}
