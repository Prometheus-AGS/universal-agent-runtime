-- Protocol correlation only; native run and approval stores own execution.
CREATE TABLE IF NOT EXISTS agui_runs (key TEXT PRIMARY KEY, owner_key TEXT NOT NULL,
    agent_id TEXT NOT NULL, thread_id TEXT NOT NULL, data JSONB NOT NULL);
CREATE INDEX IF NOT EXISTS agui_runs_owner_thread ON agui_runs(owner_key,agent_id,thread_id);
CREATE TABLE IF NOT EXISTS agui_contexts (key TEXT PRIMARY KEY, run_key TEXT NOT NULL);
