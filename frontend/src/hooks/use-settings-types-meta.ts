import { useEffect } from "react";

import { useSettingsTypesMetaStore } from "@/stores/settings-types-meta-store";

export function useSettingsTypesMeta() {
  const types = useSettingsTypesMetaStore((s) => s.types);
  const load = useSettingsTypesMetaStore((s) => s.load);

  useEffect(() => {
    void load();
  }, [load]);

  return types;
}
