-- Separate from provider credential secrets. Keep revoked records permanently;
-- creation never replaces an existing ID, including during rollback/reissue.
CREATE TABLE IF NOT EXISTS auth_api_keys (
    key_id TEXT PRIMARY KEY,
    data JSONB NOT NULL,
    CHECK (data->>'id' = key_id),
    CHECK (jsonb_typeof(data->'revoked') = 'boolean')
);
