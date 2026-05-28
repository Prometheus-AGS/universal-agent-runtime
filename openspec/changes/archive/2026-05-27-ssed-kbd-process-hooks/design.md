## Context

The `kbd-process-orchestrator` skill set ships at `~/.claude/skills/kbd-process-orchestrator/` (symlinked into `prometheus-skill-system/skills/process/kbd-process-orchestrator/`). Two files are already in place that this change extends rather than replaces:

- `hooks/hooks.json` — ships eight built-in entries today (`on_phase_complete`, `on_change_complete`, `on_assessment_complete`, `on_plan_complete`, `on_cross_tool_handoff`, `on_blocker_detected`, `on_reflection_complete`, plus the structural envelope).
- `references/schemas/hooks-config.schema.json` — already describes a `<step>:<event>` event pattern (`assess:begin`, `*:end`, etc.), `command` actions with `${PHASE}`, `${STEP}`, `${EVENT}` substitution, `timeout` (default 15 s), and `on_failure` ∈ `warn | error | ignore`.

This means three distinct event-naming conventions are at play right now:

1. The **snake_case milestone events** actually shipped in `hooks.json`.
2. The **`<step>:<event>` pattern** described in the schema (but no skill emits these — the wiring is missing).
3. The **`<kind>:<edge>` form** introduced by this change, which generalises (2).

The change defines an overlap: `<kind>:<edge>` *replaces* the schema's `<step>:<event>` form (the schema becomes a strict subset relabelled), and the snake_case events become aliases. We pick one canonical form going forward.

This is also the first change in W1 that depends on `shared/lib/waypoint.sh` (added in the previous change) — we reuse `chain_separator` and waypoint loading rather than reimplementing them.

## Goals / Non-Goals

**Goals**
- A single dispatcher entry point any KBD skill can call: `kbd_hooks_fire <kind> <edge> <name> <index> <total>` — uniform, side-effect only.
- Backward-compatible: every existing `hooks-config.json` continues to work without edits.
- Augment-vs-override resolution is deterministic and traceable (you can ask "why didn't my hook fire?" and get an answer from the log).
- The default `report-progress` reporter is itself a hook entry — no hard-coded behavior. Override semantics are tested by overriding the built-in.
- Per-skill wiring is **one line per emit point** (`kbd_hooks_fire phase before "$phase" 1 1`), no boilerplate.

**Non-Goals**
- **No event bus, no IPC, no remote dispatch.** The dispatcher is in-process; hook commands are spawned children.
- **No log rotation policy** in this change. JSONL grows; users can rotate manually. Filed as a follow-up.
- **No removal of the snake_case events** in this change. They're aliases now; deprecation is staged separately (after `/kbd-new-child` / `/kbd-next-child` ship in W1's tail).
- **No new dependencies.** Bash 3.2 + `jq` (already a documented orchestrator dependency). No node, no python, no awk-fragile parsers.
- **No automatic hook execution platform.** Hook commands remain plain shell, as the existing schema already specifies.

## Decisions

### D1. Dispatcher is a sourceable bash helper, not a separate binary

The dispatcher lives at `skills/process/kbd-process-orchestrator/shared/lib/hooks.sh`. Skills source it the same way they source `waypoint.sh`:

```sh
. "${KBD_ORCHESTRATOR_ROOT}/shared/lib/hooks.sh"
kbd_hooks_fire phase before "$phase_name" 1 1
```

Rationale: a single source-able file is the smallest possible footprint and integrates cleanly with the existing skill ecosystem, which is markdown-described. A separate binary would force a PATH entry and a build step for what is ultimately string manipulation over `hooks.json`.

### D2. Canonical event form is `<kind>:<edge>`; aliases are normalised at registration time

The dispatcher normalises every loaded hook's `event` string to the canonical form before dispatching. The mapping table lives **inside the dispatcher** (not in user files), so users see the original strings in `hooks.json` but the matcher only ever evaluates canonical events.

Aliases:

| Legacy event | Canonical event |
|---|---|
| `on_phase_complete` | `phase:after` |
| `on_plan_complete` | `plan:after` |
| `on_reflection_complete` | `reflect:after` |
| `on_assessment_complete` | `assess:after` |
| `on_change_complete` | (sentinel) — see D5 |
| `on_blocker_detected` | (unchanged — situational, not lifecycle) |
| `on_cross_tool_handoff` | (unchanged — situational, not lifecycle) |
| `<step>:begin` | `<step>:before` |
| `<step>:end` | `<step>:after` |

The schema's older `begin`/`end` edge names are aliased to `before`/`after`. We keep `before`/`after` as the canonical English because they match conventional middleware vocabulary and the user's request.

### D3. Override resolution is layer-first, last-wins within a layer

For any given `(kind, edge)` dispatch point:

1. Collect all matching entries across all three layers.
2. If zero or one `mode: "override"` entries exist, fire them as-is plus all `augment` entries.
3. If more than one `override` exists, pick the entry from the highest layer (project > user > builtin). Within a single layer (uncommon but possible), pick the last-registered entry. Emit a single warning naming both the winner and every loser, e.g.:

