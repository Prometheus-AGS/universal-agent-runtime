## 1. Store implementation (shipped prior turn, commit `3f7a40b`)

- [x] 1.1 `PostgresCredentialStore` implements `CredentialStore` at parity
      with `SurrealCredentialStore` (`src/uar/security/credentials/store.rs`,
      gated `#[cfg(feature = "sqlx")]`).
- [x] 1.2 `server.rs` constructs `PostgresCredentialStore` on the Postgres
      persistence path; in-memory fallback removed for that path.
- [x] 1.3 `sqlx`/`postgres-backend` Cargo features and dependency already
      present (`sqlx = "0.8.6"`, `postgres-backend = ["sqlx", "dep:pgvector"]`).

## 2. Close remaining gaps (this pass)

- [x] 2.1 `migrations/20260703000000_provider_credentials.sql` — turns the
      `CREATE TABLE` DDL (previously only a code comment) into a real sqlx
      migration.
- [x] 2.2 `postgres_tests` module in `store.rs` (gated `#[cfg(all(test,
      feature = "sqlx"))]`) mirrors the four `InMemoryCredentialStore` tests
      against a real Postgres pool via `DATABASE_URL` (defaults to
      `docker-compose.prod.postgres.yaml`'s local credentials).
      `#[ignore]`d by default per this repo's live-infra test convention.
- [x] 2.3 This proposal + tasks doc backfills the OpenSpec record.

## 3. Verify

- [x] 3.1 `cargo check --lib --tests --features postgres-backend` green.
- [ ] 3.2 `cargo test --features postgres-backend -- --ignored
      postgres_tests` green against a live `docker-compose.prod.postgres.yaml`
      Postgres — **not run this pass** (no local Postgres instance available
      in this environment); left `#[ignore]`d for operator/CI verification.
