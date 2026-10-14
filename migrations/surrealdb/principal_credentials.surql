-- Legacy provider_credentials remains quarantined from principal resolution.
DEFINE TABLE IF NOT EXISTS provider_credentials SCHEMALESS;
DEFINE TABLE IF NOT EXISTS provider_principal_credentials SCHEMALESS;
DEFINE INDEX IF NOT EXISTS principal_credential_unique ON provider_principal_credentials FIELDS owner_key, scope, scope_id, provider_id UNIQUE;
DEFINE TABLE IF NOT EXISTS provider_credential_owner_migrations SCHEMALESS;
