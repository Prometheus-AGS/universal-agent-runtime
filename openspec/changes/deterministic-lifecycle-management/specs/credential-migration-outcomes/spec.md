## ADDED Requirements

### Requirement: Durable migration identity
Migration SHALL atomically commit destination and receipt, join identical local requests with independently checked authority, and preserve typed conflict and unknown outcomes.

#### Scenario: Identical requests race
- **WHEN** authorized identical migration requests arrive concurrently
- **THEN** one durable effect is committed and validated matching receipts are replayed

#### Scenario: Commit response is lost
- **WHEN** transport loss prevents observing the commit result
- **THEN** the response identifies an unknown outcome and reconciliation retains the same operation identity
