DEFINE TABLE IF NOT EXISTS agui_runs SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS owner_key ON agui_runs TYPE string;
DEFINE FIELD IF NOT EXISTS agent_id ON agui_runs TYPE string;
DEFINE FIELD IF NOT EXISTS thread_id ON agui_runs TYPE string;
DEFINE FIELD IF NOT EXISTS data ON agui_runs TYPE string;
DEFINE INDEX IF NOT EXISTS agui_runs_owner_thread ON agui_runs FIELDS owner_key,agent_id,thread_id;
DEFINE TABLE IF NOT EXISTS agui_contexts SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS run_key ON agui_contexts TYPE string;
