# Admin UX & Entity Management Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace isolated Zustand stores with prometheus-entity-management normalized graph, add provider search/sort, agent CRUD with AI compilation, chat agent selection, skill import, tool playground, and global UX polish.

**Architecture:** Local-first entity graph (PGLite cache + provider-aware sync transport) replaces per-page stores. Three-layer model: Components -> Hooks -> Entity Graph Store. Agent definitions support per-session overrides in the chat interface.

**Tech Stack:** React 18, @prometheus-ags/prometheus-entity-management 1.2.0, @electric-sql/pglite 0.3.15, Zustand 5, Axum (Rust), SurrealDB/Postgres, SSE/WebSocket/ElectricSQL sync

---

## Phase 0: Global UX Quick Wins

### Task 0.1: Global Cursor Pointer Fix

**Files:**
- Modify: `frontend/src/index.css:156-224`

**Step 1: Add cursor rule to base layer**

In `frontend/src/index.css`, add after the existing `@layer base` block (after line 154):

```css
/* Global interactive cursor */
button,
[role="button"],
a,
[tabindex="0"],
input[type="checkbox"],
input[type="radio"],
select,
summary,
label[for] {
  cursor: pointer;
}
```

**Step 2: Verify in browser**

Run: `bun run build && cargo run --bin universal-agent-runtime`
Open http://localhost:3002/admin and hover over sidebar nav items, provider cards, buttons, toggles.
Expected: All interactive elements show pointer cursor.

**Step 3: Commit**

```bash
git add frontend/src/index.css
git commit -m "fix: add cursor:pointer to all interactive elements globally"
```

---

### Task 0.2: Rename LLM Settings Category

**Files:**
- Modify: backend settings type registration (find where "LLM (liter-llm)" is defined)
- Grep: `rg "liter-llm\|LLM.*liter" src/ frontend/src/`

**Step 1: Find and update the display name**

Search for the settings type definition that registers "LLM (liter-llm)" as a settings category name. Update the display name to "LLM Configuration" and the description to "Global defaults for LLM model, protocol, timeouts, and cost tracking".

**Step 2: Verify the settings page shows the new name**

Open http://localhost:3002/admin -> Settings. The first item should read "LLM Configuration" instead of "LLM (liter-llm)".

**Step 3: Commit**

```bash
git add -A
git commit -m "fix: rename LLM settings category from 'LLM (liter-llm)' to 'LLM Configuration'"
```

---

## Phase 1: Entity Graph Bootstrap

### Task 1.1: Define Entity Types

**Files:**
- Create: `frontend/src/entities/types.ts`

**Step 1: Write entity type definitions**

```ts
// frontend/src/entities/types.ts
// Canonical entity types for the normalized graph.
// These mirror backend API response shapes.

export interface ProviderEntity {
  id: string;
  display_name: string;
  base_url?: string;
  configured: boolean;
  auth_env_var?: string;
  endpoints: string[];
  model_count: number;
}

export interface ModelEntity {
  id: string;
  name: string;
  provider_id: string;
  context: number;
  tool_call: boolean;
  reasoning: boolean;
  vision: boolean;
}

export interface AgentEntity {
  id: string;
  name: string;
  description: string;
  system_prompt: string;
  model?: string;
  protocol?: "auto" | "openai-chat" | "openai-responses";
  skills: string[];
  tools: string[];
  knowledge_bases: string[];
  mcp_servers: string[];
  context_strategy: ContextStrategy;
  tool_approval: "auto" | "ask" | "deny";
  status: "active" | "draft" | "disabled";
  spec_id?: string;
  created_at: string;
  updated_at: string;
}

export interface ContextStrategy {
  max_history_messages: number;
  inject_memory: boolean;
  inject_knowledge: boolean;
  memory_scope: "session" | "agent" | "global";
  auto_capture: boolean;
}

export interface AgentSessionEntity {
  id: string;
  agent_id: string;
  session_id: string;
  model?: string;
  skills?: string[];
  tools?: string[];
  knowledge_bases?: string[];
  mcp_servers?: string[];
  context_strategy?: Partial<ContextStrategy>;
  tool_approval?: "auto" | "ask" | "deny";
}

export interface SkillEntity {
  id: string;
  title: string;
  version: string;
  description: string;
  triggers: { keywords: string[]; semantic?: boolean };
  prompt_overlay?: string;
  preferred_tools: string[];
  enabled: boolean;
  provider_id?: string;
  source?: string;
  source_path?: string;
}

export interface ToolEntity {
  id: string;
  name: string;
  description: string;
  namespace: string;
  input_schema: Record<string, unknown>;
  output_schema?: Record<string, unknown>;
  transport: "internal" | "http" | "mcp";
  built_in: boolean;
}

export interface KnowledgeBaseEntity {
  id: string;
  name: string;
  description?: string;
  document_count: number;
  config?: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface DocumentEntity {
  id: string;
  kb_id: string;
  filename: string;
  status: "pending" | "processing" | "indexed" | "failed";
  chunk_count: number;
  mime_type?: string;
  error_message?: string;
  created_at: string;
  updated_at: string;
}

export interface ThreadEntity {
  id: string;
  title: string;
  agent_id?: string;
  agent_name?: string;
  last_message_preview?: string;
  created_at: string;
  updated_at: string;
}
```

