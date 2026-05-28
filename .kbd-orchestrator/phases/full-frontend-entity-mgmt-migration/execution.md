# Execution — `full-frontend-entity-mgmt-migration`

**Date started:** 2026-05-27
**Tool:** claude-code (`/kbd-execute`)
**Backend:** OpenSpec
**Strategy:** sequential. Changes 1–3 are foundational and execute in this session. Changes 4–7 are page-by-page migrations (large surface) executed with pauses between cross-cutting PRs so we can verify cross-view propagation in the browser. Change 8 closes the phase.

## QA gate

`artifact-refiner` is not installed; per-change validation is inline (compile + smoke).

## Change cursor

- **Next:** `bootstrap-entity-engine-and-realtime`
