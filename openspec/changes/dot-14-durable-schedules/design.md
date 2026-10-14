# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. Observer polling does not define durable user scheduling.

## Goals / Non-Goals

Goal: Schedules persist intent and unique governed occurrences. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

The source-qualified choice is the existing UAR Tokio scanner plus exact already-locked Jiff 0.2.35 for IANA calendar conversion. The bounded Analyze correction rejected tokio-cron-scheduler 0.15.1 because it stores a fixed offset for subsequent ticks; see implementation/dot14-timing-fit.md in the parent phase. Executable qualification is deferred until the full phase boundary. Implement owner-scoped one-shot UTC and recurring calendar schedules, IANA timezone, revision/cancel/pause and atomic cursor-plus-occurrence admission. Default no overlap; coalesce backlog to the latest due occurrence with recorded skipped range, skip nonexistent DST local times and select the earlier repeated instant. Current authority, budget and approval apply on every admission.

Keep all mutations in trusted host services, retain complete verified owner scope, and expose durable business state through explicit inspectable records. Use fallible typed persistence outcomes; do not mask an unsupported backend or outage with success. No kernel mutation authority, no automatic replay of unknown effects, and no new lease expiry replacing exclusion.

Alternatives: existing UAR services are preferred over an independent queue/workflow authority because admission, receipt and owner transactions must agree. Apalis/Temporal remain analysis references, not dependencies. Rebuilding existing approvals/observer/credential machinery is excluded. The conditional timing library owns wakeups only; UAR owns schedule/occurrence/cursor transactions.



Recurrence input contract: a tagged object accepts once(at: RFC3339 timestamp with UTC offset), daily(time: HH:mm, timezone: IANA identifier), or weekly(weekdays: nonempty unique ISO weekdays 1–7, time: HH:mm, timezone: IANA identifier). These are proposed wire fields, not existing APIs. Reject seconds, invalid zones/dates, empty weekday sets, cron/RRULE strings and past one-shot times at creation. A timezone or recurrence edit creates a new revision effective at commit; old unadmitted occurrences are cancelled, while an already admitted run retains its original instant and authority checks. Daily/weekly wall times use the specified skip/earlier DST policy. The timing adapter fit gate must preserve this schema rather than broaden it.

Quiet hours input contract: an optional daily start/end HH:mm pair and explicit IANA timezone; interval is start-inclusive/end-exclusive, crossing midnight when start > end. Reject start = end; absence disables quiet hours. Evaluate membership by converting each candidate UTC instant to that zone, so both repeated wall-clock instants are quiet if their wall time is inside the interval. Release at the earliest future real instant outside it; a nonexistent end boundary advances to the first real instant outside the window. A policy/zone edit increments its revision and re-evaluates queued delivery before dispatch. Example: 22:00–07:00 suppresses 22:00 and 06:59 but permits 07:00; America/Chicago weekly Sunday 02:30 skips the spring gap and Sunday 01:30 selects the earlier autumn instant. Integration tests must use exact dates from the installed timezone rules and retain those fixture dates in evidence.


## Busy-time coalescing

Operator decision (2026-10-11): coalesce busy-time occurrences to the latest pending occurrence until the previous run is safely resolved. Maintain one durable overlap guard per complete owner and stable schedule identity, spanning every schedule revision. Claimed/queued execution, running work, pending approval, recoverable suspended work, cancellation in progress, and uncertain execution/effects hold the guard. Restart, wall-clock expiry and an edit do not release it. Release requires a durable terminal disposition with the previous execution excluded and its effects resolved; when required accounting remains unknown, admission must still respect its reserved liability.

While the guard is held, atomically advance the schedule cursor and replace the unadmitted pending slot only with a later due instant for the current revision, recording coalesced/skipped range and reason. On safe release, recheck revision, pause/cancel state, owner/destination authority, budget and approval, then atomically claim at most one current pending occurrence. Schedule edits cancel the old revision's unadmitted slot but retain the cross-revision guard; only due occurrences from the new revision can fill it. Cancellation clears pending intent; pausing prevents admission while retaining current valid intent for the normal coalescing policy on resume. Test competing scanners, multiple due instants during running/approval/unknown states, restart and a revision edit; no intermediate occurrence is executed and no second run starts before safe resolution.

