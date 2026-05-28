/**
 * Optimistic mutation helpers for the entity graph.
 *
 * These wrap the snapshot → optimistic mutation → rollback-on-failure pattern
 * around `useGraphStore` so callers don't repeat it. On rejection the helpers
 * restore the captured snapshot and re-throw; callers handle UI error state.
 *
 * Contract pinned by `__tests__/optimistic-rollback.test.tsx`.
 */
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";

/**
 * Optimistically merge `patch` into the entity at (`type`, `id`), then run
 * `serverCall`. If the server call rejects, the prior snapshot is restored
 * and the error is re-thrown.
 */
export async function optimisticUpsert<T extends Record<string, unknown>>(
  type: string,
  id: string,
  patch: Partial<T>,
  serverCall: () => Promise<void>,
): Promise<void> {
  const graph = useGraphStore.getState();
  const snapshot = graph.entities[type]?.[id] as
    | Record<string, unknown>
    | undefined;
  if (snapshot) {
    graph.upsertEntity(type, id, { ...snapshot, ...patch });
  }
  try {
    await serverCall();
  } catch (err) {
    if (snapshot) {
      useGraphStore.getState().upsertEntity(type, id, snapshot);
    }
    throw err;
  }
}

/**
 * Optimistically remove the entity at (`type`, `id`), then run `serverCall`.
 * If the server call rejects, the prior snapshot is re-upserted and the
 * error is re-thrown.
 */
export async function optimisticRemove(
  type: string,
  id: string,
  serverCall: () => Promise<void>,
): Promise<void> {
  const graph = useGraphStore.getState();
  const snapshot = graph.entities[type]?.[id] as
    | Record<string, unknown>
    | undefined;
  graph.removeEntity(type, id);
  try {
    await serverCall();
  } catch (err) {
    if (snapshot) {
      useGraphStore.getState().upsertEntity(type, id, snapshot);
    }
    throw err;
  }
}
