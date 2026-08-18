# Decisions — `skill-scoped-governance`

## Iteration 1 — 2026-08-15T07:31:55Z

- **Decision:** terminate.
- **Iteration:** 1 of 5.
- **Blocking violations remaining:** 0.
- **Rationale:** all four B4 constraints have deterministic evidence, both fail-closed inversions were observed to fail, exact source restoration was proven, and the restored assertions passed.
- **Uncomfortable result:** the first permitted-surface amendment was insufficient because `RunManager` built its policy universe from globally enabled skills. The final implementation required the operator-approved one-line switch to all registered skills so narrower scopes can actually win.

## Iteration 2 — 2026-08-15T07:49:56Z

- **Decision:** continue, correct, then terminate subject to renewed independent review.
- **Iteration:** 2 of 5.
- **Blocking violations remaining after correction:** 0 deterministically observed; independent review pending.
- **Rationale:** the first history-free critic and judge rejected the same-handle restart proof, found a legacy agent-binding regression, required durable deletion proof, and rejected summarized positive receipts. B4 now uses three child processes, retains unknown legacy bindings alongside durable loaded-skill overrides, proves database/filesystem deletion across reopen, and stores literal positive commands/output.
- **Uncomfortable result:** the initial refiner marked 4/4 while its own restart description was inaccurate. A new service over the same live storage handle is not a cold restart.
