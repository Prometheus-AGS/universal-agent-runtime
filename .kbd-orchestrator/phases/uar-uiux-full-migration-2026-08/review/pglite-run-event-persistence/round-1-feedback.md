# C-07 adversarial-review round 1 feedback

The first packet was incomplete because the review packet builder used a plain
Git diff, which omitted C-07's untracked new platform, test, OpenSpec, and
refinement files. The next packet makes those files visible through
intent-to-add without staging their content.

Re-evaluate the first terminal-path critical against
`frontend/src/platform/pglite/run-event-persistence.ts`: every adapted official
row is awaited before the chat-store switch, and `ingest()` flushes buffered
content, persists the terminal row, and updates run status for `RUN_FINISHED`
and `RUN_ERROR`. The separate `finish()` call is intentionally limited to
legacy `[DONE]`, stream exhaustion, cancellation, and transport-error paths
that have no official terminal row.

Changes to unrelated KBD projections, local settings, Rust SSE code, and prior
wave artifacts predate C-07 and are outside this review target. Report them only
if the C-07 implementation causes or depends on the defect.

The rejected-promise warning was valid. C-07 now disposes a runtime whose
hydration or sync startup fails, clears the cached attempt, and has a focused
retry regression test.
