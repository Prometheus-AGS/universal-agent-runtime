# Linux native qualification selection


## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.

Reconciled artifacts:

- .kbd-orchestrator/phases/dot-feature-support/plan.md
- openspec/changes/dot-11-supervisor-fencing/proposal.md
- openspec/changes/dot-11-supervisor-fencing/design.md
- openspec/changes/dot-11-supervisor-fencing/tasks.md
- openspec/changes/dot-11-supervisor-fencing/specs/dot-supervisor-recovery/spec.md
- openspec/changes/dot-17-assistant-release-acceptance/design.md
- openspec/changes/dot-17-assistant-release-acceptance/tasks.md
- openspec/changes/dot-17-assistant-release-acceptance/specs/product-validation-evidence/spec.md
- docs/operations/supervisor-recovery.md
- docs/operations/dot-supervisor-qualification.md
- docs/operations/dot-validation-targets.md
- docs/operations/dot-release-acceptance.md
- docs/durable-schedules.md
- implementation/dot17-scenario-matrix.json (release applicability only; IDs, profiles, drivers and observations preserved).

Unverified: remaining memory source gates, actual guest/systemd supervisor installation and physical qualification, LKE faults, dual architecture artifacts, customer build, installed release and M3/K3 acceptance.
