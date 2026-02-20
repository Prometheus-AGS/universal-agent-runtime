-- Federated agent registry table for PostgreSQL backend.
-- Provides persistent storage for registered remote agents,
-- equivalent to the SurrealDB table uar_agents.

CREATE TABLE IF NOT EXISTS uar_agents (
    id           TEXT        PRIMARY KEY,
    name         TEXT        NOT NULL,
    description  TEXT        NOT NULL,
    base_url     TEXT        NOT NULL,
    capabilities TEXT[]      NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_uar_agents_name ON uar_agents(name);
