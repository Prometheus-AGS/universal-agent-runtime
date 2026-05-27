import { useEntityList } from "@prometheus-ags/prometheus-entity-management";
import type { UarCompilerSession } from "@/types";

/** Live list of CompilerSession entities from the graph. */
export function useCompilerSessions() {
  return useEntityList<UarCompilerSession>("CompilerSession");
}
