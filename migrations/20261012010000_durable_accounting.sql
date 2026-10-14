-- Independent identities for new enforcement; legacy cost_ledger remains history.
CREATE TABLE IF NOT EXISTS accounting_scopes (key TEXT PRIMARY KEY, data JSONB NOT NULL);
CREATE TABLE IF NOT EXISTS accounting_calls (
    key TEXT PRIMARY KEY, owner_key TEXT NOT NULL, run_id TEXT NOT NULL, data JSONB NOT NULL
);
CREATE INDEX IF NOT EXISTS accounting_calls_owner_run ON accounting_calls(owner_key, run_id);
CREATE TABLE IF NOT EXISTS accounting_delegations (digest TEXT PRIMARY KEY, data JSONB NOT NULL);
CREATE TABLE IF NOT EXISTS accounting_remote_grants (key TEXT PRIMARY KEY, data JSONB NOT NULL);
