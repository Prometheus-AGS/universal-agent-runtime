# Base UI Verification Refinement Decisions

## 2026-08-08 — Iteration 1

- **Delta:** The old cmdk facade made Radix ownership indirect and the broad browser
  suite could not distinguish component regressions from absent-backend profiles.
- **Correction:** Use Base UI Autocomplete behind the stable facade, remove cmdk, and
  validate the two live selector/palette flows with deterministic browser evidence.
- **Decision:** Keep Assistant UI and vaul at their existing versions because current
  upstream Assistant UI still declares Radix and no Base UI-compatible upgrade removes it.
- **Convergence:** Pending fresh artifact-only adversarial review.

## 2026-08-08 — Iteration 2

- **Delta:** The initial dependency conclusion ignored the authoritative repository-root
  importer, allowing a stale production install graph to contradict the nested graph.
- **Correction:** Treat both root and nested workspaces as binding install surfaces and
  require their lock importers, frozen installs, and `pnpm why` outputs to agree.
- **Decision:** Include root `pnpm-lock.yaml` in C-14d scope and retain repeatable action,
  browser, and strict-spec evidence as the convergence packet.
- **Convergence:** The isolated resolution review returned `PASS` with no remaining
  critical findings. Terminate refinement at iteration 2.
