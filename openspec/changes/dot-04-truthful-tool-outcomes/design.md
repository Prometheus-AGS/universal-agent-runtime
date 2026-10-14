# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. MCP transport Ok can classify isError=true as execution success.

## Goals / Non-Goals

Goal: Tool execution preserves application and uncertain outcomes. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

Propagate application failure and original content through registry, ToolEnd, host admission and durable connector receipts. Keep transport failure distinct from post-dispatch outcome unknown; only successful receipts advance success dependencies. Reuse existing effect classification/reconciliation and never broaden retry authority.

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

Version receipt outcome interpretation without rewriting historical evidence into invented certainty. Legacy ambiguous records remain non-replayable until reconciled; rollback must not allow a newly failed/unknown effect to become success.

Dependencies: dot-02-principal-credentials. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

Use real scoped and direct MCP dispatch with isError=true, successful calls, pre-dispatch transport failure and disconnect after an external effect. Assert every consumer agrees on outcome, content survives and unknown work is not replayed or marked successful.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.

