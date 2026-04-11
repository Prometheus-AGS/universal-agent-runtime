# Admin UX & Entity Management Overhaul

**Date:** 2026-04-11
**Status:** Approved
**Scope:** Frontend admin UI, chat interface, entity management, sync transport, backend API extensions

---

## 1. Foundation: Entity Graph Bootstrap & Sync Transport

### Problem
Each admin page uses its own isolated Zustand store with independent fetch/cache logic. Data is duplicated across views, there is no offline support, and no realtime updates when backend state changes.

### Solution
Replace all per-page Zustand stores with the `@prometheus-ags/prometheus-entity-management` normalized graph, backed by PGLite for offline persistence and provider-aware realtime sync.

### Entity Types

| Entity | Key Fields | Relations |
|--------|-----------|-----------|
| Provider | id, display_name, base_url, configured, auth_env_var, endpoints, model_count | Provider -> Model[] |
| Model | id, name, provider_id, context, tool_call, reasoning, vision | Model -> Provider |
| Agent | id, name, description, system_prompt, model, skills[], tools[], knowledge_bases[], mcp_servers[], context_strategy, tool_approval, status, spec_id | Agent -> Skill[], Agent -> Tool[], Agent -> KnowledgeBase[] |
| Skill | id, title, version, description, triggers, enabled, provider_id, source, source_path | |
| Tool | id, name, description, namespace, input_schema, output_schema, transport, built_in | |
| KnowledgeBase | id, name, description, document_count, config | KB -> Document[] |
| Document | id, kb_id, filename, status, chunk_count, mime_type | Document -> KB |
| AgentSession | agent_id, session_id, model?, skills?, tools?, knowledge_bases?, mcp_servers?, context_strategy?, tool_approval? | AgentSession -> Agent, AgentSession -> Thread |
| Thread | id, title, agent_id, created_at, updated_at, last_message_preview | Thread -> AgentSession |

### Bootstrap Sequence (app init)

```
1. configureEngine({ baseFetch: wrappedFetch, staleTime: 30_000, retry: 2 })
2. registerSchema(providerSchema)   // with relations: Provider -> Model[]
3. registerSchema(agentSchema)      // with relations: Agent -> Skill[], Agent -> Tool[], Agent -> KB[]
4. registerSchema(kbSchema)         // with relations: KB -> Document[]
5. registerSchema(threadSchema)     // with relations: Thread -> AgentSession
6. startLocalFirstGraph({ storage: pgliteAdapter })  // hydrate from PGLite
7. detectSyncTransport() -> register appropriate RealtimeManager adapter
```

### Sync Transport Matrix

| Backend Provider | Mode | Sync Transport | Mechanism |
|-----------------|------|---------------|-----------|
| Postgres | remote | ElectricSQL | Shape streams <-> PGLite bidirectional |
| SurrealDB | remote | WebSocket | Direct WS to SurrealDB server -> LIVE SELECT -> RealtimeManager |
| SurrealDB | embedded | SSE Bridge | Axum subscribes internally -> pushes via SSE -> createSSEAdapter |

### Transport Detection

```ts
async function detectSyncTransport(): SyncAdapter {
  const { provider, mode } = await fetch("/api/config/persistence").then(r => r.json());
  
  if (provider === "postgres") {
    return createElectricAdapter({ url: electricUrl, tables: [...] });
  }
  if (provider === "surreal" && mode === "remote") {
    return createWebSocketAdapter({ url: surrealWsUrl, ... });
  }
  // surreal + embedded -> SSE bridge
  return createSSEAdapter({ url: "/api/uar/sync/stream" });
}
```

### New Backend Endpoint

```
GET /api/config/persistence
  Response: { provider: "surreal"|"postgres", mode: "embedded"|"remote", database_url: "..." }

GET /api/uar/sync/stream  (SSE, for embedded SurrealDB only)
  - Server runs LIVE SELECT on each entity table
  - Pushes ChangeSet events: { type, id, action: "create"|"update"|"delete", data }
  - Client RealtimeManager coalesces at 16ms flush interval
```

### Three-Layer Architecture Rule

