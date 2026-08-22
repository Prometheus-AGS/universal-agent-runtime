# Artifact critic — approved cleanup-scope revision

Date: 2026-08-22

Verdict: BLOCK

The critic accepted the proposed resolutions for:

- immediate concurrent MCP shutdown initiation and a barrier-based ordering test;
- affirmative full-graph SQLx absence and Redis configuration-to-composition ownership evidence;
- explicit manifest-diff and added-public-item gates.

One blocker remains: post-exit SurrealDB reopening cannot prove release before
process exit. The corrected evidence must reopen the same path from a second
process while the original UAR process remains alive at a pre-exit barrier, or
perform an in-process reopen after all owners drop and before the normal
completion markers.

Read-only ownership tracing found that `LiveQueryBus` spawns untracked topic
tasks that retain cloned SurrealDB handles. Satisfying the blocker therefore
requires adding `src/uar/realtime/surreal_bus.rs` to the child write surface so
those tasks can be cancelled and joined before normal completion.

## Follow-up resolution

Verdict: PASS on the revised evidence design.

The existing process harness runs UAR in a dedicated server thread and Tokio
runtime. The accepted test waits for that thread/runtime to join, publishes a
`resources-released` barrier while keeping the helper process alive, and
requires a second UAR process to become ready on the identical SurrealKV path
before publishing `allow-exit`. This directly proves pre-exit release and makes
`src/uar/realtime/surreal_bus.rs` unnecessary for this child.

The critic also accepted the concurrent MCP/held-ingestion ordering test,
full-graph SQLx and Redis ownership evidence, and manifest/public-surface gates
as resolutions to the other findings.
