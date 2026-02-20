-- Compiler storage tables for PostgreSQL backend.
-- Provides persistent storage for agent specs, compile reports, and compiler sessions,
-- equivalent to the SurrealDB tables uar_specs, uar_reports, uar_compiler_sessions.

-- Agent specification documents
CREATE TABLE IF NOT EXISTS uar_specs (
    id               TEXT        PRIMARY KEY,
    name             TEXT        NOT NULL,
    content          TEXT        NOT NULL,
    description      TEXT,
    latest_report_id TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Compile reports (linked to a spec)
CREATE TABLE IF NOT EXISTS uar_reports (
    id         TEXT        PRIMARY KEY,
    spec_id    TEXT        NOT NULL REFERENCES uar_specs(id) ON DELETE CASCADE,
    data       JSONB       NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_uar_reports_spec ON uar_reports(spec_id);
CREATE INDEX IF NOT EXISTS idx_uar_reports_created ON uar_reports(created_at DESC);

-- Compiler sessions (multi-turn agent building conversations)
CREATE TABLE IF NOT EXISTS uar_compiler_sessions (
    id         TEXT        PRIMARY KEY,
    data       JSONB       NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
