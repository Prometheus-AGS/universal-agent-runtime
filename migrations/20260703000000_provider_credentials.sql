-- Multi-tenant encrypted provider credentials (CH-02 postgres-credential-store).
--
-- Backs `PostgresCredentialStore` (src/uar/security/credentials/store.rs),
-- the Postgres-parity implementation of `CredentialStore` alongside
-- `SurrealCredentialStore`. Only ciphertext is stored here — encryption is
-- applied by the caller (`ProviderService`) via AES-256-GCM
-- (src/uar/security/credentials/encryption.rs).
CREATE TABLE IF NOT EXISTS provider_credentials (
    scope             TEXT NOT NULL,
    scope_id          TEXT NOT NULL,
    provider_id       TEXT NOT NULL,
    api_key_encrypted TEXT NOT NULL,
    api_key_hint      TEXT NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (scope, scope_id, provider_id)
);
