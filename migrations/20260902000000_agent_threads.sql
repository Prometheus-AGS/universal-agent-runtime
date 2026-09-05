-- Thread state and immutable spawn lineage. There is deliberately no cascade
-- deletion: these records are the recovery source for root-run lifetime limits.
CREATE TABLE IF NOT EXISTS agent_threads (
    owner_id TEXT NOT NULL,
    thread_id TEXT NOT NULL,
    root_thread_id TEXT NOT NULL,
    root_run_id TEXT NOT NULL,
    parent_thread_id TEXT,
    canonical_path TEXT COLLATE "C" NOT NULL,
    revision BIGINT NOT NULL CHECK (revision >= 0),
    data JSONB NOT NULL,
    PRIMARY KEY (owner_id, thread_id),
    UNIQUE (owner_id, root_run_id, canonical_path),
    FOREIGN KEY (owner_id, root_thread_id) REFERENCES agent_threads(owner_id, thread_id),
    FOREIGN KEY (owner_id, parent_thread_id) REFERENCES agent_threads(owner_id, thread_id)
);

CREATE TABLE IF NOT EXISTS agent_edges (
    owner_id TEXT NOT NULL,
    child_thread_id TEXT NOT NULL,
    parent_thread_id TEXT NOT NULL,
    root_thread_id TEXT NOT NULL,
    root_run_id TEXT NOT NULL,
    canonical_path TEXT COLLATE "C" NOT NULL,
    data JSONB NOT NULL,
    PRIMARY KEY (owner_id, child_thread_id),
    UNIQUE (owner_id, root_run_id, canonical_path),
    FOREIGN KEY (owner_id, child_thread_id) REFERENCES agent_threads(owner_id, thread_id),
    FOREIGN KEY (owner_id, parent_thread_id) REFERENCES agent_threads(owner_id, thread_id),
    FOREIGN KEY (owner_id, root_thread_id) REFERENCES agent_threads(owner_id, thread_id)
);
