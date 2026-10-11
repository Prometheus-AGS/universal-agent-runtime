-- Proposed: Projects (see ../project-management-design.md §5.2). Not yet in migrations/.
-- Projects: owner-partitioned like presentations; membership grants cross-principal access.
CREATE TABLE IF NOT EXISTS projects (
    owner_id        TEXT        NOT NULL,
    project_id      TEXT        NOT NULL,
    workspace_id    TEXT,
    slug            TEXT        NOT NULL,
    name            TEXT        NOT NULL CHECK (char_length(name) BETWEEN 1 AND 120),
    status          TEXT        NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'paused', 'archived', 'deleted')),
    visibility      TEXT        NOT NULL DEFAULT 'private' CHECK (visibility IN ('private', 'shared')),
    memory_mode     TEXT        NOT NULL DEFAULT 'project_only'
                    CHECK (memory_mode IN ('project_only', 'inherit_personal')),
    brief           TEXT        NOT NULL DEFAULT '' CHECK (char_length(brief) <= 8000),
    budget_monthly_usd NUMERIC(12, 2),
    budget_enforcement TEXT     CHECK (budget_enforcement IN ('hard', 'soft')),
    revision        BIGINT      NOT NULL DEFAULT 1 CHECK (revision > 0),
    data            JSONB       NOT NULL DEFAULT '{}'::jsonb,  -- description, icon, goal, template_id
    created_by      TEXT        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at     TIMESTAMPTZ,
    deleted_at      TIMESTAMPTZ,
    purge_after     TIMESTAMPTZ,
    PRIMARY KEY (owner_id, project_id),
    CHECK (visibility = 'private' OR memory_mode = 'project_only'),
    CHECK ((status = 'deleted') = (deleted_at IS NOT NULL))
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_owner_slug
    ON projects (owner_id, slug) WHERE status <> 'deleted';
CREATE INDEX IF NOT EXISTS idx_projects_workspace ON projects (workspace_id) WHERE workspace_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_projects_purge ON projects (purge_after) WHERE purge_after IS NOT NULL;

CREATE TABLE IF NOT EXISTS project_members (
    owner_id        TEXT        NOT NULL,
    project_id      TEXT        NOT NULL,
    principal_key   TEXT        NOT NULL,
    principal_kind  TEXT        NOT NULL DEFAULT 'user' CHECK (principal_kind IN ('user', 'group')),
    role            TEXT        NOT NULL CHECK (role IN ('owner', 'editor', 'runner', 'viewer')),
    invited_by      TEXT        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (owner_id, project_id, principal_key),
    FOREIGN KEY (owner_id, project_id) REFERENCES projects (owner_id, project_id) ON DELETE CASCADE
);
-- "Projects I can see": the access path for every list call.
CREATE INDEX IF NOT EXISTS idx_project_members_principal ON project_members (principal_key);

CREATE TABLE IF NOT EXISTS project_resources (
    owner_id        TEXT        NOT NULL,
    project_id      TEXT        NOT NULL,
    resource_kind   TEXT        NOT NULL
                    CHECK (resource_kind IN ('agent', 'knowledge_base', 'skill', 'mcp_server', 'presentation')),
    resource_id     TEXT        NOT NULL,
    binding_mode    TEXT        NOT NULL DEFAULT 'live' CHECK (binding_mode IN ('live', 'pinned')),
    pinned_revision TEXT,
    required        BOOLEAN     NOT NULL DEFAULT FALSE,
    added_by        TEXT        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (owner_id, project_id, resource_kind, resource_id),
    FOREIGN KEY (owner_id, project_id) REFERENCES projects (owner_id, project_id) ON DELETE CASCADE,
    CHECK ((binding_mode = 'pinned') = (pinned_revision IS NOT NULL))
);
CREATE INDEX IF NOT EXISTS idx_project_resources_resource ON project_resources (resource_kind, resource_id);

-- One project per conversation. conversation_id is the logical id (not the tenant-keyed sessions.id).
CREATE TABLE IF NOT EXISTS project_conversations (
    owner_id        TEXT        NOT NULL,
    project_id      TEXT        NOT NULL,
    conversation_id TEXT        NOT NULL,
    added_by        TEXT        NOT NULL,
    added_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    copied_from     TEXT,
    PRIMARY KEY (owner_id, project_id, conversation_id),
    UNIQUE (owner_id, conversation_id),
    FOREIGN KEY (owner_id, project_id) REFERENCES projects (owner_id, project_id) ON DELETE CASCADE
);

