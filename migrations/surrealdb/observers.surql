-- C07 metadata-only observer profile. Private subscription envelopes remain scoped.
DEFINE TABLE IF NOT EXISTS agent_instance_occurrences SCHEMALESS;
DEFINE INDEX IF NOT EXISTS occurrence_identity ON agent_instance_occurrences FIELDS occurrence_id UNIQUE;
DEFINE INDEX IF NOT EXISTS occurrence_source_order ON agent_instance_occurrences FIELDS owner_id, workspace_id, source_instance_id, sequence UNIQUE;
DEFINE TABLE IF NOT EXISTS observer_subscriptions SCHEMALESS;
DEFINE INDEX IF NOT EXISTS observer_subscription_scope ON observer_subscriptions FIELDS owner_id, workspace_id, subscription_id UNIQUE;
