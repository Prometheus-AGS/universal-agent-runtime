import { useEffect } from "react";

import { useA2uiSchemasStore } from "@/stores/a2ui-schemas-store";

export function useA2uiSchemas() {
  const schemas = useA2uiSchemasStore((s) => s.schemas);
  const loading = useA2uiSchemasStore((s) => s.loading);
  const error = useA2uiSchemasStore((s) => s.error);
  const load = useA2uiSchemasStore((s) => s.load);

  useEffect(() => {
    void load();
  }, [load]);

  return { schemas, loading, error, load };
}
