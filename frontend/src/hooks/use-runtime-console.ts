import { useRuntimeConsoleStore } from "@/stores/runtime-console-store";

/** Narrow Runtime Console action façade for UI components. */
export function useRuntimeConsoleActions() {
  const refresh = useRuntimeConsoleStore((state) => state.refresh);
  const resolveApproval = useRuntimeConsoleStore((state) => state.resolveApproval);
  return { refresh, resolveApproval };
}
