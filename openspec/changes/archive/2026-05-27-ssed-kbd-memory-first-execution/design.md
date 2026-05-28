## Context

The surreal-memory-server lives at `/Users/gqadonis/Projects/prometheus/surreal-memory-server`. Its MCP interface exposes `create_entity`, `add_observations`, `add_relations`, `find_relevant`, and friends. The KBD orchestrator's existing `SKILL.md` mentions it as optional. Change 3's hook dispatcher already writes a JSONL log per phase — this change adds a parallel mirror into the memory store so queries (Graph-RAG, similarity, time-windowed) become possible.

The design is **mirror, don't replace**: the JSONL file remains the in-flight source of truth (fast, local, append-only, no external dependency); the memory store is a queryable secondary index that earns its keep through cross-phase and cross-project recall.

## Goals / Non-Goals

**Goals**
- Default-on memory mirror when the endpoint is reachable.
- Zero blocking impact when unreachable (`on_failure: ignore`).
- Stable entity schema documented once, consumed by recall + future tools.
- A working `/kbd-memory-recall` that produces a useful planning artifact.
- Retention/relevance policy that's understandable without reading code.

**Non-Goals**
- No change to surreal-memory-server.
- No proactive scoring/reranking server-side. Use whatever ordering `find_relevant` returns.
- No automatic mirror of arbitrary skill output — only the structured hook events.
- No memory-driven decisions (e.g. "auto-suggest a child phase"). That's a future skill.

## Decisions

### D1. Augment hook, not override

The memory mirror is a `mode: augment` hook so it composes with whatever other hooks the project has. The default `report-progress` reporter still fires; this is an additional consumer of the same event stream.

### D2. `entityId` is deterministic and idempotent

Composed from `<project>/<phase>/<kind>/<edge>/<index>/<timestamp>`. Same event fired twice (rare but possible during retries) produces the same id, letting surreal-memory de-dupe naturally via its existing entity semantics.

### D3. `kbd-memory-recall` uses Graph-RAG `find_relevant`, not raw SQL

We don't want to bind the recall skill to surreal-memory's storage format. The MCP layer exposes `find_relevant` (or equivalent) which returns top-N entities by similarity to a query embedding. The skill passes the phase's goals + recent assessment text as the query. If the API changes underneath, the skill keeps working as long as `find_relevant` exists.

### D4. Detection caching is process-local, not on-disk

`kbd_memory_available()` caches its result in a shell variable. A new process re-probes. We don't want a stale "endpoint up" cached on disk after the endpoint has gone down — the cost of re-probing is one HEAD request per skill invocation, which is fine.

### D5. Retention is the server's job, not the hook's

The hook writes; the server retains. We document the policy in `memory-retention.md` so users can configure their surreal-memory-server retention to match. The hook itself doesn't track age or perform GC.

### D6. Recall digest is markdown, not JSON

`prior-context.md` is human-readable so the agent can read it like any other phase artifact. The structured data lives in the memory store; the digest is the agent-facing summary.

### D7. Auto-recall via assess:before hook is configurable

The `auto-memory-recall` hook is shipped enabled in built-in `hooks.json`. A project can disable it by setting `enabled: false` in its `hooks-config.json` for an entry matching `id: "auto-memory-recall"` — same disable pattern as any other hook.

### D8. Schema is additive across versions

The `kbd_lifecycle_event` entity schema starts with the keys named in the spec. New keys can be added in later changes; consumers MUST tolerate unknown keys. Removals require a new entityType.

### D9. The cross-project learning surface lives on `project:<name>` relations

Every event entity relates to `project:<name>`. Recall queries can scope by project (default) or expand across projects. Cross-project recall is the user's stated goal — it's enabled by these relations and the recall skill's `--cross-project` flag (default off; documented for future use).

## Implementation Sketch

### `shared/lib/memory.sh`

```sh
# kbd_memory_available — 0 = reachable, 1 = not
_KBD_MEMORY_PROBED=""
_KBD_MEMORY_OK=""
kbd_memory_available() {
  if [[ -n "$_KBD_MEMORY_PROBED" ]]; then
    return "$_KBD_MEMORY_OK"
  fi
  _KBD_MEMORY_PROBED=1

  # 1. MCP tool detection — agent passes its tool list as KBD_AVAILABLE_TOOLS
  if [[ "${KBD_AVAILABLE_TOOLS:-}" == *create_entity* ]]; then
    _KBD_MEMORY_OK=0; return 0
  fi

  # 2. env-var endpoint
  local url="${UAR_MEMORY_MCP_URL:-${KBD_MEMORY_MCP_URL:-}}"
  if [[ -z "$url" && -f .kbd-orchestrator/memory.config.json ]]; then
    url="$(jq -r '.mcpEndpoint // empty' .kbd-orchestrator/memory.config.json 2>/dev/null)"
  fi
  if [[ -n "$url" ]] && command -v curl >/dev/null 2>&1; then
    if curl -fsS --max-time 2 "$url/healthz" >/dev/null 2>&1; then
      _KBD_MEMORY_OK=0; return 0
    fi
  fi

  _KBD_MEMORY_OK=1; return 1
}

kbd_memory_url() {
  printf '%s' "${UAR_MEMORY_MCP_URL:-${KBD_MEMORY_MCP_URL:-}}"
}
```

