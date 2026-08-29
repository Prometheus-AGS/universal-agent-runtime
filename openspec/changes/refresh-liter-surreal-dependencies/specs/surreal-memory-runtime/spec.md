## ADDED Requirements

### Requirement: Durable operations have an additive, replay-safe schema
The refreshed Surreal Memory runtime SHALL apply migrations 20 and 21 in order,
SHALL retain prior migration history, and SHALL create the durable operation,
operation event, operation part, and executor event records with unique stable
identity indexes. Reapplying the migrations SHALL be idempotent.

#### Scenario: Existing databases advance without losing prior records
- **WHEN** a database at migration 19 starts with the refreshed runtime
- **THEN** migrations 20 and 21 are recorded once, the durable-operation and executor-journal tables and indexes exist, and records governed by earlier migrations remain readable

#### Scenario: A repeated startup does not duplicate durable schema state
- **WHEN** the refreshed runtime starts again after migration 21 is recorded
- **THEN** the migration runner leaves one accepted record per migration and preserves the unique operation, event-sequence, operation-part, and executor-progress identities

### Requirement: Embedding plans respect the active model token capacity
The refreshed Surreal Memory embedding service SHALL plan long content in the
token domain using the loaded tokenizer and model capacity. Every emitted part,
including special tokens after decode/re-encode normalization, SHALL fit the
model's maximum input length, and a direct embedding request that exceeds that
capacity SHALL fail rather than truncate silently.

#### Scenario: Long content is split into verified deterministic windows
- **WHEN** tokenized content exceeds the active model capacity
- **THEN** the planner emits ordered, overlapping parts whose token bounds and hashes are stable for the same tokenizer and input, and re-encoding each part with special tokens does not exceed the model maximum

#### Scenario: Oversized direct model input fails closed
- **WHEN** a direct embedding path receives more tokens than the model maximum
- **THEN** it returns an `input_too_long` error and does not submit a truncated tensor to the model

### Requirement: Durable indexed writes are idempotent by caller identity
The refreshed Surreal Memory storage SHALL treat the caller-supplied operation
record key as the transport idempotency boundary. It SHALL commit the indexed
memory and its creation-history record in one transaction and SHALL return the
existing record when the same key is replayed.

#### Scenario: A replay returns the original indexed memory
- **WHEN** a completed indexed-memory operation is repeated with the same stable record key
- **THEN** storage returns the existing memory without creating a duplicate memory or creation-history row

#### Scenario: A concurrent stable-key conflict resolves by authoritative read
- **WHEN** another writer commits the stable key while the indexed-memory transaction is in flight
- **THEN** the losing request reads and returns the committed record, or returns the database error when no committed record exists

### Requirement: Memory mutations preserve atomic audit history
The refreshed Surreal Memory storage SHALL commit a memory deletion and its
deletion-history row atomically. Task-stream creation and token-accounting SHALL
also commit atomically, SHALL retry only authoritative transaction-conflict or
not-executed responses, and SHALL stop after the bounded retry limit.

#### Scenario: Delete returns only after history and removal commit together
- **WHEN** deletion succeeds for an existing memory
- **THEN** the memory is no longer readable and exactly one corresponding deletion-history record is committed in the same transaction

#### Scenario: A persistent task-stream conflict terminates
- **WHEN** every task-stream transaction attempt is rejected as a transaction conflict
- **THEN** the runtime applies bounded backoff, stops after the configured maximum attempts, and returns the authoritative conflict instead of spinning or reporting success
