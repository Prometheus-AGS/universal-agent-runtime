export interface Migration {
  version: number;
  name: string;
  up: string;
}

export const INITIAL_SCHEMA_SQL = `
  CREATE TABLE IF NOT EXISTS threads (
    id           TEXT        PRIMARY KEY,
    title        TEXT        NOT NULL DEFAULT 'New conversation',
    is_ephemeral BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

  CREATE TABLE IF NOT EXISTS messages (
    id         TEXT        PRIMARY KEY,
    thread_id  TEXT        NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
    role       TEXT        NOT NULL CHECK (role IN ('user','assistant','system')),
    content    JSONB       NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status     TEXT        NOT NULL DEFAULT 'complete'
  );

  CREATE INDEX IF NOT EXISTS idx_messages_thread_id  ON messages(thread_id);
  CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(thread_id, created_at);
`;

export const RUN_TABLE_SQL = `
  CREATE TABLE IF NOT EXISTS run (
    id            TEXT        PRIMARY KEY,
    thread_id     TEXT        NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
    message_id    TEXT        REFERENCES messages(id) ON DELETE SET NULL,
    status        TEXT        NOT NULL CHECK (status IN ('running','finished','error','cancelled')),
    started_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at   TIMESTAMPTZ,
    model         TEXT,
    usage         JSONB,
    cost_usd      NUMERIC(12,6),
    phase_timings JSONB       NOT NULL DEFAULT '{}'::jsonb
  )
`;

export const RUN_EVENT_TABLE_SQL = `
  CREATE TABLE IF NOT EXISTS run_event (
    run_id        TEXT        NOT NULL REFERENCES run(id) ON DELETE CASCADE,
    seq           BIGINT      NOT NULL,
    event_id      TEXT        NOT NULL,
    wire_sequence BIGINT      NOT NULL,
    type          TEXT        NOT NULL,
    kind          TEXT        NOT NULL,
    at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    payload       JSONB       NOT NULL,
    PRIMARY KEY (run_id, event_id),
    UNIQUE (run_id, seq)
  )
`;

export const RUN_EVENT_MIGRATION_SQL = `
  ${RUN_TABLE_SQL};
  ${RUN_EVENT_TABLE_SQL};
  CREATE INDEX IF NOT EXISTS idx_run_thread_started
    ON run (thread_id, started_at DESC);
  CREATE INDEX IF NOT EXISTS idx_run_event_kind
    ON run_event (run_id, kind);
  CREATE INDEX IF NOT EXISTS idx_run_event_wire_sequence
    ON run_event (run_id, wire_sequence);
`;

export const MESSAGE_CHUNK_MIGRATION_SQL = `
  ALTER TABLE messages ADD COLUMN IF NOT EXISTS chunks JSONB NOT NULL DEFAULT '[]'::jsonb;
`;

export const MIGRATIONS: readonly Migration[] = [
  { version: 1, name: "initial_schema", up: INITIAL_SCHEMA_SQL },
  { version: 2, name: "runs_and_events", up: RUN_EVENT_MIGRATION_SQL },
  { version: 3, name: "message_chunks", up: MESSAGE_CHUNK_MIGRATION_SQL },
];