**Step 2: Commit**

```bash
git add frontend/src/entities/types.ts
git commit -m "feat: define entity types for normalized graph"
```

---

### Task 1.2: Create Schema Registry

**Files:**
- Create: `frontend/src/entities/schemas.ts`

**Step 1: Write schema registration**

```ts
// frontend/src/entities/schemas.ts
import { registerSchema } from "@prometheus-ags/prometheus-entity-management";

export function registerAllSchemas() {
  registerSchema({
    type: "Provider",
    idField: "id",
    relations: [
      { type: "Model", field: "provider_id", kind: "hasMany" },
    ],
  });

  registerSchema({
    type: "Model",
    idField: "id",
    relations: [
      { type: "Provider", field: "provider_id", kind: "belongsTo" },
    ],
  });

  registerSchema({
    type: "Agent",
    idField: "id",
    relations: [
      { type: "Skill", field: "skills", kind: "hasMany" },
      { type: "Tool", field: "tools", kind: "hasMany" },
      { type: "KnowledgeBase", field: "knowledge_bases", kind: "hasMany" },
    ],
  });

  registerSchema({
    type: "AgentSession",
    idField: "id",
    relations: [
      { type: "Agent", field: "agent_id", kind: "belongsTo" },
      { type: "Thread", field: "session_id", kind: "belongsTo" },
    ],
  });

  registerSchema({
    type: "Skill",
    idField: "id",
  });

  registerSchema({
    type: "Tool",
    idField: "id",
  });

  registerSchema({
    type: "KnowledgeBase",
    idField: "id",
    relations: [
      { type: "Document", field: "kb_id", kind: "hasMany" },
    ],
  });

  registerSchema({
    type: "Document",
    idField: "id",
    relations: [
      { type: "KnowledgeBase", field: "kb_id", kind: "belongsTo" },
    ],
  });

  registerSchema({
    type: "Thread",
    idField: "id",
    relations: [
      { type: "AgentSession", field: "session_id", kind: "hasMany" },
    ],
  });
}
```

**Step 2: Commit**

```bash
git add frontend/src/entities/schemas.ts
git commit -m "feat: register entity schemas with relations"
```

---

### Task 1.3: Create Entity Engine Bootstrap

**Files:**
- Create: `frontend/src/entities/bootstrap.ts`

**Step 1: Write bootstrap module**

```ts
// frontend/src/entities/bootstrap.ts
import { configureEngine, startLocalFirstGraph } from "@prometheus-ags/prometheus-entity-management";
import { registerAllSchemas } from "./schemas";

let initialized = false;

export async function bootstrapEntityGraph() {
  if (initialized) return;
  initialized = true;

  // Configure the engine with base fetch wrapper and cache settings
  configureEngine({
    baseFetch: async (url: string, init?: RequestInit) => {
      const res = await fetch(url, init);
      if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
      return res.json();
    },
    staleTime: 30_000,
    retry: 2,
  });

  // Register all entity schemas and relations
  registerAllSchemas();

  // Start local-first graph with PGLite persistence
  // PGLite is already initialized via DbProvider in main.tsx
  await startLocalFirstGraph({
    storage: "pglite",
  });
}
```

