import { useMemoryAdminStore } from "@/stores/memory-admin-store";

export function useMemoryAdmin() {
  const items = useMemoryAdminStore((s) => s.items);
  const stats = useMemoryAdminStore((s) => s.stats);
  const loading = useMemoryAdminStore((s) => s.loading);
  const error = useMemoryAdminStore((s) => s.error);
  const deleting = useMemoryAdminStore((s) => s.deleting);
  const load = useMemoryAdminStore((s) => s.load);
  const loadStats = useMemoryAdminStore((s) => s.loadStats);
  const deleteOne = useMemoryAdminStore((s) => s.deleteOne);
  const bulkDelete = useMemoryAdminStore((s) => s.bulkDelete);

  return {
    items,
    stats,
    loading,
    error,
    deleting,
    load,
    loadStats,
    deleteOne,
    bulkDelete,
  };
}
