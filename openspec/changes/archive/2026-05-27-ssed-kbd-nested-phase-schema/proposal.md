## Why

The KBD orchestrator's `current-waypoint.json` schema is **flat**. It records `phase`, `previousPhase`, `change`, `status`, and a few task pointers, but it has no concept of a phase that *contains* other phases. The phase plan for `submodule-skills-and-entity-devtools-expansion` calls for two new skills, `/kbd-new-child` and `/kbd-next-child`, that create and traverse child phases owned by a parent. Those skills have nowhere to put their state today: a child phase would either overwrite its parent's row, or live in an unrelated file that no other tool reads.

This change introduces the schema extension that makes nested phases representable, and it does so in a way that every existing reader (Roo, Cursor, Codex, OpenCode, Claude Code) keeps working with **zero awareness** of the new fields. It is the foundation for changes 3–5 in the phase plan.

It also picks up two tasks that were explicitly deferred from change 1 (`ssed-worktree-persistence-convention`):

- **4.2** Update the `kbd-status` skill to render the active worktree path and warn when it sits outside the configured `worktreeRoot`.
- **4.3** Document the `worktreeRoot` field in `references/schemas/project.template.json`.

Both of those edits live in the same `kbd-process-orchestrator` skill set that this change is already opening up. Folding them in here avoids a separate cross-repo PR for the same files.

## What Changes

### Waypoint schema extension (`current-waypoint.json`)

- Add three **optional** fields, all defaulting to absent / empty:
  - `parentPhase: string | null` — name of the enclosing phase when this row represents a child.
  - `childPhases: string[]` — ordered list of child-phase names owned by this row's phase (only meaningful on parent rows).
  - `childPointer: string | null` — name of the currently-active child within `childPhases`, or `null` when no child is active.
- Readers that don't know about these fields ignore them; readers that do treat them as authoritative.
- The on-disk format remains a single JSON object — no new files, no breaking renames.

### Schema template (`references/schemas/current-waypoint.template.json`)

- Mirror the new fields in the template with defaults (`null`, `[]`, `null`) and a short comment explaining their meaning.
- Bump the template's version comment so consumers can detect the schema generation.

### Project schema template (`references/schemas/project.template.json`)

- Add the `worktreeRoot` field carried over from `ssed-worktree-persistence-convention` (default `${HOME}/.claude/worktrees`, optional).
- Annotate as additive / backward-compatible.

### `kbd-status` skill update (cross-repo: `prometheus-skill-system`)

- Read `parentPhase`, `childPhases`, `childPointer` from the waypoint and render the active chain as `root › child › grand-child`.
- Read `worktreeRoot` from `project.json` (default `${HOME}/.claude/worktrees`).
- Compute `git rev-parse --show-toplevel` and print a `worktree:` line.
- Emit a `⚠ outside worktreeRoot` annotation when the top-level path is not a descendant of `worktreeRoot`.
- Treat all four reads as fully optional — the skill must still work against a pre-schema waypoint.

### Migration / compatibility tests

- Add a regression case (in the orchestrator skill's `references/schemas/` test corpus or equivalent) that loads a pre-schema waypoint and asserts no fields are required to be present.
- Add a positive case with all new fields populated to confirm rendering.

### Documentation

- Update `kbd-process-orchestrator/SKILL.md` to describe nested-phase semantics so the later `/kbd-new-child` skill (change 5) has a documented foundation to build on.

### Non-changes

- **No new skills** introduced here — `/kbd-new-child`, `/kbd-next-child`, and `/kbd-new-phase` are separate changes (4, 5) in the wave plan. This change only lays the schema.
- **No data migration script** — every existing waypoint file remains valid as-is; the new fields appear only when a future change actually writes them.
- **Worktree relocation is still off-limits** (carried over from change 1).

## Capabilities

### New Capabilities

- `kbd-nested-phase-schema`: The waypoint and project schemas support optional `parentPhase`, `childPhases[]`, `childPointer`, and `worktreeRoot` fields with documented defaults, additive semantics, and backward-compatible loading.
- `kbd-status-worktree-awareness`: The `kbd-status` skill renders the active worktree path against the configured `worktreeRoot`, surfaces a warning when the checkout is outside the root, and renders the parent → child phase chain when the nested-phase fields are populated.

### Modified Capabilities

- None. (The skill set is extended; no existing capability changes its contract.)

## Impact

- **Risk**: Low. All four schema additions are optional + additive. The `kbd-status` rendering changes are presentation-only.
- **Affected files**:
  - This repo: `.kbd-orchestrator/current-waypoint.json` (no edit — schema is consumed, not redefined here), tests for cross-tool readers.
  - `prometheus-skill-system`: `skills/kbd-process-orchestrator/SKILL.md`, `skills/kbd-process-orchestrator/references/schemas/current-waypoint.template.json`, `skills/kbd-process-orchestrator/references/schemas/project.template.json`, `skills/kbd-process-orchestrator/skills/kbd-status/SKILL.md`, plus any shared rendering helpers under `shared/`.
- **Cross-repo**: Yes — most of the work lands in `prometheus-skill-system`. The change is staged in this worktree but committed to the skill-system origin, then re-installed locally so `~/.claude/skills/` picks up the update.
- **Reversibility**: Trivial — fields are optional, skill renders are presentation-only.
- **Unblocks**: change 3 (`ssed-kbd-process-hooks`), change 4 (`ssed-kbd-new-phase-skill`), change 5 (`ssed-kbd-child-phase-skills`). Also closes the deferred items from change 1.
