DEFINE TABLE IF NOT EXISTS accounting_scopes SCHEMALESS;
DEFINE TABLE IF NOT EXISTS accounting_calls SCHEMALESS;
DEFINE INDEX IF NOT EXISTS accounting_calls_owner_run ON accounting_calls FIELDS owner_key, run_id;
DEFINE TABLE IF NOT EXISTS cost_ledger SCHEMALESS;
DEFINE TABLE IF NOT EXISTS accounting_delegations SCHEMALESS;
DEFINE TABLE IF NOT EXISTS accounting_remote_grants SCHEMALESS;
