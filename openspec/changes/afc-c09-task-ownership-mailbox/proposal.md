# C09.2: Durable task ownership and team inbox

## Why

C09.1 persists a planning board, but members cannot own work or exchange durable messages. The next usable increment is a revisioned claim and mailbox surface that survives a restart. Actual model turns and effects require C09.3 budget admission.

## What changes

- Add owner/workspace-scoped claim, reassign, reviewer and limited task-state commands with expected revisions, idempotent command IDs and monotonically increasing ownership epochs.
- Add an addressed durable team inbox with accepted, delivered and processed receipts. `trigger-turn` records activation intent; it does not dispatch a model in C09.2.
- Expose fixed authenticated REST routes and descriptor entries. Keep `teamInstance=false` until executable admission exists.

## Risk

A saved assignment is not a running attempt. The API and host must never describe an accepted message as processed, or make a claim look like execution.
