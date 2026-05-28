## Why

KBD already has the seed of a hooks system: `~/.claude/skills/kbd-process-orchestrator/hooks/hooks.json` ships built-in milestone hooks (`on_phase_complete`, `on_plan_complete`, `on_reflection_complete`, etc.) and `references/schemas/hooks-config.schema.json` documents a more general `<step>:<event>` form (e.g. `assess:begin`, `plan:end`, `*:*`) with `command` actions, `timeout`, and `on_failure` semantics. **What's missing** for the user's stated need:

1. **Coverage of every lifecycle boundary the user named.** Today the snake_case events cover phase/change/assessment/plan/reflection but **not** child phases, plan-start, execute-start, execute-end, or per-OpenSpec-task fire-around. The `<step>:<event>` schema is general enough but nothing emits those events yet.
2. **Augment-vs-override semantics.** All registered hooks fire in series today; there is no way to *replace* the default reporter line — only add more output. The user explicitly asked for both modes.
3. **A built-in progress reporter** emitting the exact line `starting/ending <kind> <name> [<i>/<n>]` so a developer always knows where they are without configuring anything. Today the only output is the per-skill `Starting/Completed kbd-* — <phase>` lines hard-coded in each skill's "Progress Signals (MANDATORY)" section. Those lines aren't structured (no index/total), aren't uniform across all loop levels, and aren't extensible.
4. **A durable log** so the sequence of hook fires can be replayed and audited after the fact — useful for surreal-memory logging (change 6) and the rule-injector skill (change 7) which will both consume this stream.
5. **Per-skill wiring.** Each KBD skill (and the forthcoming `/kbd-new-child`, `/kbd-next-child`) must emit hook events at the documented boundaries without duplicating boilerplate — a shared helper is the right shape, designed alongside `shared/lib/waypoint.sh` introduced in the previous change.

This change is the foundation for changes 4, 5, 6, 7 in the same wave — every one of them either *consumes* hooks (memory-log, rule-injector) or *emits* them (new-phase, child-phase skills). Landing it now keeps the rest of W1 hook-aware from day 1.

## What Changes

### Event taxonomy (extension)

The canonical event form becomes `<kind>:<edge>` where:

- `kind` ∈ `phase` | `child` | `plan` | `execute` | `reflect` | `task` | `*`
- `edge` ∈ `before` | `after` | `*`

Aliases for backward compatibility (kept working but deprecated in docs):
- `on_phase_complete` → `phase:after`
- `on_plan_complete` → `plan:after`
- `on_reflection_complete` → `reflect:after`
- `on_assessment_complete` → `assess:after` (note: `assess` keeps its existing kind name from the schema)
- `on_change_complete` → `execute:task:after` (special-case: an OpenSpec-style change reaching DONE is the *final* task event of its execute loop)
- `on_blocker_detected`, `on_cross_tool_handoff` keep their event names — they aren't lifecycle boundaries, they're situational events.

### Hook context payload

