# Caller-budgeted shutdown host integration

Execute phase: `function-first-release-certification`, task9.8. The operator
approved budgeted Pending reporting on2026-10-21. Verification is CLOSED; no
test, compiler, formatter, lint, build, review gate or packaging command ran.

Changed production source: `src/embedded.rs` adds
`EmbeddedRuntime::shutdown_with_budget(Duration) -> ShutdownObservation`.
It first requests the existing embedded run-root cancellation, then uses the
supervisor's retained shutdown observation. This preserves the distinction
between `EmbeddedRuntime::shutdown`, which owns the root token, and a bare
`RuntimeHandle`, which owns only the supervisor capability.

The wrapper does not consume or drop the runtime, replace cleanup ownership,
change engine authority, close prerequisites on expiry or report Pending as
termination. Existing unbounded shutdown remains available for eventual
acknowledgement. Repeated observers share the supervisor's shutdown identity
and finalizer. No implicit budget or force-stop policy is introduced.

The runtime writer released `src/uar/runtime/lifecycle.rs` and the shared
observation types, using the agreed `ShutdownObservation` return type. Source
integration corrected a constant-derived identity and a missing helper call;
the final source uses the existing UUID v4 dependency and actual closure
primitives, clones capabilities under a short mutex then reads snapshots outside it, and
rechecks completion before returning Pending. These are source observations,
not compiler or execution results. The lead owns this wrapper, OpenSpec/canonical integration and final
documentation. The read-only host integration notes are
`lifecycle-budget-host-integration-notes.md`. The private sidecar protocol,
launcher, process exit and normal server cleanup continue to await actual
completion; this additive Rust host API does not change them.

The uncomfortable limit is executor lifetime. A caller receiving Pending must
keep the Tokio host running so retained cleanup can finish. Process termination,
executor destruction or uninterruptible native work cannot supply graceful
completion evidence. Compilation, timer/acknowledgement races and host wrapper
behavior remain unverified. The earlier embedding pass predates this change and
does not certify it. No new test or verification gate is added.
