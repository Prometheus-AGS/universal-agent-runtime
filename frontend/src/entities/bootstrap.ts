// frontend/src/entities/bootstrap.ts
import { configureEngine } from "@prometheus-ags/prometheus-entity-management";
import { registerAllSchemas } from "./schemas";
import { initSyncTransport } from "./sync";

let initialized = false;
let syncCleanup: (() => void) | null = null;

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

  // Initialize realtime sync transport (SSE/WebSocket/ElectricSQL)
  syncCleanup = await initSyncTransport();
}

/**
 * Cleanup sync transport. Call on app unmount if needed.
 */
export function teardownEntityGraph() {
  if (syncCleanup) {
    syncCleanup();
    syncCleanup = null;
  }
}
