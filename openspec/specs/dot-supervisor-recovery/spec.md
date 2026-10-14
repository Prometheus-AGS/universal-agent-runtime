# dot-supervisor-recovery Specification

## Purpose
Define owner recovery requires trusted exclusion before epoch advancement for reliable always-on assistant operation and honest support claims.

## Requirements

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
