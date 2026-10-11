# Projects for the Universal Agent Runtime — feature and data-model design

Status: **proposed**. The runtime has no project entity today. This document describes the feature, the
scoping rules, and the changes to the domain model, the `PersistenceLayer` trait, both persistence
adapters (PostgreSQL and SurrealDB), Cedar, the HTTP API and the web client. The prototype screen is
`projects.html`.

Date: 2026-10-05. Sources are listed at the end; claims about other products are marked where they
could not be verified first-hand.

---

## 1. What exists today (the starting point)

Verified in the repository:

- **No project, workspace, member or grant model.** Isolation is single-owner: almost every
  `PersistenceLayer` method takes `owner_id`, and records are keyed `(owner_id, id)`
  (`src/uar/persistence/mod.rs`, `tenant_storage_key`). There is no users table; the principal comes from
  JWT claims (`UserContext { user_id, tenant_id, claims }`).
- **Two owner-key conventions coexist.** Knowledge, memory and discovery use the raw `user.user_id`;
  collaboration, user settings and principal conversation policies use `principal_storage_key(user)`
  (`"v1:t:{len}:{tenant}:s:{len}:{sub}"`). Projects must pick one — this design uses the principal key.
- **`workspace_id` is an opaque string** on agent instances and observers, read from the
  `x-uar-workspace-id` header. There is no workspace table.
- **Conversations are `sessions` rows** (`id`, `data JSONB`) with tenant-keyed ids; there is no separate
  conversations table. Run policy is persisted only at conversation scope
  (`conversation_policies`, `principal_conversation_policies`). `PolicyScope` is
  `Global | Agent | Conversation | Turn | Legacy`.
- **Knowledge bases and documents are owner-scoped** (`20260818000000_knowledge_tenant_ownership.sql`,
  the best template for adding columns to existing tables). Agents and skills are global.
- **Memory** comes from `surreal-memory` with `MemoryScope { Global, Agent, User, Session, Task }`;
  the Postgres `memories` table has no owner.
- **Cost ledger** already takes a free-form `(scope, scope_id)`.
- **`project_instructions.rs`** loads `AGENTS.md` from a trusted filesystem workspace for a session. It is
  not persisted and is a different thing from the project brief proposed here (see §4.3).
- **Cedar entity types:** `Agent` (principal), `Action`, `Tool`, `Skill`. There are no User, Project or
  Conversation types.
- **Adapters:** Postgres runs sqlx migrations from `migrations/` (`YYYYMMDDHHMMSS_name.sql`, `TEXT` ids,
  `TIMESTAMPTZ`, `data JSONB`, composite `(owner_id, …)` keys, no RLS). Surreal runs the
  `migrations/surrealdb/*.surql` files listed in `SurrealDbProvider::new_with_auth` at connect time,
  `SCHEMALESS` tables with `DEFINE INDEX … FIELDS owner_id, <id> UNIQUE`, records keyed
  `(table, tenant_storage_key(owner, id))`, no `RELATE` edges in use.
- **Web client** keeps threads in PGlite (`threads`, `messages`, `run`, `run_event`; migrations v1–v3).

## 2. What leading products converge on

From first-party documentation (ChatGPT Projects, Claude Projects, Claude Code Projects, Claude Cowork,
Perplexity Projects, Gemini Notebooks, Open WebUI, LibreChat, LangGraph):

| Concern | Convergent behaviour |
|---|---|
| Contents | name/icon, instructions ("brief"), files or knowledge, the chats that belong to it, and increasingly its own memory, goal, connectors/tools, skills, default model or mode |
| Instructions | project instructions override personal custom instructions inside the project; repository/agent rules load on top |
| Files | uploads are copies owned by the project; large sets switch to retrieval automatically |
| Memory | per-project memory is now standard; shared projects are forced to project-only memory (ChatGPT) |
| Roles | Owner · Edit · View/Chat; group and individual grants resolve to the higher role |
| Move/copy | moving a chat in makes it inherit the project's instructions and files; some kinds of chat cannot move |
| Lifecycle | Claude Code separates **Pause** (stop work), **Archive** (hide, stop, restorable) and **Delete** |
| Work view | Claude Code Projects show a board: ready for review · waiting on you · working · resolved, with per-project usage |