### `hooks/hooks.json` additions

```jsonc
{
  "id": "kbd-memory-log",
  "event": "*:*",
  "mode": "augment",
  "description": "Mirror every hook fire to surreal-memory (default-on when endpoint is reachable; no-op otherwise).",
  "action": {
    "type": "command",
    "command": "$KBD_ORCHESTRATOR_ROOT/shared/lib/memory-log.sh",
    "timeout": 5,
    "on_failure": "ignore"
  }
},
{
  "id": "auto-memory-recall",
  "event": "assess:before",
  "mode": "augment",
  "description": "Populate prior-context.md from surreal-memory before each /kbd-assess.",
  "action": {
    "type": "command",
    "command": "$KBD_ORCHESTRATOR_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh \"$KBD_HOOK_NAME\"",
    "timeout": 30,
    "on_failure": "ignore"
  }
}
```

### `shared/lib/memory-log.sh`

```sh
#!/usr/bin/env bash
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/memory.sh"
kbd_memory_available || exit 0

url="$(kbd_memory_url)"
[[ -n "$url" ]] || exit 0

project="$(jq -r '.project // "unknown"' .kbd-orchestrator/project.json 2>/dev/null || echo unknown)"
phase="${KBD_HOOK_PHASE_PATH%% *}"
entity_id="$project/$phase/$KBD_HOOK_KIND/$KBD_HOOK_EDGE/$KBD_HOOK_INDEX/$KBD_HOOK_STARTED_AT"

payload="$(jq -c -n \
  --arg eid "$entity_id" --arg proj "$project" --arg phase "$phase" \
  --arg kind "$KBD_HOOK_KIND" --arg edge "$KBD_HOOK_EDGE" --arg name "$KBD_HOOK_NAME" \
  --argjson index "$KBD_HOOK_INDEX" --argjson total "$KBD_HOOK_TOTAL" \
  --arg phasePath "$KBD_HOOK_PHASE_PATH" --arg srcTool "$KBD_HOOK_SOURCE_TOOL" \
  --arg ts "$KBD_HOOK_STARTED_AT" '
  {
    entityType: "kbd_lifecycle_event",
    entityId: $eid,
    observations: [{
      kind: $kind, edge: $edge, name: $name,
      index: $index, total: $total, phasePath: $phasePath,
      sourceTool: $srcTool, project: $proj, ts: $ts
    }],
    relations: [
      {from: $eid, to: ("phase:" + $phase),  label: "fires-in"},
      {from: $eid, to: ("project:" + $proj), label: "belongs-to"}
    ]
  }
')"

curl -fsS -X POST --max-time 3 \
  -H 'content-type: application/json' \
  -d "$payload" \
  "$url/api/entities" >/dev/null 2>&1 || true
```

### `kbd-memory-recall.sh` (high level)

Reads phase name (arg or waypoint), reads the phase's `goals.md` + `assessment.md` (if present) as the recall query, POSTs to `<url>/api/find_relevant` with `entityType=kbd_lifecycle_event`, top-N (default 5), writes a markdown digest. Falls back to a stub `prior-context.md` when memory unavailable.

## Risks

1. **Memory schema drift.** Future changes might want more fields. Mitigation: D8 (additive).
2. **Secret leakage.** Third-party hooks could print secrets to stderr. Spec req 2 scenario "No leakage" prevents the memory writer from capturing stderr; only the structured `KBD_HOOK_*` payload is mirrored.
3. **Probe latency stacking.** Each skill invocation re-probes once. 2s timeout × N skills could feel slow. Mitigated by D4's process-local cache.
4. **Cross-project privacy.** Cross-project recall could leak project A's patterns into project B. Mitigation: D9 — cross-project recall is opt-in via `--cross-project` flag (off by default; documented for future use).
5. **`auto-memory-recall` runs on every assess:before.** Could be expensive on small projects. Mitigation: D7 — disable via project hooks-config.

## Alternatives Considered

- **Write directly from each KBD skill (no hook).** Rejected — duplicates the dispatch logic and bypasses the audit log.
- **Use `find_path` instead of `find_relevant`.** Rejected — `find_path` is for tracing specific paths; recall wants similarity, which is `find_relevant`.
- **Store the JSONL log itself in the memory backend.** Rejected — the JSONL is line-oriented + append; memory stores entities; round-tripping loses structure.
- **Push assessment.md content into memory entirely.** Tempting but tangential — the assessment is the consumer of recall, not the source.
