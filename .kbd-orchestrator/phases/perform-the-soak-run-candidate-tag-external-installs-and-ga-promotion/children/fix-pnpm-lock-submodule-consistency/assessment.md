ASSESSMENT: fix-pnpm-lock-submodule-consistency
Project: universal-agent-runtime
Date: 2026-08-20
Codebase baseline: source commit `fa4ffb96` pins entity-management at `0352c83`, but its committed root pnpm lock predates that manifest.
Cross-tool progress: the preceding embedded-SSE child is complete; no other tool owns this child.

IMPLEMENTATION STATUS
- committed root lock: MISSING — a clean worktree running `pnpm install --frozen-lockfile` exits 1 with `ERR_PNPM_OUTDATED_LOCKFILE`, 17 added dependencies, and 12 mismatched specifiers under the pinned entity-management importer.
- operator lock candidate: SUPERSEDED — SHA-256 `fab6bb643301f98e5eed826960a3c3059d09fbf164a8d93fec7731a34162e64b` passed metadata-only frozen validation but moved two pre-existing edges. The corrected candidate is `645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350`.
- clean regeneration comparison: DONE — two disposable-worktree regenerations from the committed lock both produced SHA-256 `8706080edcdbdd35c39f867a5af648aacb0ce484348e847be37a681f1b205af3`. Direct HEAD audit identified and restored the two noncausal edges; regeneration still moves `lucide-react` 1.32.0 to 1.33.0 and collapses preserved `y-webrtc` onto the new direct `ws` pin.
- parent certification: BLOCKED — no browser preparation or browser suite ran after the frozen-lock failure.

CROSS-TOOL PROGRESS
- fix-embedded-sse-offline-reconnect: DONE (Codex) — archived, committed, and approved by the corrected history-free critic and judge.
- screen-by-screen-validation: IN PROGRESS (Codex) — source assertions are committed; immutable certification waits on this lock repair.

SPEC GAP SUMMARY
- No canonical requirement currently states that the root workspace lock must remain frozen-installable after advancing a workspace submodule manifest.
- Fresh non-frozen resolution would silently move allowed transitive ranges. The minimum repair is to retain the already exercised operator candidate and make its frozen consistency the acceptance boundary, not to accept unrelated latest-range movement.

BUILD HEALTH
- build check: FAIL — `pnpm install --frozen-lockfile` fails against the committed lock before any build starts.
- candidate check: PASS — metadata-only and clean full frozen installs exit 0, the supply-chain policy validates 1,482 entries, 1,345 packages install from empty dependency directories, and candidate SHA-256 remains unchanged.
- known violations: the lock candidate is operator-owned and uncommitted; the child must explicitly adopt and scope it before staging.
- test coverage: PARTIAL — dependency resolution is directly covered; product and browser behavior remain parent work.

CONSTRAINT CHECK
- AGENTS.md violations: NONE in the candidate; certification would violate evidence rules if run against the stale committed lock or a non-frozen regenerated graph.
- constraints.md violations: N/A — no repository constraints file is present.

GOAL PROGRESS
- make the committed root pnpm lock match the pinned entity-management manifest: NOT MET — the matching candidate is uncommitted.
- prove frozen installation leaves the lock unchanged: MET for the candidate only — the observed pre/post SHA-256 is identical.
- return control to screen-by-screen-validation for clean certification: NOT MET — child verification, review, and commit remain.

ASSESSMENT COMPLETE
