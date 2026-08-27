/**
 * Module-level draft-value cache for the settings page.
 *
 * Complements the domain-owned settings store by retaining only unsaved
 * presentation drafts across component re-mounts within a session.
 *
 * Designed for `useSyncExternalStore` consumption from `use-settings.ts`.
 */

export interface SettingsDirtyState {
  values: Record<string, unknown>;
  versions: Record<string, number>;
}

const EMPTY_DIRTY: SettingsDirtyState = {
  values: {},
  versions: {},
};

const cache = new Map<string, SettingsDirtyState>();
const listeners = new Map<string, Set<() => void>>();
let nextVersion = 0;

function notify(ns: string): void {
  const set = listeners.get(ns);
  if (!set) return;
  for (const cb of set) cb();
}

export function getDirty(ns: string): SettingsDirtyState {
  return cache.get(ns) ?? EMPTY_DIRTY;
}

export function setDirty(ns: string, key: string, value: unknown): void {
  const cur = cache.get(ns) ?? EMPTY_DIRTY;
  nextVersion += 1;
  cache.set(ns, {
    ...cur,
    values: { ...cur.values, [key]: value },
    versions: { ...cur.versions, [key]: nextVersion },
  });
  notify(ns);
}

export function clearDirty(ns: string): void {
  const cur = cache.get(ns);
  if (!cur || Object.keys(cur.values).length === 0) return;
  cache.set(ns, { values: {}, versions: {} });
  notify(ns);
}

export function reconcileSubmittedDirty(
  ns: string,
  submitted: SettingsDirtyState,
  successfulKeys: Iterable<string>,
): void {
  const current = cache.get(ns);
  if (!current) return;
  const values = { ...current.values };
  const versions = { ...current.versions };
  let changed = false;
  for (const key of successfulKeys) {
    if (
      submitted.versions[key] !== undefined &&
      current.versions[key] === submitted.versions[key]
    ) {
      delete values[key];
      delete versions[key];
      changed = true;
    }
  }
  if (!changed) return;
  cache.set(ns, { values, versions });
  notify(ns);
}

export function subscribe(ns: string, cb: () => void): () => void {
  let set = listeners.get(ns);
  if (!set) {
    set = new Set();
    listeners.set(ns, set);
  }
  set.add(cb);
  return () => {
    const s = listeners.get(ns);
    s?.delete(cb);
  };
}

/** Test helper — resets ALL cached state. Not exported in production usage paths. */
export function __resetForTests(): void {
  cache.clear();
  listeners.clear();
  nextVersion = 0;
}
