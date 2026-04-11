import { create } from "zustand";

import { fetchHealthz, type HealthzData } from "@/services/health-api";

interface HealthState {
  health: HealthzData | null;
}

interface HealthActions {
  load: () => Promise<void>;
}

export const useHealthStore = create<HealthState & HealthActions>((set) => ({
  health: null,

  load: async () => {
    try {
      const health = await fetchHealthz();
      set({ health });
    } catch {
      set({ health: null });
    }
  },
}));
