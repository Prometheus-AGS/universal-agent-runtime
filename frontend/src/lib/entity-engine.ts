/**
 * Entity-graph engine bootstrap.
 *
 * Import this module once (side-effectfully) at the top of `main.tsx`,
 * before React renders, to register process-wide defaults for the
 * `@prometheus-ags/prometheus-entity-management` runtime.
 *
 * All views that consume `useEntity` / `useEntityList` inherit these
 * settings; per-query overrides on the call site take precedence.
 */
import { configureEngine } from "@prometheus-ags/prometheus-entity-management";

configureEngine({
  defaultStaleTime: 30_000,
  defaultGcTime: 5 * 60_000,
  gcInterval: 60_000,
  maxRetries: 3,
  retryBaseDelay: 250,
  revalidateOnFocus: true,
  revalidateOnReconnect: true,
});
