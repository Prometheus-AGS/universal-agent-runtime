-- Full caller-tool handoffs and atomic claims survive runtime restarts.
DEFINE TABLE IF NOT EXISTS client_tool_continuations SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS owner_key ON client_tool_continuations TYPE string;
DEFINE FIELD IF NOT EXISTS agent_id ON client_tool_continuations TYPE string;
DEFINE FIELD IF NOT EXISTS data ON client_tool_continuations TYPE string;
DEFINE INDEX IF NOT EXISTS client_tool_continuations_owner_agent ON client_tool_continuations FIELDS owner_key, agent_id;
DEFINE FIELD IF NOT EXISTS source_run_id ON client_tool_continuations TYPE string;
DEFINE INDEX IF NOT EXISTS client_tool_continuations_run ON client_tool_continuations FIELDS owner_key, agent_id, source_run_id;