## Candidate evidence

### cand-001: Existing UAR observers, host admission, approvals and team catalog — adapt

Extend current contracts with stable scheduled occurrences and a delivery outbox; avoid a second authority or task-state system.

- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/runtime/observer/delivery.rs) — Durable subscription/inbox recovery and bounded occurrence delivery already exist.
- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/compiler/collaboration/team_execution/ownership.rs) — Catalog claim/reclaim preserves epochs, queued transfers and uncertain running work.

Risks: Observer polling is not a user recurrence contract. Existing owner identity includes tenant/subject but needs issuer policy reconciliation. Kernel execution remains behind trusted host admission.

### cand-002: tokio-cron-scheduler 0.15.1 — rejected after source qualification

Historical candidate, now rejected for the fixed-offset continuation described in the source-fit report. Do not adopt its persistence as a second source of truth. No dependency introduced or final pin authorized by this artifact.

- Tier 1: [source](https://github.com/mvniekerk/tokio-cron-scheduler/blob/v0.15.1/README.md) — Tagged README documents cron, one-shot/repeated jobs, timezone APIs and optional Postgres/Nats stores.
- Tier 2: [source](https://github.com/mvniekerk/tokio-cron-scheduler/blob/main/_autodocs/api-reference/job-builder.md) — Context7 documents timezone-aware JobBuilder; main-branch snippets are not pinned API certification.
- Tier 3: [source](https://crates.io/api/v1/crates/tokio-cron-scheduler) — Registry reports stable 0.15.1, released 2025-10-28, and dual MIT/Apache licensing.

Risks: Adoption is conditional on exact-version DST/missed-run/restart tests; no such tests run in Analyze. Durable UAR records must remain authoritative; callbacks may request admission, never execute tools directly. It adds a scheduler dependency; if its timing adapter duplicates UAR scanning without reducing recurrence complexity, reject it at the planned fit gate.

### cand-006: Existing UAR persistence adapters with transactional outbox and settlement records — adapt

Keep data and admission state in current stores. Build narrow domain transactions and uniqueness/CAS contracts rather than introduce a generic job framework as a correctness substitute.

- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/persistence/mod.rs) — Existing persistence boundary includes durable capabilities and also checkpoint/cost defaults requiring explicit unsupported behavior.
- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/persistence/providers/postgres.rs) — Postgres cost recording currently appends scope/amount rows without a run/call settlement key.

Risks: Atomicity must be implemented and tested per supported backend, not assumed from a trait. Source-blob/outbox coordination requires a durable content-reference protocol if bytes are not in the same transaction. Shared-host consistency and single-device restart must be separately proven.

## Risks / Trade-offs

The uncomfortable thing: a persisted row or green isolated test does not prove cross-process authority and restart correctness. Validate the actual production boundary and retain negative controls. New retries/guards must trace to the reported failure or real owner/effect/consent boundary; do not harden unrelated code.

The operator selected latest-pending coalescing. The busy-time contract above resolves the review's policy question and is part of timing qualification and production acceptance; no-overlap must remain durable across approvals, uncertainty, restart and schedule revisions.

## Migration Plan

New schedules start in the new versioned store with no inferred import from observer polling. Edits increment revision and cancel old unadmitted work. Retain occurrence/admission deduplication identity for at least the maximum supported replay horizon; deletion leaves a non-content tombstone to reject stale work. Rollback pauses admissions.

Dependencies: dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract, dot-06-fresh-process-resume, dot-07-approval-recovery, dot-10-durable-budget-admission, dot-11-supervisor-fencing. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

After full phase implementation, prove the selected timing fit (DST, cancel, downtime, no duplicate authoritative state); failure returns the choice to Analyze. The manifest only adds an exact direct reference to the already locked Jiff0.2.35; no Cargo resolution ran during implementation. Then race two scanners, crash at cursor/admission, restart, pause/cancel and change schedule revision. Verify one logical occurrence and receipt, truthful skipped/coalesced state and no replay of uncertain work.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.

