# Implementation Tasks — ssed-kbd-new-phase-skill

> Target repo: `prometheus-skill-system` (via symlink). All edits land in
> the same topic-branch commit as changes 2 + 3.
> Smoke tests pass 12/12. Live verification + rollback succeeded.

## 1. Skill directory + SKILL.md

- [x] 1.1 Create directory `skills/process/kbd-process-orchestrator/skills/kbd-new-phase/`
- [x] 1.2 Wrote `kbd-new-phase/SKILL.md` per the skeleton in design.md §Implementation Sketch:
  - [x] 1.2.1 YAML front matter
  - [x] 1.2.2 "What this does" 8-step list
  - [x] 1.2.3 "When to use" — manual vs `/kbd-next-phase`
  - [x] 1.2.4 "Progress Signals (MANDATORY)"
  - [x] 1.2.5 "Prerequisites"
  - [x] 1.2.6 "How to invoke" — single-script entry point + 10-step inline workflow
  - [x] 1.2.7 "Hook integration" subsection
  - [x] 1.2.8 "Examples" block with three call shapes

## 2. Helper script `kbd-new-phase.sh`

- [x] 2.1 Wrote the script (POSIX bash 3.2, ~115 LOC)
- [x] 2.2 Argument parsing implemented
- [x] 2.3 Name validation:
  - [x] 2.3.1 Refuses `..`
  - [x] 2.3.2 Refuses `/`
  - [x] 2.3.3 Regex `^[a-z0-9][a-z0-9._-]*$`
  - [x] 2.3.4 Refuses on existing-phase collision
- [x] 2.4 Atomic `goals.md` write
- [x] 2.5 Atomic `progress.json` with canonical field set
- [x] 2.6 Waypoint update:
  - [x] 2.6.1 Rewrite via `jq` preserving unknown keys (verified live — 5 fields passed through)
  - [x] 2.6.2 Reject malformed JSON BEFORE any on-disk state changes (ordering fixed per §5.3)
  - [x] 2.6.3 First-write skeleton when waypoint absent
  - [x] 2.6.4 Atomic `mv`
- [x] 2.7 `project.json` update:
  - [x] 2.7.1 Update + atomic `mv` when present
  - [x] 2.7.2 Warn + continue when absent
- [x] 2.8 Hook fire:
  - [x] 2.8.1 `KBD_ORCHESTRATOR_ROOT` fallback to `$HOME/.claude/skills/kbd-process-orchestrator`
  - [x] 2.8.2 Sources both `waypoint.sh` and `hooks.sh` when present
  - [x] 2.8.3 `kbd_hooks_fire phase before "$name" 1 1`
  - [x] 2.8.4 Warn + continue when hooks subsystem unavailable (phase persists)
- [x] 2.9 Confirmation banner: phase + goals.md path + `Next:` line
- [x] 2.10 `chmod +x kbd-new-phase.sh`
- [x] 2.11 `bash -n` syntax check passes

## 3. Orchestrator SKILL.md confirmation edit

- [x] 3.1 Located the `/kbd-new-phase` entry in "Quick Start Commands"
- [x] 3.2 Appended `(implemented in skills/kbd-new-phase/)` inline note
- [x] 3.3 Verified no "referenced but not implemented" wording remains

## 4. Smoke tests

- [x] 4.1 Created `shared/lib/tests/test-kbd-new-phase.sh`
- [x] 4.2 Each test isolated in `$(mktemp -d)`
- [x] 4.3 All 12 cases pass:
  - [x] 4.3.1 Happy path, no goals → TBD stub, full progress.json
  - [x] 4.3.2 Happy path with 3 goals → bullets in order
  - [x] 4.3.3 Missing name → usage error
  - [x] 4.3.4 Uppercase name → regex error
  - [x] 4.3.5 Name with `..` → traversal error
  - [x] 4.3.6 Collision → refused, existing phase untouched
  - [x] 4.3.7 First-waypoint write → `previousPhase: null`
  - [x] 4.3.8 Malformed waypoint → abort, NO on-disk state changed (validated upfront)
  - [x] 4.3.9 Absent `project.json` → warn, phase still created
  - [x] 4.3.10 Hook fire → `phase:before` entry in `hooks.log.jsonl` with all keys
  - [x] 4.3.11 Hooks subsystem absent (bad `KBD_ORCHESTRATOR_ROOT`) → warn, phase still created
  - [x] 4.3.12 Unknown waypoint keys preserved through rewrite
