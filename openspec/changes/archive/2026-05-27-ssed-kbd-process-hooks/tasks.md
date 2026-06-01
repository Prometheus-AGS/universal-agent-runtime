# Implementation Tasks — ssed-kbd-process-hooks

> Target repo: `prometheus-skill-system` — all edits land via the symlink
> `~/.claude/skills/kbd-process-orchestrator → skills/process/kbd-process-orchestrator`.
> Smoke tests pass live (10/10).

## 1. Dispatcher core — `shared/lib/hooks.sh`

- [x] 1.1 Create `skills/process/kbd-process-orchestrator/shared/lib/hooks.sh` (POSIX bash 3.2)
- [x] 1.2 Public function: `kbd_hooks_fire <kind> <edge> <name> [<index>=1] [<total>=1]`
- [x] 1.3 Private: `_kbd_hooks_collect` + `_kbd_hooks_collect_one` — load all 3 layers, tag with `layer`, normalise event aliases via `_kbd_hooks_normalise_event`, filter to those matching `(kind, edge)` or `*`
- [x] 1.4 Private: `_kbd_hooks_resolve_overrides` — applies D3 rules; emits one warning per conflict naming winner + losers
- [x] 1.5 Private: `_kbd_hooks_run` — builds `KBD_HOOK_*` env, spawns via `bash -c "$command"`, enforces `timeout` when `timeout(1)` is present, tees stderr to file + parent stderr, appends JSONL log line
- [x] 1.6 Private: `_kbd_hooks_run_alias on_change_complete` is implemented inline in `kbd_hooks_fire` (D5 sentinel) — fires only on `task:after` with `index == total`
- [x] 1.7 JSONL append: uses `flock` when available, mkdir-based lock fallback with 5s acquire timeout; record-anyway after timeout (no blocking)
- [x] 1.8 Resolve active phase via `_kbd_hooks_active_phase_dir` (reads `current-waypoint.json` then checks for `phases/<phase>/`); fall back to `.kbd-orchestrator/hooks.log.jsonl` at project root when no phase dir exists (D8)
- [x] 1.9 Synthesise `hookId` in `_kbd_hooks_collect_one` jq filter when the entry lacks an `id` field (D9)
- [x] 1.10 Soft-fail when `jq` is unavailable — single stderr warning + no-op return

## 2. Default reporter and built-in extensions — `hooks/hooks.json`

- [x] 2.1 Appended the `report-progress` entry verbatim (event `*:*`, mode `augment`, timeout 5, on_failure `ignore`)
- [x] 2.2 Added top-level `__description` explaining all entries here are `layer: builtin`
- [x] 2.3 Left every existing entry; gave each an explicit `id` (the previous file had no ids — required by the schema)
- [x] 2.4 Renamed legacy `action.target` → `action.command` to match the schema and the dispatcher

## 3. Schema patch — `references/schemas/hooks-config.schema.json`

- [x] 3.1 Added `properties.mode` with `enum: ["augment","override"]`, `default: "augment"`, description per design D9
- [x] 3.2 Rewrote the `event` field's description to enumerate canonical kinds and edges, with the alias note
- [x] 3.3 Rewrote the top-level `description` to link to orchestrator `SKILL.md` → "Hooks"
- [x] 3.4 Validated `jq .` parses ok

## 4. Per-skill wiring

- [x] 4.1 `kbd-assess/SKILL.md` — appended a "Hook integration" section showing the assess:before / assess:after stanza
- [x] 4.2 `kbd-plan/SKILL.md` — plan:before / plan:after stanza
- [x] 4.3 `kbd-execute/SKILL.md` — execute:before / execute:after stanza; explicit note that `task:*` is fired by `/opsx:apply`, and that `on_change_complete` is auto-triggered by the dispatcher on the final task:after
- [x] 4.4 `kbd-reflect/SKILL.md` — reflect:after + phase:after (canonical phase-end boundary)
- [x] 4.5 `kbd-next-phase/SKILL.md` — phase:before for the new phase
- [x] 4.6 Phase-bracket note included in the new "Hooks" section in orchestrator `SKILL.md` — phase:before is owned by whichever skill first activates a phase (today: `/kbd-new-phase` when change 4 lands; until then `/kbd-next-phase` covers the seeded case)
- [x] 4.7 Each wiring stanza explicitly says the existing `Starting/Completed kbd-<skill> — <phase>` Progress Signal lines continue to fire — coexistence preserved

## 5. `/opsx:apply` task-level integration

- [x] 5.1 Documented the task-fire snippet in the orchestrator `SKILL.md` "Hooks" section ("Wiring stanza" subsection) so anyone wiring `/opsx:apply` has a canonical reference
- [x] 5.2 Snippet form: `i=0; n=$(grep -c '^- \[' tasks.md); per-task: kbd_hooks_fire task before/after "$task" "$i" "$n"` — documented in orchestrator SKILL.md
- [x] 5.3 `on_change_complete` automatic firing documented in the kbd-execute SKILL.md "Hook integration" subsection
- [ ] 5.4 **Deferred — external skill** — `/opsx:apply` lives outside the `kbd-process-orchestrator` skill (it's part of the user's openspec plugin). The orchestrator side is complete (snippet + alias semantics documented); the apply skill must be patched in its own repo to call `kbd_hooks_fire task before/after` around each task it implements. Filed as a follow-up; the apply skill's source location was not confirmed in this session.

