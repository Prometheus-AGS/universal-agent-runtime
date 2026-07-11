import { useThreadRegistryStore } from "@/stores/thread-registry-store";

/** Narrow UI façade for thread navigation intent. */
export function useThreadRegistryActions() {
  const setActiveThread = useThreadRegistryStore((state) => state.setActive);
  return { setActiveThread };
}
