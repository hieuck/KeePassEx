/**
 * KeePassEx Browser Extension — Content Script
 * Detects login forms and handles credential filling
 */

interface Credentials {
  username: string;
  password: string;
  url: string;
  title: string;
}

interface EntryOption {
  uuid: string;
  title: string;
  username: string;
  url: string;
}

// ─── Form Detection ───────────────────────────────────────────────────────────

function findPasswordFields(): HTMLInputElement[] {
  return Array.from(
    document.querySelectorAll<HTMLInputElement>('input[type="password"]')
  ).filter(el => isVisible(el));
}

function findUsernameField(passwordField: HTMLInputElement): HTMLInputElement | null {
  // Look for username/email field before the password field
  const form = passwordField.closest('form');
  const container = form ?? document.body;

  const candidates = Array.from(
    container.querySelectorAll<HTMLInputElement>(
      'input[type="text"], input[type="email"], input[type="tel"], input:not([type])'
    )
  ).filter(isVisible);

  // Find the last visible text input before the password field
  const pwIndex = getElementIndex(passwordField);
  const before = candidates.filter(el => getElementIndex(el) < pwIndex);
  return before[before.length - 1] ?? null;
}

function getElementIndex(el: Element): number {
  const all = Array.from(document.querySelectorAll('*'));
  return all.indexOf(el);
}

function isVisible(el: HTMLElement): boolean {
  const style = window.getComputedStyle(el);
  return (
    style.display !== 'none' &&
    style.visibility !== 'hidden' &&
    style.opacity !== '0' &&
    el.offsetWidth > 0 &&
    el.offsetHeight > 0
  );
}

// ─── Credential Filling ───────────────────────────────────────────────────────

function fillCredentials(credentials: Credentials): boolean {
  const passwordFields = findPasswordFields();
  if (passwordFields.length === 0) return false;

  let filled = false;

  for (const pwField of passwordFields) {
    const usernameField = findUsernameField(pwField);

    if (usernameField && credentials.username) {
      setNativeValue(usernameField, credentials.username);
      filled = true;
    }

    if (credentials.password) {
      setNativeValue(pwField, credentials.password);
      filled = true;
    }
  }

  return filled;
}

/**
 * Set value in a way that React/Vue/Angular detect the change
 */
function setNativeValue(el: HTMLInputElement, value: string): void {
  const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
    window.HTMLInputElement.prototype,
    'value'
  )?.set;

  if (nativeInputValueSetter) {
    nativeInputValueSetter.call(el, value);
  } else {
    el.value = value;
  }

  // Trigger events that frameworks listen to
  el.dispatchEvent(new Event('input', { bubbles: true }));
  el.dispatchEvent(new Event('change', { bubbles: true }));
  el.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true }));
  el.dispatchEvent(new KeyboardEvent('keyup', { bubbles: true }));
}

// ─── Fill Picker UI ───────────────────────────────────────────────────────────

let pickerEl: HTMLElement | null = null;

function showFillPicker(entries: EntryOption[]): void {
  removePicker();

  if (!entries || entries.length === 0) {
    showNotification('No matching entries found', 'info');
    return;
  }

  pickerEl = document.createElement('div');
  pickerEl.id = 'keepassex-picker';
  pickerEl.setAttribute('role', 'dialog');
  pickerEl.setAttribute('aria-label', 'KeePassEx — Select entry to fill');
  pickerEl.innerHTML = `
    <div class="kpx-picker-header">
      <span class="kpx-logo">🔐</span>
      <span class="kpx-title">KeePassEx</span>
      <button class="kpx-close" aria-label="Close">✕</button>
    </div>
    <div class="kpx-entries" role="list">
      ${entries.map(e => `
        <button class="kpx-entry" data-uuid="${e.uuid}" role="listitem">
          <span class="kpx-entry-title">${escapeHtml(e.title)}</span>
          <span class="kpx-entry-username">${escapeHtml(e.username)}</span>
        </button>
      `).join('')}
    </div>
  `;

  // Styles
  const style = document.createElement('style');
  style.textContent = `
    #keepassex-picker {
      position: fixed;
      top: 20px;
      right: 20px;
      z-index: 2147483647;
      background: #fff;
      border: 1px solid #e5e7eb;
      border-radius: 12px;
      box-shadow: 0 20px 60px rgba(0,0,0,0.15);
      width: 280px;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
      font-size: 14px;
      overflow: hidden;
    }
    .kpx-picker-header {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 12px 16px;
      background: #f9fafb;
      border-bottom: 1px solid #e5e7eb;
    }
    .kpx-logo { font-size: 18px; }
    .kpx-title { flex: 1; font-weight: 600; color: #111827; }
    .kpx-close {
      background: none;
      border: none;
      cursor: pointer;
      color: #6b7280;
      font-size: 14px;
      padding: 2px 6px;
      border-radius: 4px;
    }
    .kpx-close:hover { background: #f3f4f6; }
    .kpx-entries { max-height: 240px; overflow-y: auto; }
    .kpx-entry {
      display: flex;
      flex-direction: column;
      align-items: flex-start;
      width: 100%;
      padding: 10px 16px;
      border: none;
      background: none;
      cursor: pointer;
      text-align: left;
      border-bottom: 1px solid #f3f4f6;
      gap: 2px;
    }
    .kpx-entry:hover { background: #f9fafb; }
    .kpx-entry:focus-visible { outline: 2px solid #2563eb; outline-offset: -2px; }
    .kpx-entry-title { font-weight: 500; color: #111827; }
    .kpx-entry-username { font-size: 12px; color: #6b7280; }
  `;

  document.head.appendChild(style);
  document.body.appendChild(pickerEl);

  // Event listeners
  pickerEl.querySelector('.kpx-close')?.addEventListener('click', removePicker);

  pickerEl.querySelectorAll('.kpx-entry').forEach(btn => {
    btn.addEventListener('click', async () => {
      const uuid = (btn as HTMLElement).dataset.uuid;
      if (!uuid) return;

      removePicker();

      const response = await chrome.runtime.sendMessage({
        action: 'AUTOFILL',
        payload: { entryUuid: uuid, tabId: undefined },
      });
    });
  });

  // Close on outside click
  setTimeout(() => {
    document.addEventListener('click', handleOutsideClick, { once: true });
  }, 100);
}

