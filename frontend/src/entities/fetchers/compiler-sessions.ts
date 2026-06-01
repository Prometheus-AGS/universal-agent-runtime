import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { fetchCompilerSessions } from "@/services/compiler-api";
import type { UarCompilerSession } from "@/types";

/**
 * Fetch all compiler sessions and upsert into the entity graph.
 * Backend returns either `{ sessions: [...] }`, `{ data: { sessions: [...] } }`,
 * or a bare array; we normalise here.
 */
export async function loadCompilerSessionsIntoGraph(): Promise<void> {
  const data = await fetchCompilerSessions<
    | { sessions?: UarCompilerSession[]; data?: { sessions?: UarCompilerSession[] } }
    | UarCompilerSession[]
  >();
  const list: UarCompilerSession[] = Array.isArray(data)
    ? data
    : data.data?.sessions ?? (data as { sessions?: UarCompilerSession[] }).sessions ?? [];

  const { upsertEntity } = useGraphStore.getState();
  for (const s of list) {
    upsertEntity("CompilerSession", s.id, s as unknown as Record<string, unknown>);
  }
}
