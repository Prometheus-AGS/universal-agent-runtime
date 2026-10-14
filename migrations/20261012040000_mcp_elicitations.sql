-- Human interaction history only. Acceptance is never a tool execution grant.
CREATE TABLE IF NOT EXISTS mcp_elicitations (
    id TEXT PRIMARY KEY, owner_key TEXT NOT NULL, run_id TEXT NOT NULL, data JSONB NOT NULL
);
CREATE INDEX IF NOT EXISTS mcp_elicitations_owner_run ON mcp_elicitations(owner_key,run_id);
