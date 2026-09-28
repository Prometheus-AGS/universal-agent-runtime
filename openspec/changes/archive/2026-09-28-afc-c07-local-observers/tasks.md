# C07 product tasks

- [x] C07.1 Publish stable committed instance occurrences in the same SurrealDB transaction as the source CAS. Keep token streams outside the durable profile.
- [x] C07.2 Persist scoped subscription revisions, source/conversation grant intersection, independent outbox cursor, inbox admission/ack and gap state; expose authenticated administration.
- [x] C07.3 Activate bounded monitor turns through C06 with current authority, pause/backlog/dead-letter/recovery controls and replay-as-observe default.
- [x] Run the single real-provider production integration gate after C07.1–C07.3 and record its exact evidence in `gate-receipt.md`. Initiative ledger reconciliation, archive, and stacked PR publication follow the product gate.
