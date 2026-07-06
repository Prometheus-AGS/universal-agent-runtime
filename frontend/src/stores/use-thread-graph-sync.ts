/**
 * SSE-driven sync between the entity graph's `Thread` slice and the
 * client-first `thread-registry-store`.
 *
 * Threads are authoritative client-side (created in the SPA, persisted to
 * PGlite). The server writes the corresponding SurrealDB `sessions` row
 * when a chat session is persisted, and the realtime bus broadcasts those
 * events on the `threads` topic. This hook closes the loop:
 *
 *   - New server row → registry creates a persisted entry (isEphemeral=false)
 *   - Server title/updated_at change → registry merges fields, preserves id
 *   - Server delete → registry removes the entry + PGlite row
 *
 * Local-only ephemeral threads (no server row yet) are NEVER touched by
 * this hook — diff-by-keyset only fires on actual graph mutations.
 *
 * Mount once at the SPA root (`App.tsx`). No render-time return value.
 */
import { useEffect, useRef } from "react";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { useThreadRegistryStore } from "@/stores/thread-registry-store";

interface ServerThreadRow {
  title?: string;
  created_at?: string;
  updated_at?: string;
}

export function useThreadGraphSync(): void {
  // Track the prior id keyset so we can compute new/removed deltas.
  const prevIdsRef = useRef<Set<string>>(new Set());

  useEffect(() => {
    // Initial snapshot — capture without firing reconciliation; the local
    // PGlite-backed registry is the source of truth on first mount.
    const initial = useGraphStore.getState().entities["Thread"];
    prevIdsRef.current = new Set(initial ? Object.keys(initial) : []);

    const unsubscribe = useGraphStore.subscribe((state) => {
      const slice = state.entities["Thread"] as
        | Record<string, ServerThreadRow>
        | undefined;
      const nextIds = new Set(slice ? Object.keys(slice) : []);
      const prevIds = prevIdsRef.current;

      // Removed: was present, now gone.
      for (const id of prevIds) {
        if (!nextIds.has(id)) {
          useThreadRegistryStore.getState().removeThread(id);
        }
      }

      // Added or updated: present in next.
      if (slice) {
        const registry = useThreadRegistryStore.getState();
        for (const id of nextIds) {
          const row = slice[id];
          const local = registry.threads[id];
          if (!local) {
            // New server thread — register, then mark persisted.
            registry.registerThread(id);
            if (row.title) registry.setTitle(id, row.title);
            registry.markPersisted(id);
          } else {
            // Known thread — apply server fields without touching the
            // local lifecycle flags except markPersisted on transition.
            if (row.title && row.title !== local.title) {
              registry.setTitle(id, row.title);
            }
            if (local.isEphemeral) {
              registry.markPersisted(id);
            } else if (row.updated_at && row.updated_at !== local.updatedAt) {
              registry.touch(id);
            }
          }
        }
      }

      prevIdsRef.current = nextIds;
    });

    return unsubscribe;
  }, []);
}
