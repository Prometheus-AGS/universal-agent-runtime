import { useMemoryAdminStore } from "./memory-admin-store";

export function useMemoryAdmin() {
  const loading = useMemoryAdminStore((state) => state.loading);
  const deleting = useMemoryAdminStore((state) => state.deleting);
  const error = useMemoryAdminStore((state) => state.error);
  const load = useMemoryAdminStore((state) => state.load);
  const remove = useMemoryAdminStore((state) => state.remove);
  const removeVisible = useMemoryAdminStore((state) => state.removeVisible);
  const clearError = useMemoryAdminStore((state) => state.clearError);
  return { loading, deleting, error, load, remove, removeVisible, clearError };
}
