# Decisions — `skill-scoped-governance`

## Iteration 1 — 2026-08-15T07:31:55Z

- **Decision:** terminate.
- **Iteration:** 1 of 5.
- **Blocking violations remaining:** 0.
- **Rationale:** all four B4 constraints have deterministic evidence, both fail-closed inversions were observed to fail, exact source restoration was proven, and the restored assertions passed.
- **Uncomfortable result:** the first permitted-surface amendment was insufficient because `RunManager` built its policy universe from globally enabled skills. The final implementation required the operator-approved one-line switch to all registered skills so narrower scopes can actually win.