What users complain about (mostly community reports, so treat as signals, not facts):
memory leaking across projects ("project-only" behaving like a retrieval hint, not a boundary),
project instructions being ignored because they are prompt text rather than enforced rules, silent
retrieval misses, stale copied files, chats that cannot be moved, and background cost surprises.

The design below takes the convergent shape and fixes those complaints with enforcement the runtime
already has: owner-scoped storage, `RunPolicy`, Cedar and the cost ledger.

## 3. The feature

A **project** is a named container that scopes conversations, runs, files, knowledge, agents and tools,
instructions, memory, members, tasks and budget. Everything a conversation does inside a project is
bounded by the project.

### 3.1 What a project holds

| Part | Meaning | Stored as |
|---|---|---|
| Identity | name, slug, icon, description, goal | `projects` |
| Brief | the project instructions every new conversation starts from | `projects.brief` |
| Defaults | conversation target (orchestrator with routing on/off, or a pinned agent), default model, tool approval default | `project_policies` (a `RunPolicy` at the new `Project` scope) |
| Resource ceiling | which agents, knowledge bases, skills, MCP servers and A2UI presentations conversations may use | `project_resources` |
| Project files | uploads owned by the project, indexed like any knowledge base | a project-owned knowledge base (`knowledge_bases.project_id`) |
| Linked knowledge | existing knowledge bases referenced, **live** or **pinned** to a revision | `project_resources` with `binding_mode` |
| Conversations | chats that belong to the project; at most one project per conversation | `project_conversations` |
| Memory | project-only, or project plus personal memory | `projects.memory_mode`; memories carry `project_id` |
| Members | principals and roles | `project_members` |
| Tasks | work items assigned to a person or an agent, linked to a conversation or run | `project_tasks` |
| Budget | monthly limit, hard or soft enforcement | `projects.budget_*`; spend from the cost ledger (`scope = "project"`) |
| Activity | append-only audit of membership, settings, moves, lifecycle | `project_events` |

### 3.2 Roles

| Role | Can |
|---|---|
| Owner | everything, including members, budget, delete; a project always keeps at least one owner |
| Editor | change brief, defaults, resources, files, tasks; invite members up to Editor |
| Runner | chat and start runs inside the project; add tasks; read files and settings |
| Viewer | read conversations, files, board and settings |

Group and individual grants resolve to the higher role. Roles are enforced in Cedar (§6), not in the
client.

### 3.3 Lifecycle

| State | Effect | Reversible |
|---|---|---|
| `active` | normal | — |
| `paused` | live runs are cancelled at the next checkpoint; new runs and scheduled work are refused; everything stays readable and editable | yes |
| `archived` | hidden from the default list; runs stopped; read-only for everyone | yes (Owner) |
| `deleted` | soft-deleted; `purge_after = deleted_at + 30 days`; a purge job then removes the project's conversations, project files, project memory, tasks and events | until purge (Owner) |

Deletion never touches **linked** knowledge bases, agents, skills or MCP servers — they are references.
External side effects that runs already caused (pushed branches, sent messages) are out of scope and
the UI says so.

### 3.4 Move and copy semantics

- **Move a conversation in:** future turns apply the project's brief, ceiling, defaults and memory mode.
  Past turns are not rewritten. If the conversation is pinned to an agent the project does not allow, the
  move is refused with a choice: add the agent to the project, or switch the conversation to the
  orchestrator.
