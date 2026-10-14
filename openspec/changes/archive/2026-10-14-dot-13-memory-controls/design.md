# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. A storage engine alone does not give assistant users consent and deletion control.

## Goals / Non-Goals

Goal: Assistant capture and retention are owner controlled. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

Add owner-visible durable capture policy, provenance and retention controls to MemoryService APIs; new assistant automatic capture defaults off until opt-in. Require explicit retention choice at enablement. Use tombstones to immediately suppress deleted/expired content across records/index/history/queued capture; idempotent purge completes without retaining content in audit logs. Preserve existing explicit user memory behavior.

Operator-resolved retention contract (2026-10-11): accept only `{"kind":"days","days":7}`, the same shape with 30, 90 or 365, or `{"kind":"until_deleted"}`. Reject mixed forms, missing choice and other durations. A day is 24 hours in UTC from immutable first durable capture, not enqueue time, last access, retry or schedule-local midnight. Expiry is exclusive: at `now >= effective_expires_at` content is unavailable on every read surface before physical purge.

Each automatic capture records the admitted policy revision, selected retention and first durable capture timestamp. Shortening constrains existing automatic captures immediately, including until-deleted records. Existing effective expiry becomes the earlier prior bound or first-capture plus the shortened duration. Extending policy affects only new captures and cannot extend an existing record, remove an earlier shortening constraint or revive deleted/expired content. Durable constraints must survive interrupted bulk/index cleanup and another policy revision; merely consulting the latest, later-extended policy would be incorrect. Queued work rechecks current consent and retention in the capture commit transaction; repeated attempts cannot reset the first-capture anchor. Explicit user-written and legacy memory is not silently reclassified as opted-in automatic capture.

Keep all mutations in trusted host services, retain complete verified owner scope, and expose durable business state through explicit inspectable records. Use fallible typed persistence outcomes; do not mask an unsupported backend or outage with success. No kernel mutation authority, no automatic replay of unknown effects, and no new lease expiry replacing exclusion.

Alternatives: existing UAR services are preferred over an independent queue/workflow authority because admission, receipt and owner transactions must agree. Apalis/Temporal remain analysis references, not dependencies. Rebuilding existing approvals/observer/credential machinery is excluded.



## Candidate evidence

### cand-005: Existing UAR MemoryService and vendored surreal-memory 0.1.0 — adapt

Preserve the existing memory engine and extend host scoping, consent, deletion and source provenance. A new memory database would not repair the identity boundary.

- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/Cargo.toml) — Memory is a path dependency under vendor/git/surreal-memory-server; Cargo.lock reports 0.1.0 without registry source.
- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/memory/service.rs) — Service exposes retrieval/capture/mutation helpers but user scoping passes subject IDs without the complete tenant/issuer identity.

Risks: Vendor tree is committed source, not the historical revision named in versions.toml. Every memory surface, including native/MCP, legacy HTTP and embedded SDK, needs current owner enforcement. Workflow mirror persistence is not assistant memory privacy certification.

## Risks / Trade-offs

The uncomfortable thing: a persisted row or green isolated test does not prove cross-process authority and restart correctness. Validate the actual production boundary and retain negative controls. New retries/guards must trace to the reported failure or real owner/effect/consent boundary; do not harden unrelated code.

The retention warning from the independent Plan clarification review is resolved by the operator's fixed choices and the exact anchor/revision contract above. The latest full-phase no-testing instruction defers spec validation and independent artifact checking until the complete expanded implementation boundary; it does not block source implementation. Dependencies and real infrastructure availability still apply.

## Migration Plan

Do not reinterpret prior user-created memory as automatic assistant consent. Persist explicit policy revision and content tombstones before background cleanup. Tombstones survive migration/rollback until content and index purge are proven; audit retains only non-content deletion metadata.

Dependencies: dot-12-memory-isolation, dot-08-durable-ingestion. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

Opt-out/revoke during queued capture, expire during retrieval and crash during deletion/index cleanup; no deleted content is returned and purge resumes. Verify both processes honor policy revision, export/history redact removed content and legacy records are not silently assigned new consent.

Add exact-boundary cases for all four day values and until-deleted; reject unknown/mixed/missing input. Shorten an existing until-deleted or longer-lived record, crash before purge, extend the policy, and prove two runtimes cannot read expired content or change its prior bound. Commit queued work after policy change using current retention and prove retries keep the original first durable capture timestamp. These are production-path acceptance criteria, not new unit-only evidence.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.
