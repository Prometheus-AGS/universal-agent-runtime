## ADDED Requirements

### Requirement: Ordered explicit transport and subscription recovery
The trusted host SHALL serialize transport recovery with dependent subscription registration for each connection pair. A transport availability event SHALL NOT itself authorize reconnection. Snapshots SHALL distinguish transport generations, subscription command generations and resource lifecycle generations.

#### Scenario: Transport and subscription aliases receive concurrent commands
- **WHEN** commands target either alias of the same transport/subscription pair
- **THEN** they enter one controller, validate the target generation under its lock, and acknowledge recovery only after transport availability and all enrolled subscriptions are ready

#### Scenario: Transport recovery succeeds but subscription recovery fails
- **WHEN** one explicit recovery operation advances transport state before a later subscription failure
- **THEN** the correlated error retains safe actual transport and subscription state for a subsequent explicit decision without claiming rollback or automatically retrying

### Requirement: Explicit subscription generations
Subscriptions SHALL expose retained readiness and generation gaps, recover through authoritative events or explicit authenticated commands, and reconcile subscriptions with snapshots without timer-driven correctness.

#### Scenario: Subscription ends unexpectedly
- **WHEN** a live subscription ends outside shutdown
- **THEN** the topic becomes unavailable and consumers receive a gap instead of silently retaining healthy state

#### Scenario: A consumer lags
- **WHEN** events are lost to bounded-channel lag
- **THEN** the affected generation is invalidated and a fresh authoritative snapshot is required

### Requirement: Authorized business observation events
Release fixtures and consumers SHALL use owner-authorized event or completion contracts for run progression, approval, ingestion status, provider contact and team run registration. Notifications SHALL trigger authoritative reads rather than expose private shared database rows.

#### Scenario: Ingestion status races stream installation
- **WHEN** an authorized caller opens an ingestion status stream while a job changes
- **THEN** UAR subscribes before reading the caller's exact knowledge-base/document job, emits its authoritative snapshot, and rereads only after a retained change or lifecycle transition

#### Scenario: Team admission precedes local run registration
- **WHEN** an authorized caller awaits a durable attempt's run registration
- **THEN** UAR waits on retained registration and catalog changes, acknowledges only the exact owner's installed run, and reports settlement without registration or runtime unavailability instead of polling
