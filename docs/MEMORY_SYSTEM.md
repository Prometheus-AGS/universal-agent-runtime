# Memory System

UAR ships a production-grade, multi-scope memory system built on [surreal-memory](https://github.com/Prometheus-AGS/surreal-memory-server) and SurrealDB. It gives every agent durable, cross-session recall — turning stateless LLMs into agents that remember users, learn from interactions, and build a persistent knowledge graph over time.

---

## 1. Overview

Stateless LLMs forget everything the moment a conversation ends. UAR's memory system fixes this by:

- **Auto-capturing** important facts from each assistant response
- **Injecting** relevant past memories into the system prompt before each LLM call
- **Hybrid search** (vector similarity + BM25 full-text) to surface the most contextually relevant memories, ranked by recency and importance
- **Exposing all memory operations as MCP tools** so agents can explicitly read, write, and delete memories mid-conversation
- **Building a knowledge graph** of entities, relationships, and observations over time

Memory is **opt-in** — set `UAR_MEMORY__ENABLED=true` to activate it.

---

## 2. Memory Scopes

Every memory record is tagged with a scope that determines its lifetime and who can access it.

| Scope | Env context | Lifetime | Best for |
|-------|------------|----------|----------|
| `session` | Current conversation ID | Until session ends (or explicitly cleared) | Immediate context like user intent for this chat |
| `user` | User ID | Permanent (until deleted) | Persistent user preferences, facts, history |
| `agent` | Agent ID | Permanent (until deleted) | Agent-specific knowledge base, domain facts |
| `global` | None | Permanent, shared | Cross-agent shared knowledge, organization-wide facts |
| `task` | Task stream ID | Until task completes | Multi-step workflow state, intermediate results |

**Scope resolution at injection time**: UAR retrieves memories ranked across all scopes, weighted by relevance to the current query and message. Session and user scope memories are given higher weight for personalization.

---

## 3. Memory Types

| Type | Purpose | Examples |
|------|---------|---------|
| `episodic` | Facts and events from past conversations | "User prefers dark mode", "Called API at 3pm on Feb 20" |
| `semantic` | Structured knowledge graph entries | Entities (User, Product), Relations (works_at, prefers), Observations |
| `procedural` | Instructions, workflows, learned patterns | "When user asks about billing, always check Stripe" |
| `associative` | Connections between concepts | "Topic X relates to Topic Y based on conversation history" |

The `memory_type` field is optional on write — UAR auto-classifies if not specified.

---

## 4. Enabling Memory

Memory is **disabled by default**. Enable it in your config or via environment variables:

### Config file (`config.toml` / `config.yaml`)

```toml
[memory]
enabled = true
embedding_provider = "openai"
embedding_model = "text-embedding-3-small"
auto_capture = true
inject_context = true
```

### Environment variables

```bash
UAR_MEMORY__ENABLED=true
UAR_MEMORY__EMBEDDING_PROVIDER=openai   # openai | cohere | local
UAR_MEMORY__EMBEDDING_MODEL=text-embedding-3-small
OPENAI_API_KEY=sk-...                   # required when provider=openai
```

### Embedding providers

| Provider | Env var | Notes |
|----------|---------|-------|
| `openai` | `OPENAI_API_KEY` | Default. Uses `text-embedding-3-small` by default |
| `cohere` | `UAR_MEMORY__COHERE_API_KEY` | Use `embed-english-v3.0` or multilingual equivalent |
| `local` | _(none)_ | Requires a local embedding server at `UAR_MEMORY__LOCAL_EMBEDDING_URL` |

---

## 5. Auto-Capture

When `auto_capture = true` (the default when memory is enabled), UAR automatically extracts and stores key facts from each assistant response without the agent needing to call any tool.

### How it works

After each assistant turn completes:
1. The auto-capture pipeline sends the assistant response + conversation context to the LLM with an extraction prompt
2. The LLM identifies facts worth remembering (user preferences, stated facts, decisions made)
3. Each fact is embedded and stored in `scope = "session"` or `scope = "user"` depending on content
4. Duplicates are detected and merged or skipped

### Configure capture behavior

```toml
[memory]
auto_capture = true        # Enable/disable (default: true when memory.enabled=true)
```

To disable auto-capture while keeping memory enabled (manual-only mode):

```bash
UAR_MEMORY__AUTO_CAPTURE=false
```

---

## 6. Context Injection

When `inject_context = true`, UAR prepends a structured memory block to the LLM's system prompt before each call.

### Injection format

The injected block looks like:

```
## Recalled Memories

1. [user/2025-11-01] User prefers responses in bullet points. (relevance: 0.94)
2. [session] User is asking about the quarterly report. (relevance: 0.87)
3. [agent] This agent specializes in financial data analysis. (relevance: 0.81)
```

### Tuning injection

```toml
[memory]
inject_context = true
max_context_tokens = 2000      # Max tokens to reserve for memory context
vector_weight = 0.7            # Weight for vector (semantic) similarity score
bm25_weight = 0.3              # Weight for BM25 (keyword) match score
```

- **`max_context_tokens`**: Limits how many tokens are used by injected memories. Increase for deeper recall; decrease to leave more room for content.
- **`vector_weight` + `bm25_weight`**: Must sum to 1.0. Higher `vector_weight` favors semantic similarity; higher `bm25_weight` favors exact keyword matches. The default (0.7/0.3) is well-suited for general use.

---

## 7. MCP Access

The memory system is exposed as a full MCP server at `/mcp/memory`. Any MCP-compatible client (Claude Desktop, LangGraph, AutoGen, Cursor, etc.) can use all memory tools directly.

Enable the HTTP MCP endpoint:

```toml
[memory]
mcp_http_enabled = true
mcp_http_path = "/mcp/memory"   # default
```

### Available MCP tools

**Scoped memory (mem0-compatible)**

| Tool | Description |
|------|-------------|
| `memory_add` | Store a new memory with content, scope, and type |
| `memory_get` | Retrieve a memory by ID |
| `memory_update` | Update the content of an existing memory |
| `memory_delete` | Delete a memory by ID |
| `memory_delete_all` | Delete all memories matching a scope filter |
| `memory_list` | List all memories for a scope (paginated) |
| `memory_search` | Semantic vector search |
| `memory_hybrid_search` | Combined vector + BM25 search (recommended) |
| `memory_history` | Retrieve access/mutation history for a memory |
| `memory_compress` | Deduplicate and consolidate related memories |
| `memory_extract_from_conversation` | Extract facts from a conversation turn |

**Knowledge graph**

| Tool | Description |
|------|-------------|
| `kg_create_entity` | Create a named entity (person, place, concept) |
| `kg_add_observations` | Attach factual observations to an entity |
| `kg_create_relation` | Define a directed relation between two entities |
| `kg_read` | Read an entity's full profile |
| `kg_search` | Exact-name entity search |
| `kg_semantic_search` | Semantic search across entities |
| `kg_expand_neighbors` | Expand the graph from a starting entity |
| `kg_find_path` | Find connection paths between two entities |
| `kg_get_related` | Get entities related to a target entity |
| `kg_delete_entity` | Remove an entity and its relations |
| `kg_delete_relation` | Remove a specific relation |

**Task streams**

| Tool | Description |
|------|-------------|
| `task_stream_create` | Create a named task stream for multi-step workflows |
| `task_stream_add` | Append an event to a task stream |
| `task_stream_get` | Get a task stream's full history |
| `task_stream_context` | Get condensed context for the current task |
| `task_stream_list` | List active task streams |
| `task_stream_archive` | Archive a completed task stream |
| `task_stream_auto_summarize` | LLM-driven task stream compression |

### MCP client configuration (Claude Desktop example)

```json
{
  "mcpServers": {
    "uar-memory": {
      "url": "http://localhost:1906/mcp/memory"
    }
  }
}
```

---

## 8. Knowledge Graph

The knowledge graph provides a structured, entity-relationship model on top of the scoped memory system. It enables graph-RAG workflows where agents reason across connected entities.

### Conceptual model

```
Entity (User: "Alice")
  → Observation: "Works at Acme Corp"
  → Observation: "Prefers TypeScript over Python"
  → Relation: works_at → Entity (Company: "Acme Corp")
  → Relation: knows → Entity (User: "Bob")
```

### Use case: personal assistant

```python
# MCP tool call from an agent
kg_create_entity(name="Alice", entity_type="user", observations=["Alice works at Acme Corp"])
kg_create_relation(source="Alice", relation_type="works_at", target="Acme Corp")

# Later in a new session:
kg_expand_neighbors(entity="Alice")  # Returns: Acme Corp, Bob, ...
```

### Visualization

Surrealist (included in the Docker Compose stack) provides a visual graph explorer. Connect to `ws://localhost:8000` with your SurrealDB credentials to browse the knowledge graph interactively.

---

## 9. Annotated Configuration

Complete `[memory]` config block with every field documented:

```toml
[memory]
# ── Core ────────────────────────────────────────────────────────────────
enabled = true                        # Master switch — false by default

# ── Storage backend ──────────────────────────────────────────────────────
db_path = "./data/memory.db"          # RocksDB path (embedded mode only)

# External SurrealDB (production — overrides embedded mode)
# surreal_endpoint = "ws://surreal-svc:8000"
# surreal_user = "root"
# surreal_pass = "changeme"
# namespace = "uar"
# database = "memory"

# ── Embeddings ───────────────────────────────────────────────────────────
embedding_provider = "openai"         # "openai" | "cohere" | "local"
embedding_model = "text-embedding-3-small"
# openai_api_key = ""                 # or use OPENAI_API_KEY env var
# cohere_api_key = ""                 # or use UAR_MEMORY__COHERE_API_KEY

# ── Capture ──────────────────────────────────────────────────────────────
auto_capture = true                   # Extract memories after each turn

# ── Context injection ────────────────────────────────────────────────────
inject_context = true                 # Prepend memory block to LLM prompt
max_context_tokens = 2000             # Max tokens for injected memories
vector_weight = 0.7                   # Semantic similarity weight (0–1)
bm25_weight = 0.3                     # BM25 keyword weight (0–1, sum must be 1)

# ── MCP HTTP ─────────────────────────────────────────────────────────────
mcp_http_enabled = true               # Expose memory tools as MCP server
mcp_http_path = "/mcp/memory"         # HTTP path for MCP endpoint
```

### Environment variable equivalents

```bash
UAR_MEMORY__ENABLED=true
UAR_MEMORY__DB_PATH=./data/memory.db
UAR_MEMORY__EMBEDDING_PROVIDER=openai
UAR_MEMORY__EMBEDDING_MODEL=text-embedding-3-small
UAR_MEMORY__AUTO_CAPTURE=true
UAR_MEMORY__INJECT_CONTEXT=true
UAR_MEMORY__MAX_CONTEXT_TOKENS=2000
UAR_MEMORY__VECTOR_WEIGHT=0.7
UAR_MEMORY__BM25_WEIGHT=0.3
UAR_MEMORY__MCP_HTTP_ENABLED=true
UAR_MEMORY__MCP_HTTP_PATH=/mcp/memory
UAR_MEMORY__SURREAL_ENDPOINT=ws://surreal-svc:8000
UAR_MEMORY__SURREAL_USER=root
UAR_MEMORY__SURREAL_PASS=changeme
```

---

## 10. Quick Start (5 steps)

### Step 1 — Enable memory

```bash
export UAR_MEMORY__ENABLED=true
export OPENAI_API_KEY=sk-...
```

### Step 2 — Start UAR

```bash
docker compose -f docker-compose.prod.yml up -d
# or: cargo run
```

### Step 3 — Run your first conversation

Send a message through the UAR web UI or via the API:

```bash
curl -X POST http://localhost:1906/api/uar/runs \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "your-agent-id", "input": "My name is Alice and I work at Acme Corp."}'
```

### Step 4 — Verify recall

After the run completes, check what was remembered:

```bash
# Via the memory admin API
curl "http://localhost:1906/api/admin/memories?scope=session"
```

Or connect Claude Desktop to the MCP endpoint:
```json
{ "mcpServers": { "uar-memory": { "url": "http://localhost:1906/mcp/memory" } } }
```

Then ask: `memory_list` → you should see the extracted fact about Alice.

### Step 5 — Inspect the knowledge graph

Start a new conversation:

```
"What do you know about Alice?"
```

UAR will inject the recalled memory and the agent will answer with the stored fact — even in a brand new session.

To explore the graph visually, open Surrealist at `http://localhost:8080` and connect to `ws://localhost:8000`.

---

## Further Reading

- [Dependency Management](./DEPENDENCY_MANAGEMENT.md)
- [Architecture Overview](./ARCHITECTURE.md)
- [surreal-memory library](https://github.com/Prometheus-AGS/surreal-memory-server)
