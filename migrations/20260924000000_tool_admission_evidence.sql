CREATE TABLE IF NOT EXISTS tool_admission_evidence (
    owner_id TEXT NOT NULL,
    invocation_id TEXT NOT NULL,
    state TEXT NOT NULL,
    root_run_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    data JSONB NOT NULL,
    PRIMARY KEY (owner_id, invocation_id, state)
);

CREATE INDEX IF NOT EXISTS tool_admission_evidence_owner_time
    ON tool_admission_evidence(owner_id, occurred_at, invocation_id);