- **Move out:** the conversation keeps its history and loses the project's context from the next turn.
- **Copy:** forks the conversation into the target with `copied_from` lineage.
- **Leave a shared project:** offers to copy your conversations into a new private project.
- **Convert a conversation into a project:** creates a project with that conversation as its first member,
  seeding the brief from the conversation's settings.

### 3.5 Work board

The project overview is a board, not a chat list. Columns are derived, not stored:

| Column | Derived from |
|---|---|
| Waiting on you | runs with pending tool approval or HITL gate (`/runs/{id}/tool-approval/pending`, `waiting` status); tasks assigned to you in `blocked` |
| Working | runs `running` / `queued`; tasks `doing` |
| Ready for review | runs `completed` with artifacts not yet acknowledged; tasks `review` |
| Resolved | finished and acknowledged; auto-resolve after 7 days idle |

## 4. Scoping rules

### 4.1 Policy precedence

`PolicyScope` gains `Project`, between `Agent` and `Conversation`:

```
Global  →  Agent  →  Project  →  Conversation  →  Turn
```

Merging rules:

1. Scalar fields (`model`, `tool_approval`, `memory_enabled`, `prompt_caching_enabled`,
   `context_strategy`, `agent_id`, `chat_mode`, the new `routing`) — the most specific scope that sets a
   value wins, as today.
2. `ResourceSelection` fields (`skills`, `tools`, `mcp_servers`, `knowledge_bases`, `presentations`) —
   the **project is a ceiling**. A conversation inside a project can narrow (`selected`, `none`,
   `denied_ids`) but cannot select an id outside the project's resources. The merge intersects with the
   project ceiling after resolving the conversation's selection.
3. `tool_approval` can only become **stricter** below the project (`auto` → `ask` → `deny`), never looser.
4. Cedar decides last. A Cedar forbid is final regardless of any setting.

### 4.2 Conversation target

`RunPolicy` gains `routing: Option<RoutingMode>` with `Enabled | Disabled`. A project sets the default
target: orchestrator with routing on, orchestrator with routing off (everything to `default-agent` with
workspace defaults plus conversation policy), or a pinned `agent_id`. When the orchestrator routes, it may
only choose among agents in the project ceiling.

### 4.3 Instruction stack

The runtime composes, in order, and the UI shows the effective result:

1. Workspace policy (Cedar, guardrails) — **enforced**, not text.
2. Project brief.
3. Agent `prompt.system` and `instructions[]`.
4. Skill prompt overlays.
5. Repository instructions from `project_instructions.rs` (`AGENTS.md`), only when the conversation is
   bound to a trusted filesystem workspace.
6. Conversation settings.
7. The message.

Anything that must hold (no writes to production, ask before deleting) belongs in Cedar or approvals, not
in the brief. The Instructions tab labels each layer as *enforced* or *advisory*.

### 4.4 Memory isolation by construction

- Every memory written inside a project carries `project_id`.
- Retrieval inside a project filters to `project_id = $p`; with `memory_mode = inherit_personal` it may
  also read the caller's personal memories (`project_id IS NULL`, same principal). It never reads another
  project's memories.
- `shared` projects are forced to `project_only`, and the setting cannot be relaxed while shared.
- Isolation is a storage filter, not a prompt hint, which is the complaint this fixes.

### 4.5 Knowledge

- Project files are a knowledge base owned by the project (`knowledge_bases.project_id`,
  `kind = 'project'`). Deleting the project deletes it.
- Linked knowledge bases are references. `binding_mode = live` follows the latest index;
  `pinned` records `pinned_revision`. Each run records the knowledge revisions it used in its context so
  results are reproducible.

## 5. Data model

### 5.1 Domain types (`src/uar/domain/project.rs`, new)

