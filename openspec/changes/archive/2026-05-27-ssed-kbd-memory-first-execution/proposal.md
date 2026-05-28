## Why

The user's phase-defining message asked for "more intentional execution of phases using a combination of whatever memory system is available (emphasizing surreal memory mcp server) and the pk-based skills for my rust version of the karpathy skills" — so that iterative improvements compound across phases AND across projects.

Today:
- `kbd-process-orchestrator/SKILL.md` already documents surreal-memory as *optional* ("Detection: check if `create_entity` tool is available. Fallback: filesystem state").
- `surreal-memory-server` exists at `/Users/gqadonis/Projects/prometheus/surreal-memory-server` with TaskStreams API + MCP wiring.
- `karpathy-tokenizer` skill exists for tokenizer training but **no `pk-recall` skill** that queries prior memory for similar phases / patterns.
- Hook dispatcher from change 3 already writes a JSONL audit log; that log is a natural source for the memory writer to consume.

The substrate is ready; what's missing is **discipline + tooling** to use it by default. This change:

1. Promotes surreal-memory from "optional" to "default-on when reachable", with an explicit detection contract and a graceful degradation path.
2. Adds a `kbd-memory-log` augment hook that mirrors every hook fire into surreal-memory as a structured entity observation.
3. Adds a `kbd-memory-recall` skill (`/kbd-memory-recall`) that, given a phase or change context, queries surreal-memory for prior similar work and writes a digest to `.kbd-orchestrator/phases/<phase>/prior-context.md` for the agent to consult during assess/plan.
4. Defines the memory **entity schema** for KBD events so any tool reading the memory can interpret it consistently — this is the cross-project learning surface the user described.
5. Adds a documented retention/relevance policy so the log doesn't balloon.

## What Changes

### Detection contract

A small helper `shared/lib/memory.sh` exposes `kbd_memory_available()` returning 0 when the surreal-memory MCP endpoint is reachable. Detection is one of:

- The orchestrator's existing convention: `create_entity` tool is in the calling tool's tool list.
- Environment variable `UAR_MEMORY_MCP_URL` (or `KBD_MEMORY_MCP_URL`) is set AND responds to a `GET /healthz` probe.
- A `.kbd-orchestrator/memory.config.json` file declares the endpoint and credentials.

Detection runs once per skill invocation; the result is cached in a process-local variable.

### `kbd-memory-log` augment hook

Registered in `hooks/hooks.json` as a built-in `mode: augment` entry covering `*:*`. On every hook fire, it serialises a structured observation matching this schema:

```json
{
  "entityType": "kbd_lifecycle_event",
  "entityId": "<project>/<phase>/<kind>/<edge>/<index>/<timestamp>",
  "observations": [
    { "kind": "<kind>", "edge": "<edge>", "name": "<name>",
      "index": <i>, "total": <n>, "phasePath": "<chain>",
      "sourceTool": "<tool>", "project": "<project>", "ts": "<iso>" }
  ],
  "relations": [
    { "from": "<this entityId>", "to": "phase:<phase>", "label": "fires-in" },
    { "from": "<this entityId>", "to": "project:<project>", "label": "belongs-to" }
  ]
}
```

