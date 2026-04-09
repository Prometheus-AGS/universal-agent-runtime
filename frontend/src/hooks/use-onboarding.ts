import { useCallback, useSyncExternalStore } from "react";

/**
 * Lightweight onboarding state backed by localStorage.
 *
 * Each hint/banner has a string key (e.g. "admin-welcome", "kb-pipeline").
 * Once dismissed, the key is stored in localStorage and the hint won't show again.
 *
 * Usage:
 *   const { dismissed, dismiss } = useOnboarding("admin-welcome");
 *   if (!dismissed) return <Banner onDismiss={dismiss} />;
 */

const STORAGE_PREFIX = "uar-onboarding-";

function getKey(id: string): string {
  return `${STORAGE_PREFIX}${id}`;
}

function isDismissed(id: string): boolean {
  try {
    return localStorage.getItem(getKey(id)) === "1";
  } catch {
    return false;
  }
}

// Notify all subscribers when any onboarding state changes.
const listeners = new Set<() => void>();

function subscribe(cb: () => void) {
  listeners.add(cb);
  return () => listeners.delete(cb);
}

function notify() {
  for (const cb of listeners) cb();
}

export function useOnboarding(id: string) {
  const dismissed = useSyncExternalStore(
    subscribe,
    () => isDismissed(id),
    () => false, // SSR: always show
  );

  const dismiss = useCallback(() => {
    try {
      localStorage.setItem(getKey(id), "1");
    } catch {
      /* quota exceeded — ignore */
    }
    notify();
  }, [id]);

  const reset = useCallback(() => {
    try {
      localStorage.removeItem(getKey(id));
    } catch {
      /* ignore */
    }
    notify();
  }, [id]);

  return { dismissed, dismiss, reset } as const;
}

/** Reset all onboarding state — useful for testing or a "Show tips again" button. */
export function resetAllOnboarding() {
  try {
    const toRemove: string[] = [];
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (key?.startsWith(STORAGE_PREFIX)) toRemove.push(key);
    }
    for (const key of toRemove) localStorage.removeItem(key);
  } catch {
    /* ignore */
  }
  notify();
}
