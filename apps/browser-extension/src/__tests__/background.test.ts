/**
 * Browser extension background script tests
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock webextension-polyfill
vi.mock('webextension-polyfill', () => ({
  default: {
    runtime: {
      connectNative: vi.fn(() => ({
        postMessage: vi.fn(),
        onMessage: { addListener: vi.fn() },
        onDisconnect: { addListener: vi.fn() },
      })),
      onMessage: { addListener: vi.fn() },
      sendMessage: vi.fn(),
    },
    tabs: {
      query: vi.fn().mockResolvedValue([{ id: 1, url: 'https://github.com' }]),
      sendMessage: vi.fn().mockResolvedValue(undefined),
    },
    contextMenus: {
      create: vi.fn(),
      onClicked: { addListener: vi.fn() },
    },
    commands: {
      onCommand: { addListener: vi.fn() },
    },
    storage: {
      local: {
        get: vi.fn().mockResolvedValue({}),
        set: vi.fn().mockResolvedValue(undefined),
      },
    },
  },
}));

// ─── URL domain extraction ────────────────────────────────────────────────────

describe('URL domain extraction', () => {
  function extractDomain(url: string): string {
    try {
      const u = new URL(url);
      return u.hostname.replace(/^www\./, '');
    } catch {
      return url;
    }
  }

  it('extracts domain from https URL', () => {
    expect(extractDomain('https://github.com/user/repo')).toBe('github.com');
  });

  it('strips www prefix', () => {
    expect(extractDomain('https://www.google.com')).toBe('google.com');
  });

  it('handles http', () => {
    expect(extractDomain('http://example.com/path')).toBe('example.com');
  });

  it('handles invalid URL', () => {
    expect(extractDomain('not-a-url')).toBe('not-a-url');
  });

  it('handles URL with port', () => {
    expect(extractDomain('https://localhost:3000')).toBe('localhost');
  });
});

// ─── Native message ID generation ────────────────────────────────────────────

describe('Message ID generation', () => {
  it('generates unique IDs', () => {
    const ids = new Set(
      Array.from({ length: 100 }, () => crypto.randomUUID())
    );
    expect(ids.size).toBe(100);
  });

  it('generates UUID format', () => {
    const id = crypto.randomUUID();
    expect(id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
  });
});

// ─── Content script form detection ───────────────────────────────────────────

describe('Form detection logic', () => {
  // Simulate DOM environment
  beforeEach(() => {
    document.body.innerHTML = '';
  });

  it('detects password field', () => {
    document.body.innerHTML = '<input type="password" />';
    const fields = document.querySelectorAll<HTMLInputElement>('input[type="password"]');
    expect(fields.length).toBe(1);
  });

  it('detects multiple password fields', () => {
    document.body.innerHTML = `
      <input type="password" name="password" />
      <input type="password" name="confirm" />
    `;
    const fields = document.querySelectorAll<HTMLInputElement>('input[type="password"]');
    expect(fields.length).toBe(2);
  });

  it('finds username field before password', () => {
    document.body.innerHTML = `
      <input type="email" id="email" />
      <input type="password" id="password" />
    `;
    const emailField = document.getElementById('email') as HTMLInputElement;
    const passwordField = document.getElementById('password') as HTMLInputElement;

    expect(emailField).toBeTruthy();
    expect(passwordField).toBeTruthy();

    // Email field should come before password in DOM order
    const allInputs = Array.from(document.querySelectorAll('input'));
    const emailIdx = allInputs.indexOf(emailField);
    const passIdx = allInputs.indexOf(passwordField);
    expect(emailIdx).toBeLessThan(passIdx);
  });

  it('handles form with no password field', () => {
    document.body.innerHTML = '<input type="text" /><input type="email" />';
    const fields = document.querySelectorAll<HTMLInputElement>('input[type="password"]');
    expect(fields.length).toBe(0);
  });
});

// ─── Credential filling ───────────────────────────────────────────────────────

describe('Credential filling', () => {
  beforeEach(() => {
    document.body.innerHTML = `
      <form>
        <input type="email" id="username" />
        <input type="password" id="password" />
        <button type="submit">Login</button>
      </form>
    `;
  });

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
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
  }

  it('fills username field', () => {
    const usernameField = document.getElementById('username') as HTMLInputElement;
    setNativeValue(usernameField, 'user@example.com');
    expect(usernameField.value).toBe('user@example.com');
  });

  it('fills password field', () => {
    const passwordField = document.getElementById('password') as HTMLInputElement;
    setNativeValue(passwordField, 'SecureP@ss123!');
    expect(passwordField.value).toBe('SecureP@ss123!');
  });

  it('dispatches input event after filling', () => {
    const usernameField = document.getElementById('username') as HTMLInputElement;
    const inputHandler = vi.fn();
    usernameField.addEventListener('input', inputHandler);

    setNativeValue(usernameField, 'test@example.com');
    expect(inputHandler).toHaveBeenCalled();
  });
});

// ─── HTML escaping ────────────────────────────────────────────────────────────

describe('HTML escaping', () => {
  function escapeHtml(str: string): string {
    return str
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  it('escapes ampersand', () => {
    expect(escapeHtml('AT&T')).toBe('AT&amp;T');
  });

  it('escapes angle brackets', () => {
    expect(escapeHtml('<script>')).toBe('&lt;script&gt;');
  });

  it('escapes quotes', () => {
    expect(escapeHtml('"hello"')).toBe('&quot;hello&quot;');
  });

  it('handles clean string', () => {
    expect(escapeHtml('Hello World')).toBe('Hello World');
  });

  it('prevents XSS in entry titles', () => {
    const malicious = '<img src=x onerror=alert(1)>';
    const escaped = escapeHtml(malicious);
    expect(escaped).not.toContain('<img');
    expect(escaped).toContain('&lt;img');
  });
});
