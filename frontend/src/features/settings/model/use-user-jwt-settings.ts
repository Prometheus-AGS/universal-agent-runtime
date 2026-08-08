import { useEffect } from "react";

import { useUserJwtSettingsStore } from "./user-jwt-settings-store";

export function useUserJwtSettings() {
  const settings = useUserJwtSettingsStore((s) => s.settings);
  const loading = useUserJwtSettingsStore((s) => s.loading);
  const saving = useUserJwtSettingsStore((s) => s.saving);
  const error = useUserJwtSettingsStore((s) => s.error);
  const load = useUserJwtSettingsStore((s) => s.load);
  const save = useUserJwtSettingsStore((s) => s.save);

  useEffect(() => {
    void load();
  }, [load]);

  return { settings, loading, saving, error, load, save };
}
