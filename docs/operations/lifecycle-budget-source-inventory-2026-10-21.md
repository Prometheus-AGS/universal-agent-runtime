# Approved budgeted shutdown implementation inventory

Execute phase: `uar-outstanding-issues-v1-1-1::function-first-release-certification`.
Waypoint3196, plan86. Canonical task9.8 and runtime/state/lead source handoffs
are complete. The aggregate138/153 counter and pending release/acceptance tasks
are unchanged; this inventory cannot override their incomplete state.

The operator explicitly approved budgeted Pending reporting with retained
cleanup. It did not authorize tests, builds, other verification or new gates.

| Source | Implemented contract | Evidence |
| --- | --- | --- |
| `src/uar/runtime/lifecycle.rs` | Additive typed Pending/Completed observation on supervisor and handle; caller budget wraps only observation; one UUID-backed retained owner starts before registry-lock await; current unreleased inventory or explicit SnapshotPending; retained completion preference | Runtime handoff; saved source only, uncompiled/unexecuted |
| `src/embedded.rs` | Additive budgeted wrapper requests existing embedded root cancellation before the same supervisor operation; unbounded shutdown preserved | Lead handoff and host-integration notes; source only |
| Runtime resource lifecycle delta, task ledger, architecture and operator docs | Budget expiry never means release/rollback; failed/unresolved obligations retained; no sidecar protocol or default duration added | Authored source/docs, canonical supported task transitions |

All files have one released writer. The source integration corrected a reused
identity and a nonexistent helper call before completion; no compiler probe was
used. Existing UUID1.21.0 v4, Tokio/Tokio-util and all dependency pins remain.
No test, dependency manifest, fixture, wire format, launcher or workflow changed.
Existing unrelated edits, including the seed-tool worktree changes, are preserved.

Handoffs: `lifecycle-budget-runtime-handoff.md`,
`lifecycle-budget-lead-handoff.md`, and
`lifecycle-budget-host-integration-notes.md`. Operator API usage and limits are
in `lifecycle-budgeted-shutdown.md`; whole-plan status is in
`lifecycle-plan-implementation-evidence-checklist.md`.

The companion JSON records a new aggregate saved-source identity, canonical
ledgers and handoffs. Its compact representation references the previous full
saved-file inventory and records changed/added/removed file records. Applying
those changes reconstructs the complete current record set without repeating
the earlier catalogue. Hashes describe saved source; no acceptance or
source-identity checker is executed. The inventory excludes itself.

The uncomfortable limit remains explicit: bounded observation does not guarantee
finite termination. Pending requires the host executor to stay alive; a silent
peer or uninterruptible operation can remain owned indefinitely. Completed may
contain failures or unreleased resources. Physical release, timer/acknowledgement
races, repeated-caller identity and registry-lock cancellation remain unverified.

The earlier embedding pass and image32cb3bcf predate this source. No new test
or gate is added or run, and no previous pass is relabeled as current evidence.
The Boss pin and image remain unchanged. Native payload binding, same-image
preparation, full frozen embedding evidence, independent review, acceptance and
reflection remain pending under the closed gate.
