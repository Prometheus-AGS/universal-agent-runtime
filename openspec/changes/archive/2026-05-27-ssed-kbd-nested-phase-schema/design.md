## Context

The `kbd-process-orchestrator` skill set lives at `~/.claude/skills/kbd-process-orchestrator/`, sourced from `/Users/gqadonis/Projects/prometheus/prometheus-skill-system`. Its current `references/schemas/` directory contains three files: `hooks-config.schema.json`, `progress.schema.json`, and `project.template.json`. **There is no `current-waypoint.template.json` today** — the waypoint format is documented only inline in `SKILL.md` and inferred from `project.json` writes in each KBD skill (`kbd-assess`, `kbd-plan`, `kbd-execute`, `kbd-reflect`, `kbd-status`, `kbd-next-phase`).

This change does three things at once because they touch the same files in the same skill set:

1. **Introduces** `current-waypoint.template.json` as a first-class artifact.
2. **Extends** the in-flight waypoint object with three optional fields (`parentPhase`, `childPhases`, `childPointer`).
3. **Extends** `project.template.json` with one optional field (`worktreeRoot`, carried over from change 1's deferred task 4.3).
4. **Updates** the `kbd-status` skill to render worktree path + the parent-child phase chain (carried over from change 1's deferred task 4.2).

The cross-tool consumers of `current-waypoint.json` are at least five: Roo, Cursor, Codex, OpenCode, and Claude Code. None of them parse a JSON Schema — they all read the file as plain JSON and pluck out a handful of well-known fields. That fact dictates several decisions below.

## Goals / Non-Goals

**Goals**
- Add `parentPhase`, `childPhases`, `childPointer`, and `worktreeRoot` as **optional, additive** fields.
- Define cross-field invariants (pointer must reference a valid child; no duplicate children) and enforce them on **write**, not on **read** — readers stay lenient, writers stay strict.
- Ship a `current-waypoint.template.json` so future tools have a canonical reference.
- Update `kbd-status` to render worktree + nested-phase information without breaking pre-schema waypoints.
- Provide regression fixtures covering both pre-schema and fully-populated waypoints.

**Non-Goals**
- **No JSON Schema validator at runtime.** Adding ajv/jsonschema would introduce a node dependency to a skill that's currently pure markdown + jq-friendly JSON; not worth it for four optional fields.
- **No new skills.** `/kbd-new-child`, `/kbd-next-child`, `/kbd-new-phase` are separate changes.
- **No migration of in-flight waypoint files.** The new fields appear only when a future writer populates them.
- **No change to existing writers** (`kbd-plan`, `kbd-execute`, `kbd-next-phase`, etc.). They keep emitting the same fields they emit today; the new fields are populated only by future skills that explicitly opt in. Adding emission to existing writers is deliberately out of scope (the field they would emit — `parentPhase` — would always be `null` for them, so omitting it is correct).

## Decisions

### D1. Duck-typed loader, not JSON Schema enforcement

Every existing waypoint reader does `JSON.parse(...)` (or `jq`) and pulls out `phase`, `previousPhase`, etc. with safe defaults. We keep that style: the documented behavior is "any field that's not present takes its documented default; any field whose value has the wrong type is treated as absent and a warning is logged." No runtime validator is introduced. The `template.json` and the prose in `SKILL.md` are the source of truth.

Rationale: the alternative (ship a schema, require every tool to validate) creates a new failure mode (validator disagrees with tool's actual usage) and a new dependency tree (ajv, etc.). The cost outweighs the four optional fields' weight.

### D2. Cross-field invariants are writer-enforced

`childPointer` must be in `childPhases`; `childPhases` must have no duplicates. These are enforced wherever a tool writes the file (i.e. `/kbd-new-child` and `/kbd-next-child` in change 5). The waypoint *reader* in `kbd-status` does not fail when invariants are violated — it renders best-effort and warns. This keeps a corrupted file diagnosable rather than catastrophic.

### D3. The U+203A `›` separator with ASCII fallback

The phase-chain rendering uses `›` (U+203A) because it's the standard visual breadcrumb separator and renders in every modern terminal. For terminals that explicitly opt out of UTF-8 (`LANG=C`, `LC_ALL=POSIX`) the skill falls back to ` > ` (space-greater-space). This is implemented as a single helper `chain_separator()` in the orchestrator skill's shared script library (or inline if no shared library yet — the rendering is small enough to inline).

### D4. `worktreeRoot` storage format is a *literal* string with documented expansion semantics

The `project.json` field holds the string `"${HOME}/.claude/worktrees"` literally — not the expanded path. Each consumer expands `${HOME}` (and any other documented variable) at the point of use against its own environment. This keeps the file portable across users and machines without per-user diffing.

Consumers that want a stricter "no env variables, just an absolute path" mode can write an absolute path directly; the spec already allows any non-empty string and treats it as authoritative.

### D5. Fixtures live next to the template

Regression fixtures go under `references/schemas/fixtures/waypoint/`:
- `pre-schema.json` — a flat waypoint as produced by today's `kbd-plan`.
- `parent-with-children.json` — `parentPhase=null`, `childPhases=["a","b","c"]`, `childPointer="b"`.
- `child-row.json` — `parentPhase="outer"`, `childPhases=[]`, `childPointer=null` (representing the active child).

A simple `bash` driver in `references/schemas/fixtures/test.sh` loads each via `jq` and asserts a single line of expected output. Bats is not required — `jq` is already a documented orchestrator dependency.

### D6. `kbd-status` rendering — order and stability

The rendered status is **append-only over time**: existing lines (`phase: …`, `change: …`, `status: …`, `progress: N / M`, etc.) keep their positions and formatting. The new lines (`worktree: …`, optional `children: i/n`, optional `(children defined, none active)` annotation) are added as a block immediately after `phase:` and before `change:`. This keeps human muscle memory and any grep-based scripts working.

### D7. Pre-schema detection without a schema version field

The schema's optional-field design means a single waypoint file can omit all four new fields and still be valid. We don't need a `__schemaVersion` *requirement*; we add a `__schemaVersion: "2"` field to the template **as documentation**, but the loader never reads it. Writers that produce the new format may set it; writers that don't, don't. This decision intentionally diverges from spec requirement 2 scenario "Template version marker" — the version marker exists in the template only, not in user files. The spec scenario stands; we just clarify here that it does not imply runtime enforcement.

### D8. Cross-repo commit strategy

This change writes files in two locations:
- `prometheus-skill-system` repo: `skills/kbd-process-orchestrator/references/schemas/*`, `skills/kbd-process-orchestrator/SKILL.md`, `skills/kbd-process-orchestrator/skills/kbd-status/SKILL.md`, and any shared rendering helpers.
- `universal-agent-runtime` repo (this worktree): tasks log, archived OpenSpec change.

The OpenSpec change *lives* in this repo; the actual file edits land in the skill-system repo. The `tasks.md` records both target paths and the commit SHAs after each landed batch.

## Implementation Sketch

### `references/schemas/current-waypoint.template.json` (new file)

```jsonc
{
  "__schemaVersion": "2",
  "__description": "Active KBD waypoint. All fields documented in kbd-process-orchestrator/SKILL.md.",

  "phase": "<phase-name>",
  "previousPhase": null,
  "change": null,
  "status": "assessment_ready",
  "currentTask": "<short imperative>",
  "nextPendingChange": null,
  "sourceTool": "<tool-name>",
  "exactNextCommand": "<slash command>",

  "parentPhase": null,
  "childPhases": [],
  "childPointer": null,

  "updatedAt": "<ISO-8601 UTC>"
}
```

### `references/schemas/project.template.json` patch

Add one field next to `active_phase`:

```jsonc
"worktreeRoot": "${HOME}/.claude/worktrees",
```

### `kbd-status` rendering pseudocode

```
project       = read_project_json()             # tolerant
waypoint      = read_waypoint_json()             # tolerant
worktreeRoot  = expand(project.worktreeRoot or "${HOME}/.claude/worktrees")
top           = try { git rev-parse --show-toplevel } else None

# phase chain
chain = []
if waypoint.parentPhase: chain.append(waypoint.parentPhase)
chain.append(waypoint.phase)
if waypoint.childPointer: chain.append(waypoint.childPointer)
phase_line = "phase: " + " " + sep + " ".join(chain).replace(" " + sep + " ", " " + sep + " ")
# (sep = "›" or " > " per D3)

# worktree line
if top is None:
    worktree_line = "worktree: (none — not inside a git checkout)"
elif top == worktreeRoot or not is_descendant(top, worktreeRoot):
    worktree_line = f"worktree: {top}  ⚠ outside worktreeRoot ({worktreeRoot})"
else:
    worktree_line = f"worktree: {top}"

# children annotation
if waypoint.childPhases and waypoint.childPointer:
    i = index_of(waypoint.childPhases, waypoint.childPointer) + 1
    children_line = f"children: {i}/{len(waypoint.childPhases)}"
elif waypoint.childPhases:
    children_line = "(children defined, none active)"
```

Per D6 the lines are emitted in this order: `phase`, `children?` (when present), `worktree`, then the rest (`change`, `status`, …).

## Risks

1. **Existing writers omit new fields permanently.** That's by design (D2 / Non-Goals) but means a child phase introduced by a future tool has to add the field to *its* waypoint write. Documented; not a blocker.
2. **`is_descendant` on macOS** — the check uses `pwd -P` on both sides to canonicalize, same pattern as change 1's `worktree-new.sh`. Symlinked `$HOME` is the one edge case; we accept the slight unsoundness rather than depend on GNU `realpath`.
3. **`__schemaVersion` confusion (D7)** — readers may see the field and try to switch behavior. The mitigation is to call out in `SKILL.md` that the marker is documentation only and that the *only* contract is the per-field default.
4. **`›` rendering in legacy terminals** — fall-back is `> ` (D3); not perfect but recognizable.
5. **`worktreeRoot` containing `${HOME}` not expanded by a consumer** — every consumer must remember to expand. We provide an `expand_kbd_path()` helper in the orchestrator's shared library to make this a one-liner; documented in `SKILL.md`.
6. **Cross-repo PR coordination (D8)** — the OpenSpec change archived here references commit SHAs that land in a different repo. The reader must follow the link; this is the same arrangement used for every cross-repo change in this phase.

## Alternatives Considered

- **JSON Schema enforcement.** Rejected — see D1. Heavy machinery for four optional fields.
- **Versioned waypoint files** (`current-waypoint.v2.json` next to the old one). Rejected — readers would need to know which to read, doubling parsing complexity for no compatibility win.
- **`children: <name1>, <name2>, …` rendering** instead of `<i>/<n>`. Rejected — long child lists make the status output noisy. Index + count is compact and combined with the chain renders enough context.
- **Embedding the rendering in each KBD skill rather than a shared helper.** Tolerated for now (the rendering is ~20 LOC); revisit when a third skill needs the same logic.