**Step 2: Wire into app entry point**

In `frontend/src/main.tsx`, import and call `bootstrapEntityGraph()` inside the app initialization, before React renders. Add it to the existing `useEffect` or initialization block.

**Step 3: Verify app still loads without errors**

Run: `bun run build`
Expected: No TypeScript errors. App loads normally at http://localhost:3002.

**Step 4: Commit**

```bash
git add frontend/src/entities/bootstrap.ts frontend/src/main.tsx
git commit -m "feat: bootstrap entity graph engine at app init"
```

---

### Task 1.4: Create Sync Transport Detection

**Files:**
- Create: `frontend/src/entities/sync.ts`

**Step 1: Write sync transport detector**

```ts
// frontend/src/entities/sync.ts
import {
  getRealtimeManager,
  createWebSocketAdapter,
} from "@prometheus-ags/prometheus-entity-management";

interface PersistenceInfo {
  provider: "surreal" | "postgres";
  mode: "embedded" | "remote";
  database_url?: string;
}

// Custom SSE adapter for embedded SurrealDB
function createSSEAdapter(url: string) {
  return {
    type: "sse" as const,
    connect() {
      const eventSource = new EventSource(url);
      const manager = getRealtimeManager();

      eventSource.onmessage = (event) => {
        try {
          const change = JSON.parse(event.data);
          manager.applyChanges([{
            type: change.entity_type,
            id: change.id,
            action: change.action, // "create" | "update" | "delete"
            data: change.data,
          }]);
        } catch {
          // Skip malformed events
        }
      };

      return () => eventSource.close();
    },
  };
}

export async function initSyncTransport(): Promise<() => void> {
  let info: PersistenceInfo;
  try {
    const res = await fetch("/api/config/persistence");
    if (!res.ok) return () => {};
    info = await res.json();
  } catch {
    // Fallback: no realtime sync, rely on REST polling + staleTime
    return () => {};
  }

  const manager = getRealtimeManager({ flushInterval: 16 });

  if (info.provider === "postgres") {
    // ElectricSQL sync -- will be implemented in Phase 6
    // For now, return no-op; REST polling handles data freshness
    return () => {};
  }

  if (info.provider === "surreal" && info.mode === "remote" && info.database_url) {
    // Direct WebSocket to SurrealDB server
    const wsUrl = info.database_url.replace(/^https?/, "ws") + "/rpc";
    const adapter = createWebSocketAdapter({ url: wsUrl });
    const unregister = manager.register(adapter);
    return unregister;
  }

  // surreal + embedded -> SSE bridge
  const adapter = createSSEAdapter("/api/uar/sync/stream");
  const cleanup = adapter.connect();
  return cleanup;
}
```

**Step 2: Commit**

```bash
git add frontend/src/entities/sync.ts
git commit -m "feat: sync transport detection for SurrealDB/Postgres"
```

---

## Phase 2: Migrate Providers & Models to Entity Graph

### Task 2.1: Create Provider Entity Hooks

**Files:**
- Create: `frontend/src/entities/hooks/use-providers.ts`

**Step 1: Write provider hooks using entity graph**

```ts
// frontend/src/entities/hooks/use-providers.ts
import { useEntityView, useEntityCRUD, useEntityList } from "@prometheus-ags/prometheus-entity-management";
import type { ProviderEntity, ModelEntity } from "../types";

const EMPTY_MODELS: ModelEntity[] = [];

export function useProviders(searchTerm: string, filter: "all" | "configured" | "unconfigured") {
  const filterClauses = filter === "all" ? [] : [
    { field: "configured", operator: "eq" as const, value: filter === "configured" },
  ];

  const view = useEntityView<ProviderEntity>("Provider", {
    sort: [
      { field: "configured", direction: "desc" },
      { field: "display_name", direction: "asc" },
    ],
    search: searchTerm ? { fields: ["display_name", "id"], query: searchTerm } : undefined,
    filter: filterClauses.length > 0 ? { clauses: filterClauses } : undefined,
  });

  return view;
}

export function useProviderModelsEntity(providerId: string) {
  const list = useEntityList<ModelEntity>("Model", {
    filter: { clauses: [{ field: "provider_id", operator: "eq", value: providerId }] },
    sort: [{ field: "name", direction: "asc" }],
  });

  return {
    models: list.entities ?? EMPTY_MODELS,
    loading: list.loading,
  };
}
```