```rust
pub struct Project {
    pub id: String,                 // ULID, stable
    pub owner_id: String,           // partition key: principal_storage_key of the creator
    pub workspace_id: Option<String>,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub goal: Option<String>,
    pub brief: String,              // project instructions, ≤ 8_000 chars
    pub status: ProjectStatus,      // Active | Paused | Archived | Deleted
    pub visibility: Visibility,     // Private | Shared
    pub memory_mode: MemoryMode,    // ProjectOnly | InheritPersonal
    pub budget: Option<ProjectBudget>, // { monthly_limit_usd, enforcement: Hard | Soft }
    pub template_id: Option<String>,
    pub revision: u64,              // optimistic concurrency, like presentations
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub purge_after: Option<DateTime<Utc>>,
}

pub enum ProjectRole { Owner, Editor, Runner, Viewer }
pub struct ProjectMember { project_id, owner_id, principal_key, principal_kind /* User | Group */, role, invited_by, created_at }

pub enum ProjectResourceKind { Agent, KnowledgeBase, Skill, McpServer, Presentation }
pub struct ProjectResource { project_id, owner_id, kind, resource_id, binding_mode /* Live | Pinned */, pinned_revision: Option<String>, required: bool, added_by, created_at }

pub struct ProjectConversation { project_id, owner_id, conversation_id, added_by, added_at, copied_from: Option<String> }

pub enum TaskStatus { Todo, Doing, Blocked, Review, Done }
pub struct ProjectTask { task_id, project_id, owner_id, title, body, status, assignee: Option<Assignee /* User(key) | Agent(id) */>, conversation_id, run_id, due_at, position: i64, created_by, created_at, updated_at, resolved_at }

pub struct ProjectEvent { event_id, project_id, owner_id, kind, actor, payload: Value, occurred_at }
```

`RunPolicy` changes: add `routing: Option<RoutingMode>`; add `PolicyScope::Project`.
`Run.context` gains `project_id` and `knowledge_revisions`.

### 5.2 PostgreSQL migration

`migrations/20261006000000_projects.sql`, following the existing conventions (`TEXT` ids, composite
owner keys, `IF NOT EXISTS`, JSONB `data` for forward-compatible fields, no RLS — the adapter enforces
ownership like every other table).

```sql
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
```

The cost ledger needs no schema change: write entries with `scope = 'project'`, `scope_id = project_id`
alongside the existing scopes, and sum the current month for budget checks.

### 5.3 SurrealDB schema

`migrations/surrealdb/projects.surql`, registered in the list `SurrealDbProvider::new_with_auth`
executes at connect time. It follows the provider's existing style (`SCHEMALESS` tables, unique indexes on
the owner pair, records written through `save_tenant_record` with key
`tenant_storage_key(owner_id, <id>)`; child records use `tenant_storage_key(owner_id, project_id + "/" + child_id)`).
Field-level `ASSERT`s are used only where the adapter cannot otherwise guarantee an invariant.

