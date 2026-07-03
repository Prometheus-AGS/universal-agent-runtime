# CH-02 postgres-credential-store

## Why

Multi-tenant deployments running on the Postgres persistence backend
(`UAR_PERSISTENCE__PROVIDER=postgres`) had no encrypted-credential parity with
the SurrealDB path: `CredentialStore` only had `SurrealCredentialStore` and
`InMemoryCredentialStore` implementations, so a Postgres deployment silently
fell back to the in-memory store — per-user provider keys did not survive a
restart or a multi-instance deployment.

## What changed

- `PostgresCredentialStore` (`src/uar/security/credentials/store.rs`,
  feature-gated behind `sqlx`/`postgres-backend`) implements `CredentialStore`
  at parity with `SurrealCredentialStore`: same `(scope, scope_id,
  provider_id)` triple key, same ciphertext-only persistence contract (the
  caller — `ProviderService` — applies AES-256-GCM via `encryption.rs` before
  `put`, and decrypts after `get`).
- `server.rs`'s Postgres boot path constructs `PostgresCredentialStore`
  directly; the in-memory fallback is gone for that backend.
- New migration `migrations/20260703000000_provider_credentials.sql` creates
  the `provider_credentials` table (composite primary key on the scope
  triple), closing the gap where the DDL previously existed only as a comment.
- Parity test module (`postgres_tests`, gated `#[cfg(all(test, feature =
  "sqlx"))]`) mirrors the four existing `InMemoryCredentialStore` tests
  (store/retrieve, cross-user isolation, delete, provider isolation) against
  a real pool. `#[ignore]`d by default — run with `cargo test --features
  postgres-backend -- --ignored` against
  `docker-compose.prod.postgres.yaml`'s `postgres` service, consistent with
  this repo's existing live-infra test convention
  (`tests/integration/live/*`). This repo has no established testcontainers
  wiring for Postgres, so a docker-compose target was used instead of adding
  that dependency for one test module.

## Scope note

The store + wiring (this proposal's core deliverable) were implemented and
committed in a prior turn (`3f7a40b`) without a corresponding OpenSpec
proposal/tasks pair — this document backfills that record and closes the
remaining gaps (migration, tests, docs) identified during the uar-next-harness
phase's G1/G2/G3 completion pass.
