-- Immutable pre-format tool results. The run row serializes admissions so the
-- 64 MiB ceiling remains true when parallel tool calls finish concurrently.
CREATE TABLE IF NOT EXISTS canonical_tool_receipt_runs (
    owner_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    retained_bytes BIGINT NOT NULL DEFAULT 0
        CHECK (retained_bytes >= 0 AND retained_bytes <= 67108864),
    PRIMARY KEY (owner_id, run_id)
);

CREATE TABLE IF NOT EXISTS canonical_tool_receipts (
    owner_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    call_id TEXT NOT NULL,
    sequence BIGINT NOT NULL CHECK (sequence >= 0),
    retained_bytes BIGINT NOT NULL
        CHECK (retained_bytes >= 0 AND retained_bytes <= 16777216),
    data JSONB NOT NULL,
    PRIMARY KEY (owner_id, run_id, call_id),
    FOREIGN KEY (owner_id, run_id)
        REFERENCES canonical_tool_receipt_runs(owner_id, run_id)
);

CREATE INDEX IF NOT EXISTS canonical_tool_receipts_run_order
    ON canonical_tool_receipts(owner_id, run_id, sequence, call_id);