```sql
-- Projects
DEFINE TABLE IF NOT EXISTS projects SCHEMALESS;
DEFINE INDEX IF NOT EXISTS project_owner_id   ON projects FIELDS owner_id, project_id UNIQUE;
DEFINE INDEX IF NOT EXISTS project_owner_slug ON projects FIELDS owner_id, slug;
DEFINE INDEX IF NOT EXISTS project_workspace  ON projects FIELDS workspace_id;
DEFINE INDEX IF NOT EXISTS project_purge      ON projects FIELDS purge_after;
DEFINE FIELD IF NOT EXISTS status      ON projects TYPE string
    ASSERT $value IN ['active', 'paused', 'archived', 'deleted'];
DEFINE FIELD IF NOT EXISTS memory_mode ON projects TYPE string
    ASSERT $value IN ['project_only', 'inherit_personal'];
DEFINE FIELD IF NOT EXISTS brief       ON projects TYPE string ASSERT string::len($value) <= 8000;

-- Members
DEFINE TABLE IF NOT EXISTS project_members SCHEMALESS;
DEFINE INDEX IF NOT EXISTS project_member_key       ON project_members FIELDS owner_id, project_id, principal_key UNIQUE;
DEFINE INDEX IF NOT EXISTS project_member_principal ON project_members FIELDS principal_key;
DEFINE FIELD IF NOT EXISTS role ON project_members TYPE string
    ASSERT $value IN ['owner', 'editor', 'runner', 'viewer'];

-- Resource ceiling and linked knowledge
DEFINE TABLE IF NOT EXISTS project_resources SCHEMALESS;
DEFINE INDEX IF NOT EXISTS project_resource_key ON project_resources
    FIELDS owner_id, project_id, resource_kind, resource_id UNIQUE;
DEFINE INDEX IF NOT EXISTS project_resource_ref ON project_resources FIELDS resource_kind, resource_id;

-- Conversations (one project per conversation)
DEFINE TABLE IF NOT EXISTS project_conversations SCHEMALESS;
DEFINE INDEX IF NOT EXISTS project_conversation_key  ON project_conversations FIELDS owner_id, project_id, conversation_id UNIQUE;
DEFINE INDEX IF NOT EXISTS project_conversation_once ON project_conversations FIELDS owner_id, conversation_id UNIQUE;

-- Project-scope RunPolicy
DEFINE TABLE IF NOT EXISTS project_policies SCHEMALESS;
DEFINE INDEX IF NOT EXISTS project_policy_key ON project_policies FIELDS owner_id, project_id UNIQUE;

-- Tasks
DEFINE TABLE IF NOT EXISTS project_tasks SCHEMALESS;
DEFINE INDEX IF NOT EXISTS project_task_key   ON project_tasks FIELDS owner_id, project_id, task_id UNIQUE;
DEFINE INDEX IF NOT EXISTS project_task_board ON project_tasks FIELDS owner_id, project_id, status, position;
DEFINE FIELD IF NOT EXISTS status ON project_tasks TYPE string
    ASSERT $value IN ['todo', 'doing', 'blocked', 'review', 'done'];

-- Activity: append-only via table permissions for record users; the root/namespace connection the
-- provider uses still bypasses permissions, so the adapter must only ever CREATE here.
DEFINE TABLE IF NOT EXISTS project_events SCHEMALESS
    PERMISSIONS FOR select, create FULL, FOR update, delete NONE;
DEFINE INDEX IF NOT EXISTS project_event_key  ON project_events FIELDS owner_id, project_id, event_id UNIQUE;
DEFINE INDEX IF NOT EXISTS project_event_time ON project_events FIELDS owner_id, project_id, occurred_at;

DEFINE TABLE IF NOT EXISTS project_templates SCHEMALESS;
DEFINE INDEX IF NOT EXISTS project_template_key ON project_templates FIELDS owner_id, template_id UNIQUE;
```

Existing Surreal tables written by the provider gain plain fields, no schema change required because
they are `SCHEMALESS`: `knowledge_bases.project_id` and `.kind`, `agent_threads.project_id`, and
`project_id` on memory records written through the memory service. Add indexes:

```sql
DEFINE INDEX IF NOT EXISTS knowledge_base_project ON knowledge_bases FIELDS owner_id, project_id;
DEFINE INDEX IF NOT EXISTS agent_thread_project   ON agent_threads   FIELDS owner_id, project_id;
```

**Optional graph view.** Surreal could model membership as
`DEFINE TABLE member_of TYPE RELATION IN principal OUT projects`. This design does **not** do that, because
the provider uses no `RELATE` edges and keeping both adapters on the same row model keeps the
`PersistenceLayer` contract identical. Revisit if a later feature needs graph traversal (for example,
"everything this principal can reach").

**Memory service.** `surreal-memory`'s `MemoryScope` has no project variant. Two options:
(a) add `MemoryScope::Project(id)` upstream in the vendored crate, or (b) keep scope `User` and write
`project_id` as a metadata field with a mandatory filter in `src/uar/memory/service.rs`. Recommend (a):
a scope is checked by the store; a metadata filter can be forgotten by a caller, which is the leak users
report elsewhere.

