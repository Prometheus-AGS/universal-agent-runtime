import { useUiStore } from "@/stores/ui-store";

/** Subscribe to shared navigation/sidebar UI state. */
export function useUiState() {
  return {
    mobileSidebarOpen: useUiStore((state) => state.mobileSidebarOpen),
    setMobileSidebarOpen: useUiStore((state) => state.setMobileSidebarOpen),
    toggleMobileSidebar: useUiStore((state) => state.toggleMobileSidebar),
    navRailCollapsed: useUiStore((state) => state.navRailCollapsed),
    setNavRailCollapsed: useUiStore((state) => state.setNavRailCollapsed),
    toggleNavRail: useUiStore((state) => state.toggleNavRail),
    commandPaletteOpen: useUiStore((state) => state.commandPaletteOpen),
    setCommandPaletteOpen: useUiStore((state) => state.setCommandPaletteOpen),
    shellSheet: useUiStore((state) => state.shellSheet),
    setShellSheet: useUiStore((state) => state.setShellSheet),
    closeShellOverlays: useUiStore((state) => state.closeShellOverlays),
  };
}
