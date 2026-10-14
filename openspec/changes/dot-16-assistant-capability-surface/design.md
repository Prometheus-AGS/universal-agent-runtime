# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. Current HTTP/embedded method reachability is uneven and blanket parity claims would be misleading.

## Goals / Non-Goals

Goal: Assistant capability discovery matches reachable operations. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

Expose the six contracts through authenticated server/sidecar HTTP management and supported MCP/native assistant tools using the same host services. Publish per-operation capability descriptors for memory controls, schedule CRUD/pause/cancel, follow-up policy/inbox, tool outcomes, approval reads/decisions and checkpoint recovery. Postgres/in-memory/unattested remote and direct embedded Runtime must explicitly identify unsupported operations; no silent fallback. Provide API examples for consumers, not a new assistant UI.

Keep all mutations in trusted host services, retain complete verified owner scope, and expose durable business state through explicit inspectable records. Use fallible typed persistence outcomes; do not mask an unsupported backend or outage with success. No kernel mutation authority, no automatic replay of unknown effects, and no new lease expiry replacing exclusion.

Alternatives: existing UAR services are preferred over an independent queue/workflow authority because admission, receipt and owner transactions must agree. Apalis/Temporal remain analysis references, not dependencies. Rebuilding existing approvals/observer/credential machinery is excluded.



## Candidate evidence

### cand-001: Existing UAR observers, host admission, approvals and team catalog — adapt

Extend current contracts with stable scheduled occurrences and a delivery outbox; avoid a second authority or task-state system.

- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/runtime/observer/delivery.rs) — Durable subscription/inbox recovery and bounded occurrence delivery already exist.
- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/compiler/collaboration/team_execution/ownership.rs) — Catalog claim/reclaim preserves epochs, queued transfers and uncertain running work.

Risks: Observer polling is not a user recurrence contract. Existing owner identity includes tenant/subject but needs issuer policy reconciliation. Kernel execution remains behind trusted host admission.

### cand-005: Existing UAR MemoryService and vendored surreal-memory 0.1.0 — adapt

Preserve the existing memory engine and extend host scoping, consent, deletion and source provenance. A new memory database would not repair the identity boundary.

- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/Cargo.toml) — Memory is a path dependency under vendor/git/surreal-memory-server; Cargo.lock reports 0.1.0 without registry source.
- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/memory/service.rs) — Service exposes retrieval/capture/mutation helpers but user scoping passes subject IDs without the complete tenant/issuer identity.

Risks: Vendor tree is committed source, not the historical revision named in versions.toml. Every memory surface, including native/MCP, legacy HTTP and embedded SDK, needs current owner enforcement. Workflow mirror persistence is not assistant memory privacy certification.

## Risks / Trade-offs

The uncomfortable thing: a persisted row or green isolated test does not prove cross-process authority and restart correctness. Validate the actual production boundary and retain negative controls. New retries/guards must trace to the reported failure or real owner/effect/consent boundary; do not harden unrelated code.

No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

## Migration Plan

Advertise only proven per-operation capability revisions. Existing clients keep their current APIs; new methods use documented unsupported responses where unavailable. Version discovery additively; do not claim direct embedded Runtime parity.

Dependencies: dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract, dot-06-fresh-process-resume, dot-07-approval-recovery, dot-08-durable-ingestion, dot-10-durable-budget-admission, dot-11-supervisor-fencing, dot-13-memory-controls, dot-14-durable-schedules, dot-15-proactive-followups. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

Generate an operation-by-profile inventory from actual registrations and exercise every advertised operation through real entry points. Existing HTTP Rust client can wrap APIs; do not certify absent direct embedded APIs. Assert unsupported requests fail explicitly and shared/local owners receive identical contract semantics.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.