```
warn: override conflict on phase:after — using project/id-foo, suppressing project/id-bar, user/id-baz
```

The warning goes to stderr. The dispatch does not abort.

### D4. JSONL append uses `flock` for safety, falls back gracefully on macOS without `flock`

Writes to `hooks.log.jsonl` are append-only one-line-at-a-time. Two skills running concurrently (e.g. Roo + Codex) could interleave. We use `flock` on the log file when available; macOS lacks `flock(1)` by default, so the fallback is a `mkdir`-based lock directory next to the log (`hooks.log.jsonl.lockd`) with a 5-second acquire timeout. Past the timeout, append anyway and record `"status": "lock-timeout"` in the entry — losing a line is worse than blocking the user's skill.

Each entry is built with `jq -c -n …` so the JSON is well-formed even when names contain special characters.

### D5. `on_change_complete` alias: detect "final task" with a sentinel

Today `on_change_complete` is one event. After this change it's *the last `task:after` of the change's execute loop*. The dispatcher detects "final" by comparing `KBD_HOOK_INDEX == KBD_HOOK_TOTAL` at the time of the fire and, only then, ALSO firing any registered `on_change_complete` hooks. This preserves the existing semantics for projects that haven't migrated yet and gives new code a cleaner per-task hook.

This is the **only** alias that requires runtime conditional dispatch; every other alias is a pure rename normalized at registration time.

### D6. Default reporter lives in `hooks.json`, not in the dispatcher

The built-in `report-progress` hook is an entry in `hooks/hooks.json`:

```json
{
  "id": "report-progress",
  "event": "*:*",
  "mode": "augment",
  "description": "Built-in: emits 'starting/ending <kind> <name> [<i>/<n>]' to stderr.",
  "action": {
    "type": "command",
    "command": "if [ \"$KBD_HOOK_EDGE\" = \"before\" ]; then verb=starting; else verb=ending; fi; printf '%s %s %s [%s/%s]\\n' \"$verb\" \"$KBD_HOOK_KIND\" \"$KBD_HOOK_NAME\" \"$KBD_HOOK_INDEX\" \"$KBD_HOOK_TOTAL\" >&2",
    "timeout": 5,
    "on_failure": "ignore"
  }
}
```

This is deliberately a regular hook entry so that the override path is exercised by the default behavior itself. A project's `mode: "override"` covering `"*:*"` will *actually replace* this entry; you don't need a special "disable default" flag.

### D7. Per-skill wiring is one line per emit point

The dispatcher exposes one user-facing function:

```sh
kbd_hooks_fire <kind> <edge> <name> [<index> <total>]
```

`<index>` and `<total>` default to `1`. Skills add two calls — one before their work, one after — at the boundaries specified by spec §"Per-Skill Wiring". A shared snippet documented in `SKILL.md` shows the exact form to paste; no helper library beyond `hooks.sh` is needed.

### D8. Forward dispatch points without active phases write to a fallback log path

`kbd_hooks_fire` resolves the current phase from `.kbd-orchestrator/current-waypoint.json` (via `waypoint_load`). When no phase directory exists yet (i.e. `/kbd-init` is mid-flight), the log writes to `.kbd-orchestrator/hooks.log.jsonl` at the project root. This is the only place the log path differs from the in-phase path, and the spec scenario for it is explicit.

### D9. Schema additions are strictly additive

`hooks-config.schema.json` gains:

- `properties.mode` with `enum: ["augment","override"]`, `default: "augment"`.
- `event` description updated to enumerate canonical kinds and edges.

The `required: ["id", "event", "action"]` list does not change. Older config files (no `id`, e.g. the built-in `hooks.json` entries today) are kept working by **synthesising** an id from the file path and event when one isn't present: `<layer>/<event>/<index>`. The schema doesn't require this synthesis, but the dispatcher performs it so every log entry has a stable `hookId`.

### D10. No native `task:*` events for `/kbd-execute` until `/opsx:apply` integration is wired

Today `/opsx:apply` is the routine that iterates OpenSpec tasks. `/kbd-execute` writes the dispatch contract but doesn't itself walk tasks. The wiring described in spec §"Per-Skill Wiring" — "`/kbd-execute` (or `/opsx:apply`) advances from one OpenSpec task to the next" — therefore lands in two places: a `kbd-execute` emit at execute boundary, and an `opsx-apply` integration that calls `kbd_hooks_fire task before/after` around each task it processes. The latter is in scope for this change and is documented in tasks.md.

## Implementation Sketch

### `shared/lib/hooks.sh` (new)

