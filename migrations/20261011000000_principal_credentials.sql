-- Separate from arbitrary legacy subjects. No subject-only ownership backfill.
CREATE TABLE IF NOT EXISTS provider_principal_credentials (
    owner_key         TEXT NOT NULL,
    scope             TEXT NOT NULL CHECK (scope IN ('session', 'agent', 'user')),
    scope_id          TEXT NOT NULL,
    provider_id       TEXT NOT NULL,
    api_key_encrypted TEXT NOT NULL,
    api_key_hint      TEXT NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL,
    updated_at        TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (owner_key, scope, scope_id, provider_id)
);

-- One legacy source can be mapped exactly once. Receipt and ciphertext copy
-- commit in the same transaction; resuming never overwrites rotation/deletion.
CREATE TABLE IF NOT EXISTS provider_credential_owner_migrations (
    source_key TEXT PRIMARY KEY,
    receipt JSONB NOT NULL
);
