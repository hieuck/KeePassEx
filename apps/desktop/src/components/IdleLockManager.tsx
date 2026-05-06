/**
 * IdleLockManager — auto-lock vault after configurable idle period
 * Listens to mouse/keyboard activity and locks when idle too long
 */
import { useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useVaultStore } from '../store/vault';
import { useSettingsStore } from '../store/settings';

export function IdleLockManager() {
  const { isOpen, isLocked, lockVault } = useVaultStore();
  const { settings } = useSettingsStore();
  const navigate = useNavigate();
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const resetTimer = () => {
    if (timerRef.current) clearTimeout(timerRef.current);

    const minutes = settings.lockAfterIdleMinutes;
    if (!minutes || !isOpen || isLocked) return;

    timerRef.current = setTimeout(async () => {
      await lockVault();
      navigate('/unlock');
    }, minutes * 60 * 1000);
  };

  useEffect(() => {
    if (!isOpen || isLocked || !settings.lockAfterIdleMinutes) return;

    const events = ['mousemove', 'mousedown', 'keydown', 'touchstart', 'scroll'];
    events.forEach(e => window.addEventListener(e, resetTimer, { passive: true }));
    resetTimer();

    return () => {
      events.forEach(e => window.removeEventListener(e, resetTimer));
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [isOpen, isLocked, settings.lockAfterIdleMinutes]);

  // Lock on screen lock (via visibility change)
  useEffect(() => {
    if (!settings.lockOnScreenLock) return;

    const handleVisibility = () => {
      if (document.hidden && isOpen && !isLocked) {
        lockVault();
      }
    };

    document.addEventListener('visibilitychange', handleVisibility);
    return () => document.removeEventListener('visibilitychange', handleVisibility);
  }, [isOpen, isLocked, settings.lockOnScreenLock]);

  return null; // No UI — pure logic component
}