Every hook receives a uniform context, accessible as environment variables (matching the existing schema's substitution surface):

```
KBD_HOOK_KIND        # phase | child | plan | execute | reflect | task
KBD_HOOK_EDGE        # before | after
KBD_HOOK_NAME        # the active item's canonical name (phase name, change id, task title)
KBD_HOOK_INDEX       # 1-based index within the containing loop (or 1 when no loop)
KBD_HOOK_TOTAL       # total count in the containing loop (or 1 when no loop)
KBD_HOOK_PHASE_PATH  # parent › phase › child chain (uses chain_separator from waypoint.sh)
KBD_HOOK_CHILD_PATH  # current child name or empty
KBD_HOOK_SOURCE_TOOL # claude-code | codex | roo | cursor | opencode | …
KBD_HOOK_STARTED_AT  # ISO-8601 UTC timestamp
```

Existing `${PHASE}`, `${STEP}`, `${EVENT}`, `${KBD_PROJECT_NAME}`, `${KBD_PROJECT_ROOT}`, `${CHANGES_TOTAL}`, `${CHANGES_COMPLETED}`, `${LAST_UPDATED_BY}`, `${TIMESTAMP}` substitutions continue to work; the new variables are additive.

### Discovery order

Hooks are loaded in three layers and concatenated in the order each layer is listed:

1. **Built-in defaults** — `~/.claude/skills/kbd-process-orchestrator/hooks/hooks.json` (this repo).
2. **User-level** — `~/.claude/skills/kbd-process-orchestrator/hooks/user.json` if present.
3. **Project-local** — `.kbd-orchestrator/hooks-config.json` in the active project (already documented in the existing schema).

### Augment vs. override mode

Each hook entry gains an optional `mode` field:

- `mode: "augment"` (default) — the hook fires alongside every other hook for the same event.
- `mode: "override"` — the hook replaces the built-in default reporter for this event. At most one `override` per `(kind, edge)` may resolve; if more than one is declared across layers, **the last-loaded wins** (project beats user beats built-in) and a warning is emitted naming both override sources.

### Default `report-progress` reporter

Ship a built-in hook with `event: "*:*"` that emits the exact line the user requested:

```
starting <kind> <name> [<index>/<total>]   # on every *:before
ending  <kind> <name> [<index>/<total>]    # on every *:after
```

Lines go to **stderr** so they don't pollute structured stdout. They are emitted as plain text — no slash, no decoration — and follow the spacing convention `[<i>/<n>]` so a grep like `^starting ` works for downstream tooling.

A project that wants to customize wholesale ships a single `override` hook covering `*:*`. A project that wants to *add* (e.g. push to Slack on `phase:after`) ships an `augment` hook for that specific event.

### Per-event dispatch wiring

Every KBD skill is wired to fire `<kind>:before` and `<kind>:after` around its work:

| Skill | Emits |
|---|---|
| `kbd-assess` | `phase:before`/`assess:before` *(pair)*, … `assess:after`/`phase:after` *(pair)* — `phase:*` only fires if assess is the first/last skill of a phase, see "Pairing rules" |
| `kbd-plan`   | `plan:before` … `plan:after` |
| `kbd-execute` | `execute:before` … `execute:after`; inside, per OpenSpec task or per native KBD task: `task:before` … `task:after` |
| `kbd-reflect` | `reflect:before` … `reflect:after` |
| `kbd-new-phase` (change 4) | `phase:before` for the new phase |
| `kbd-next-phase` | `phase:after` for the closing phase, `phase:before` for the new phase |
| `kbd-new-child` (change 5) | `child:before` |
| `kbd-next-child` | `child:after` for the closing child, `child:before` for the new one |

#### Pairing rules

- `phase:before` fires exactly once per phase, at the start of the first skill invocation that owns the new phase (today: `/kbd-new-phase`, or the `/kbd-plan` step that flips `activePhase` if `/kbd-new-phase` isn't run — both will be auditable).
- `phase:after` fires exactly once per phase, at the *end* of `/kbd-reflect` (the canonical "phase done" boundary).
- `child:before`/`after` fire only when `/kbd-new-child`/`/kbd-next-child` is the active control flow — i.e. only when the waypoint's nested-phase fields are populated.
- `task:before`/`after` fire **per OpenSpec task** during `/opsx:apply` (and per native-KBD task when OpenSpec isn't used). Index = task number within the change; Total = task count for the change.

### Hook log

Persist every hook fire to `.kbd-orchestrator/phases/<phase>/hooks.log.jsonl`, one JSON object per line:

```json
{"ts":"2026-05-27T00:00:00Z","kind":"task","edge":"before","name":"1.1 create script","index":1,"total":7,"phasePath":"ssed › w1","sourceTool":"claude-code","status":0}
```

`status` is the exit code of the hook command (0 = success). Failures recorded with non-zero exit codes plus the captured stderr line (truncated to 200 chars). Log rotation is the user's responsibility for now; a rotation policy is out of scope here (filed as a follow-up).

### Coexistence with existing `Progress Signals (MANDATORY)` sections

Every KBD skill still emits its `Starting/Completed kbd-<skill> — <phase>` line per its current contract — those are agent-facing signals. The new hook output is *complementary*, not a replacement; the `report-progress` reporter line is on stderr and uses a distinctly different format (`starting <kind> <name> [<i>/<n>]` vs `Starting kbd-<skill> — <phase>`).

### Documentation

- Update `kbd-process-orchestrator/SKILL.md` with a "Hooks" section: event taxonomy, discovery order, augment-vs-override, payload, JSONL log location.
- Cross-link from each KBD skill's `SKILL.md` to that section so the wiring is discoverable from where the events fire.
- Update `references/schemas/hooks-config.schema.json` to add the `mode` field and the new event kinds.

### Non-changes

- **No removal** of existing snake_case events. They become documented aliases; remove no earlier than after `/kbd-new-child` and `/kbd-next-child` (changes 5) are in production.
- **No mandatory hook execution platform.** Hook commands are still plain shell, as today; we don't introduce a runtime (no Node, no Python).
- **No remote hook dispatch.** Hooks run locally only; remote/webhook integration is out of scope.

## Capabilities

### New Capabilities

- `kbd-process-hooks`: Event-driven extensibility surface across the KBD lifecycle — `<kind>:<edge>` events (`phase`, `child`, `plan`, `execute`, `reflect`, `task`), augment-vs-override mode, three-layer discovery (built-in / user / project), a uniform context payload, a built-in `report-progress` reporter emitting `starting/ending <kind> <name> [<i>/<n>]`, and a `.kbd-orchestrator/phases/<phase>/hooks.log.jsonl` audit trail.

### Modified Capabilities

- None as separate spec entries. The existing KBD skills (`kbd-assess`, `kbd-plan`, `kbd-execute`, `kbd-reflect`, plus the forthcoming `/kbd-new-phase`, `/kbd-new-child`, `/kbd-next-child`) each gain a documented "fires hook events at boundary X" contract; that's a wiring obligation captured in tasks.md, not a stand-alone capability spec.

## Impact

- **Risk**: Medium. The wiring touches every KBD skill, and the event-name aliasing across the old `on_*` form and the new `<kind>:<edge>` form must not break any existing project-local `hooks-config.json`.
- **Affected files** (skill-system):
  - `skills/process/kbd-process-orchestrator/hooks/hooks.json` (extend defaults)
  - `skills/process/kbd-process-orchestrator/references/schemas/hooks-config.schema.json` (add `mode`, new event kinds)
  - `skills/process/kbd-process-orchestrator/shared/lib/hooks.sh` (new dispatcher helper, mirrors `waypoint.sh`)
  - `skills/process/kbd-process-orchestrator/SKILL.md` (new "Hooks" section)
  - `skills/process/kbd-process-orchestrator/skills/{kbd-assess,kbd-plan,kbd-execute,kbd-reflect,kbd-next-phase}/SKILL.md` (each gains the wiring stanza)
- **Cross-repo**: Yes — all edits in `prometheus-skill-system`; this UAR repo only sees archive + task log.
- **Reversibility**: Trivial — remove the dispatcher source, drop the new `mode` field, restore the bare snake_case event handlers. No data migration required.
- **Unblocks**: changes 4, 5 (new-phase / child-phase skills can emit events from day 1), 6 (memory-log via an augment hook), 7 (rule-injector consumes `phase:before`).
