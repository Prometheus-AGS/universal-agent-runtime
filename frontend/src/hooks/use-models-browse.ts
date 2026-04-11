import { useEffect } from "react";

import { useModelsBrowseStore } from "@/stores/models-browse-store";

export function useModelsBrowse() {
  const response = useModelsBrowseStore((s) => s.response);
  const loading = useModelsBrowseStore((s) => s.loading);
  const error = useModelsBrowseStore((s) => s.error);
  const load = useModelsBrowseStore((s) => s.load);

  useEffect(() => {
    void load();
  }, [load]);

  return { response, loading, error, load };
}