function handleOutsideClick(e: MouseEvent): void {
  if (pickerEl && !pickerEl.contains(e.target as Node)) {
    removePicker();
  }
}

function removePicker(): void {
  pickerEl?.remove();
  pickerEl = null;
}

function showNotification(message: string, type: 'info' | 'success' | 'error'): void {
  const el = document.createElement('div');
  el.style.cssText = `
    position: fixed;
    bottom: 20px;
    right: 20px;
    z-index: 2147483647;
    background: ${type === 'error' ? '#DC2626' : type === 'success' ? '#16A34A' : '#2563EB'};
    color: white;
    padding: 10px 16px;
    border-radius: 8px;
    font-family: -apple-system, sans-serif;
    font-size: 13px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    animation: kpx-slide-in 0.2s ease;
  `;
  el.textContent = `🔐 ${message}`;
  document.body.appendChild(el);
  setTimeout(() => el.remove(), 3000);
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

// ─── Message Listener ─────────────────────────────────────────────────────────

chrome.runtime.onMessage.addListener((message: { action: string; payload?: unknown }) => {
  switch (message.action) {
    case 'FILL_CREDENTIALS': {
      const creds = message.payload as Credentials;
      const filled = fillCredentials(creds);
      if (filled) {
        showNotification(`Filled: ${creds.title}`, 'success');
      } else {
        showNotification('No login form found on this page', 'info');
      }
      break;
    }

    case 'SHOW_FILL_PICKER': {
      const entries = message.payload as EntryOption[];
      showFillPicker(entries);
      break;
    }

    case 'COPY_TO_CLIPBOARD': {
      const { text } = message.payload as { text: string };
      navigator.clipboard.writeText(text).catch(() => {
        // Fallback for older browsers
        const el = document.createElement('textarea');
        el.value = text;
        el.style.position = 'fixed';
        el.style.opacity = '0';
        document.body.appendChild(el);
        el.select();
        document.execCommand('copy');
        el.remove();
      });
      break;
    }

    case 'FILL_GENERATED_PASSWORD': {
      const { password } = message.payload as { password: string };
      const active = document.activeElement as HTMLInputElement;
      if (active && (active.type === 'password' || active.type === 'text')) {
        setNativeValue(active, password);
        showNotification('Password generated and filled', 'success');
      }
      break;
    }
  }
});

// ─── Page Load Detection ──────────────────────────────────────────────────────

// Notify background when a login form is detected
function detectLoginForms(): void {
  const pwFields = findPasswordFields();
  if (pwFields.length > 0) {
    chrome.runtime.sendMessage({
      action: 'LOGIN_FORM_DETECTED',
      payload: { url: window.location.href },
    }).catch(() => {}); // Ignore if background not ready
  }
}

// Run on load and on DOM changes
detectLoginForms();

const observer = new MutationObserver(() => {
  detectLoginForms();
});

observer.observe(document.body, {
  childList: true,
  subtree: true,
});

console.log('[KeePassEx] Content script loaded on', window.location.hostname);
