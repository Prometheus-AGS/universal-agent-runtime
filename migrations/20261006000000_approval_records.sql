CREATE TABLE IF NOT EXISTS approval_records (
    owner_key TEXT NOT NULL,
    issuer_id TEXT NOT NULL,
    challenge_id TEXT NOT NULL,
    root_run_id TEXT NOT NULL,
    state TEXT NOT NULL,
    data JSONB NOT NULL,
    PRIMARY KEY (owner_key, issuer_id, challenge_id)
);
CREATE INDEX IF NOT EXISTS approval_records_owner_run ON approval_records (owner_key, root_run_id);
