# Tasks

Assignments: see the Task model assignments table in ../../../../.kbd-orchestrator/phases/dot-feature-support/plan.md, scoped by phase dot-feature-support, change dot-17-assistant-release-acceptance, and backend ordinals 1–3. Dotted labels below are display labels; kbd-apply list ordinals are canonical task IDs. Backend task 3 is scheduled before 1 and 2 despite its later ordinal.

Prerequisites: dot-01-release-provenance, dot-02-principal-credentials, dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract, dot-06-fresh-process-resume, dot-07-approval-recovery, dot-08-durable-ingestion, dot-09-idempotent-settlement, dot-10-durable-budget-admission, dot-11-supervisor-fencing, dot-12-memory-isolation, dot-13-memory-controls, dot-14-durable-schedules, dot-15-proactive-followups, dot-16-assistant-capability-surface. Changes18–20 are registered canonically and all nine issue379 items remain required before final acceptance; effective source prerequisites are recorded in the sequencing manifest. No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

Workflow/bootstrap: use KBD begin/end task ownership, read applicable AGENTS.md and skills, verify current source and route, and preserve unrelated work before editing. No workflow tool integration changes are requested here; if scope changes, validate Codex, Claude Code, Cursor and OpenCode integrations explicitly.

Current Dot scheduling (2026-10-12): Existing task labels remain unchanged and backend task 3 adds source-only runner implementation. Future executable verification is deferred until the lead records an explicit inventory that **every production implementation item in the expanded Dot phase, including changes 18–20 and the Dot 17 runner, is complete**. Before that boundary, use source/document inspection and author tests or runners without executing them. Do not run tests, lint, format, typecheck, compiler probes, builds, policy validators, QA detectors, real-path probes, or product review gates at a per-edit, per-task, or per-change boundary. After the inventory, run the planned production integration and customer-build gates serially. Checked 01–06 receipts are historical evidence for their earlier source, not acceptance of the final combined revision. This scheduling note supersedes earlier Tier 0 and completed-change timing language below; it changes no completion checkbox.

## 1. Full-phase production acceptance

- [ ] 1.1 After the lead records the full expanded Dot production implementation inventory, run both complete assistant profile journeys with the task 3 runner and reconcile local product evidence. Exercise memory, schedule, follow-up, connected effect, approval, crash recovery and all nine issue #379 requirements through local SurrealKV and two-runtime durable networked SurrealDB. Record exact source revision, backend, entry point and observed result for each scenario. After local acceptance, run the required customer build and record its artifact. Keep implementation, product evidence, certification and publication statuses independent.

## 2. Published-artifact acceptance

- [ ] 2.1 After task 1 local acceptance and the required customer build, record the installed published artifact's exact tag, source revision, immutable amd64/arm64 digests or package checksum, all nine issue #379 conformance results and narrow deployment health. Actions never runs product suites. Missing infrastructure, pin resolution, any required implementation, or either profile means blocked certification, never a pass.

## 3. Runner production implementation — scheduled before tasks 1 and 2

- [x] 3.1 Implement the local and shared assistant profile scenario runner, conformance fixtures, operator instructions and evidence matrix. Connect each spec scenario and all nine issue #379 items to the intended source revision, backend, entry point and expected observation. Prepare installed-artifact and provenance capture instructions. Author the complete memory, schedule, follow-up, connected effect, approval and crash-recovery journeys without executing them. This runner and documentation are production implementation items required before the full-phase boundary; no test, probe, build or validator runs in this task.

Source closure: 109 scenarios authored, all not_executed. The 24-item inventory
records code complete with verification_authorized false. Tasks1 and2 remain
pending; source completion does not start a gate or certify runtime behavior.

## Workflow follow-up

- Reflect and archive only after required product evidence and independent review pass; do not treat file existence as completed implementation.



## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.


## Superseding delivery acceptance — operator decision 2026-10-14

The operator directed that code-complete UAR delivery stop being blocked by qualification suites and instead be accepted through actual use by agents and dependent projects. The 109-case qualification matrix, synthetic/conformance/fault suites, native reboot and clock/disk qualification, and review-suite certification are no longer prerequisites for this release. Preserve their actual historical passes/failures and unexecuted status. This decision does not mark a cancelled check passed and does not remove any implemented feature or any of the nine reliability or nine issue #379 requirements.

The release exit criteria are usable production implementation, a build of the actual current source, installation on existing authorized capacity, and a useful task executed by an existing configured agent/provider through UAR. Record the real turn, the actual tools/approvals used and their independently observed result, memory/document/accounting persistence actually exercised, exact artifact/source identity and consumer availability. Do not force unused features into artificial scenarios. Report unused protocols, adversarial edge cases and crash/reboot guarantees as unverified rather than blocking unrelated consumers. Keep actual defects, failed builds and failed consumer use actionable; code presence alone is not a passing consumer result.

Necessary artifact building and deployment health remain delivery work. All production implementation and this scope reconciliation precede any build or consumer-use execution. Gates remain serial; no new qualification scripts, matrix expansion, cache-copy workaround or paid resources. Existing node/volume/VM approvals and immutable cleanup deadlines remain. Independent teammates may inspect and fix observed code problems without making their review a new release gate. Merged-main releases for other projects may proceed using their explicitly selected source; unfinished Dot qualification is not a dependency-wide freeze.

Cargo cache IDs are reusable namespace identifiers, not source-attestation hashes. The delivery may reuse the genuinely populated existing architecture/toolchain/profile namespace. Actual source identity, dependency pins, Cargo fingerprints and produced artifact digests remain independently recorded. Do not relabel an old source archive, copy compiled outputs under a fabricated cache identity, prune caches or claim reproducibility from cache reuse.
