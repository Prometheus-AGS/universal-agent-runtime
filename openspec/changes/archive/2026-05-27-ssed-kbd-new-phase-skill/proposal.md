## Why

`kbd-process-orchestrator/SKILL.md` documents `/kbd-new-phase <name> [goals...]` as one of the core commands ("Quick Start Commands" lists it; the orchestrator's `/kbd-full-phase` flow references it) — but **no skill directory exists** under `skills/process/kbd-process-orchestrator/skills/`. The companion `/kbd-next-phase` *is* implemented (it auto-seeds from the previous reflection); `/kbd-new-phase` is the manual-entry counterpart for cases where:

- The user is starting the very first phase of a new project (no prior reflection exists).
- The user wants to pivot to a phase that isn't the suggestion in the prior reflection.
- An operator is initialising state by hand and needs a one-shot skill instead of editing `current-waypoint.json` directly.

This change closes the documented-but-missing gap. It is also the **first** skill in this phase plan to act on the schema introduced by change 2 (`kbd-nested-phase-schema`): every new phase row this skill writes carries the documented `parentPhase: null`, `childPhases: []`, `childPointer: null` defaults so downstream tools see a complete waypoint from day 1. And it is the **first writer** of `phase:before` per change 3 (`kbd-process-hooks`) — a phase newly created here fires its own opening boundary event.

## What Changes

### New skill directory

Create `skills/process/kbd-process-orchestrator/skills/kbd-new-phase/` with:

- `SKILL.md` — front-matter, "/kbd-new-phase" command, "What this does" steps, Progress Signals contract, prerequisites, How to invoke, Examples, **Hook integration** subsection (mirroring the pattern landed in change 3).
- `kbd-new-phase.sh` — POSIX bash 3.2 script that the SKILL.md "How to invoke" steps call. Validates the name, refuses to overwrite an existing phase, creates the phase directory tree, writes the initial `goals.md` and `progress.json`, flips `current-waypoint.json` and `project.json`'s `active_phase`, and emits the confirmation banner.

### Argument shape

```
/kbd-new-phase <name> [goal-1] [goal-2] [goal-n]...
```

- `<name>`: required. Validated identically to `/kbd-next-phase` — kebab-case (`^[a-z0-9][a-z0-9._-]*$`), no path traversal, no slashes, refuses if `.kbd-orchestrator/phases/<name>/` already exists.
- `[goals…]`: every remaining argument is treated as one bullet in `goals.md`. If no goals are provided, `goals.md` carries a single `<!-- TBD: enumerate goals before /kbd-assess -->` stub line so it's never written empty.

### Files written

```
.kbd-orchestrator/phases/<name>/
├── goals.md         # bulleted list from CLI args, or TBD stub
└── progress.json    # skeleton — see below
```

`progress.json` skeleton (matches the shape used by existing phases, plus the nested-phase fields from change 2):

```jsonc
{
  "phase": "<name>",
  "parentPhase": null,
  "childPhases": [],
  "childPointer": null,
  "assessment_complete": false,
  "plan_complete": false,
  "execute_complete": false,
  "reflect_complete": false,
  "changes_total": 0,
  "changes_completed": 0,
  "completed_changes": [],
  "active_change": null,
  "blocked_changes": [],
  "sourceTool": "<from current-waypoint or 'unknown'>",
  "createdBy": "kbd-new-phase",
  "updatedAt": "<ISO-8601 UTC>"
}
```

### Waypoint flip

`current-waypoint.json` updates:

- `previousPhase` ← prior `phase`
- `phase` ← `<name>`
- `change` ← null
- `status` ← `"assessment_ready"`
- `currentTask` ← `"run kbd-assess for <name>"`
- `nextPendingChange` ← null
- `exactNextCommand` ← `/kbd-assess <name>`
- `parentPhase` / `childPhases` / `childPointer` ← `null` / `[]` / `null` (top-level)
- `updatedAt` ← now

`project.json` updates:
- `activePhase` ← `<name>`
- `updatedAt` ← now

### Hook fire

The skill fires `phase:before` exactly once for `<name>` after the waypoint flip and before emitting its `Completed kbd-new-phase — <name>` Progress Signal. This is the *opening* boundary for the new phase; the *closing* `phase:after` is the responsibility of `/kbd-reflect` for the previous phase (which should have run already).

### Orchestrator documentation closes the gap

Update `kbd-process-orchestrator/SKILL.md`:

- Remove the implicit "referenced but not implemented" status of `/kbd-new-phase` (today the skill is listed in "Quick Start Commands" without a directory backing it).
- Confirm `/kbd-new-phase` is now a real skill in the per-skill list.
- Cross-link the new "Hook integration" subsection back to the "Hooks" section.

### Non-changes

- **No change to `/kbd-next-phase`.** They are siblings, not refactor targets.
- **No child-phase support.** `/kbd-new-child` is change 5; this skill creates top-level phases only.
- **No reflection-reading.** That's `/kbd-next-phase`'s job. `/kbd-new-phase` is deliberately manual — the user supplies the name and goals.
- **No deletion or override of existing phases.** If `.kbd-orchestrator/phases/<name>/` exists, the skill refuses with a clear error and a hint to run `/kbd-next-phase` or pick a different name.

## Capabilities

### New Capabilities

- `kbd-new-phase-skill`: A first-class `/kbd-new-phase <name> [goals…]` command that creates a fresh top-level KBD phase, seeds `goals.md` and `progress.json` (including the nested-phase defaults introduced by `kbd-nested-phase-schema`), flips `current-waypoint.json` and `project.json` atomically, fires `phase:before` per `kbd-process-hooks`, and emits the canonical Progress Signals — closing the orchestrator's documented-but-missing gap.

### Modified Capabilities

- None as separate spec entries. The `kbd-process-hooks` capability gains its first real `phase:before` writer; the `kbd-nested-phase-schema` capability gains its first real consumer of the new field defaults — both contracts are unchanged.

## Impact

- **Risk**: Low. Net-new skill that mirrors `/kbd-next-phase`'s structure; no behavior change to any existing skill. All file writes are atomic (each file is written whole; no partial writes).
- **Affected files** (skill-system):
  - `skills/process/kbd-process-orchestrator/skills/kbd-new-phase/SKILL.md` *(new)*
  - `skills/process/kbd-process-orchestrator/skills/kbd-new-phase/kbd-new-phase.sh` *(new)*
  - `skills/process/kbd-process-orchestrator/SKILL.md` — small edit to confirm `/kbd-new-phase` is now backed by a skill directory
- **Cross-repo**: Yes — same `prometheus-skill-system` repo as changes 2 and 3, naturally folds into the same topic-branch commit.
- **Reversibility**: Trivial — remove the new skill directory; the orchestrator documentation correction is a one-line revert.
- **Unblocks**: `/kbd-new-child` and `/kbd-next-child` in change 5 (both mirror this skill's structure for nested phases); also enables `/kbd-full-phase` to actually invoke `/kbd-new-phase` instead of describing it.