**Step 2: Commit**

```bash
git add frontend/src/entities/hooks/use-providers.ts
git commit -m "feat: entity graph hooks for providers and models"
```

---

### Task 2.2: Create Entity Fetcher Registration for Providers

**Files:**
- Create: `frontend/src/entities/fetchers/providers.ts`

**Step 1: Write fetcher that loads providers into entity graph**

```ts
// frontend/src/entities/fetchers/providers.ts
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { fetchCatalog, fetchConfiguredProviders } from "@/services/providers-api";
import { fetchModelsCatalog, modelsRowsForProvider } from "@/services/models-api";

export async function loadProvidersIntoGraph() {
  const [catalogData, providersData] = await Promise.all([
    fetchCatalog(),
    fetchConfiguredProviders(),
  ]);

  const configuredIds = new Set(
    (providersData.providers ?? []).map((p: { id: string }) => p.id)
  );

  const store = useGraphStore.getState();

  // Upsert all providers
  for (const p of catalogData.providers) {
    store.upsertEntity("Provider", p.id, {
      ...p,
      configured: configuredIds.has(p.id),
    });
  }

  return {
    defaultId: providersData.default_id,
    count: catalogData.providers.length,
  };
}

export async function loadModelsForProvider(providerId: string) {
  const data = await fetchModelsCatalog();
  const rows = modelsRowsForProvider(data, providerId);
  const store = useGraphStore.getState();

  for (const m of rows) {
    store.upsertEntity("Model", m.id, {
      ...m,
      provider_id: providerId,
    });
  }
}
```

**Step 2: Commit**

```bash
git add frontend/src/entities/fetchers/providers.ts
git commit -m "feat: entity fetchers for providers and models"
```

---

### Task 2.3: Add Search to Providers Page

**Files:**
- Modify: `frontend/src/admin/pages/providers-page.tsx`

**Step 1: Add search input state and filter the provider list**

Add a `searchTerm` state variable alongside the existing `filter` state. Add a search `<Input>` component above the filter tabs in the provider sidebar. Wire the search into the provider list filtering logic so it filters by `display_name` and `id` (case-insensitive substring match). Ensure configured providers always sort above unconfigured.

**Step 2: Verify search works**

Open http://localhost:3002/admin -> Providers. Type "open" in search. Expected: only providers matching "open" appear (e.g., OpenAI, OpenRouter). Configured ones appear first.

**Step 3: Commit**

```bash
git add frontend/src/admin/pages/providers-page.tsx
git commit -m "feat: add search and configured-first sorting to providers list"
```

---

## Phase 3: Agent CRUD

### Task 3.1: Add Agent CRUD Backend Endpoints

