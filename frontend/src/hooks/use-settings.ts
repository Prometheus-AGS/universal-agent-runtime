import { useEffect, useMemo } from "react";

import { useSettingsStore } from "@/stores/settings-store";
import type { SettingWithMeta } from "@/types";

export interface UseSettingsReturn {
  values: Record<string, unknown>;
  settings: Record<string, SettingWithMeta>;
  loading: boolean;
  saving: boolean;
  error: string | null;
  setSetting: (key: string, value: unknown) => void;
  saveAll: () => Promise<void>;
  reload: () => Promise<void>;
}

export function useSettings(namespace: string): UseSettingsReturn {
  const slice = useSettingsStore((s) => s.namespaces[namespace]);
  const load = useSettingsStore((s) => s.load);
  const setSettingStore = useSettingsStore((s) => s.setSetting);
  const saveAllStore = useSettingsStore((s) => s.saveAll);

  useEffect(() => {
    void load(namespace);
  }, [namespace, load]);

  const setSetting = useMemo(
    () => (key: string, value: unknown) => setSettingStore(namespace, key, value),
    [namespace, setSettingStore],
  );

  const saveAll = useMemo(
    () => () => saveAllStore(namespace),
    [namespace, saveAllStore],
  );

  const reload = useMemo(() => () => load(namespace), [namespace, load]);

  return {
    values: slice?.values ?? {},
    settings: slice?.settings ?? {},
    loading: slice?.loading ?? true,
    saving: slice?.saving ?? false,
    error: slice?.error ?? null,
    setSetting,
    saveAll,
    reload,
  };
}
