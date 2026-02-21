-- Enhanced settings schema with automatic updated_at triggers.
-- Tables are created with IF NOT EXISTS so this is safe to run after
-- 05_create_settings.sql; the triggers and indexes are additive.

CREATE TABLE IF NOT EXISTS settings_types (
    id          UUID        PRIMARY KEY,
    name        TEXT        UNIQUE NOT NULL,
    key         TEXT        UNIQUE NOT NULL,
    schema      JSONB       NOT NULL,
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

DROP TRIGGER IF EXISTS trg_settings_types_updated ON settings_types;
CREATE TRIGGER trg_settings_types_updated
    BEFORE UPDATE ON settings_types
    FOR EACH ROW EXECUTE FUNCTION _uar_settings_types_set_updated_at();

CREATE TABLE IF NOT EXISTS settings (
    id               UUID        PRIMARY KEY,
    settings_type_id UUID        NOT NULL REFERENCES settings_types(id) ON DELETE CASCADE,
    name             TEXT        NOT NULL,
    key              TEXT        UNIQUE NOT NULL,
    data             JSONB       NOT NULL,
    parent_id        UUID        REFERENCES settings(id) ON DELETE SET NULL,
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

DROP TRIGGER IF EXISTS trg_settings_updated ON settings;
CREATE TRIGGER trg_settings_updated
    BEFORE UPDATE ON settings
    FOR EACH ROW EXECUTE FUNCTION _uar_settings_set_updated_at();

CREATE INDEX IF NOT EXISTS idx_settings_key       ON settings(key);
CREATE INDEX IF NOT EXISTS idx_settings_type_id   ON settings(settings_type_id);
CREATE INDEX IF NOT EXISTS idx_settings_parent_id ON settings(parent_id);
