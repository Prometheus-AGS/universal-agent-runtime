ASSESSMENT: handle-kbd-uninitialized-runtime
Project: universal-agent-runtime
Date: 2026-08-25
Codebase baseline: origin/main at 05e0c61f pins prometheus-skill-system f1e58b25; issue #265 remains open and its uninitialized-runtime path is still present in the pinned CLI.
Cross-tool progress: none

IMPLEMENTATION STATUS
- Registered-project discovery: [DONE] — `ProjectRegistry::register_existing` records a replica with an immutable project identity, but intentionally does not create the first runtime event.
- First typed mutation: [MISSING] — `state_or_replay` falls back from the unreachable TCP control plane to `runtime.replay()` and propagates `RuntimeError::NotInitialized`; it does not call the existing legacy-aware `ensure_runtime` initializer.
- Initialization primitive: [PARTIAL] — `ensure_runtime` already initializes from the legacy waypoint and preserves phase, lifecycle, plan revision, and exact-next-work, but only `kbd migrate --apply` calls it.
- Status remediation: [MISSING] — an uninitialized runtime still prints `KBD mode: legacy (run prometheus kbd migrate --apply)` even when migration inventory reports no required journal migration.
- Failure exit semantics: [PARTIAL] — the Rust entry point returns `anyhow::Result`, which normally maps an error to a non-zero process status, but no CLI regression test covers the issue's observed zero exit status.
- Unix-socket transport: [DEFERRED] — the local canonical-runtime fallback is the existing correctness path; issue #265 explicitly marks Unix-socket transport as optional and lower priority.

CROSS-TOOL PROGRESS
- NONE — the new phase contains no registered changes or tasks.

SPEC GAP SUMMARY
- The upstream OpenSpec baseline has no KBD runtime-initialization capability defining the boundary between project registration and the first signed run event.
- No requirement states that the first typed mutation self-initializes from compatible legacy projections, preserves recorded work, or reports a non-zero failure when initialization cannot complete.
- No requirement prevents `kbd status` from recommending a destructive or irrelevant migration command for an empty registered runtime.

BUILD HEALTH
- build check: [UNKNOWN] — no build was run during Assess.
- known violations: issue #265 remains reproducible by source inspection because `state_or_replay` propagates `NotInitialized` and `status` still recommends `migrate --apply`.
- test coverage: [PARTIAL] — runtime initialization and migration safety tests exist, but there is no CLI test for a registered, uninitialized runtime's first typed mutation, status guidance, or failure exit code.

CONSTRAINT CHECK
- AGENTS.md violations: NONE introduced during assessment. The upstream submodule checkout remains unedited.
- constraints.md violations: N/A — no additional phase constraint file exists.

GOAL PROGRESS
- Resolve GitHub issue 265 with evidence: [NOT MET] — the issue remains open and the reported first-mutation failure path remains in the pinned source.
- Preserve registered project history and typed command correctness: [PARTIAL] — `ensure_runtime` provides a preservation-aware initializer, but mutations do not invoke it.
- Close the issue only after the reported failure modes are handled or proven obsolete: [NOT MET] — the primary failure is not obsolete; closure would be premature.

UNCOMFORTABLE FINDING
- The existing `migrate --apply` hint directs operators toward the one path whose comments document prior loss of projection-only work, even though the missing action is initialization rather than migration.

ASSESSMENT COMPLETE
