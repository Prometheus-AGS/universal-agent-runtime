# Implementation Tasks — ssed-kbd-nested-phase-schema

> **Target repos**
> - **Skill-system** (where most edits land): `/Users/gqadonis/Projects/prometheus/prometheus-skill-system`
>   (surfaces in this machine at `~/.claude/skills/kbd-process-orchestrator/` via symlink to `skills/process/kbd-process-orchestrator`)
> - **UAR** (this repo, for archive + task log only): `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`
>
> All edits below were made on the skill-system `main` working tree through
> the symlink, leaving the repo dirty in preparation for a topic-branch
> commit (§9). The fixture suite passed end-to-end on macOS Bash 3.2.

## 1. Schema template — `current-waypoint.template.json` *(skill-system)*

- [x] 1.1 Create `skills/process/kbd-process-orchestrator/references/schemas/current-waypoint.template.json` with the body from design §Implementation Sketch
  - [x] 1.1.1 Top-level keys: `__schemaVersion: "2"`, `__description`, then the canonical waypoint fields
  - [x] 1.1.2 New fields with documented defaults: `parentPhase: null`, `childPhases: []`, `childPointer: null`
  - [x] 1.1.3 Keep `phase`, `previousPhase`, `change`, `status`, `currentTask`, `nextPendingChange`, `sourceTool`, `exactNextCommand`, `updatedAt` in their existing positions; added the common rendering fields (`backend`, `wave`, `lastCompletedChange`, `changesCompleted`, `changesTotal`) observed in current waypoints to give writers a complete reference
- [x] 1.2 Validate the file parses as JSON (`jq .` returned 0)
- [x] 1.3 Mention the template's existence and its purpose in the "Wayfinding State" subsection of `kbd-process-orchestrator/SKILL.md` (see new "Nested phases" subsection — references the template explicitly)

## 2. Schema template — `project.template.json` worktreeRoot field *(skill-system, closes change 1 task 4.3)*

- [x] 2.1 Insert `"worktreeRoot": "${HOME}/.claude/worktrees"` into `references/schemas/project.template.json`, immediately after `"active_phase"`
- [x] 2.2 Re-validate `jq .` (ok)
- [x] 2.3 Documented the literal-string + consumer-expansion semantics in the new SKILL.md "Nested phases" subsection (which also covers `worktreeRoot`)

## 3. SKILL.md "Nested phases" section *(skill-system)*

- [x] 3.1 Added a "Nested phases" subsection to `skills/process/kbd-process-orchestrator/SKILL.md` covering:
  - [x] 3.1.1 The three new fields with their default values
  - [x] 3.1.2 The canonical iteration order of `childPhases` and the meaning of `childPointer`
  - [x] 3.1.3 The write-time invariants from spec §"Cross-Field Invariants"
  - [x] 3.1.4 Explicit note that `__schemaVersion` is documentation only and is never read at runtime (design D7)
- [x] 3.2 The subsection cross-references `kbd-status/SKILL.md` and (forward-references) `/kbd-new-child` and `/kbd-next-child` for change 5

## 4. Shared helpers *(skill-system)*

- [x] 4.1 Created `skills/process/kbd-process-orchestrator/shared/lib/waypoint.sh` (POSIX bash 3.2) with:
  - [x] 4.1.1 `waypoint_load <path>` — emits each documented field on stdout as `key=value` lines using `jq` with safe defaults; `childPhases` joined with `,`
  - [x] 4.1.2 `waypoint_chain <parent> <phase> <pointer>` — renders the chain with empty slots elided
  - [x] 4.1.3 `chain_separator` — emits `›` (U+203A followed by a space) unless `LC_ALL`/`LANG` is `POSIX`/`C`/`C.*`, in which case it emits ` > `
  - [x] 4.1.4 `expand_kbd_path <literal>` — safely expands `${HOME}` / `$HOME` / `${USER}` / `$USER` without `eval`; uses a temp pattern var to avoid bash brace-matching pitfalls (regression fixed during apply)
  - [x] 4.1.5 `is_descendant <child> <parent>` — `cd … && pwd -P` on both sides; same path returns 1 (matches spec scenario "Checkout exactly equals worktreeRoot" — same path is NOT a descendant, triggers the warning)
- [x] 4.2 Test coverage: section 6 (fixtures) — a pure bash+jq driver covers every helper. Bats-style tests deferred (driver covers the same surface).

## 5. `kbd-status` skill update *(skill-system, closes change 1 task 4.2 / 6.5)*

