CREATE TABLE IF NOT EXISTS conversation_policies (
    conversation_id TEXT PRIMARY KEY,
    policy JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_conversation_policies_updated_at
    ON conversation_policies(updated_at);
