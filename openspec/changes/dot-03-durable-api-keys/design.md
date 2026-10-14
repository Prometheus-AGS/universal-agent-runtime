# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. Server startup selects in-memory API-key storage.

## Goals / Non-Goals

Goal: API-key authority survives restart and revocation is current. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

Wire durable production key storage for both existing database providers, retaining hashed secrets, authority metadata, expiry and owner identity. Read authoritative revocation/expiry before admission; storage outage cannot fall back to memory or stale allow decisions. Preserve explicit development mode.

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

Existing in-memory keys cannot be assumed recoverable after restart. Provide an explicit authenticated in-process migration/import or reissue path without storing plaintext; persist revocation before rollout completes. Rollback must not revive revoked keys.

Dependencies: dot-02-principal-credentials. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

Create a key through management, restart and authenticate, revoke from runtime A and deny new admission on B, then restart both and deny again. Cover expiry, database outage and unauthorized management; do not expose key hashes or plaintext in receipts.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.