```
UI (Components) -> Hooks -> Graph Store + Engine -> APIs/Adapters
```

- Components NEVER import `useGraphStore` directly. Use public hooks: `useEntity`, `useEntityList`, `useEntityView`, `useEntityCRUD`.
- Hooks orchestrate graph reads/writes; allowed to call `useGraphStore.getState()` internally.
- Lists store IDs only (not data copies). Entities live exactly once in the normalized graph.

### Migration Path

Existing stores (`providers-admin-store.ts`, `agents-admin-store.ts`, etc.) become thin wrappers around entity graph hooks during transition, then are removed.

---

## 2. Providers & Models UI

### Global Cursor Fix

Add to the base Tailwind layer:

```css
button, [role="button"], a, [tabindex="0"], input[type="checkbox"],
input[type="radio"], select, summary { cursor: pointer; }
```

### Provider List Redesign

**Search:** Sticky search input at top of provider sidebar. Client-side filter by `display_name` and `id`.

**Sort order:**
1. Configured providers first (alphabetical within group)
2. Unconfigured providers second (alphabetical)

**Visual distinction:** Configured providers get `border-l-2 border-primary` accent.

**Entity integration:** `useEntityView` powers the list:

```ts
const providerView = useEntityView<Provider>("Provider", {
  sort: [
    { field: "configured", direction: "desc" },
    { field: "display_name", direction: "asc" },
  ],
  search: { fields: ["display_name", "id"], query: searchTerm },
});
```

### Model List

Models load via `useEntityList` with `provider_id` filter. Relation `Provider -> Model[]` registered in schema for `cascadeInvalidation`.

### LLM Settings Clarification

Rename "LLM (liter-llm)" to **"LLM Configuration"**. This page configures global LLM defaults:

| Field | Type | Maps to |
|-------|------|---------|
| Default model | provider/model dropdown | `LlmConfig.model` |
| Protocol | auto / openai-chat / openai-responses select | `LlmConfig.protocol` |
| Timeout | number input (seconds) | `LlmConfig.timeout_secs` |
| Max retries | number input | `LlmConfig.max_retries` |
| Cost tracking | toggle | `LlmConfig.cost_tracking` |
| Thinking budget | number input (tokens) | `LlmConfig.thinking_budget` |
| Rate limiting | requests/sec + burst size | `LlmConfig.rate_limit` |

Individual provider configuration remains on the Providers page.

---

## 3. Agent Definitions

### Agent Definition Schema

```ts
interface AgentDefinition {
  id: string;
  name: string;
  description: string;
  system_prompt: string;
  model?: string;                        // provider/model override
  protocol?: "auto" | "openai-chat" | "openai-responses";
  
  // Capabilities (defaults for all sessions)
  skills: string[];                      // bound skill IDs
  tools: string[];                       // available tool IDs
  knowledge_bases: string[];             // attached KB IDs for RAG
  mcp_servers: string[];                 // which MCP servers this agent can use
  
  // Context Management
  context_strategy: {
    max_history_messages: number;
    inject_memory: boolean;
    inject_knowledge: boolean;
    memory_scope: "session" | "agent" | "global";
    auto_capture: boolean;
  };
  
  // Governance
  tool_approval: "auto" | "ask" | "deny";
  
  // Metadata
  status: "active" | "draft" | "disabled";
  spec_id?: string;                      // compiled UAR-AGENT-MD link
  created_at: string;
  updated_at: string;
}
```

### Agent Session (Per-Conversation Override)

```ts
interface AgentSession {
  agent_id: string;
  session_id: string;
  
  // All optional -- null means "use agent default"
  model?: string;
  skills?: string[];
  tools?: string[];
  knowledge_bases?: string[];
  mcp_servers?: string[];
  context_strategy?: Partial<AgentDefinition["context_strategy"]>;
  tool_approval?: "auto" | "ask" | "deny";
}
```

Effective config = `merge(agentDefinition, agentSession)`.

### Two Creation Paths

**Path A: Manual Creation**
Multi-step form: Identity -> Capabilities -> Behavior -> Context -> Governance -> Review.

