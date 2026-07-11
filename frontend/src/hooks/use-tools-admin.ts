import { useToolsAdminStore } from "@/stores/tools-admin-store";

export function useToolsAdmin() {
  const loading = useToolsAdminStore((state) => state.loading);
  const error = useToolsAdminStore((state) => state.error);
  const executing = useToolsAdminStore((state) => state.executing);
  const executionError = useToolsAdminStore((state) => state.executionError);
  const load = useToolsAdminStore((state) => state.load);
  const execute = useToolsAdminStore((state) => state.execute);
  return { loading, error, executing, executionError, load, execute };
}
