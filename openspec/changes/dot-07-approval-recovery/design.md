# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. Persisted approval rows do not alone reconstruct a safe executable continuation.

## Goals / Non-Goals

Goal: Recovered approvals require current executable authority. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

Connect existing durable approval records to restored run/checkpoint disposition. Expose current pending/expired/invalidated decisions through existing approval reads/events; claim approved continuation once under current owner, binding and policy. Preserve timeout/denial semantics and invalidate stale approval rather than replaying a captured future.

Keep all mutations in trusted host services, retain complete verified owner scope, and expose durable business state through explicit inspectable records. Use fallible typed persistence outcomes; do not mask an unsupported backend or outage with success. No kernel mutation authority, no automatic replay of unknown effects, and no new lease expiry replacing exclusion.

Alternatives: existing UAR services are preferred over an independent queue/workflow authority because admission, receipt and owner transactions must agree. Apalis/Temporal remain analysis references, not dependencies. Rebuilding existing approvals/observer/credential machinery is excluded.



## Candidate evidence

### cand-001: Existing UAR observers, host admission, approvals and team catalog — adapt

Extend current contracts with stable scheduled occurrences and a delivery outbox; avoid a second authority or task-state system.

- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/runtime/observer/delivery.rs) — Durable subscription/inbox recovery and bounded occurrence delivery already exist.
- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/compiler/collaboration/team_execution/ownership.rs) — Catalog claim/reclaim preserves epochs, queued transfers and uncertain running work.

Risks: Observer polling is not a user recurrence contract. Existing owner identity includes tenant/subject but needs issuer policy reconciliation. Kernel execution remains behind trusted host admission.

## Risks / Trade-offs

The uncomfortable thing: a persisted row or green isolated test does not prove cross-process authority and restart correctness. Validate the actual production boundary and retain negative controls. New retries/guards must trace to the reported failure or real owner/effect/consent boundary; do not harden unrelated code.

No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

## Migration Plan

Preserve existing pending/denied/expired decisions and current timeout policies. Link only to verified resume envelopes; stale/unmappable records become visibly invalidated. Rollback cannot restore consumed approval authority.

Dependencies: dot-06-fresh-process-resume. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

Restart while pending, read it through the shipped interface and decide on a second runtime. Race duplicate decisions, expiration, revoked authority and changed bindings; assert at most one approved effect and no effect for invalidated/denied decisions.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.

The complete approval acceptance command is:

```bash
cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test approval_recovery -- --nocapture --test-threads=1
```

`in-memory-backend` is explicit because the production memory deadline fixture
imports a library module that integration-test compilation does not expose
through internal `cfg(test)`. The observed command without that feature failed
with E0433 before scenarios. Preserve that failed attempt. The additive feature
belongs to acceptance; do not expose memory in the release profile or remove the
deadline negative to make qualification compile.

The fixture's production binary inherits the additive acceptance features.
After the whole approval target in the same serialized failed-gate rerun,
produce the exact customer-profile artifact separately:

```bash
cargo build --locked --no-default-features --features server-full,postgres-backend --bin universal-agent-runtime
```

Record both commands and feature sets, retain separate artifact hashes, and
preserve the exact-profile output before another profile replaces it. The
customer build establishes compilation for that profile; the additive fixture
does not certify the runtime behavior of an exact-profile customer binary.

