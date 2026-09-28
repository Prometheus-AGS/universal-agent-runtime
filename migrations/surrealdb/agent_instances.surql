-- C06 logical instances: versioned whole-record CAS with private JSON envelope.
DEFINE TABLE IF NOT EXISTS agent_instances SCHEMALESS;
DEFINE INDEX IF NOT EXISTS agent_instance_scope ON agent_instances FIELDS owner_id, workspace_id, instance_id UNIQUE;
