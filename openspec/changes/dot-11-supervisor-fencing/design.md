# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. An abruptly dead owner cannot record the existing orderly-join evidence.

## Goals / Non-Goals

Goal: Owner recovery requires trusted exclusion before epoch advancement. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

Add host-supervisor-issued exclusion evidence bound to owner incarnation and monotonic epoch. Local supervisor must terminate and reap the managed process group/tree; shared-host supervisor must attest infrastructure-level exclusion of the old execution domain, not a heartbeat or PID. Authenticate the evidence and CAS ownership transfer; preserve non-expiring claims and existing uncertainty reconciliation. Apply the exclusion protocol to ordinary checkpoint continuation claims as well as team owners; 06 creates fail-closed claims and this change enables safe abrupt takeover.

Keep all mutations in trusted host services, retain complete verified owner scope, and expose durable business state through explicit inspectable records. Use fallible typed persistence outcomes; do not mask an unsupported backend or outage with success. No kernel mutation authority, no automatic replay of unknown effects, and no new lease expiry replacing exclusion.

Alternatives: existing UAR services are preferred over an independent queue/workflow authority because admission, receipt and owner transactions must agree. Apalis/Temporal remain analysis references, not dependencies. Rebuilding existing approvals/observer/credential machinery is excluded.



## Selected supervisor environments

Operator decision (2026-10-11): use the existing native service managers locally—macOS LaunchAgent com.prometheus.universal-agent-runtime, Linux systemd uar.service, and Windows SCM as named in the native-service-deployment contract—and Kubernetes for shared hosting. Keep qualification evidence separate per platform; packaging/template checks do not certify recovery. Shared acceptance uses at least two UAR runtime Pods backed by the same durable networked SurrealDB.

The trusted host adapter records supervisor identity, runtime incarnation, process/container identities, containment of descendants, exclusion observation, and a single-use challenge tied to the current owner epoch. Evidence is accepted only over an authenticated host/controller boundary, never from an agent. A replacement process started automatically by a native manager must still acquire a new authorized epoch before effects. Qualification verifies actual root and descendant termination, not just a manager's stopped status.

The Kubernetes adapter binds evidence to cluster, namespace, Pod UID, node UID and container identities. On a reachable node, request controlled termination and obtain authenticated runtime-level evidence that all containers and governed descendants are stopped before reclaim. A missing Pod, force deletion, lease expiry or an unreachable node is insufficient. If node/runtime termination cannot be observed, recovery stays blocked until a trusted infrastructure controller supplies independently verifiable node exclusion evidence; support for a particular power-fencing provider is not implied. Preserve unknown external effects for reconciliation even after exclusion. Qualification must record actual adapter mechanisms and access and prove these cases before enabling reclaim.

Kubernetes evidence source (Context7, 2026-10-11): [Pod lifecycle — forced termination](https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-termination-forced) documents that API deletion does not wait for node termination. [Non-graceful node shutdown](https://kubernetes.io/docs/concepts/cluster-administration/node-shutdown/#non-graceful-node-shutdown) requires verifying shutdown before out-of-service handling. These sources support refusing API-object disappearance as exclusion proof; they do not qualify a UAR adapter.

## Candidate evidence

### cand-001: Existing UAR observers, host admission, approvals and team catalog — adapt

Extend current contracts with stable scheduled occurrences and a delivery outbox; avoid a second authority or task-state system.

- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/runtime/observer/delivery.rs) — Durable subscription/inbox recovery and bounded occurrence delivery already exist.
- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/compiler/collaboration/team_execution/ownership.rs) — Catalog claim/reclaim preserves epochs, queued transfers and uncertain running work.

Risks: Observer polling is not a user recurrence contract. Existing owner identity includes tenant/subject but needs issuer policy reconciliation. Kernel execution remains behind trusted host admission.

## Risks / Trade-offs

The uncomfortable thing: a persisted row or green isolated test does not prove cross-process authority and restart correctness. Validate the actual production boundary and retain negative controls. New retries/guards must trace to the reported failure or real owner/effect/consent boundary; do not harden unrelated code.

The deployment-manager choice is resolved by the operator. Before enabling reclaim, qualify the selected native-manager and Kubernetes adapters against the containment, authenticated evidence and partition scenarios above. Infrastructure access and actual runtime qualification remain unverified; a generic signed assertion is insufficient.

## Migration Plan

Preserve non-expiring ownership rows and add evidence/epoch transitions with authenticated host access. No timer-based migration. Local adapter uses supervisor-controlled child lifetime plus joined descendants; shared adapter must bind the actual deployment manager's exclusion receipt to the old incarnation. Unknown environments stay unsupported. Rollback prevents new reclaim rather than lowering epochs.

Dependencies: dot-06-fresh-process-resume, dot-07-approval-recovery, dot-10-durable-budget-admission. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

Kill owner with active descendants; deny reclaim until exclusion evidence covers root/children, reject replayed/wrong-incarnation evidence and stale executor effects. Transfer only safe queued work. Exercise real local and shared supervisor adapters; unavailable infrastructure evidence blocks shared recovery certification. After proven supervisor exclusion, successfully resume an ordinary checkpoint and a safe queued team task in a fresh runtime; repeat the competing-resume race with old-epoch effect attempts denied.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.


Operator-approved recovery availability (2026-10-12): native takeover may require a confirmed reboot of the same stable host when descendant exclusion cannot be proven. Same-boot takeover is available only through an implemented, qualified containment mechanism. Service stopped, PID absence, timeout, lease expiry and Pod deletion are insufficient. Kubernetes requires actual execution-domain/container/node fencing. A reboot proves prior execution excluded, not the outcome of external actions; uncertain effects and unsettled accounting remain held. Ship the concrete controller/observation path, not merely a receipt schema. Qualification runs only at the full-phase boundary.


## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.
