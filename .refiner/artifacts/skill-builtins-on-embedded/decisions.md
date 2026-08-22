# Decisions — `skill-builtins-on-embedded`

## Iteration 1 — 2026-08-14T17:27:51Z

- **Decision:** terminate.
- **Iteration:** 1 of 5.
- **Blocking violations remaining:** 0.
- **Rationale:** all four B3 constraints have deterministic evidence; the availability assertion failed under registration removal, exact source restoration was proven, and the restored assertion passed.
- **Uncomfortable result:** a first attempt to reopen the same SurrealKV directory with a second provider hit its live datastore lock. The test now reconstructs the full runtime against the same durable provider, matching the repository's established restart-test boundary without adding retries or changing persistence.

## Iteration 2 — 2026-08-14T17:41:06Z

- **Decision:** terminate after correction.
- **Iteration:** 2 of 5.
- **Blocking violations remaining:** 0.
- **Rationale:** independent critic and judge correctly rejected iteration 1 because one live provider did not prove process-exit durability and enabled re-seeding could mask a load failure. The corrected assertion uses three child processes against one SurrealKV directory: seed; reopen and load with seeding disabled; reopen and re-register with seeding enabled. The exact positive and registration-removal control were both observed again.
- **Uncomfortable result:** iteration 1 declared convergence on weaker evidence. Its decision remains above as history rather than being rewritten.
