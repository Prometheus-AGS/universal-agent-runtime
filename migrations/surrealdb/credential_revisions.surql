-- Existing credential and receipt identities remain immutable. Missing optional
-- revisions represent historical rows until explicit host initialization.
DEFINE FIELD IF NOT EXISTS row_revision ON TABLE provider_credentials TYPE option<string>;
DEFINE FIELD IF NOT EXISTS row_revision ON TABLE provider_principal_credentials TYPE option<string>;

-- provider_credential_owner_migrations remains SCHEMALESS so its existing
-- immutable receipt accepts additive revision and precondition metadata.
