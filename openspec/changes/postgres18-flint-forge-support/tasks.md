# Tasks — postgres18-flint-forge-support

Verification is BDD only (`postgres-flint-forge.feature`, `remote-surreal-restart.feature`), run once the batch is implemented and the operator has approved the gate.

## 1. Infrastructure
- [x] 1.1 `dedicated-runtime.mjs` fixture with `surreal` and `postgres` backends and a keyless stub LLM option.
- [x] 1.2 `flint-forge-uar-db.sh` init script creating the `uar` role and database with pgvector.
- [x] 1.3 Compose, `.env.example`, config example and `docs/operations/postgres-flint-forge.md`.

## 2. Store fixes
- [x] 2.1 #389 settings `source` column and provenance-aware upsert.
- [x] 2.2 #390 legacy session keys and SQL-filtered expiry.
- [x] 2.3 #393 truthful memory refusal.
- [x] 2.4 #395 unconstrained skill vectors with fingerprints and re-embedding.
- [x] 2.5 #394 `capability_unsupported` refusals and `GET /api/uar/persistence/capabilities`.

## 3. Verification
- [ ] 3.1 Builds for `server-full` and `server-full,postgres-backend` with zero warnings.
- [ ] 3.2 `openspec validate postgres18-flint-forge-support`.
- [ ] 3.3 `postgres-flint-forge` and `remote-surreal-restart` scenarios once each.