- [x] 4.4 Driver matches the pattern used by `test-hooks.sh` and the fixture driver
- [x] 4.5 **12 / 12 assertions pass live** on macOS Bash 3.2

## 5. Cross-script ordering audit

- [x] 5.1 Order audited: validate-args → validate-waypoint → check-collision → mkdir → goals → progress.json → waypoint → project.json → hook → signal → banner
- [x] 5.2 Order matches design D5 (with the D7 refinement)
- [x] 5.3 **Ordering correction applied**: malformed-waypoint check moved BEFORE `mkdir -p $phase_dir` so a bad waypoint leaves on-disk state untouched. Verified by test 4.3.8.

## 6. Live verification against UAR

- [x] 6.1 Ran `bash $KBD_ORCHESTRATOR_ROOT/skills/kbd-new-phase/kbd-new-phase.sh ssed-smoke-phase "Live verification only — rolled back"` against the real `.kbd-orchestrator/`
- [x] 6.2 `phase:before` entry confirmed in `phases/ssed-smoke-phase/hooks.log.jsonl`:

  ```
  {"ts":"2026-05-27T14:20:28Z","kind":"phase","edge":"before","name":"ssed-smoke-phase",
   "index":1,"total":1,"phasePath":"ssed-smoke-phase","sourceTool":"claude-code",
   "hookId":"report-progress","layer":"builtin","mode":"augment","status":0}
  ```

- [x] 6.3 Waypoint flip confirmed: `phase: ssed-smoke-phase`, `previousPhase: submodule-skills-and-entity-devtools-expansion`. Additional fields (`backend`, `wave`, `lastCompletedChange`, `changesCompleted`, `changesTotal`) preserved by the unknown-field passthrough — schema fault-tolerance from change 2 working as designed.
- [x] 6.4 **Rollback complete** — restored prior waypoint + project.json, deleted `phases/ssed-smoke-phase/`. UAR active phase reverted to `submodule-skills-and-entity-devtools-expansion`.

## 7. Documentation

- [x] 7.1 `kbd-new-phase/SKILL.md` cross-links to orchestrator "Hooks" section
- [x] 7.2 Cross-link to `references/schemas/current-waypoint.template.json` for the field set
- [x] 7.3 No README/index update required

## 8. Cross-repo commit + verification

- [ ] 8.1 Add new files + orchestrator SKILL.md edit to the in-progress topic branch in `prometheus-skill-system` — **pending user-driven git operation** (combines naturally with changes 2 + 3)
- [ ] 8.2 Push, open PR, capture URL + SHA — **pending**
- [ ] 8.3 Post-merge re-run of §4 + §6 — **pending merge**

```
prometheus-skill-system commit: eb3134bce2b1f1a956acfd6d264f2b7d8862974f
PR URL:                         https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3
```

## 9. Closeout

- [x] 9.1 `progress.json` will be updated with `changes_completed: 4` and `active_change: "ssed-kbd-child-phase-skills"` after archive
- [ ] 9.2 `/opsx:verify ssed-kbd-new-phase-skill` — **pending user-driven**
- [ ] 9.3 `/opsx:archive ssed-kbd-new-phase-skill` — **pending verify pass**
- [x] 9.4 `current-waypoint.json` will be refreshed after archive

## Files touched in `prometheus-skill-system`

New (3):
- `skills/process/kbd-process-orchestrator/skills/kbd-new-phase/SKILL.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-new-phase/kbd-new-phase.sh`
- `skills/process/kbd-process-orchestrator/shared/lib/tests/test-kbd-new-phase.sh`

Modified (1):
- `skills/process/kbd-process-orchestrator/SKILL.md` (single-line `(implemented in skills/kbd-new-phase/)` note)

Combined with changes 2 + 3, the topic branch now carries 11 modified + 3 untracked-dir entries — a single coherent "W1 orchestrator foundation" PR.
