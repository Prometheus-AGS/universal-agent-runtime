-- API-key authority is distinct from provider credentials. Revocation retains
-- the record; CREATE cannot replace its tombstone with an active key.
DEFINE TABLE IF NOT EXISTS auth_api_keys SCHEMALESS;
