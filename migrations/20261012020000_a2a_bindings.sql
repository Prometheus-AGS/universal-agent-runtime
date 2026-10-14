-- Transport correlation. This table neither grants execution nor expires claims.
CREATE TABLE IF NOT EXISTS a2a_bindings (
    owner_key TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    data JSONB NOT NULL,
    PRIMARY KEY (owner_key, agent_id, task_id)
);
-- One durable submission slot protects a shared conversation from concurrent roots.
CREATE TABLE IF NOT EXISTS a2a_contexts (
    owner_key TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    context_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    PRIMARY KEY (owner_key, agent_id, context_id)
);
