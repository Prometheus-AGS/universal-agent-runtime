# Implementation Tasks — ssed-kbd-child-phase-skills

> Target repo: `prometheus-skill-system`. Folds into the same topic-branch commit as 2 + 3 + 4.

## 1. kbd-new-child skill

- [ ] 1.1 Create `skills/process/kbd-process-orchestrator/skills/kbd-new-child/`
- [ ] 1.2 Write `SKILL.md` (mirrors `kbd-new-phase` shape; describes child-scoped behavior)
- [ ] 1.3 Write `kbd-new-child.sh` per design §Implementation Sketch
  - [ ] 1.3.1 Name validation (regex + `..` + `/` rejection)
  - [ ] 1.3.2 Refuse when no top-level phase active or waypoint malformed
  - [ ] 1.3.3 Refuse when currently inside a child (single-level nesting only — D7)
  - [ ] 1.3.4 Refuse duplicate child name (with `/kbd-next-child` hint)
  - [ ] 1.3.5 Atomic goals.md + progress.json writes inside `phases/<parent>/children/<child>/`
  - [ ] 1.3.6 progress.json has `parentPhase: "<parent>"` and the rest of the canonical field set
  - [ ] 1.3.7 Atomic waypoint update appending child + setting pointer + updating currentTask / exactNextCommand
  - [ ] 1.3.8 Invariant check (no duplicates, pointer ∈ children) before mv
  - [ ] 1.3.9 Fire `child:before` with index = new count, total = new count
- [ ] 1.4 `chmod +x`; `bash -n` passes

## 2. kbd-next-child skill

- [ ] 2.1 Create `skills/process/kbd-process-orchestrator/skills/kbd-next-child/`
- [ ] 2.2 Write `SKILL.md`
- [ ] 2.3 Write `kbd-next-child.sh` per design §Implementation Sketch
  - [ ] 2.3.1 No-arg implicit advance + explicit-name jump (D3)
  - [ ] 2.3.2 Refuse when children empty
  - [ ] 2.3.3 Refuse when implicit advance past last (with `/kbd-reflect` + `/kbd-next-phase` hint)
  - [ ] 2.3.4 Refuse explicit jump to unknown name (list available)
  - [ ] 2.3.5 Fire `child:after` for closing pointer while old waypoint is still on disk (D5)
  - [ ] 2.3.6 Atomic waypoint update setting new pointer
  - [ ] 2.3.7 Fire `child:before` for new active child
- [ ] 2.4 `chmod +x`; `bash -n` passes

## 3. Orchestrator SKILL.md

- [ ] 3.1 Add `/kbd-new-child` and `/kbd-next-child` to "Quick Start Commands" / per-skill list
- [ ] 3.2 Confirm the "Nested phases" subsection (added by change 2) lists both skills as the writers

## 4. Smoke tests

- [ ] 4.1 `shared/lib/tests/test-kbd-child-phase.sh` covering the full lifecycle:
  - [ ] 4.1.1 Set up: create a parent via `kbd-new-phase.sh parent-x`
  - [ ] 4.1.2 `kbd-new-child a "first goal"` → child dir, child added to `childPhases`, pointer set
  - [ ] 4.1.3 `kbd-new-child b` → second child appended, pointer moves to `b`
  - [ ] 4.1.4 `kbd-next-child a` → explicit jump to `a`; child:after for `b` + child:before for `a` in log
  - [ ] 4.1.5 `kbd-next-child` (no arg) → implicit advance from `a` to `b`
  - [ ] 4.1.6 `kbd-next-child` past last → refusal with `/kbd-reflect` hint
  - [ ] 4.1.7 `kbd-next-child unknown` → refusal listing available children
  - [ ] 4.1.8 `kbd-new-child a` (duplicate) → refusal with `/kbd-next-child a` hint
  - [ ] 4.1.9 Invariant: never observe waypoint where `childPointer ∉ childPhases`
  - [ ] 4.1.10 `kbd-new-child x` while no parent active → refusal
- [ ] 4.2 Run live; record pass/fail

## 5. Live verification + rollback

- [ ] 5.1 Save current waypoint + project.json
- [ ] 5.2 Run `kbd-new-child.sh ssed-smoke-child` against this UAR worktree (the active phase IS our omnibus phase, so this is a real-world test)
- [ ] 5.3 Verify `phases/submodule-skills-and-entity-devtools-expansion/children/ssed-smoke-child/` exists with goals.md + progress.json
- [ ] 5.4 Verify waypoint shows `childPhases: ["ssed-smoke-child"]`, `childPointer: "ssed-smoke-child"`
- [ ] 5.5 Verify a `child:before` entry appears in the hook log
- [ ] 5.6 **Rollback**: restore waypoint, delete the child dir

## 6. Cross-repo commit + closeout

- [ ] 6.1 Stage in `prometheus-skill-system` topic branch (combines with 2+3+4)
- [ ] 6.2 progress.json `changes_completed: 5`, active_change → `ssed-kbd-memory-first-execution`
- [ ] 6.3 `/opsx:verify` → `/opsx:archive`

```
prometheus-skill-system commit: eb3134bce2b1f1a956acfd6d264f2b7d8862974f
PR URL:                         https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3
```
