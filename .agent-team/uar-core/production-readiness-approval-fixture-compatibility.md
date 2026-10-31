# Approval fixture applicability handoff

Phase: `production-readiness-476-baseline`, Execute. Verification gate: **CLOSED** during the final fixture/API compatibility cohort. This is an implementation handoff, not acceptance evidence.

## Observed defect and source-grounded reproducer

The existing `.kbd-orchestrator/phases/production-readiness-476-baseline/evidence/full-inventory-attempt-002.txt` records `E0433` at `tests/support/approval_recovery_process.rs:1229` on candidate `9ed71e93cb2b3e130ff2c042a04b6ed3bcbd51ca`: `providers::memory::InMemoryProvider` is unavailable in the shipping profile. The recorded command was `cargo test --locked --no-default-features --features server-full`; compilation failed before test execution. It was not rerun for this handoff.

Source mechanism: `src/uar/persistence/providers/mod.rs` exposes `memory` only under `feature = "in-memory-backend"` or library `cfg(test)`. The integration fixture referenced it unconditionally in `deadline_profiles`. `server-full` does not enable `in-memory-backend`. The legacy qualifier already required explicit `postgres-backend` at entry, but lacked the corresponding memory-control applicability requirement.

## Exact owned changes

- `tests/support/approval_recovery_process.rs`: add an explicit `in-memory-backend` assertion at the start of `qualify()`, before the existing Postgres assertion, environment lookup, evidence-directory creation, scenarios, or child processes. Compile only the existing memory atomic-deadline block under that feature, including its acceptance-file write.
- `.agent-team/uar-core/production-readiness-approval-fixture-compatibility.md`: this handoff.

No production API, persistence provider, dependency feature, manifest, identity, approval, effect, resource binding, budget, cancellation, redaction, or authentication plumbing changed. No assertion or target was deleted or marked ignored. No unrelated addition was made.

## Coverage and applicability

With explicit `in-memory-backend`, the same `InMemoryProvider` still runs the unchanged `stale_deadline_intents` helper. It retains the unbound-denial control and both approval/denial stale-intent cases: intent predates expiry, a late transition cannot win, stored state becomes Expired with no decision, and a repeated transition cannot win. The memory receipt remains after those assertions. The local/shared/Postgres deadline branches and all other qualifier assertions are unchanged.

Without the feature, the memory block and its receipt are absent from compilation; invoking `qualify()` fails its first precondition rather than reporting a pass or starting external work. This does not alter the existing child-job dispatcher, which precedes `qualify()` in the integration entrypoint. The prerequisite is for the parent legacy qualifier, not a new child authentication or admission rule.

The uncomfortable limit is that compiling this fixture without the memory feature does not qualify its memory controls, and baseline compilation does not qualify installed recovery. The corrected phase plan compiles all targets with `--no-run` and runs the safe library inventory separately. It does not execute this historical Dot07 qualifier or revive cancelled historical gates. Selected actual installed recovery remains in #465.

## Evidence, deferred work, and decisions

Commands in this task were classified as source inspection (`pwd`, `cat`, `sed`, `rg`) or authorized implementation/documentation writing (`apply_patch`). Source reads established the fixture's original control flow, provider feature gate, manifest features, active waypoint, final cohort plan and existing diagnostic. No verification command or independent review gate ran. No Cargo, rustc, tests, builds, format, lint, diff check, external mutation or paid call ran.

The new applicability assertion traces directly to the observed feature-gated compile failure and the final cohort requirement to fail before side effects. The cfg boundary preserves the original backend and assertions without exposing the backend in the shipping production profile. No speculative guard was added.

Unverified: compilation and warnings in shipping/explicit-memory configurations; actual early failure when the feature is absent; runtime execution of retained controls. These remain unverified because the gate is closed. The lead owns the renewed whole-cohort inventory and later authorized, serialized acceptance/review. No unresolved implementation decision remains in these owned files; there is no claim that the entire phase is complete.
