import { useUiStore } from "@/stores/ui-store";

/** Subscribe to shared navigation/sidebar UI state. */
export function useUiState() {
  return {
    mobileSidebarOpen: useUiStore((state) => state.mobileSidebarOpen),
    setMobileSidebarOpen: useUiStore((state) => state.setMobileSidebarOpen),
    toggleMobileSidebar: useUiStore((state) => state.toggleMobileSidebar),
  };
}
