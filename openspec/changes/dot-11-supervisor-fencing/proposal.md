# Proposal

## Why

An abruptly dead owner cannot record the existing orderly-join evidence. This change supplies the R9 acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Add host-supervisor-issued exclusion evidence bound to owner incarnation and monotonic epoch. Local supervisor must terminate and reap the managed process group/tree; shared-host supervisor must attest infrastructure-level exclusion of the old execution domain, not a heartbeat or PID. Authenticate the evidence and CAS ownership transfer; preserve non-expiring claims and existing uncertainty reconciliation. Apply the exclusion protocol to ordinary checkpoint continuation claims as well as team owners; 06 creates fail-closed claims and this change enables safe abrupt takeover.

## Capabilities

### New Capabilities

- `dot-supervisor-recovery`: Owner recovery requires trusted exclusion before epoch advancement.

### Modified Capabilities

None.

## Impact

The operator selected existing native service managers locally (macOS LaunchAgent, Linux systemd and Windows SCM under the existing platform contract) and Kubernetes shared hosting. The selected-supervisor evidence and partition requirements in design.md/specs are binding; actual per-platform qualification remains unverified.

Code ownership: `src/uar/compiler/collaboration/team_execution/ownership.rs`, `src/uar/compiler/collaboration/team_execution`, `src/server.rs`. Dependencies: dot-06-fresh-process-resume, dot-07-approval-recovery, dot-10-durable-budget-admission. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.



## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.
