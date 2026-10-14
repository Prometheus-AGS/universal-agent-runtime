-- Human interaction history only. Acceptance is never a tool execution grant.
DEFINE TABLE IF NOT EXISTS mcp_elicitations SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS owner_key ON mcp_elicitations TYPE string;
DEFINE FIELD IF NOT EXISTS run_id ON mcp_elicitations TYPE string;
DEFINE FIELD IF NOT EXISTS data ON mcp_elicitations TYPE string;
DEFINE INDEX IF NOT EXISTS mcp_elicitations_owner_run ON mcp_elicitations FIELDS owner_key, run_id;
