-- Full caller-tool handoffs and atomic claims survive runtime restarts.
CREATE TABLE IF NOT EXISTS client_tool_continuations (
    key TEXT PRIMARY KEY, owner_key TEXT NOT NULL, agent_id TEXT NOT NULL, source_run_id TEXT NOT NULL, data JSONB NOT NULL
);
CREATE INDEX IF NOT EXISTS client_tool_continuations_owner_agent ON client_tool_continuations(owner_key,agent_id);
CREATE INDEX IF NOT EXISTS client_tool_continuations_run ON client_tool_continuations(owner_key,agent_id,source_run_id);
