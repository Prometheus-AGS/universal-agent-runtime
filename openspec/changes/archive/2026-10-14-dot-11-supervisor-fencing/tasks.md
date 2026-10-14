# Tasks

Assignments: see the Task model assignments table in ../../../../.kbd-orchestrator/phases/dot-feature-support/plan.md, scoped by phase dot-feature-support, change dot-11-supervisor-fencing, and backend ordinals 1–2. Dotted labels below are display labels; kbd-apply list ordinals are canonical task IDs.

Prerequisites: dot-06-fresh-process-resume, dot-07-approval-recovery, dot-10-durable-budget-admission. Operator-selected scope: existing native service managers locally and Kubernetes shared hosting. Both tasks must apply design.md's selected-supervisor contract: platform-specific qualification, incarnation-bound evidence, descendant containment and refusal of Pod deletion or unreachable-node status as exclusion. Actual adapter qualification gates enabling reclaim; manager selection is resolved.

Workflow/bootstrap: use KBD begin/end task ownership, read applicable AGENTS.md and skills, verify current source and route, and preserve unrelated work before editing. No workflow tool integration changes are requested here; if scope changes, validate Codex, Claude Code, Cursor and OpenCode integrations explicitly.

Current Dot scheduling (2026-10-12): These task labels remain unchanged, but future executable verification is deferred until the lead records an explicit inventory that **every production implementation item in the expanded Dot phase, including changes 18–20 and the Dot 17 runner, is complete**. Before that boundary, use source/document inspection and author tests or runners without executing them. Do not run tests, lint, format, typecheck, compiler probes, builds, policy validators, QA detectors, real-path probes, or product review gates at a per-edit, per-task, or per-change boundary. After the inventory, run the planned production integration and customer-build gates serially. Checked 01–06 receipts are historical evidence for their earlier source, not acceptance of the final combined revision. This scheduling note supersedes earlier Tier 0 and completed-change timing language below; it changes no completion checkbox.

## 1. Complete capability slice

- [x] 1.1 Implement recover dead owners only after trusted exclusion. Add host-supervisor-issued exclusion evidence bound to owner incarnation and monotonic epoch. Local supervisor must terminate and reap the managed process group/tree; shared-host supervisor must attest infrastructure-level exclusion of the old execution domain, not a heartbeat or PID. Authenticate the evidence and CAS ownership transfer; preserve non-expiring claims and existing uncertainty reconciliation. Apply the exclusion protocol to ordinary checkpoint continuation claims as well as team owners; 06 creates fail-closed claims and this change enables safe abrupt takeover. Author integration tests and operator/API documentation in the same production slice. Inspect source wiring and explicit state/error paths without executing checks. Run the next task's real-path acceptance only after the full expanded Dot production implementation inventory is complete.

## 2. Production-boundary acceptance

- [ ] 2.1 Verify recover dead owners only after trusted exclusion through production boundaries. Kill owner with active descendants; deny reclaim until exclusion evidence covers root/children, reject replayed/wrong-incarnation evidence and stale executor effects. Transfer only safe queued work. Exercise real local and shared supervisor adapters; unavailable infrastructure evidence blocks shared recovery certification. After proven supervisor exclusion, successfully resume an ordinary checkpoint and a safe queued team task in a fresh runtime; repeat the competing-resume race with old-epoch effect attempts denied.

## Workflow follow-up

- Reflect and archive only after required product evidence and independent review pass; do not treat file existence as completed implementation.


Operator-approved recovery availability (2026-10-12): native takeover may require a confirmed reboot of the same stable host when descendant exclusion cannot be proven. Same-boot takeover is available only through an implemented, qualified containment mechanism. Service stopped, PID absence, timeout, lease expiry and Pod deletion are insufficient. Kubernetes requires actual execution-domain/container/node fencing. A reboot proves prior execution excluded, not the outcome of external actions; uncertain effects and unsettled accounting remain held. Ship the concrete controller/observation path, not merely a receipt schema. Qualification runs only at the full-phase boundary.


## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.


## Operator closure — 2026-10-14

Production implementation is committed on main. Remaining unchecked verification tasks were cancelled by explicit operator instruction and remain unchecked; no new passing result is asserted. This change is archived and does not block consumers.