**Path B: AI-Assisted Compilation**
- User types plain-language description
- Sends to `POST /api/a2a/compiler` using `uar.compile.conversational` skill
- Compiler returns structured agent spec (UAR-AGENT-MD)
- Spec pre-fills agent editor form for review and tweaking
- Compiled spec stored via CompilerService and linked to agent via `spec_id`

### Agent Editor UI Tabs

| Tab | Fields |
|-----|--------|
| Identity | name, description, status, model selector |
| Prompt | system prompt (markdown editor with preview) |
| Capabilities | skills, tools, knowledge bases, MCP servers (searchable chip lists) |
| Context | history window slider, memory toggles, scope selector, auto-capture |
| Governance | tool approval policy |
| Spec | read-only compiled spec, "Compile"/"Recompile with AI" buttons |

### Backend Changes

```
POST   /api/agents          -- create agent
PUT    /api/agents/{id}     -- full update
DELETE /api/agents/{id}     -- delete agent
POST   /api/agents/{id}/compile -- trigger compilation from existing config
```

Existing `PATCH /api/agents/{id}` stays for partial updates.

---

## 4. Chat Interface

### Layout

```
+----------+----------------------------------------------+
| THREADS  |  Agent: [Research Agent v]  [gear Session Config]|
| + New    |----------------------------------------------|
| Search   |    Messages...                               |
|          |                                              |
| Thread 1 |                                              |
| Thread 2 |                                              |
|          |----------------------------------------------|
|          | [clip] [Type a message...            ] [Send] |
|          | [brain KB: default] [wrench Tools: 14] [zap Skills: 2]  |
+----------+----------------------------------------------+
```

### Agent Selector

- Dropdown at top of chat area showing all active agents
- Grouped: Recently used (top 3) then All agents (alphabetical)
- Each option: agent name, model badge, description snippet
- Selecting agent for new thread creates `AgentSession` linking agent -> thread
- Changing agent mid-conversation creates new `AgentSession`
- Default: "Default Assistant" using global LLM config

### Input Bar Capability Toggles

| Toggle | Behavior |
|--------|----------|
| Knowledge Bases | Popover with KB checklist, check/uncheck per-session |
| Tools | Popover with tool list, toggle individual tools |
| Skills | Popover with agent's skills, toggle on/off |
| Web Search | Single toggle for web search tool |
| Memory | Toggle memory injection on/off |
| Attachments | File picker |

Toggles override agent definition defaults for this session only, persisted to `AgentSession` entity.

### Thread Sidebar Enhancement

- Each thread shows: title, agent name/icon, last message preview, timestamp
- "New thread" button prompts agent selection
- Right-click context menu: rename, delete, duplicate, change agent

### Per-Session Config Panel

`[gear Session Config]` button opens slide-over:
- Current agent (with "Change" link)
- Model override dropdown
- Context strategy overrides
- Tool approval mode
- Active KBs, tools, skills (structured form)

### Data Flow

```
1. User creates new thread -> agent selector opens -> selects agent
2. AgentSession created: { agent_id, session_id }
3. Effective config computed: merge(agentDefinition, agentSession)
4. User toggles capabilities in input bar -> AgentSession updated
5. User sends message -> backend receives session_id
6. Backend resolves effective config from AgentSession + AgentDefinition
7. Orchestrator runs with resolved tools, skills, KBs, model
```

### Backend Changes

```
POST /api/sessions/{id}/agent-session    -- create/update agent session config
GET  /api/sessions/{id}/agent-session    -- get effective config (merged)
GET  /api/sessions/{id}/effective-config -- resolved config for orchestrator
```

---

## 5. Skill Import from Disk

### Import Flow

```
"Import from Disk" -> path input -> Backend parses SKILL.md + validates ->
Preview shown -> User confirms -> Skill saved + registered
```

### Backend Endpoint