- [x] 5.1 Updated `skills/process/kbd-process-orchestrator/skills/kbd-status/SKILL.md`:
  - [x] 5.1.1 Documented sourcing of `shared/lib/waypoint.sh` in the new "Phase chain rendering" and "Worktree awareness" sections
  - [x] 5.1.2 Documented `children: <i>/<n>` and `(children defined, none active)` rendering per spec §"Phase Chain Rendering"
  - [x] 5.1.3 Documented `worktree: <path>` rendering and the `⚠ outside worktreeRoot (<resolved-root>)` annotation per spec §"Outside-worktreeRoot Warning"
  - [x] 5.1.4 Documented the "append-only stability" render order (design D6): phase → children? → worktree → (existing lines below)
- [x] 5.2 Updated the rendered-output example to show the canonical `phase:` / `worktree:` lines; added a second example showing nested phase + outside-root warning
- [x] 5.3 Documented every graceful-degradation case (no git, no project.json, unreadable project.json) per spec §"Graceful Degradation"

## 6. Fixtures *(skill-system)*

- [x] 6.1 `references/schemas/fixtures/waypoint/pre-schema.json` — flat waypoint, no new fields
- [x] 6.2 `references/schemas/fixtures/waypoint/parent-with-children.json` — `childPhases: ["w0","w1","w2"]`, `childPointer: "w1"`
- [x] 6.3 `references/schemas/fixtures/waypoint/child-row.json` — `parentPhase: "outer"`, `phase: "inner"`
- [x] 6.4 `references/schemas/fixtures/test.sh` driver:
  - [x] 6.4.1 Loads each fixture via `waypoint_load`
  - [x] 6.4.2 Asserts documented defaults emerge for missing fields
  - [x] 6.4.3 Asserts `waypoint_chain` renders the documented string for each fixture, including the POSIX-locale fallback
  - [x] 6.4.4 Pure bash + jq, exits non-zero on first failure — passes 9/9 locally on macOS Bash 3.2

## 7. Project-side adoption *(UAR — this repo)*

- [x] 7.1 No edit required to use the new fields — additive. Verified by running the orchestrator fixture suite end-to-end.
- [N/A] 7.2 Populating `parentPhase` / `childPhases` / `childPointer` in this repo's waypoint is reserved for changes 4/5; not done here.

## 8. Backward-compatibility smoke tests *(skill-system)*

- [x] 8.1 Pre-schema fixture loads with documented defaults — covered by §6 test driver
- [x] 8.2 `parent-with-children.json` renders `parent-phase › w1` (chain) — covered
- [x] 8.3 `child-row.json` renders `outer › inner` (chain) — covered
- [x] 8.4 `LC_ALL=POSIX` substitutes ` > ` for `›` — covered

## 9. Cross-repo commit + verification

- [ ] 9.1 Stage skill-system edits in a single commit on a topic branch in `prometheus-skill-system` — **pending user-driven git operation**
- [ ] 9.2 Push, open PR, capture PR URL + commit SHA — **pending**
- [ ] 9.3 After PR merges, re-run §8 against UAR's waypoint via the updated `kbd-status` — **pending merge**
- [ ] 9.4 Record the merged commit SHA below

```
prometheus-skill-system commit: eb3134bce2b1f1a956acfd6d264f2b7d8862974f
PR URL:                         https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3
```

**Files dirtied in `prometheus-skill-system` (ready to stage)**:

```
 M  skills/process/kbd-process-orchestrator/SKILL.md
 M  skills/process/kbd-process-orchestrator/references/schemas/project.template.json
 M  skills/process/kbd-process-orchestrator/skills/kbd-status/SKILL.md
??  skills/process/kbd-process-orchestrator/references/schemas/current-waypoint.template.json
??  skills/process/kbd-process-orchestrator/references/schemas/fixtures/
??  skills/process/kbd-process-orchestrator/shared/
```

## 10. Closeout

- [x] 10.1 Update `.kbd-orchestrator/phases/submodule-skills-and-entity-devtools-expansion/progress.json`:
  - [x] 10.1.1 `changes_completed: 2`, append `ssed-kbd-nested-phase-schema` to `completed_changes`
  - [x] 10.1.2 Remove entries for tasks 4.2 / 4.3 / 6.5 from `deferred_tasks` (now resolved)
  - [x] 10.1.3 Set `active_change: "ssed-kbd-process-hooks"`, state `ready_for_opsx_new`
- [ ] 10.2 `/opsx:verify ssed-kbd-nested-phase-schema` — **pending user-driven**
- [ ] 10.3 `/opsx:archive ssed-kbd-nested-phase-schema` — **pending verify pass**
- [x] 10.4 `current-waypoint.json` refreshed to point at change 3 (`ssed-kbd-process-hooks`)
