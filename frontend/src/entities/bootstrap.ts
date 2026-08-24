// frontend/src/entities/bootstrap.ts
import {
  configureEngine,
  createPGlitePersistenceAdapter,
  registerSessionConfigurationEntities,
  startLocalFirstGraph,
} from "@/platform/entities";
import type {
  GraphPersistenceAdapter,
  LocalFirstGraphRuntime,
  PGlitePersistenceClient,
} from "@/platform/entities";
import { registerAllSchemas } from "./schemas";
import { initSyncTransport } from "./sync";

let initialized = false;
let durableBootstrap: Promise<void> | null = null;
let localFirstRuntime: LocalFirstGraphRuntime | null = null;
let syncCleanup: (() => void) | null = null;

export interface DurableEntityGraphDependencies {
  createStorage: (db: PGlitePersistenceClient) => Promise<GraphPersistenceAdapter>;
  startLocalFirst: (storage: GraphPersistenceAdapter) => LocalFirstGraphRuntime;
  startSync: () => Promise<() => void>;
}

const durableDependencies: DurableEntityGraphDependencies = {
  createStorage: (db) => createPGlitePersistenceAdapter(db),
  startLocalFirst: (storage) => startLocalFirstGraph({
    storage,
    key: "uar:entity-graph",
    replayPendingActions: true,
    retryPolicy: {
      maxAttempts: 5,
      initialDelayMs: 500,
      maxDelayMs: 30_000,
      backoffFactor: 2,
      jitter: "equal",
      poisonHandler: (action, error) => {
        console.error("[entity-graph] pending action exhausted", {
          id: action.id,
          key: action.key,
          errorType: error instanceof Error ? error.name : typeof error,
        });
      },
    },
  }),
  startSync: initSyncTransport,
};

/**
 * Bootstrap the entity graph engine.
 * Call once at app init, before React renders entity-backed components.
 */
export async function bootstrapEntityGraph() {
  if (initialized) return;
  initialized = true;

  // Configure the engine with cache and retry settings
  configureEngine({
    defaultStaleTime: 30_000,
    maxRetries: 2,
    revalidateOnFocus: true,
    revalidateOnReconnect: true,
  });

  // Register all entity schemas and relations
  registerAllSchemas();
  registerSessionConfigurationEntities();
}

export async function initializeDurableEntityGraph(
  db: PGlitePersistenceClient,
  dependencies: DurableEntityGraphDependencies,
): Promise<{ runtime: LocalFirstGraphRuntime; cleanup: () => void }> {
  const storage = await dependencies.createStorage(db);
  const runtime = dependencies.startLocalFirst(storage);
  try {
    await runtime.ready;
    const cleanup = await dependencies.startSync();
    return { runtime, cleanup };
  } catch (error) {
    runtime.dispose();
    throw error;
  }
}

/**
 * Hydrate the durable graph once, then attach realtime synchronization.
 * A failed attempt is cleared so a later database initialization can retry.
 */
export function bootstrapDurableEntityGraph(
  db: PGlitePersistenceClient,
  dependencies: DurableEntityGraphDependencies = durableDependencies,
): Promise<void> {
  durableBootstrap ??= initializeDurableEntityGraph(db, dependencies)
    .then(({ runtime, cleanup }) => {
      localFirstRuntime = runtime;
      syncCleanup = cleanup;
    })
    .catch((error: unknown) => {
      durableBootstrap = null;
      throw error;
    });
  return durableBootstrap;
}

/**
 * Cleanup sync transport. Call on app unmount if needed.
 */
export function teardownEntityGraph() {
  localFirstRuntime?.dispose();
  localFirstRuntime = null;
  if (syncCleanup) {
    syncCleanup();
    syncCleanup = null;
  }
  durableBootstrap = null;
}
