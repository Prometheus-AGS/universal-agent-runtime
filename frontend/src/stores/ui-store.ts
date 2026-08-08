import { create } from "zustand";

export type ShellSheet = "configure" | null;

interface UiState {
  mobileSidebarOpen: boolean;
  navRailCollapsed: boolean;
  commandPaletteOpen: boolean;
  shellSheet: ShellSheet;
}
interface UiActions {
  setMobileSidebarOpen: (open: boolean) => void;
  toggleMobileSidebar: () => void;
  setNavRailCollapsed: (collapsed: boolean) => void;
  toggleNavRail: () => void;
  setCommandPaletteOpen: (open: boolean) => void;
  setShellSheet: (sheet: ShellSheet) => void;
  closeShellOverlays: () => void;
}
type UiStore = UiState & UiActions;

export const useUiStore = create<UiStore>((set) => ({
  mobileSidebarOpen: false,
  navRailCollapsed: false,
  commandPaletteOpen: false,
  shellSheet: null,
  setMobileSidebarOpen: (open) => set({ mobileSidebarOpen: open }),
  toggleMobileSidebar: () => set((s) => ({ mobileSidebarOpen: !s.mobileSidebarOpen })),
  setNavRailCollapsed: (collapsed) => set({ navRailCollapsed: collapsed }),
  toggleNavRail: () => set((state) => ({ navRailCollapsed: !state.navRailCollapsed })),
  setCommandPaletteOpen: (open) => set({ commandPaletteOpen: open }),
  setShellSheet: (sheet) => set({ shellSheet: sheet }),
  closeShellOverlays: () => set({ commandPaletteOpen: false, shellSheet: null }),
}));
