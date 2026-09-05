-- Applied by the provider at connection time, including embedded deployments.
-- Data is a JSON string envelope to preserve Rust timestamps and unsigned
-- history revisions without provider-specific scalar coercion.
DEFINE TABLE IF NOT EXISTS agent_threads SCHEMALESS;
DEFINE INDEX IF NOT EXISTS agent_thread_owner_id ON agent_threads FIELDS owner_id, thread_id UNIQUE;
DEFINE INDEX IF NOT EXISTS agent_thread_owner_path ON agent_threads FIELDS owner_id, root_run_id, canonical_path UNIQUE;

DEFINE TABLE IF NOT EXISTS agent_edges SCHEMALESS;
DEFINE INDEX IF NOT EXISTS agent_edge_owner_child ON agent_edges FIELDS owner_id, child_thread_id UNIQUE;
DEFINE INDEX IF NOT EXISTS agent_edge_owner_path ON agent_edges FIELDS owner_id, root_run_id, canonical_path UNIQUE;