```sh
# shellcheck shell=bash
# kbd-process-orchestrator/shared/lib/hooks.sh

kbd_hooks_fire() {
  local kind="$1" edge="$2" name="$3"
  local index="${4:-1}" total="${5:-1}"
  : "${KBD_ORCHESTRATOR_ROOT:?must be set by caller}"

  # 1. Discover layer paths
  local builtin="$KBD_ORCHESTRATOR_ROOT/hooks/hooks.json"
  local user_layer="$KBD_ORCHESTRATOR_ROOT/hooks/user.json"
  local project_layer=".kbd-orchestrator/hooks-config.json"

  # 2. Load + normalise into a single jq pipeline that emits each
  #    matching entry as one line of { id, mode, command, layer }
  local matches
  matches="$(_kbd_hooks_collect "$builtin" "$user_layer" "$project_layer" "$kind" "$edge")"

  # 3. Override resolution (D3)
  matches="$(_kbd_hooks_resolve_overrides "$matches")"

  # 4. For each remaining entry, build the context env and run the command
  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    _kbd_hooks_run "$entry" "$kind" "$edge" "$name" "$index" "$total"
  done <<< "$matches"

  # 5. on_change_complete alias (D5): runtime conditional
  if [ "$kind" = "task" ] && [ "$edge" = "after" ] && [ "$index" = "$total" ]; then
    _kbd_hooks_run_alias on_change_complete "$name" "$index" "$total"
  fi
}
```

The four helpers (`_kbd_hooks_collect`, `_kbd_hooks_resolve_overrides`, `_kbd_hooks_run`, `_kbd_hooks_run_alias`) are private and live in the same file. `_kbd_hooks_run` is responsible for the JSONL append; it uses `flock` when present (`type flock >/dev/null 2>&1`) and the directory-lock fallback otherwise.

### `hooks/hooks.json` (extended)

Existing entries remain. The new built-in `report-progress` entry is appended.

### `references/schemas/hooks-config.schema.json` (patched)

```jsonc
"mode": {
  "type": "string",
  "enum": ["augment", "override"],
  "default": "augment",
  "description": "augment: this hook fires alongside others on the same event (default). override: this hook replaces the built-in default reporter for this dispatch point; at most one override per (kind, edge) resolves — last layer wins (project > user > builtin)."
}
```

`event` description rewritten to enumerate canonical kinds + edges and call out alias compatibility.

### Per-skill wiring (one snippet)

Documented in orchestrator `SKILL.md` → "Hooks" section:

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"
kbd_hooks_fire plan before "$phase_name" 1 1
# ... do the work ...
kbd_hooks_fire plan after "$phase_name" 1 1
```

For per-task wiring inside `/opsx:apply`:

```sh
i=0; n=$(grep -c '^- \[' "$tasks_md" || echo 0)
while read -r task; do
  i=$((i+1))
  kbd_hooks_fire task before "$task" "$i" "$n"
  # ... implement task ...
  kbd_hooks_fire task after "$task" "$i" "$n"
done < <(extract_tasks "$tasks_md")
```

## Risks

1. **Concurrency**: Two skills running concurrently in different worktrees both write to the same phase log. Mitigated by D4 (`flock` + directory-lock fallback). The fallback's 5-second timeout means a degenerate case may produce one entry with `status: "lock-timeout"`; that's acceptable signal vs. losing the line entirely.
2. **`jq` availability on minimal systems**: `jq` is already a documented orchestrator dependency (`waypoint.sh` requires it too). If absent, `kbd_hooks_fire` logs a single warning to stderr and silently no-ops. We don't want hooks to be the reason a fresh checkout fails.
3. **Stderr noise**: the default reporter writes to stderr on every dispatch — potentially many lines per change. Mitigated because (a) stderr is independently capturable, (b) the line format is stable and greppable, (c) a project can disable the reporter wholesale with a single `mode: "override"` entry covering `"*:*"`.
4. **Override conflict warnings**: developers may accidentally introduce multiple overrides across layers. The warning names both winner and losers so the conflict is debuggable from a single log read.
5. **Backward-compat surface**: every existing project-level `hooks-config.json` may use `<step>:<event>` or `<step>:begin`/`end` or snake_case. The alias table (D2) covers them all; the dispatcher logs an info-level "normalised X → Y" line for each loaded entry when `KBD_HOOK_DEBUG=1` is set in the environment, to make migration debuggable.
6. **`task:*` index/total accuracy**: counting tasks in `tasks.md` is regex-based (`grep -c '^- \['`). If a `tasks.md` uses non-standard checkbox indent or different markers, the count is wrong. Mitigated because every change in this phase uses the same checkbox convention; if a future change deviates, it must override the counter.

## Alternatives Considered

- **Rewrite as a node CLI** for richer JSON handling and a real event emitter. Rejected — adds a node dependency and a build step for what is, structurally, "loop over JSON entries, spawn a child process per match". The shell version is ~150 LOC.
- **Drop snake_case aliases**, force migration. Rejected — `hooks.json` ships them today; existing user configs would break for zero gain.
- **Make `report-progress` part of the dispatcher**, not a hook entry. Rejected — D6's "default reporter is itself a hook" gives us free dogfooding of the override path.
- **Per-event log files** (`phase-after.jsonl`, `task-after.jsonl`, …). Rejected — `hooks.log.jsonl` is small enough; one file makes replay and audit trivial; a downstream tool can split on `kind`/`edge` keys.
- **Synchronous-only dispatch.** Today's spec is synchronous (each hook command runs to completion before the next fires). An async option would need a per-event ordering policy we don't yet need. Filed as a possible future option behind a `mode: "augment-async"` value.
