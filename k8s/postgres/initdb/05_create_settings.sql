-- Initial settings schema (v1). The enhanced version with update triggers
-- is applied by 10_settings_tables.sql; both are idempotent.
CREATE TABLE IF NOT EXISTS settings_types (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    key TEXT NOT NULL UNIQUE,
    description TEXT,
    display_mode TEXT NOT NULL DEFAULT 'form',
    schema JSONB,
    icon_url TEXT
);

CREATE TABLE IF NOT EXISTS settings (
    id UUID PRIMARY KEY,
    settings_type_id UUID NOT NULL REFERENCES settings_types(id),
    name TEXT NOT NULL,
    key TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    data JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_settings_key ON settings(key);
