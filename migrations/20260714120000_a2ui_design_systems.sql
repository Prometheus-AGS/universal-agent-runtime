-- A2UI design-system bridge: design systems, base component catalog,
-- per-design-system component overrides, and component embeddings.
--
-- Ported from flint-forge's `flint_a2ui` schema for the Postgres backend
-- (`postgres-backend` / `sqlx` Cargo feature). Backs
-- `src/uar/a2ui/design_systems/store.rs`'s `PostgresDesignSystemStore`.
--
-- Source: flint-forge/migrations/0009_flint_a2ui_design_systems.sql, adapted:
--   - Dropped the `flint_a2ui.` schema prefix (tables live in the default
--     schema alongside UAR's other tables, matching this repo's convention).
--   - `flint_a2ui.design_systems` / `flint_a2ui.components` did not exist in
--     UAR (they were created by earlier flint-forge migrations 0002/0004/0006
--     that are out of scope for this change), so this migration also creates
--     minimal `a2ui_design_systems` and `a2ui_components` tables rather than
--     assuming they already exist. See the change proposal's "Out of scope"
--     section for the full component-catalog-vendoring note.
--   - RLS policies dropped: UAR's Postgres backend does not use Postgres
--     row-level security for multi-tenancy (see `src/uar/security/`); no
--     equivalent policy exists elsewhere in this repo's migrations.
--   - `embeddings` renamed to `a2ui_component_embeddings` and narrowed to one
--     row per component (flint-forge's schema supports multiple aspects per
--     entity; this port only embeds the component "description" aspect, matching
--     `src/uar/a2ui/design_systems/embedder.rs`'s scope).

CREATE TABLE IF NOT EXISTS a2ui_design_systems (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    tokens          JSONB NOT NULL DEFAULT '{}',
    -- 'design_md' | 'w3c_tokens' | 'figma_tokens' | 'manual'
    source_format   TEXT NOT NULL DEFAULT 'manual'
        CHECK (source_format IN ('design_md', 'w3c_tokens', 'figma_tokens', 'manual')),
    source_content  TEXT,
    imported_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS a2ui_components (
    id              TEXT PRIMARY KEY,
    slug            TEXT NOT NULL UNIQUE,
    primitive_type  TEXT NOT NULL,
    category        TEXT NOT NULL,
    schema          JSONB NOT NULL DEFAULT '{}',
    description     TEXT,
    usage_examples  JSONB,
    -- { "react": bool, "flutter": bool, "htmx": bool }
    renderers       JSONB NOT NULL DEFAULT '{"react": true, "flutter": true, "htmx": true}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS a2ui_component_overrides (
    id                  TEXT PRIMARY KEY,
    design_system_id    TEXT NOT NULL
        REFERENCES a2ui_design_systems(id) ON DELETE CASCADE,
    component_id        TEXT NOT NULL
        REFERENCES a2ui_components(id) ON DELETE CASCADE,
    prop_defaults        JSONB NOT NULL DEFAULT '{}',
    css_vars             JSONB NOT NULL DEFAULT '{}',
    react_component      TEXT,
    flutter_widget       TEXT,
    htmx_template        TEXT,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (design_system_id, component_id)
);

CREATE INDEX IF NOT EXISTS a2ui_component_overrides_ds_idx
    ON a2ui_component_overrides (design_system_id);

-- One embedding row per component ("description" aspect only — see header
-- note). Vector dimension left generic (`pgvector::Vector` bound at query
-- time) since UAR's `EmbeddingBackend` trait supports multiple providers
-- with different output dimensions (384 for local BGE, 1536 for OpenAI
-- text-embedding-3-*, etc.), unlike flint-forge's fixed `vector(1536)`.
CREATE TABLE IF NOT EXISTS a2ui_component_embeddings (
    component_id    TEXT PRIMARY KEY
        REFERENCES a2ui_components(id) ON DELETE CASCADE,
    embedding       vector NOT NULL,
    model           TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