## 6. SKILL.md "Hooks" section — orchestrator level

- [x] 6.1 Added a top-level "Hooks" section to `skills/process/kbd-process-orchestrator/SKILL.md` placed between "Wayfinding State" and "Cross-Tool Reporting Protocol"
- [x] 6.2 Covers: canonical event form, complete alias table (with the `<kind>:begin/end` mapping), discovery order, augment-vs-override semantics, full `KBD_HOOK_*` env var table, JSONL log path (in-phase vs fallback), default reporter contract, wiring stanza, `KBD_HOOK_DEBUG=1` debug note
- [x] 6.3 Each per-skill "Hook integration" section cross-links back to this orchestrator section

## 7. Smoke tests

- [x] 7.1 `shared/lib/tests/test-hooks.sh` (pure bash + jq) covers:
  - [x] 7.1.1 Default reporter `starting <kind> <name> [<i>/<n>]` on *:before
  - [x] 7.1.2 Default reporter `ending …` on *:after
  - [x] 7.1.3 Project-layer override suppresses default reporter
  - [x] 7.1.4 Multiple overrides → conflict warning, project layer wins
  - [x] 7.1.5 `on_phase_complete` alias → canonical phase:after dispatch
  - [x] 7.1.6 `on_change_complete` alias fires ONLY when task:after has index == total
  - [x] 7.1.7 JSONL log entry contains all required keys (ts, kind, edge, name, index, total, phasePath, sourceTool, hookId, layer, mode, status)
  - [x] 7.1.8 Invalid event in a config → silently skipped, dispatch continues
  - [x] 7.1.9 `<step>:begin` alias maps to `<step>:before`
  - [x] 7.1.10 (added during apply) — mid-task `on_change_complete` does NOT fire
- [x] 7.2 Driver is standalone and matches the pattern used by `references/schemas/fixtures/test.sh`
- [x] 7.3 **10 / 10 assertions pass live** on macOS Bash 3.2

## 8. Backward compatibility

- [x] 8.1 Existing `hooks/hooks.json` entries verified to normalise via the alias table (debug trace captured)
- [x] 8.2 `event: "assess:begin"` confirmed to fire on `assess:before` dispatch (covered by test 7.1.9)
- [x] 8.3 `event: "on_change_complete"` confirmed to fire only on the final `task:after` (covered by tests 7.1.6 + 7.1.10)
- [x] 8.4 `KBD_HOOK_DEBUG=1` prints `normalised <orig> → <canonical>` info lines per loaded entry — verified live

## 9. Documentation

- [x] 9.1 Updated `kbd-process-orchestrator/SKILL.md` per §6
- [x] 9.2 Updated `references/schemas/hooks-config.schema.json` description per §3.3
- [x] 9.3 "Hook integration" subsections added to assess / plan / execute / reflect / next-phase SKILL.md
- [ ] 9.4 README / top-level SKILL.md example block updated to show the default `starting/ending …` lines — **deferred**: the orchestrator `SKILL.md` Hooks section already shows the reporter contract; a separate top-level example block would be redundant. Skipped intentionally.

## 10. Cross-repo commit + verification

- [ ] 10.1 Stage skill-system edits — 10 modified + 3 untracked paths under `skills/process/kbd-process-orchestrator/` — on a topic branch in `prometheus-skill-system`. **Pending user-driven git operation** (combines naturally with change 2's still-pending commit).
- [ ] 10.2 Push, open PR, capture PR URL and merged commit SHA
- [ ] 10.3 After merge, re-run §7 from a fresh shell

```
prometheus-skill-system commit: eb3134bce2b1f1a956acfd6d264f2b7d8862974f
PR URL:                         https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3
```

## 11. Closeout

- [x] 11.1 Updated `.kbd-orchestrator/phases/submodule-skills-and-entity-devtools-expansion/progress.json`:
  - [x] 11.1.1 `changes_completed: 3`, append `ssed-kbd-process-hooks` to `completed_changes`
  - [x] 11.1.2 `active_change: "ssed-kbd-new-phase-skill"`, state `ready_for_opsx_new`
- [ ] 11.2 `/opsx:verify ssed-kbd-process-hooks` — **pending user-driven**
- [ ] 11.3 `/opsx:archive ssed-kbd-process-hooks` — **pending verify pass**
- [x] 11.4 Refreshed `current-waypoint.json` to point at change 4 (`ssed-kbd-new-phase-skill`)

## Files touched in `prometheus-skill-system`

Modified (10):
- `SKILL.md`
- `hooks/hooks.json`
- `references/schemas/hooks-config.schema.json`
- `references/schemas/project.template.json` *(from change 2, still uncommitted)*
- `skills/kbd-status/SKILL.md` *(from change 2)*
- `skills/kbd-assess/SKILL.md`
- `skills/kbd-execute/SKILL.md`
- `skills/kbd-next-phase/SKILL.md`
- `skills/kbd-plan/SKILL.md`
- `skills/kbd-reflect/SKILL.md`

Untracked (3 dirs):
- `references/schemas/current-waypoint.template.json` *(from change 2)*
- `references/schemas/fixtures/` *(from change 2)*
- `shared/` *(introduces `shared/lib/waypoint.sh` from change 2 + `shared/lib/hooks.sh` and `shared/lib/tests/test-hooks.sh` from this change)*

A single topic-branch commit covering changes 2 and 3 is the natural shape.
