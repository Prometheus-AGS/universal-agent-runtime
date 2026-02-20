-- Migration: 0010 — Settings Tables
-- Settings Types: one row per named settings namespace.
-- Plugins and extensions register here to add their own domains.
-- No code changes are needed to support new setting domains.
CREATE TABLE IF NOT EXISTS settings_types (
    id          UUID        PRIMARY KEY,
    name        TEXT        UNIQUE NOT NULL,   -- "Server Configuration"
    key         TEXT        UNIQUE NOT NULL,   -- "server"
    schema      JSONB       NOT NULL,          -- JSON Schema (Draft 7) for .data validation + UI generation
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ
);

CREATE OR REPLACE FUNCTION _uar_settings_types_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_settings_types_updated
    BEFORE UPDATE ON settings_types
    FOR EACH ROW EXECUTE FUNCTION _uar_settings_types_set_updated_at();

-- Settings: one row per concrete setting value.
-- data is validated in the application layer against the parent type's JSON Schema.
CREATE TABLE IF NOT EXISTS settings (
    id               UUID        PRIMARY KEY,
    settings_type_id UUID        NOT NULL REFERENCES settings_types(id) ON DELETE CASCADE,
    name             TEXT        NOT NULL,         -- Human-readable label
    key              TEXT        UNIQUE NOT NULL,  -- Dotted key, e.g. "server.port"
    data             JSONB       NOT NULL,         -- Validated against parent type's schema
    parent_id        UUID        REFERENCES settings(id) ON DELETE SET NULL,  -- Optional hierarchy
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ
);

CREATE OR REPLACE FUNCTION _uar_settings_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_settings_updated
    BEFORE UPDATE ON settings
    FOR EACH ROW EXECUTE FUNCTION _uar_settings_set_updated_at();

CREATE INDEX IF NOT EXISTS idx_settings_key        ON settings(key);
CREATE INDEX IF NOT EXISTS idx_settings_type_id    ON settings(settings_type_id);
CREATE INDEX IF NOT EXISTS idx_settings_parent_id  ON settings(parent_id);
