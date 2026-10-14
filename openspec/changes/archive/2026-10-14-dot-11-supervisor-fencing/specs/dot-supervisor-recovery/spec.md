# Spec Delta

## Purpose

Define owner recovery requires trusted exclusion before epoch advancement for reliable always-on assistant operation and honest support claims.

## ADDED Requirements

### Requirement: Owner recovery requires trusted exclusion before epoch advancement

Ownership recovery SHALL require authenticated trusted evidence excluding the predecessor execution domain before advancing a monotonic epoch. Only safe queued work SHALL transfer; uncertain effects SHALL remain held.

#### Scenario: Abrupt owner loss

- **WHEN** a runtime dies without recording orderly shutdown
- **THEN** a trusted supervisor must establish exclusion of its entire execution domain before a new epoch can be admitted

#### Scenario: Stale executor

- **WHEN** an old epoch attempts an effect or forged/replayed exclusion is submitted
- **THEN** host admission rejects it and uncertain previous effects remain unreplayed

### Requirement: Selected supervisors prove complete execution exclusion

Recovery SHALL qualify existing native service managers for local deployments and Kubernetes for shared hosting. Evidence SHALL authenticate the supervisor, predecessor incarnation, contained descendants and current ownership challenge. Manager stopped status, Pod deletion, node unreachability and lease expiry alone SHALL NOT authorize reclaim. Unobservable exclusion SHALL leave recovery blocked.

#### Scenario: Native service restarts with surviving child

- **WHEN** a native service manager replaces a crashed root while a governed descendant may still execute
- **THEN** the replacement cannot acquire effect authority until the trusted adapter proves exclusion of the old root and descendants

#### Scenario: Kubernetes Pod disappears on an unreachable node

- **WHEN** the Pod object is deleted while termination of its containers and descendants cannot be established
- **THEN** ownership remains unreclaimed until authenticated runtime termination or independently verifiable infrastructure exclusion is proven

#### Scenario: Validated supervisor evidence is replayed

- **WHEN** an exclusion receipt is reused for a different Pod UID, service incarnation or ownership challenge
- **THEN** the host rejects it and the ownership epoch is unchanged




## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.
