import { useGraphEntities } from "@/entities/hooks/use-graph-entities";
import type { UarCompilerSession } from "@/types";

/** Live list of CompilerSession entities from the graph. */
export function useCompilerSessions(): { items: UarCompilerSession[] } {
  return { items: useGraphEntities<UarCompilerSession>("CompilerSession") };
}
