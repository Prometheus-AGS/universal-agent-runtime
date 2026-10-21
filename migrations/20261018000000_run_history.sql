-- Restart-durable public run history (run metadata + published events).
CREATE TABLE IF NOT EXISTS durable_run_records (key TEXT PRIMARY KEY, owner_key TEXT,
    run_id TEXT NOT NULL, data JSONB NOT NULL, revision BIGINT NOT NULL);
CREATE INDEX IF NOT EXISTS durable_run_records_owner ON durable_run_records(owner_key);
CREATE TABLE IF NOT EXISTS durable_run_events (key TEXT PRIMARY KEY, run_id TEXT NOT NULL,
    event_id BIGINT NOT NULL, data JSONB NOT NULL);
CREATE INDEX IF NOT EXISTS durable_run_events_run ON durable_run_events(run_id, event_id);
