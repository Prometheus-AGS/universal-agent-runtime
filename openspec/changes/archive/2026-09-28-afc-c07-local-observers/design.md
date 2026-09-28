# C07 design

## Authority and source boundary

UAR remains the execution and scheduling owner. The source key is verified owner, workspace, logical instance, stable conversation, and source event sequence. A deterministic occurrence ID derives from that tuple and event kind; attempt/root IDs remain provenance, not new occurrence IDs. Only a successful source CAS can publish an occurrence. The SurrealDB transaction updates the instance and inserts occurrence rows together; failure of either aborts both. The outbox is separate from the bounded C06 UI replay list. Token, thought and tool deltas are ephemeral and cannot enter it.

Each subscription binds an observer instance, exact source-instance allowlist, optional exact conversation intersection, a current private projection grant and finite limits. Registration and delivery use the verified owner/workspace and current catalog binding. Subscription revision is monotonic. The projection initially contains committed control-event metadata and IDs, no prompt, outcome, credentials or raw conversation content. A later content projection requires a separate grant contract. Revocation blocks delivery of already-admitted but unacknowledged content.

## Delivery and recovery

Poll the committed outbox from a persisted source watermark. Admit each permitted occurrence once per subscription using a unique subscription/occurrence key in its bounded inbox; separately persist acknowledgement after C06 returns a durable command receipt. Duplicate admission or restart recovers that receipt, while a changed projection/revision conflicts. An observer command ID is deterministic from the subscription and occurrence. The monitor turn is a fresh C06 root with the observer's current authority. At most one mutating turn runs per observer instance. A pause stops new admission and keeps backlog visible. Failure consumes a finite retry budget and ends in an inspectable dead-letter state. Expired source retention reports the exact missing cursor interval and requires resnapshot or operator acknowledgement; it cannot silently advance.

The outbox is not coupled to the SSE lifetime. Its retention may only advance after all active subscription watermarks permit it, or it must leave a recorded gap for a lagging subscription. Snapshots report source high watermark, retained low watermark, subscription cursor, inbox depth, pause and dead-letter counts. The server may replay observable events, but replay does not repeat an external source effect.

## Product ownership

- Persistence owner: `src/uar/persistence/observers.rs`, `src/uar/persistence/mod.rs`, Surreal provider and migration. One writer owns the Surreal provider file.
- Runtime owner: focused `src/uar/runtime/observer/` service using the C06 controller, catalog and current authority. It never uses RunManager's in-memory broadcast as durability evidence.
- API owner: focused authenticated observer routes and capability flag. Scope and grant checks occur in trusted server code, not the UI.
- Integration gate: one real SurrealKV 3.3.0/UAR host scenario after all three tasks are implemented. Two independent observers, source and conversation exclusions, publication/admission/ack restart, revocation before queued delivery, pause/backlog, gap, finite dead-letter and old ordinary-run behavior. The receipt records exact source, SDK, policy and binding revisions.

## Rollback

Disable new observer admission while retaining source occurrences and subscription records. A rollback does not replay or reverse any source effect. A version without the schema must refuse the durable observer profile rather than treating in-memory streams as equivalent.
