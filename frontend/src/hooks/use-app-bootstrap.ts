import { useThreadHydration } from "@/stores/thread-registry-store";
import { useThreadGraphSync } from "@/stores/use-thread-graph-sync";

/** Initialize durable and realtime thread state for the application shell. */
export function useAppBootstrap() {
  useThreadHydration();
  useThreadGraphSync();
}
