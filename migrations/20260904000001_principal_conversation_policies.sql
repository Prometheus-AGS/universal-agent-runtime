-- Verified principal policies must never share a namespace with raw legacy subjects.
CREATE TABLE IF NOT EXISTS principal_conversation_policies (
    owner_id TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    policy JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (owner_id, conversation_id)
);
