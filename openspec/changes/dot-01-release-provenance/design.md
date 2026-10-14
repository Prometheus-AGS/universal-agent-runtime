# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. The frozen-install and schema fixes already landed; policy pins disagree with effective sources and Docker omits --locked.

## Goals / Non-Goals

Goal: Release artifacts have traceable locked inputs. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

Prepare an exact intended-versus-effective pin reconciliation for the operator; do not edit versions.toml. Once policy resolution is recorded, align source/lock/build inputs, retain frozen pnpm installs and enforce locked Cargo resolution without redoing landed fixes. Preserve embedded schema inclusion and deployment-only workflow policy.

Keep all mutations in trusted host services, retain complete verified owner scope, and expose durable business state through explicit inspectable records. Use fallible typed persistence outcomes; do not mask an unsupported backend or outage with success. No kernel mutation authority, no automatic replay of unknown effects, and no new lease expiry replacing exclusion.

Alternatives: existing UAR services are preferred over an independent queue/workflow authority because admission, receipt and owner transactions must agree. Apalis/Temporal remain analysis references, not dependencies. Rebuilding existing approvals/observer/credential machinery is excluded.



## Candidate evidence

No library adoption is needed. This is domain/build/evidence work identified in library-candidates.json build_required; use current pinned repository components.

## Risks / Trade-offs

The uncomfortable thing: a persisted row or green isolated test does not prove cross-process authority and restart correctness. Validate the actual production boundary and retain negative controls. New retries/guards must trace to the reported failure or real owner/effect/consent boundary; do not harden unrelated code.

No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

## Migration Plan

Do not edit protected versions.toml. Produce an exact proposed reconciliation and wait for the operator's deliberate policy resolution before applying dependent input changes. Keep original checkout and immutable dependency identities recoverable; rollback packaging only to an already coherent source/lock set.

Dependencies: none. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

From a clean recorded checkout, demonstrate frozen dependency closure and locked production packaging. Capture source/gitlinks and effective skill provenance. Publication/health of the final phase revision belongs to change 17; the existing digests prove only the analyzed revision.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.

