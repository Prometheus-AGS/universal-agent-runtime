# Decisions — `skill-builtins-on-embedded`

## Iteration 1 — 2026-08-14T17:27:51Z

- **Decision:** terminate.
- **Iteration:** 1 of 5.
- **Blocking violations remaining:** 0.
- **Rationale:** all four B3 constraints have deterministic evidence; the availability assertion failed under registration removal, exact source restoration was proven, and the restored assertion passed.
- **Uncomfortable result:** a first attempt to reopen the same SurrealKV directory with a second provider hit its live datastore lock. The test now reconstructs the full runtime against the same durable provider, matching the repository's established restart-test boundary without adding retries or changing persistence.
