-- Proposed: Projects (see ../project-management-design.md §5.3). Add to the list run by SurrealDbProvider::new_with_auth.
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

DEFINE INDEX IF NOT EXISTS knowledge_base_project ON knowledge_bases FIELDS owner_id, project_id;
DEFINE INDEX IF NOT EXISTS agent_thread_project   ON agent_threads   FIELDS owner_id, project_id;

