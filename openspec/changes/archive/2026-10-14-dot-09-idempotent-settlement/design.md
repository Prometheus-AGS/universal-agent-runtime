# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. Detached ledger writes can outlive terminal completion and have no idempotency identity.

## Goals / Non-Goals

Goal: Usage settlement has stable durable identity. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

Replace detached acceptance with durable idempotent settlement keyed by stable run/call/attempt and scope identity. Preserve per-call/team charging without double counting. Separate execution outcome from accounting state; a persistence outage leaves durable reconciliation work or visible unresolved accounting, never an invented settlement.

Keep all mutations in trusted host services, retain complete verified owner scope, and expose durable business state through explicit inspectable records. Use fallible typed persistence outcomes; do not mask an unsupported backend or outage with success. No kernel mutation authority, no automatic replay of unknown effects, and no new lease expiry replacing exclusion.

Alternatives: existing UAR services are preferred over an independent queue/workflow authority because admission, receipt and owner transactions must agree. Apalis/Temporal remain analysis references, not dependencies. Rebuilding existing approvals/observer/credential machinery is excluded.



## Candidate evidence

### cand-006: Existing UAR persistence adapters with transactional outbox and settlement records — adapt

Keep data and admission state in current stores. Build narrow domain transactions and uniqueness/CAS contracts rather than introduce a generic job framework as a correctness substitute.

- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/persistence/mod.rs) — Existing persistence boundary includes durable capabilities and also checkpoint/cost defaults requiring explicit unsupported behavior.
- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/persistence/providers/postgres.rs) — Postgres cost recording currently appends scope/amount rows without a run/call settlement key.

Risks: Atomicity must be implemented and tested per supported backend, not assumed from a trait. Source-blob/outbox coordination requires a durable content-reference protocol if bytes are not in the same transaction. Shared-host consistency and single-device restart must be separately proven.

## Risks / Trade-offs

The uncomfortable thing: a persisted row or green isolated test does not prove cross-process authority and restart correctness. Validate the actual production boundary and retain negative controls. New retries/guards must trace to the reported failure or real owner/effect/consent boundary; do not harden unrelated code.

No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

## Migration Plan

Add stable identities without fabricating identities for legacy append-only history. Preserve old reporting rows, isolate new enforcement epochs and require a proven opening balance or explicit operator reconciliation before enforcing old scope budgets. Replays never append a second charge.

Dependencies: dot-02-principal-credentials, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

Replay settlements and crash between provider usage, persistence and terminal publication; assert one charge per logical scope settlement on both database providers. Verify existing legacy rows remain history and are not ambiguously deduplicated or fabricated into current enforceable spend.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.

