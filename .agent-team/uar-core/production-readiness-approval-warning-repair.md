# Approval shared-fixture warning repair

Phase: `production-readiness-476-baseline`, Execute, observed zero-warning repair cohort, implementation item 12. Verification gate: **CLOSED**. This handoff records implementation only.

## Observed failure

The existing `target/production-readiness-476/approval-dev-compile-003.log` records compilation of `approval_recovery` with `server-full,postgres-backend,in-memory-backend` and `--no-run`. It finished compilation but reported 17 `dead_code` warnings from `tests/support/fresh_process_resume_process.rs`, plus one separate sidecar deprecation warning owned by another writer. Under the repository zero-warning rule, that is a failed gate despite compilation completing. No test body executed in that recorded compile.

The 17 warnings concern `Peer::start`/`saw`, completion/source/envelope/proof helpers, readback types and operations, cold-run inspection, wait/corruption/scenario/unsupported helpers, and the fresh-process qualifier. Source inspection shows the approval fixture includes that file as `mod resume`, reuses its process/database/identity/file helpers, and supplies its own completion, readback and qualification scenarios.

## Exact owned changes

- `tests/support/approval_recovery_process.rs`: add `#[expect(dead_code, reason = "Approval recovery reuses process helpers but supplies its own qualifier and readback scenarios")]` to this target's existing `mod resume` inclusion, immediately after its path attribute.
- `.agent-team/uar-core/production-readiness-approval-warning-repair.md`: this handoff.

The shared module implementation is unchanged. The expectation applies only to this inclusion and only to `dead_code`; there is no crate-wide expectation, blanket `allow`, lowered lint level or compiler flag. No target or assertion is removed or ignored. The original feature-gated memory backend/control/receipt and fail-fast memory-feature precondition are preserved. No production interface, identity, approval, effect, resource, budget, authentication or cancellation behavior changes.

The uncomfortable limit is that a module-level expectation also covers future unused items in this inclusion; its scope is intentional because this target consumes a subset of a shared fixture. It does not establish runtime recovery correctness or qualify the historical Dot07 scenarios.

## Evidence and remaining limits

This task used only source inspection (`cat`, `tail`, `sed`, `rg`) and authorized implementation/documentation writing (`apply_patch`). One bounded search returned no matches; no compiler or verification process was started. No tests, Cargo, rustc, format/lint/check/build, diff check, independent review gate, external mutation or paid call ran.

The only new annotation traces to the recorded 17 warnings and amended plan item 12. No unrelated addition or speculative guard was made. No unresolved implementation decision remains in these owned files.

Unverified: whether the affected explicit-feature compile now finishes with zero warnings and whether an independent reviewer accepts the annotation scope. The gate remains CLOSED pending both cohort handoffs, the lead's renewed complete implementation inventory, and the applicable authorization. The lead owns later serialized acceptance. The separate sidecar warning and all prior unrelated full-target/full-library failures are outside this writer's ownership; this handoff does not claim they are fixed.
