# Decisions — `emit-rag-retrieval-events`

## Iteration 1 — 2026-08-18T18:48:34Z

- **Decision:** continue to independent review before convergence.
- **Iteration:** 1 of 5.
- **Blocking violations remaining locally:** 0.
- **Rationale:** the minimum product delta reuses the existing citation UI and
  hardened retrieval pipeline, adds the missing KB identity, and surfaces the
  Surreal statement error that had left status pending.
- **Uncomfortable result:** the first browser attempt never exercised retrieval
  because the entity workspace package was unbuilt. After that was repaired,
  the existing BDD helper silently skipped thread creation and the hover locator
  was ambiguous with its screen-reader duplicate. Both fixture defects are now
  explicit waits/scopes, and the final scenario reaches the actual product path.
- **Independent review:** pending critic and judge.

## Iteration 2 — 2026-08-18T19:08:41Z

- **Decision:** terminate with all four constraints satisfied.
- **Iteration:** 2 of 5.
- **Blocking violations remaining:** 0.
- **Independent review:** artifact critic PASS and independent artifact judge
  PASS on the same candidate hashes and refreshed receipts.
- **Rationale:** the reviewers independently confirmed the provenance path,
  hardened pre-injection retrieval, post-score limiting, structured audit event,
  checked Surreal status transition, provider/browser receipts, OpenSpec validity,
  and artifact schema consistency.
- **Commit exclusion:** `.claude/settings.local.json`, `pnpm-lock.yaml`,
  `.refiner/registry.json`, and unrelated KBD projection churn remain outside the
  change.