```
POST /api/uar/skills/import
  Body: { "path": "/absolute/path/to/skill-directory" }
  
  Response: {
    "parsed": {
      "name": "...", "description": "...", "version": "...",
      "triggers": { "keywords": [...] },
      "prompt_overlay": "...",
      "references": ["ref1.md", ...],
      "scripts": ["setup.sh", ...],
      "source": "filesystem",
      "source_path": "/absolute/path"
    },
    "validation": {
      "valid": true,
      "warnings": [...],
      "detected_formats": ["agentskills.io", "claude-code-plugin"]
    }
  }
```

### Format Detection

| Format | Detection | Parsing |
|--------|-----------|---------|
| agentskills.io | `SKILL.md` exists (case-sensitive) | YAML frontmatter -> metadata; body -> prompt_overlay |
| Claude Code plugin | `.claude/` dir with `skills/` or `CLAUDE.md` | Extract skill definitions |
| Marketplace bundle | `SKILLS.md` catalog + subdirs | Parse as multi-skill; import each sub-skill |

### Frontend UI

- "Import" button on Skills page next to "New Skill"
- Path input field + "Parse" button
- Preview card: name, description, version, keywords, format badges, warnings
- "Import" button saves to persistence
- Multi-skill bundles show checklist for selective import

---

## 6. Tool Playground

### Tool Detail View

Click a tool -> detail panel with tabs:

| Tab | Content |
|-----|---------|
| Test | Schema-driven form + Execute button + result viewer |
| Schema | Raw JSON Schema (syntax-highlighted, collapsible) |
| Metrics | Execution history: last N calls, avg duration, success rate |

### Schema-Driven Form Generation

`JsonSchemaForm` component renders form from JSON Schema:
- `string` -> text input
- `number`/`integer` -> number input
- `boolean` -> toggle
- `enum` -> select dropdown
- `array` -> repeatable field group
- `object` -> nested fieldset (collapsible)
- Required fields validated before execution

### Backend Endpoint

```
POST /api/tools/{namespaced_name}/execute
  Body: { arguments: { ... } }
  Response: { result: ..., duration_ms: 1200, success: true }
```

Routes through existing `McpRegistry.call_namespaced_tool()`.

---

## 7. UI/UX Quality

### Design System

- Typography: `font-display` (headings), `font-mono` (data), `font-body` (text)
- Color: ShadCN HSL tokens. Configured = `text-success`, warnings = `text-amber-400`, errors = `text-destructive`
- Spacing: 4px grid. Consistent `gap-2`/`gap-3`/`p-4`/`px-6`
- Motion: `transition-colors` on hover, `animate-spin` on loading. No gratuitous animation.

### Interaction Polish

- `cursor: pointer` on all interactive elements (global CSS)
- Focus rings: `focus-visible:ring-2 focus-visible:ring-primary/50`
- Loading states: spinner in triggering button
- Empty states: icon + message + CTA for every list
- Error states: inline `font-mono text-xs text-destructive`
- Keyboard: arrow keys in lists, Enter to select, Escape to deselect

### Responsive

- Sidebar collapses on mobile with back-button navigation
- Detail panels full-width on mobile
- Input bar toggles wrap on narrow screens

---

## Implementation Phases (Recommended)

| Phase | Scope | Dependencies |
|-------|-------|-------------|
| **P0** | Global cursor fix, provider search/sort, LLM settings rename | None |
| **P1** | Entity graph bootstrap, PGLite, configureEngine, registerSchema | @prometheus-ags/prometheus-entity-management |
| **P2** | Migrate providers + models to entity graph with useEntityView | P1 |
| **P3** | Agent CRUD backend + frontend editor with full config schema | P1 |
| **P4** | Chat interface agent selector, AgentSession, input bar toggles | P3 |
| **P5** | Sync transport detection + SSE bridge (embedded SurrealDB) | P1 |
| **P6** | ElectricSQL adapter (Postgres) + WebSocket adapter (SurrealDB remote) | P5 |
| **P7** | Skill import from disk (backend parser + frontend UI) | P1 |
| **P8** | Tool playground (JsonSchemaForm + execute endpoint) | P1 |
| **P9** | Migrate remaining stores (agents, skills, tools, KBs) to entity graph | P2 |
| **P10** | Impeccable design pass: polish, a11y audit, responsive QA | P0-P9 |
