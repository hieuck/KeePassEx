/**
 * Sync state store — Zustand
 */
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { SyncStatus, SyncResult, SyncConfig } from '@keepassex/types';

interface SyncState {
  status: SyncStatus | null;
  syncing: boolean;
  lastResult: SyncResult | null;
  error: string | null;

  fetchStatus: () => Promise<void>;
  configure: (config: SyncConfig & { username?: string; password?: string; serverUrl?: string }) => Promise<void>;
  syncNow: () => Promise<SyncResult>;
  testConnection: (provider: string, remotePath: string, username?: string, password?: string) => Promise<boolean>;
}

export const useSyncStore = create<SyncState>((set) => ({
  status: null,
  syncing: false,
  lastResult: null,
  error: null,

  fetchStatus: async () => {
    try {
      const status = await invoke<SyncStatus>('get_sync_status');
      set({ status });
    } catch {
      // Ignore
    }
  },

  configure: async (config) => {
    await invoke('configure_sync', {
      args: {
        provider: config.provider,
        remote_path: config.remotePath,
        auto_sync: config.autoSync,
        sync_interval_seconds: config.syncIntervalSeconds,
        conflict_resolution: config.conflictResolution,
        username: config.username ?? null,
        password: config.password ?? null,
        server_url: config.serverUrl ?? null,
      },
    });
    await useSyncStore.getState().fetchStatus();
  },

  syncNow: async () => {
    set({ syncing: true, error: null });
    try {
      const result = await invoke<SyncResult>('sync_now');
      set({ syncing: false, lastResult: result });
      return result;
    } catch (e: unknown) {
      const error = e instanceof Error ? e.message : String(e);
      set({ syncing: false, error });
      throw e;
    }
  },

  testConnection: async (provider, remotePath, username, password) => {
    return invoke<boolean>('test_sync_connection', {
      provider,
      remote_path: remotePath,
      username: username ?? null,
      password: password ?? null,
    });
  },
}));