**Files:**
- Modify: `src/uar/api/agents.rs` (or create if module doesn't exist)
- Modify: `src/server.rs` (register new routes)

**Step 1: Implement POST /api/agents (create)**

Add a handler that accepts an `AgentDefinition` JSON body, generates an ID, persists to the persistence layer, and returns the created agent.

**Step 2: Implement PUT /api/agents/{id} (full update)**

Add a handler that accepts a full `AgentDefinition` body and overwrites the existing agent.

**Step 3: Implement DELETE /api/agents/{id}**

Add a handler that deletes the agent by ID.

**Step 4: Implement POST /api/agents/{id}/compile**

Add a handler that takes the existing agent config, feeds it to the CompilerService, and stores the resulting spec. Links spec_id back to the agent.

**Step 5: Register routes in server.rs**

Add the new routes under `/api/agents` alongside existing ones.

**Step 6: Test with curl**

```bash
# Create
curl -X POST http://localhost:3002/api/agents \
  -H 'Content-Type: application/json' \
  -d '{"name":"Test Agent","description":"A test","system_prompt":"You are helpful","skills":[],"tools":[],"knowledge_bases":[],"mcp_servers":[],"context_strategy":{"max_history_messages":50,"inject_memory":true,"inject_knowledge":true,"memory_scope":"session","auto_capture":true},"tool_approval":"auto","status":"active"}'

# List
curl http://localhost:3002/api/agents

# Delete
curl -X DELETE http://localhost:3002/api/agents/{id}
```

**Step 7: Commit**

```bash
git add src/uar/api/ src/server.rs
git commit -m "feat: add agent CRUD endpoints (POST, PUT, DELETE, compile)"
```

---

### Task 3.2: Agent Editor Frontend — Multi-Tab Form

**Files:**
- Create: `frontend/src/admin/components/agent-editor.tsx`
- Modify: `frontend/src/admin/pages/agents-page.tsx`

**Step 1: Build AgentEditor component**

Create a multi-tab form component with tabs: Identity, Prompt, Capabilities, Context, Governance, Spec. Use the existing Dialog/Sheet and Tab components from shadcn. Each tab maps to fields from `AgentEntity`.

- **Identity tab:** name (Input), description (Textarea), status (Select), model (Select dropdown populated from Model entities)
- **Prompt tab:** system_prompt (full-height Textarea with markdown preview toggle)
- **Capabilities tab:** skills, tools, knowledge_bases, mcp_servers — each as a searchable multi-select chip list. Load available options from entity graph.
- **Context tab:** max_history_messages (number slider), inject_memory (Switch), inject_knowledge (Switch), memory_scope (Select), auto_capture (Switch)
- **Governance tab:** tool_approval (RadioGroup: auto/ask/deny)
- **Spec tab:** read-only code block showing compiled spec if spec_id exists, "Compile with AI" button

**Step 2: Wire into agents-page.tsx**

Add "New Agent" button in the agents page header. Clicking opens AgentEditor in create mode. Clicking an existing agent opens it in edit mode. Add delete confirmation dialog.

**Step 3: Commit**

```bash
git add frontend/src/admin/components/agent-editor.tsx frontend/src/admin/pages/agents-page.tsx
git commit -m "feat: agent editor with multi-tab form (identity, prompt, capabilities, context, governance)"
```

---

### Task 3.3: AI-Assisted Agent Compilation

**Files:**
- Create: `frontend/src/admin/components/agent-ai-builder.tsx`
- Modify: `frontend/src/admin/pages/agents-page.tsx`

**Step 1: Build conversational agent builder component**

Create a component with a text area where the user describes what they want the agent to do in natural language. On submit, send to `POST /api/a2a/compiler` with method `message/send` using the `uar.compile.conversational` skill. Display the compiler's responses inline. When compilation completes, parse the resulting agent spec and pre-fill the AgentEditor form.

**Step 2: Add "Create with AI" button to agents page**

Next to "New Agent", add "Create with AI" button that opens the AI builder panel.

**Step 3: Test the flow**

Type a description like "An agent that searches the web and summarizes results". Expected: compiler returns structured output, AgentEditor opens with pre-filled fields.

**Step 4: Commit**

```bash
git add frontend/src/admin/components/agent-ai-builder.tsx frontend/src/admin/pages/agents-page.tsx
git commit -m "feat: AI-assisted agent creation via compiler service"
```

---

## Phase 4: Chat Interface Agent Selection

### Task 4.1: Agent Session Backend Endpoints

**Files:**
- Create: `src/uar/api/agent_sessions.rs`
- Modify: `src/server.rs`

**Step 1: Implement agent session endpoints**

```
POST /api/sessions/{id}/agent-session    -- create/update
GET  /api/sessions/{id}/agent-session    -- get current
GET  /api/sessions/{id}/effective-config -- merged agent def + session overrides
```

The effective-config endpoint resolves `merge(agentDefinition, agentSession)` and returns the fully resolved configuration the orchestrator should use.

**Step 2: Wire into orchestrator**

Modify the run creation flow (`POST /api/uar/runs`) to resolve the effective config from the agent session when a `session_id` is provided, instead of requiring a full artifact inline.

**Step 3: Test with curl**

```bash
curl -X POST http://localhost:3002/api/sessions/thread-123/agent-session \
  -H 'Content-Type: application/json' \
  -d '{"agent_id":"research-agent","tools":["tavily__tavily_search"]}'

curl http://localhost:3002/api/sessions/thread-123/effective-config
```

**Step 4: Commit**

```bash
git add src/uar/api/agent_sessions.rs src/server.rs
git commit -m "feat: agent session endpoints with effective config resolution"
```

---

### Task 4.2: Agent Selector Dropdown in Chat

**Files:**
- Create: `frontend/src/features/chat/agent-selector.tsx`
- Modify: `frontend/src/pages/chat-page.tsx`

**Step 1: Build AgentSelector component**

A dropdown/combobox that lists all active agents. Grouped into "Recently used" (top 3) and "All agents" (alphabetical). Each item shows agent name, model badge, description snippet. Selection creates/updates the AgentSession for the current thread.

**Step 2: Add to chat page header**

Place the AgentSelector at the top of the chat area (between the header and messages). Show current agent name with a dropdown arrow.

**Step 3: Wire agent selection to thread creation**

When user clicks "New thread", show agent selector first. After selection, create thread + agent session, then open the thread.

**Step 4: Commit**

```bash
git add frontend/src/features/chat/agent-selector.tsx frontend/src/pages/chat-page.tsx
git commit -m "feat: agent selector dropdown in chat interface"
```

---

### Task 4.3: Input Bar Capability Toggles

**Files:**
- Create: `frontend/src/features/chat/capability-toggles.tsx`
- Modify: `frontend/src/pages/chat-page.tsx`

**Step 1: Build CapabilityToggles component**

A row of compact toggle buttons below the message input:
- Knowledge Bases (popover with KB checklist)
- Tools (popover with tool toggle list)
- Skills (popover with skill toggle list)
- Web Search (single toggle)
- Memory (single toggle)

Each toggle shows a count badge. Changes update the AgentSession entity.

**Step 2: Add below message input in chat page**

Wire the toggles into the chat page layout, positioned between the input and the bottom of the viewport.

**Step 3: Commit**

```bash
git add frontend/src/features/chat/capability-toggles.tsx frontend/src/pages/chat-page.tsx
git commit -m "feat: per-session capability toggles in chat input bar"
```

---

### Task 4.4: Enhanced Thread Sidebar

**Files:**
- Modify: `frontend/src/components/layout/left-sidebar.tsx`

**Step 1: Enhance thread list items**

Each thread item should show: title, agent name/icon, last message preview (truncated), timestamp. Add a right-click context menu with: Rename, Delete, Change Agent.

**Step 2: Commit**

```bash
git add frontend/src/components/layout/left-sidebar.tsx
git commit -m "feat: enhanced thread sidebar with agent info and context menu"
```

---

### Task 4.5: Per-Session Config Panel

**Files:**
- Create: `frontend/src/features/chat/session-config-panel.tsx`
- Modify: `frontend/src/pages/chat-page.tsx`

**Step 1: Build SessionConfigPanel as a sheet/slide-over**

Triggered by a gear icon next to the agent selector. Shows:
- Current agent (with "Change" button)
- Model override dropdown
- Context strategy overrides (history window, memory toggles)
- Tool approval mode
- Active KBs, tools, skills lists with toggles

All changes persist to AgentSession entity.

**Step 2: Add gear icon to chat header**

**Step 3: Commit**

```bash
git add frontend/src/features/chat/session-config-panel.tsx frontend/src/pages/chat-page.tsx
git commit -m "feat: per-session configuration panel in chat interface"
```

---

## Phase 5: SSE Sync Bridge (Embedded SurrealDB)

### Task 5.1: Backend SSE Sync Endpoint

**Files:**
- Create: `src/uar/api/sync.rs`
- Modify: `src/server.rs`

**Step 1: Add persistence info endpoint**

```
GET /api/config/persistence
Response: { "provider": "surreal", "mode": "embedded", "database_url": "rocksdb://..." }
```

Reads from `AppConfig.persistence` to determine provider and mode. Mode is "embedded" if database_url starts with `rocksdb://` or `mem://`, otherwise "remote".

**Step 2: Add SSE sync stream endpoint**

```
GET /api/uar/sync/stream
```

For embedded SurrealDB: start LIVE SELECT queries on key tables (knowledge_bases, knowledge_documents, skills, agents). Marshal changes into SSE events with format:

```json
{"entity_type": "KnowledgeBase", "id": "abc", "action": "update", "data": {...}}
```

For Postgres or remote SurrealDB: return 404 (clients use other sync transports).

**Step 3: Test**

```bash
curl -N http://localhost:3002/api/uar/sync/stream
# Should see SSE events when data changes
```

**Step 4: Commit**

```bash
git add src/uar/api/sync.rs src/server.rs
git commit -m "feat: SSE sync bridge for embedded SurrealDB LIVE SELECT"
```

---

## Phase 6: ElectricSQL + WebSocket Adapters

### Task 6.1: ElectricSQL Adapter for Postgres

**Files:**
- Modify: `frontend/src/entities/sync.ts`

**Step 1: Implement Postgres branch in detectSyncTransport**

When `provider === "postgres"`, use `createElectricAdapter` from the entity management library. Configure shape streams for all entity tables. This syncs PGLite <-> Postgres bidirectionally.

**Step 2: Test with Postgres backend**

Set `UAR_PERSISTENCE__PROVIDER=postgres` and verify data syncs.

**Step 3: Commit**

```bash
git add frontend/src/entities/sync.ts
git commit -m "feat: ElectricSQL sync adapter for Postgres backend"
```

---

### Task 6.2: WebSocket Adapter for Remote SurrealDB

**Files:**
- Modify: `frontend/src/entities/sync.ts`

**Step 1: Implement remote SurrealDB branch**

When `provider === "surreal" && mode === "remote"`, use `createWebSocketAdapter` pointing to the SurrealDB server's WebSocket endpoint. Subscribe to LIVE SELECT channels.

**Step 2: Test with remote SurrealDB**

Configure a remote SurrealDB server and verify live updates flow.

**Step 3: Commit**

```bash
git add frontend/src/entities/sync.ts
git commit -m "feat: WebSocket sync adapter for remote SurrealDB"
```

---

## Phase 7: Skill Import from Disk

### Task 7.1: Backend Skill Import Endpoint

**Files:**
- Modify: `src/uar/api/skills.rs`

**Step 1: Add POST /api/uar/skills/import endpoint**

Accepts `{ "path": "/absolute/path" }`. Reads the directory:
1. Check for `SKILL.md` (agentskills.io format) — parse YAML frontmatter + markdown body
2. Check for `.claude/` directory (Claude Code plugin format)
3. Check for `SKILLS.md` (marketplace bundle) — return multiple parsed skills
4. Discover `references/`, `scripts/`, `assets/` directories

Return parsed skill data + validation result.

**Step 2: Test with curl**

```bash
curl -X POST http://localhost:3002/api/uar/skills/import \
  -H 'Content-Type: application/json' \
  -d '{"path":"/Users/gqadonis/Projects/prometheus/universal-agent-runtime/.claude/skills/prometheus-entity-skills/entity-graph-setup"}'
```

Expected: parsed skill with name, description, detected format.

**Step 3: Commit**

```bash
git add src/uar/api/skills.rs
git commit -m "feat: skill import endpoint with agentskills.io/Claude Code/marketplace parsing"
```

---

### Task 7.2: Skill Import UI

**Files:**
- Modify: `frontend/src/admin/pages/skills-page.tsx`
- Create: `frontend/src/admin/components/skill-import-dialog.tsx`

**Step 1: Build import dialog**

Dialog with path input, "Parse" button, preview card showing parsed skill data + validation badges, and "Import" button that calls `POST /api/uar/skills` with the parsed data.

For marketplace bundles, show a checklist of sub-skills for selective import.

**Step 2: Add "Import" button to skills page header**

**Step 3: Commit**

```bash
git add frontend/src/admin/components/skill-import-dialog.tsx frontend/src/admin/pages/skills-page.tsx
git commit -m "feat: skill import from disk with format detection and preview"
```

---

## Phase 8: Tool Playground

### Task 8.1: Tool Execute Backend Endpoint

**Files:**
- Modify: `src/server.rs` or appropriate API module

**Step 1: Add POST /api/tools/{name}/execute**

Accepts `{ "arguments": { ... } }`. Routes through `McpRegistry.call_namespaced_tool()`. Returns `{ "result": ..., "duration_ms": 1200, "success": true }`.

**Step 2: Test with curl**

```bash
curl -X POST http://localhost:3002/api/tools/time__now/execute \
  -H 'Content-Type: application/json' \
  -d '{"arguments":{}}'
```

**Step 3: Commit**

```bash
git add src/server.rs
git commit -m "feat: tool execute endpoint for playground testing"
```

---

### Task 8.2: JsonSchemaForm Component

**Files:**
- Create: `frontend/src/components/json-schema-form.tsx`

**Step 1: Build recursive JSON Schema form renderer**

Takes a JSON Schema object, renders appropriate form fields:
- `string` -> Input
- `number`/`integer` -> Input type=number
- `boolean` -> Switch
- `enum` -> Select
- `array` -> repeatable field group with + button
- `object` -> nested fieldset (collapsible)
- Required fields: asterisk label + validation

Returns form data as a plain object matching the schema shape.

**Step 2: Test with a complex schema**

Render the form with `time__now` tool's input_schema (empty) and `tavily__tavily_search` tool's schema (has query, search_depth, max_results fields). Verify correct rendering.

**Step 3: Commit**

```bash
git add frontend/src/components/json-schema-form.tsx
git commit -m "feat: recursive JSON Schema form component for tool playground"
```

---

### Task 8.3: Tool Detail Panel with Playground

**Files:**
- Modify: `frontend/src/admin/pages/tools-page.tsx`
- Create: `frontend/src/admin/components/tool-detail-panel.tsx`

**Step 1: Build tool detail panel**

When a tool is clicked in the tools list, show a detail panel with three tabs:
- **Test:** JsonSchemaForm for input_schema + Execute button + result viewer (syntax-highlighted JSON)
- **Schema:** raw JSON schema viewer
- **Metrics:** placeholder for future execution history

**Step 2: Wire into tools-page.tsx**

Convert from simple list to master-detail layout. Left: tool list (unchanged). Right: ToolDetailPanel for selected tool.

**Step 3: Test execution**

Click a tool, fill in arguments, click Execute. Verify result appears.

**Step 4: Commit**

```bash
git add frontend/src/admin/components/tool-detail-panel.tsx frontend/src/admin/pages/tools-page.tsx
git commit -m "feat: tool playground with schema-driven form and execution"
```

---

## Phase 9: Migrate Remaining Stores to Entity Graph

### Task 9.1: Migrate Agents Store

**Files:**
- Create: `frontend/src/entities/hooks/use-agents.ts`
- Create: `frontend/src/entities/fetchers/agents.ts`
- Modify: `frontend/src/hooks/use-agents-admin.ts` (thin wrapper)
- Modify: `frontend/src/admin/pages/agents-page.tsx`

**Step 1:** Create entity hooks that replace `agents-admin-store.ts`
**Step 2:** Create fetcher that loads agents into graph via `upsertEntity`
**Step 3:** Update the admin hook to delegate to entity hooks
**Step 4:** Verify agents page works identically
**Step 5:** Commit

---

### Task 9.2: Migrate Skills Store

Same pattern as 9.1 for skills.

---

### Task 9.3: Migrate Tools Store

Same pattern as 9.1 for tools.

---

### Task 9.4: Migrate Knowledge Base Store

Same pattern as 9.1 for knowledge bases and documents.

---

### Task 9.5: Remove Legacy Stores

Once all pages use entity graph hooks, remove the old store files:
- `providers-admin-store.ts`
- `provider-models-store.ts`
- `agents-admin-store.ts`
- `skills-admin-store.ts`
- `tools-discovery-store.ts`
- `knowledge-admin-store.ts`

And their corresponding hooks that were thin wrappers.

**Commit:**
```bash
git commit -m "refactor: remove legacy per-page Zustand stores (replaced by entity graph)"
```

---

## Phase 10: Impeccable Design Pass

### Task 10.1: Run Impeccable Audit

Use the `impeccable:audit` skill to run a full accessibility, performance, theming, and responsive audit across all admin pages.

### Task 10.2: Apply Polish

Use the `impeccable:polish` skill for final alignment, spacing, and consistency fixes.

### Task 10.3: Responsive QA

Test all pages at mobile (375px), tablet (768px), and desktop (1280px+) breakpoints. Fix any layout breaks.

### Task 10.4: Final Commit

```bash
git commit -m "style: impeccable design pass — polish, a11y, responsive fixes"
```