### 5.4 `PersistenceLayer` additions

All take `owner_id` (the project partition) plus a `principal_key` where access must be checked.
Default implementations return `Error::Unsupported` so adapters can land incrementally, as other groups
do.

```rust
// projects
async fn create_project(&self, project: &Project) -> Result<()>;
async fn load_project(&self, owner_id: &str, project_id: &str) -> Result<Option<Project>>;
async fn compare_and_swap_project(&self, project: &Project, expected_revision: u64) -> Result<bool>;
async fn list_projects_for_principal(&self, principal_key: &str, filter: &ProjectFilter) -> Result<Vec<ProjectSummary>>;
async fn mark_project_deleted(&self, owner_id: &str, project_id: &str, purge_after: DateTime<Utc>) -> Result<()>;
async fn purge_projects_due(&self, now: DateTime<Utc>, limit: usize) -> Result<Vec<(String, String)>>;

// membership
async fn upsert_project_member(&self, member: &ProjectMember) -> Result<()>;
async fn remove_project_member(&self, owner_id: &str, project_id: &str, principal_key: &str) -> Result<()>; // refuses the last owner
async fn list_project_members(&self, owner_id: &str, project_id: &str) -> Result<Vec<ProjectMember>>;
async fn resolve_project_role(&self, owner_id: &str, project_id: &str, principal_keys: &[String]) -> Result<Option<ProjectRole>>; // highest of user + groups

// ceiling, policy, conversations
async fn set_project_resources(&self, owner_id: &str, project_id: &str, kind: ProjectResourceKind, resources: &[ProjectResource]) -> Result<()>;
async fn list_project_resources(&self, owner_id: &str, project_id: &str) -> Result<Vec<ProjectResource>>;
async fn save_project_policy(&self, owner_id: &str, project_id: &str, policy: &RunPolicy, expected_revision: Option<u64>) -> Result<u64>;
async fn load_project_policy(&self, owner_id: &str, project_id: &str) -> Result<Option<(RunPolicy, u64)>>;
async fn attach_conversation(&self, link: &ProjectConversation) -> Result<()>;      // fails if attached elsewhere
async fn detach_conversation(&self, owner_id: &str, conversation_id: &str) -> Result<()>;
async fn project_for_conversation(&self, owner_id: &str, conversation_id: &str) -> Result<Option<String>>;
async fn list_project_conversations(&self, owner_id: &str, project_id: &str) -> Result<Vec<ProjectConversation>>;

// tasks and activity
async fn save_project_task(&self, task: &ProjectTask) -> Result<()>;
async fn list_project_tasks(&self, owner_id: &str, project_id: &str, status: Option<TaskStatus>) -> Result<Vec<ProjectTask>>;
async fn delete_project_task(&self, owner_id: &str, project_id: &str, task_id: &str) -> Result<()>;
async fn append_project_event(&self, event: &ProjectEvent) -> Result<()>;
async fn list_project_events(&self, owner_id: &str, project_id: &str, before: Option<DateTime<Utc>>, limit: usize) -> Result<Vec<ProjectEvent>>;
```

`InMemoryProvider` implements the same set with `DashMap`s for tests. A shared conformance test suite runs
against all three adapters (create/load/CAS, last-owner guard, one-project-per-conversation, ceiling
intersection, purge cascade, memory isolation).

**Owner key.** Projects use `principal_storage_key(user)` for `owner_id`, `created_by` and
`principal_key`. Anonymous principals cannot create projects (the key is `None`), which matches how
principal conversation policies behave.

## 6. Cedar

Add entity types and actions in `src/uar/governance/policy.rs` and a new `policies/projects.cedar`.