The hook is a thin wrapper script: it reads the `KBD_HOOK_*` env vars, builds the JSON, and POSTs to the surreal-memory MCP endpoint (or no-ops when the endpoint isn't available — same soft-fail contract as the JSONL log).

### `kbd-memory-recall` skill (`/kbd-memory-recall`)

New skill at `skills/process/kbd-process-orchestrator/skills/kbd-memory-recall/` with SKILL.md + helper script:

- Accepts `<phase-name>` (defaults to active phase from waypoint).
- Calls surreal-memory Graph-RAG `find_path` / `find_relevant` (per the surreal-memory MCP API) to retrieve up to N entities matching `kind = "kbd_lifecycle_event"` and a similarity threshold against the phase's goals + recent assessment.
- Writes a digest to `.kbd-orchestrator/phases/<phase>/prior-context.md` with:
  - 3–5 most relevant prior phases (linked by project + name + timestamp)
  - Their reflection.md key takeaways (if reachable via the memory's relations)
  - A "patterns observed" section the agent can use as planning input
- Emits Progress Signals.

The skill is invoked manually (or automatically by an `assess:before` augment hook — see below) at the start of each phase so the agent's planning step benefits from prior learning.

### Default-on automation

Add a **built-in augment hook** registered in `hooks.json`:

```json
{
  "id": "auto-memory-recall",
  "event": "assess:before",
  "mode": "augment",
  "action": {
    "type": "command",
    "command": "$KBD_ORCHESTRATOR_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh \"$KBD_HOOK_NAME\" 2>/dev/null || true",
    "timeout": 30,
    "on_failure": "ignore"
  }
}
```

So every `/kbd-assess` automatically gets a fresh `prior-context.md`. The hook is `augment` (doesn't suppress anything) and `on_failure: ignore` (a missing memory endpoint never blocks assess).

### Retention/relevance policy

Documented in `shared/references/memory-retention.md` (new):

- KBD lifecycle events are retained for 365 days; older entries are aged out by surreal-memory's existing retention model (configured via the server).
- Relevance for recall: events tagged with the same `project` get the highest score; same `kind` gets next; same `phase` name pattern gets next.
- Sensitive content: hook commands may emit stderr containing secrets; the memory writer captures only the structured `KBD_HOOK_*` payload, never the raw stderr from third-party hooks.

### Orchestrator documentation

Update `kbd-process-orchestrator/SKILL.md` "Surreal-Memory Integration" section:

- Reframe from "optional" to "default-on when reachable".
- Reference the new `kbd_memory_available` helper, the `kbd-memory-log` hook, and the `/kbd-memory-recall` skill.
- Document the entity schema and retention policy.

### Non-changes

- **No change to surreal-memory-server itself.** All work lives in the orchestrator skill set; the server is consumed as-is via its existing MCP API.
- **No removal of the JSONL hook log.** The JSONL log is the source of truth for in-flight events; the memory mirror is an additional, queryable index.
- **No proactive write to memory from existing skills** beyond the hook log mirror. Future skills can write more structured artifacts (e.g. reflection insights) — out of scope here.

## Capabilities

### New Capabilities

- `kbd-memory-first-execution`: A default-on integration with surreal-memory that mirrors every KBD hook fire into a queryable entity store, provides a `/kbd-memory-recall` skill for retrieving prior similar work as planning input, and defines the canonical event entity schema + retention policy for cross-project learning.

### Modified Capabilities

- None as separate spec entries. `kbd-process-hooks` gains an additional built-in augment hook (`kbd-memory-log`); its contract is unchanged.

## Impact

- **Risk**: Low-medium. The hook-log mirror writes happen out-of-band of KBD's actual lifecycle (the hook's `on_failure: ignore`); a broken memory endpoint never blocks development. The new `/kbd-memory-recall` skill is opt-in via the assess:before hook — projects that don't want auto-recall remove that hook entry.
- **Affected files**: orchestrator skill set only (this UAR repo benefits as a consumer but needs no code change). `shared/lib/memory.sh` (new), `skills/kbd-memory-recall/` (new directory), `hooks/hooks.json` (two new entries), `SKILL.md` (Surreal-Memory Integration section rewrite), `shared/references/memory-retention.md` (new).
- **Cross-repo**: Yes — same `prometheus-skill-system` repo, same topic-branch commit.
- **Reversibility**: Trivial — remove the two new hook entries and the new skill dir; the memory.sh helper is dead code without callers.
- **Unblocks**: Cross-project learning. Subsequent phases anywhere in the prometheus-* ecosystem benefit from compounded learning automatically.
