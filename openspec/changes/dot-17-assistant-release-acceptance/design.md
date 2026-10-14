# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. Source evidence and existing image publication do not certify an always-on assistant.

## Goals / Non-Goals

Goal: Both required deployment profiles pass before first release. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

Assemble a local production-boundary scenario runner and evidence matrix connecting each spec scenario to exact source revision, backend, entry point and observed result. Run one complete always-on journey per profile with memory, schedule, follow-up, connected effect, approval and crash recovery. Keep implementation, product evidence, certification and publication statuses independent.

Keep all mutations in trusted host services, retain complete verified owner scope, and expose durable business state through explicit inspectable records. Use fallible typed persistence outcomes; do not mask an unsupported backend or outage with success. No kernel mutation authority, no automatic replay of unknown effects, and no new lease expiry replacing exclusion.

Alternatives: existing UAR services are preferred over an independent queue/workflow authority because admission, receipt and owner transactions must agree. Apalis/Temporal remain analysis references, not dependencies. Rebuilding existing approvals/observer/credential machinery is excluded.



## Candidate evidence

No library adoption is needed. This is domain/build/evidence work identified in library-candidates.json build_required; use current pinned repository components.

## Risks / Trade-offs

The uncomfortable thing: green per-change tests can still miss composition failures; both real complete journeys and final-revision evidence are required. Validate the actual production boundary and retain negative controls. New retries/guards must trace to the reported failure or real owner/effect/consent boundary; do not harden unrelated code.

No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

## Migration Plan

Certification evidence is append-only and tied to immutable revisions. Any implementation change after a passing result invalidates affected evidence until rerun. Publication rollback uses an identified earlier digest; it does not transfer certification to an untested tree.

Dependencies: dot-01-release-provenance, dot-02-principal-credentials, dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract, dot-06-fresh-process-resume, dot-07-approval-recovery, dot-08-durable-ingestion, dot-09-idempotent-settlement, dot-10-durable-budget-admission, dot-11-supervisor-fencing, dot-12-memory-isolation, dot-13-memory-controls, dot-14-durable-schedules, dot-15-proactive-followups, dot-16-assistant-capability-surface. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

Operator-selected environments: existing native service managers locally, and at least two runtime Pods on Kubernetes for shared hosting. Record native platform results separately. Exercise running/approval/uncertain busy-time coalescing, cross-revision overlap exclusion, and Kubernetes unreachable-node/Pod-deletion cases from changes 11 and 14. A chosen platform or manager is not qualification evidence.

Run local SurrealKV plus two-runtime durable networked SurrealDB; exercise negative tenant/issuer cases, fresh-process recovery, real supervisor exclusion, two-process admission/revocation/budgets and unknown effects. After local acceptance, capture final-revision amd64/arm64 immutable digests and narrow deployment health; Actions never runs product suites. Missing infrastructure or pin resolution means blocked certification, never a pass.

Operator-directed candidate deployment on 2026-10-13 may precede completion of those acceptance gates. Reuse the existing Dockerfile and deploy an actual native Linux image by immutable digest into the isolated `dot17-uar` namespace through the existing `know-me` Envoy gateway. Bind the build to the complete frozen source inventory, including the approved baseline Liter pin and its actual uncommitted changes. This operational candidate supplies a stable shared environment; it earns no clean-release, two-architecture, publication, or product acceptance credit. Keep the original receipts and final release prerequisites intact. Use existing worker capacity and the approved single 10 GiB volume; request approval before additional paid provisioning.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.



## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.


## Superseding delivery acceptance — operator decision 2026-10-14

The operator directed that code-complete UAR delivery stop being blocked by qualification suites and instead be accepted through actual use by agents and dependent projects. The 109-case qualification matrix, synthetic/conformance/fault suites, native reboot and clock/disk qualification, and review-suite certification are no longer prerequisites for this release. Preserve their actual historical passes/failures and unexecuted status. This decision does not mark a cancelled check passed and does not remove any implemented feature or any of the nine reliability or nine issue #379 requirements.

The release exit criteria are usable production implementation, a build of the actual current source, installation on existing authorized capacity, and a useful task executed by an existing configured agent/provider through UAR. Record the real turn, the actual tools/approvals used and their independently observed result, memory/document/accounting persistence actually exercised, exact artifact/source identity and consumer availability. Do not force unused features into artificial scenarios. Report unused protocols, adversarial edge cases and crash/reboot guarantees as unverified rather than blocking unrelated consumers. Keep actual defects, failed builds and failed consumer use actionable; code presence alone is not a passing consumer result.

Necessary artifact building and deployment health remain delivery work. All production implementation and this scope reconciliation precede any build or consumer-use execution. Gates remain serial; no new qualification scripts, matrix expansion, cache-copy workaround or paid resources. Existing node/volume/VM approvals and immutable cleanup deadlines remain. Independent teammates may inspect and fix observed code problems without making their review a new release gate. Merged-main releases for other projects may proceed using their explicitly selected source; unfinished Dot qualification is not a dependency-wide freeze.

Cargo cache IDs are reusable namespace identifiers, not source-attestation hashes. The delivery may reuse the genuinely populated existing architecture/toolchain/profile namespace. Actual source identity, dependency pins, Cargo fingerprints and produced artifact digests remain independently recorded. Do not relabel an old source archive, copy compiled outputs under a fabricated cache identity, prune caches or claim reproducibility from cache reuse.
