## ADDED Requirements

### Requirement: Local ingestion admission is bounded independently of durable acceptance

The runtime SHALL bound active ingestion work and locally queued descriptors independently of durable upload acceptance. Saturation SHALL leave accepted work discoverable. One ingestion job SHALL consume one concurrency slot. Locally queued equal-priority work SHALL run FIFO.

#### Scenario: Local queue is full

- **WHEN** durable upload acceptance succeeds while all active slots and local queue positions are occupied
- **THEN** acceptance remains durable and the job is discoverable for a later dispatch without a second durable queue

#### Scenario: Queued work starts in order

- **WHEN** equal-priority descriptors are queued and an active worker settles
- **THEN** the earliest locally queued eligible descriptor receives the next slot

#### Scenario: Oversized admission does not block eligible work

- **WHEN** admission rejects a demand exceeding configured capacity
- **THEN** that demand does not occupy the local queue or prevent a subsequent eligible demand from starting

### Requirement: Ingestion capacity follows actual work settlement

An ingestion slot SHALL remain occupied until actual execution and owned cleanup finish. Caller cancellation, retrieval waiting limits and shutdown requests SHALL NOT establish completion. Failures SHALL preserve durable state transitions and generation fencing.

#### Scenario: Caller abandons an executing job

- **WHEN** a caller stops waiting while extraction or embedding is held by an owned operation
- **THEN** the worker remains owned and its capacity cannot be reused until the operation and worker settle

#### Scenario: Execution fails or panics

- **WHEN** an accepted worker fails or panics
- **THEN** its failure is recorded against the current durable attempt without publishing a partial generation, and its slot is returned after actual worker completion

### Requirement: Ingestion shutdown retains dependent resources until worker joins

Shutdown SHALL close local admission, request cooperative cancellation and retain actual workers through their joins before releasing embedding or persistence dependencies. A cancelled shutdown waiter SHALL NOT discard the retained finalizer. Unsettled work SHALL NOT be reported as released.

#### Scenario: Blocking work outlives the shutdown request

- **WHEN** shutdown begins while a non-cooperative extraction operation remains held
- **THEN** new local work does not start and dependent resources remain owned until the operation exits and its worker joins

#### Scenario: Concurrent shutdown waiters

- **WHEN** multiple callers await shutdown and one waiter is cancelled
- **THEN** the retained finalizer continues and other waiters observe the same actual settlement
