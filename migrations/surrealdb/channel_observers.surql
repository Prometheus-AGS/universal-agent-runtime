-- C08 channel-source profile remains separate from C07 logical-instance observers.
DEFINE TABLE IF NOT EXISTS channel_observer_subscriptions SCHEMALESS;
DEFINE INDEX IF NOT EXISTS channel_observer_subscription_scope ON channel_observer_subscriptions FIELDS owner_id, workspace_id, subscription_id UNIQUE;
DEFINE TABLE IF NOT EXISTS channel_observer_inbox SCHEMALESS;
DEFINE INDEX IF NOT EXISTS channel_observer_delivery_identity ON channel_observer_inbox FIELDS owner_id, workspace_id, subscription_id, delivery_id UNIQUE;
