# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. Existing observers need user-controlled follow-up decisions and durable delivery.

## Goals / Non-Goals

Goal: Follow-up decisions and delivery remain governed and durable. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

Persist opt-in rules bound to owner/source/grant and authorized conversation. Consume durable occurrences and commit admission/suppression reasons plus delivery outbox. First-release destination is a durable owner conversation inbox; existing external connectors are allowed only when individually declared supported with idempotency/reconciliation and approval policy. No new email/push connectors. Recheck quiet hours, consent, cancellation and current destination authority before dispatch.

Keep all mutations in trusted host services, retain complete verified owner scope, and expose durable business state through explicit inspectable records. Use fallible typed persistence outcomes; do not mask an unsupported backend or outage with success. No kernel mutation authority, no automatic replay of unknown effects, and no new lease expiry replacing exclusion.

Alternatives: existing UAR services are preferred over an independent queue/workflow authority because admission, receipt and owner transactions must agree. Apalis/Temporal remain analysis references, not dependencies. Rebuilding existing approvals/observer/credential machinery is excluded.



Recurrence input contract: a tagged object accepts once(at: RFC3339 timestamp with UTC offset), daily(time: HH:mm, timezone: IANA identifier), or weekly(weekdays: nonempty unique ISO weekdays 1–7, time: HH:mm, timezone: IANA identifier). These are proposed wire fields, not existing APIs. Reject seconds, invalid zones/dates, empty weekday sets, cron/RRULE strings and past one-shot times at creation. A timezone or recurrence edit creates a new revision effective at commit; old unadmitted occurrences are cancelled, while an already admitted run retains its original instant and authority checks. Daily/weekly wall times use the specified skip/earlier DST policy. The timing adapter fit gate must preserve this schema rather than broaden it.

Quiet hours input contract: an optional daily start/end HH:mm pair and explicit IANA timezone; interval is start-inclusive/end-exclusive, crossing midnight when start > end. Reject start = end; absence disables quiet hours. Evaluate membership by converting each candidate UTC instant to that zone, so both repeated wall-clock instants are quiet if their wall time is inside the interval. Release at the earliest future real instant outside it; a nonexistent end boundary advances to the first real instant outside the window. A policy/zone edit increments its revision and re-evaluates queued delivery before dispatch. Example: 22:00–07:00 suppresses 22:00 and 06:59 but permits 07:00; America/Chicago weekly Sunday 02:30 skips the spring gap and Sunday 01:30 selects the earlier autumn instant. Integration tests must use exact dates from the installed timezone rules and retain those fixture dates in evidence.


## Candidate evidence

### cand-001: Existing UAR observers, host admission, approvals and team catalog — adapt

Extend current contracts with stable scheduled occurrences and a delivery outbox; avoid a second authority or task-state system.

- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/runtime/observer/delivery.rs) — Durable subscription/inbox recovery and bounded occurrence delivery already exist.
- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/compiler/collaboration/team_execution/ownership.rs) — Catalog claim/reclaim preserves epochs, queued transfers and uncertain running work.

Risks: Observer polling is not a user recurrence contract. Existing owner identity includes tenant/subject but needs issuer policy reconciliation. Kernel execution remains behind trusted host admission.

### cand-006: Existing UAR persistence adapters with transactional outbox and settlement records — adapt

Keep data and admission state in current stores. Build narrow domain transactions and uniqueness/CAS contracts rather than introduce a generic job framework as a correctness substitute.

- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/persistence/mod.rs) — Existing persistence boundary includes durable capabilities and also checkpoint/cost defaults requiring explicit unsupported behavior.
- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/persistence/providers/postgres.rs) — Postgres cost recording currently appends scope/amount rows without a run/call settlement key.

Risks: Atomicity must be implemented and tested per supported backend, not assumed from a trait. Source-blob/outbox coordination requires a durable content-reference protocol if bytes are not in the same transaction. Shared-host consistency and single-device restart must be separately proven.

## Risks / Trade-offs

The uncomfortable thing: a persisted row or green isolated test does not prove cross-process authority and restart correctness. Validate the actual production boundary and retain negative controls. New retries/guards must trace to the reported failure or real owner/effect/consent boundary; do not harden unrelated code.

No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

## Migration Plan

Rules default disabled until owner enables them and selects source/destination grants. Persist rule revisions and delivery identity; deletion revokes queued work and retains non-content deduplication evidence. Rollback pauses delivery; it cannot retry uncertain external sends.

Dependencies: dot-13-memory-controls, dot-14-durable-schedules. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

Crash at rule evaluation, decision commit and delivery acknowledgement; race scanners and revoke destination access. Prove one logical inbox item, visible suppression, quiet-hour deferral and unknown external outcome held for reconciliation. No universal exactly-once human-receipt claim.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.

