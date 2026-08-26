CREATE TABLE IF NOT EXISTS user_prompt_caching_settings (
    principal_id TEXT PRIMARY KEY,
    prompt_caching_enabled BOOLEAN NULL,
    preferred_scope JSONB NOT NULL DEFAULT '"session"'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL
);