-- Project-scope RunPolicy; mirrors conversation_policies.
CREATE TABLE IF NOT EXISTS project_policies (
    owner_id        TEXT        NOT NULL,
    project_id      TEXT        NOT NULL,
    policy          JSONB       NOT NULL,
    revision        BIGINT      NOT NULL DEFAULT 1,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (owner_id, project_id),
    FOREIGN KEY (owner_id, project_id) REFERENCES projects (owner_id, project_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS project_tasks (
    owner_id        TEXT        NOT NULL,
    project_id      TEXT        NOT NULL,
    task_id         TEXT        NOT NULL,
    title           TEXT        NOT NULL CHECK (char_length(title) BETWEEN 1 AND 200),
    status          TEXT        NOT NULL DEFAULT 'todo'
                    CHECK (status IN ('todo', 'doing', 'blocked', 'review', 'done')),
    assignee_kind   TEXT        CHECK (assignee_kind IN ('user', 'agent')),
    assignee_id     TEXT,
    conversation_id TEXT,
    run_id          TEXT,
    position        BIGINT      NOT NULL DEFAULT 0,
    due_at          TIMESTAMPTZ,
    data            JSONB       NOT NULL DEFAULT '{}'::jsonb,  -- body, labels
    created_by      TEXT        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at     TIMESTAMPTZ,
    PRIMARY KEY (owner_id, project_id, task_id),
    FOREIGN KEY (owner_id, project_id) REFERENCES projects (owner_id, project_id) ON DELETE CASCADE,
    CHECK ((assignee_kind IS NULL) = (assignee_id IS NULL))
);
CREATE INDEX IF NOT EXISTS idx_project_tasks_board ON project_tasks (owner_id, project_id, status, position);
CREATE INDEX IF NOT EXISTS idx_project_tasks_assignee ON project_tasks (assignee_kind, assignee_id) WHERE status <> 'done';

-- Append-only activity/audit.
CREATE TABLE IF NOT EXISTS project_events (
    owner_id        TEXT        NOT NULL,
    project_id      TEXT        NOT NULL,
    event_id        TEXT        NOT NULL,
    kind            TEXT        NOT NULL,
    actor           TEXT        NOT NULL,
    payload         JSONB       NOT NULL DEFAULT '{}'::jsonb,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (owner_id, project_id, event_id),
    FOREIGN KEY (owner_id, project_id) REFERENCES projects (owner_id, project_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_project_events_time ON project_events (owner_id, project_id, occurred_at DESC);
CREATE OR REPLACE FUNCTION project_events_append_only() RETURNS trigger AS $$
BEGIN RAISE EXCEPTION 'project_events is append-only'; END; $$ LANGUAGE plpgsql;
DROP TRIGGER IF EXISTS trg_project_events_append_only ON project_events;
CREATE TRIGGER trg_project_events_append_only BEFORE UPDATE ON project_events
    FOR EACH ROW EXECUTE FUNCTION project_events_append_only();
-- DELETE stays allowed so the cascade from a purged project works.

CREATE TABLE IF NOT EXISTS project_templates (
    owner_id        TEXT        NOT NULL,
    template_id     TEXT        NOT NULL,
    name            TEXT        NOT NULL,
    data            JSONB       NOT NULL,  -- brief, policy, resources, memory_mode
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (owner_id, template_id)
);

-- Project-owned knowledge base (project files). Nullable: ordinary KBs stay unchanged.
ALTER TABLE knowledge_bases ADD COLUMN IF NOT EXISTS project_id TEXT;
ALTER TABLE knowledge_bases ADD COLUMN IF NOT EXISTS kind TEXT NOT NULL DEFAULT 'standard';
DO $$ BEGIN
    ALTER TABLE knowledge_bases ADD CONSTRAINT knowledge_bases_kind_check CHECK (kind IN ('standard', 'project'));
EXCEPTION WHEN duplicate_object THEN NULL; END $$;
CREATE UNIQUE INDEX IF NOT EXISTS idx_knowledge_bases_project
    ON knowledge_bases (owner_id, project_id) WHERE kind = 'project';

-- Memory isolation: owner and project on memories (today neither exists in Postgres).
ALTER TABLE memories ADD COLUMN IF NOT EXISTS owner_id TEXT;
ALTER TABLE memories ADD COLUMN IF NOT EXISTS project_id TEXT;
CREATE INDEX IF NOT EXISTS idx_memories_scope ON memories (owner_id, project_id);

-- Agent threads (run trees) can be listed by project for the board.
ALTER TABLE agent_threads ADD COLUMN IF NOT EXISTS project_id TEXT;
CREATE INDEX IF NOT EXISTS idx_agent_threads_project ON agent_threads (owner_id, project_id) WHERE project_id IS NOT NULL;