```cedar
// Entities: User, Group, Project, Conversation, KnowledgeBase join Agent, Tool, Skill.
// Project entities carry role sets as attributes (owners, editors, runners, viewers: Set<User>),
// filled from project_members when the request is authorized; Tool entities carry their MCP server.

permit (principal, action in [Action::"project.read"], resource is Project)
  when { resource.viewers.contains(principal) || resource.runners.contains(principal)
      || resource.editors.contains(principal) || resource.owners.contains(principal) };

permit (principal, action in [Action::"project.chat", Action::"project.run"], resource is Project)
  when { (resource.runners.contains(principal) || resource.editors.contains(principal) || resource.owners.contains(principal))
      && resource.status == "active" };

permit (principal, action in [Action::"project.edit", Action::"project.files.write"], resource is Project)
  when { (resource.editors.contains(principal) || resource.owners.contains(principal)) && resource.status != "archived" };

permit (principal, action in [Action::"project.members.manage", Action::"project.delete",
                              Action::"project.budget.edit"], resource is Project)
  when { resource.owners.contains(principal) };

// Tools: a run inside a project may only call tools from MCP servers in the project ceiling.
forbid (principal, action == Action::"execute_tool", resource)
  when { context has project && !context.project.mcp_servers.contains(resource.server) };

// Hard budget.
forbid (principal, action in [Action::"call_llm", Action::"execute_tool"], resource)
  when { context has project && context.project.budget_enforcement == "hard"
      && context.project.spend_usd >= context.project.budget_limit_usd };
```

The runtime builds `context.project` from the project, its resources and the month-to-date cost ledger
sum when a run starts inside a project.

## 7. HTTP API (and CLI)

Under `/api/uar/projects`, Axum module `src/uar/api/projects.rs`, mounted in `src/server.rs` next to
`/api/uar/knowledge-bases`. Every handler extracts `UserContext`, computes the principal key, resolves the
role and asks Cedar.

| Method | Path | Purpose |
|---|---|---|
| GET | `/projects?status=&workspace_id=` | projects visible to the caller, with board counts |
| POST | `/projects` | create (optionally `template_id`, `from_conversation_id`) |
| GET · PATCH | `/projects/{id}` | read; RFC 7396 merge patch with `If-Match: <revision>` |
| POST | `/projects/{id}/pause` · `/resume` · `/archive` · `/unarchive` | lifecycle |
| DELETE | `/projects/{id}` | soft delete, returns `purge_after`; `POST /projects/{id}/restore` until then |
| GET · PUT | `/projects/{id}/policy` | project-scope `RunPolicy` |
| GET · PUT | `/projects/{id}/resources` | ceiling and linked knowledge |
| GET · POST · DELETE | `/projects/{id}/members[/{principal}]` | membership |
| GET | `/projects/{id}/conversations` | conversations in the project |
| POST | `/projects/{id}/conversations:move` | `{conversation_ids[], mode: move \| copy}`; returns per-item outcome and refusals |
| GET · POST · PATCH · DELETE | `/projects/{id}/tasks[/{task_id}]` | tasks |
| GET | `/projects/{id}/board` | derived board columns |
| GET | `/projects/{id}/events` | activity |
| GET | `/projects/{id}/usage?month=` | spend by agent and model from the cost ledger |
| GET | `/projects/{id}/effective-instructions?agent_id=` | the composed instruction stack with enforced/advisory labels |
| GET · POST | `/project-templates` | templates |

Existing endpoints accept `project_id`: `POST /api/uar/runs`, `POST /api/chat/completion`,
`POST /api/uar/knowledge-bases` (creates a project files base when `kind = project`), and
`GET /api/uar/sessions/{id}/effective-config` reports the project scope it merged. Contracts go in the
shared DTO layer so the web client and any CLI use the same shapes; every capability above should also be
reachable from the CLI (`uar project list|create|move|members|tasks …`) with `--json`.

## 8. Web client

- **Navigation:** Projects appears in the workspace sidebar and as the Projects group in the chat
  sidebar; each project opens its home (`projects.html` in the prototype).
