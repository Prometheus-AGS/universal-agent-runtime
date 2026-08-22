# Artifact judge — Surreal lifecycle scope expansion

Date: 2026-08-22

Verdict: PASS

The negative control proves real lingering ownership in the live-query
supervisors and ingestion watcher. Existing `src/server.rs` scope covers
watcher ownership, outer lifetime/drop ordering, lock observation, and deadline
coordination. Adding only `src/uar/realtime/surreal_bus.rs` is sufficient for a
crate-private cancellation and task-join operation.

Lock polling remains inside the already-armed process deadline. The manifest
and public-visibility gates prevent dependency or API expansion. The requested
single-path expansion is justified and minimal.
