# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. Resume requires a live run before loading its checkpoint; unsupported methods can succeed as no-ops.

## Goals / Non-Goals

Goal: Checkpoints restore authorized execution in a fresh process. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

Persist a complete owner-bound resume envelope atomically with the checkpoint and load it before live-manager lookup. Reconstruct through host admission using protected history/current bindings and explicit host secret/resource reattachment. Replace persistence no-op defaults with unsupported errors; never deserialize runnable authority from caller input. Before reconstruction can execute, acquire a durable CAS claim bound to source run, checkpoint revision and executor incarnation; duplicate resume returns the same continuation identity or an explicit conflict, never new execution. Reuse the existing epoch/ownership vocabulary. A predecessor that may still execute or has uncertain effects remains blocked until trusted exclusion/reconciliation; change 11 supplies abrupt-owner exclusion, so 06 alone does not certify abrupt takeover.

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

Version the durable resume envelope and claim. Older incomplete checkpoints remain inspectable but explicitly non-resumable unless protected metadata can be reconstructed with proof. Orderly predecessor evidence enables initial successful restart; abrupt takeover stays denied until change 11.

Dependencies: dot-02-principal-credentials, dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

After durable proof of orderly predecessor shutdown and joined descendants, launch a new manager against the same durable backend, list/resume the checkpoint and verify authorized continuation. Abrupt-loss cases must deny takeover at this change boundary; successful killed-owner takeover is verified in 11. Deny cross-owner, altered artifact/bindings and missing host resources; explicit unsupported on Postgres until implemented, with zero execution. Race authorized resume requests from two separate runtimes and prove one continuation/effect. Kill the claimant before and after effect dispatch; a second claimant cannot replay uncertain work.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.