- **Project home tabs:** Overview (board + tasks + activity), Chats, Knowledge & files, Agents & tools,
  Instructions, Memory, Members, Budget & usage.
- **Chat:** a conversation in a project shows the project name as its breadcrumb; the target picker only
  lists agents in the project ceiling; the route line can say "outside this project's agents" when the
  orchestrator declines.
- **PGlite:** migration v4 adds `projects(id, name, slug, status, updated_at)` as a read cache and
  `threads.project_id TEXT NULL` with an index; the server stays the source of truth.

## 9. Rollout

1. Domain types, Postgres migration, Surreal schema, `PersistenceLayer` methods and the adapter
   conformance tests.
2. `PolicyScope::Project`, ceiling merge, `routing` field, run context `project_id` and knowledge
   revisions.
3. Memory scope (`MemoryScope::Project`) and the retrieval filter.
4. Cedar entities, `projects.cedar`, budget forbid.
5. HTTP API, CLI, contracts.
6. Web client: sidebar, project home, move flow, chat integration.
7. Purge job, templates, convert-conversation-to-project.

## 10. Validation done for this document

- **PostgreSQL 18 + pgvector, throwaway cluster:** all 25 existing files in `migrations/` applied in
  order, then the §5.2 migration applied twice (idempotent). Invariant checks raised the expected errors:
  shared project with `inherit_personal` memory, a conversation in two projects, a pinned binding without
  a revision, `UPDATE` on `project_events`, `deleted` status without `deleted_at`. Deleting a project
  cascaded to members, conversations and events.
- **SurrealDB 3.3.0, in-memory:** the §5.3 statements applied cleanly and re-applied cleanly. Duplicate
  `(owner_id, project_id)`, a conversation in two projects, an unknown status and an unknown role were
  rejected by the indexes and field assertions. Note: on SurrealDB 3 the namespace and database must exist
  before the schema runs; the provider's connect path must select or define them first, as it does for
  the existing `.surql` files.
- Not done: no Rust code was written or compiled, and the Cedar policy in §6 was not run through
  `cedar validate` (the entity schema it needs does not exist yet).

## 11. Open questions

- **Partition owner for shared projects.** This design keeps the creator's principal key as the partition
  (`owner_id`) and grants others through membership. Transferring ownership then means rewriting the
  partition, or treating `owner_id` as immutable and ownership as a role (recommended: the latter).
- **Groups.** There is no group source today; `principal_kind = 'group'` is reserved until identity
  provides groups (for example from Kratos/flint-gate claims).
- **Workspace.** `workspace_id` stays an opaque string until a workspace entity exists.
- **Cross-project knowledge.** Should a knowledge base be linkable from projects owned by other
  principals? This design allows it only within the same partition owner.

## Sources

First-party pages opened during research: help.openai.com/en/articles/10169521-projects-in-chatgpt;
support.claude.com/en/articles/9517075-what-are-projects; code.claude.com/docs/en/claude-projects;
support.claude.com/en/articles/14116274-organize-your-tasks-with-projects-in-claude-cowork;
claude.com/blog/memory; learn.chatgpt.com/docs/environments/local-environment;
perplexity.ai/help-center/en/articles/10352961-what-are-projects;
blog.google/innovation-and-ai/products/gemini-app/notebooks-gemini-notebooklm/;
docs.openwebui.com/features/workspace/; docs.openwebui.com/features/chat-conversations/chat-features/conversation-organization.md;
librechat.ai/docs/features/access_control; docs.langchain.com/langsmith/assistants;
docs.langchain.com/oss/python/langgraph/persistence.
Seen only in search results or community posts (not first-hand): Dify workspace roles, Cursor multi-root
rule leakage, Kimi Projects, reports of ChatGPT project-only memory leaking and of ignored project
instructions.
Runtime repository paths are cited inline.
