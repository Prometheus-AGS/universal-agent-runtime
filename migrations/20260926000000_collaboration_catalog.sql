-- Atomic I1 collaboration catalog state. The JSON document contains immutable
-- package definitions, compatibility projections, command receipts, and private
-- deployment bindings under one monotonic generation.
CREATE TABLE IF NOT EXISTS uar_collaboration_state (
    id TEXT PRIMARY KEY,
    generation BIGINT NOT NULL,
    data JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
