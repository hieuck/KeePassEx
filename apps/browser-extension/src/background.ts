/**
 * KeePassEx Browser Extension — Background Service Worker (MV3)
 * Handles native messaging with the desktop app and coordinates autofill
 */
import browser from 'webextension-polyfill';

// ─── Native Messaging ─────────────────────────────────────────────────────────

const NATIVE_HOST = 'com.keepassex.app';
let nativePort: browser.Runtime.Port | null = null;
let pendingRequests = new Map<string, (response: NativeResponse) => void>();

interface NativeMessage {
  id: string;
  action: string;
  payload?: unknown;
}

interface NativeResponse {
  id: string;
  success: boolean;
  data?: unknown;
  error?: string;
}

function connectNative(): browser.Runtime.Port {
  if (nativePort) return nativePort;

  nativePort = browser.runtime.connectNative(NATIVE_HOST);

  nativePort.onMessage.addListener((message: NativeResponse) => {
    const resolve = pendingRequests.get(message.id);
    if (resolve) {
      pendingRequests.delete(message.id);
      resolve(message);
    }
  });

  nativePort.onDisconnect.addListener(() => {
    nativePort = null;
    // Reject all pending requests
    for (const [id, resolve] of pendingRequests) {
      resolve({ id, success: false, error: 'Native host disconnected' });
    }
    pendingRequests.clear();
  });

  return nativePort;
}

async function sendNativeMessage(action: string, payload?: unknown): Promise<NativeResponse> {
  return new Promise((resolve) => {
    const id = crypto.randomUUID();
    const message: NativeMessage = { id, action, payload };

    pendingRequests.set(id, resolve);

    try {
      const port = connectNative();
      port.postMessage(message);
    } catch (e) {
      pendingRequests.delete(id);
      resolve({ id, success: false, error: String(e) });
    }

    // Timeout after 10 seconds
    setTimeout(() => {
      if (pendingRequests.has(id)) {
        pendingRequests.delete(id);
        resolve({ id, success: false, error: 'Request timed out' });
      }
    }, 10_000);
  });
}

// ─── Message Handlers ─────────────────────────────────────────────────────────

browser.runtime.onMessage.addListener(async (message: { action: string; payload?: unknown }, sender) => {
  switch (message.action) {
    case 'GET_CREDENTIALS_FOR_URL': {
      const { url } = message.payload as { url: string };
      return sendNativeMessage('getCredentialsForUrl', { url });
    }

    case 'AUTOFILL': {
      const { entryUuid, tabId } = message.payload as { entryUuid: string; tabId: number };
      const response = await sendNativeMessage('getEntryForAutofill', { uuid: entryUuid });
      if (response.success && response.data) {
        // Send credentials to content script
        await browser.tabs.sendMessage(tabId, {
          action: 'FILL_CREDENTIALS',
          payload: response.data,
        });
      }
      return response;
    }

    case 'COPY_PASSWORD': {
      const { entryUuid } = message.payload as { entryUuid: string };
      const response = await sendNativeMessage('getEntryPassword', { uuid: entryUuid });
      if (response.success && response.data) {
        await browser.tabs.sendMessage(sender.tab!.id!, {
          action: 'COPY_TO_CLIPBOARD',
          payload: { text: response.data },
        });
      }
      return response;
    }

    case 'GENERATE_OTP': {
      const { entryUuid } = message.payload as { entryUuid: string };
      return sendNativeMessage('generateTotp', { uuid: entryUuid });
    }

    case 'CHECK_NATIVE_HOST': {
      return sendNativeMessage('ping');
    }

    case 'GET_VAULT_STATUS': {
      return sendNativeMessage('getVaultStatus');
    }

    default:
      return { success: false, error: `Unknown action: ${message.action}` };
  }
});

// ─── Context Menu ─────────────────────────────────────────────────────────────

browser.runtime.onInstalled.addListener(() => {
  browser.contextMenus.create({
    id: 'keepassex-fill',
    title: 'Fill with KeePassEx',
    contexts: ['editable'],
  });

  browser.contextMenus.create({
    id: 'keepassex-generate',
    title: 'Generate Password',
    contexts: ['editable'],
  });
});

browser.contextMenus.onClicked.addListener(async (info, tab) => {
  if (!tab?.id) return;

  if (info.menuItemId === 'keepassex-fill') {
    const url = tab.url ?? '';
    const response = await sendNativeMessage('getCredentialsForUrl', { url });
    if (response.success) {
      await browser.tabs.sendMessage(tab.id, {
        action: 'SHOW_FILL_PICKER',
        payload: response.data,
      });
    }
  }

  if (info.menuItemId === 'keepassex-generate') {
    const response = await sendNativeMessage('generatePassword', {
      mode: 'random',
      length: 20,
    });
    if (response.success) {
      await browser.tabs.sendMessage(tab.id, {
        action: 'FILL_GENERATED_PASSWORD',
        payload: response.data,
      });
    }
  }
});

// ─── Keyboard Shortcut ────────────────────────────────────────────────────────

browser.commands.onCommand.addListener(async (command) => {
  if (command === 'autofill') {
    const [tab] = await browser.tabs.query({ active: true, currentWindow: true });
    if (!tab?.id || !tab.url) return;

    const response = await sendNativeMessage('getCredentialsForUrl', { url: tab.url });
    if (response.success && Array.isArray(response.data) && response.data.length === 1) {
      // Single match — autofill immediately
      await browser.tabs.sendMessage(tab.id, {
        action: 'FILL_CREDENTIALS',
        payload: response.data[0],
      });
    } else if (response.success) {
      // Multiple matches — show picker
      await browser.tabs.sendMessage(tab.id, {
        action: 'SHOW_FILL_PICKER',
        payload: response.data,
      });
    }
  }
});

console.log('[KeePassEx] Background service worker started');
