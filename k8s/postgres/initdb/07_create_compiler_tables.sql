-- Compiler storage: specs, reports, and multi-turn compiler sessions.
CREATE TABLE IF NOT EXISTS uar_specs (
    id               TEXT        PRIMARY KEY,
    name             TEXT        NOT NULL,
    content          TEXT        NOT NULL,
    description      TEXT,
    latest_report_id TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS uar_reports (
    id         TEXT        PRIMARY KEY,
    spec_id    TEXT        NOT NULL REFERENCES uar_specs(id) ON DELETE CASCADE,
    data       JSONB       NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_uar_reports_spec    ON uar_reports(spec_id);
CREATE INDEX IF NOT EXISTS idx_uar_reports_created ON uar_reports(created_at DESC);

CREATE TABLE IF NOT EXISTS uar_compiler_sessions (
    id         TEXT        PRIMARY KEY,
    data       JSONB       NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
