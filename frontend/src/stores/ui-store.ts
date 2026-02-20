import { create } from "zustand";

interface UiState {
  mobileSidebarOpen: boolean;
}
interface UiActions {
  setMobileSidebarOpen: (open: boolean) => void;
  toggleMobileSidebar: () => void;
}
type UiStore = UiState & UiActions;

export const useUiStore = create<UiStore>((set) => ({
  mobileSidebarOpen: false,
  setMobileSidebarOpen: (open) => set({ mobileSidebarOpen: open }),
  toggleMobileSidebar: () => set((s) => ({ mobileSidebarOpen: !s.mobileSidebarOpen })),
}));
