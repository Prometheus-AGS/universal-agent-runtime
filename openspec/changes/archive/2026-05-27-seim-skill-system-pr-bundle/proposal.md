## Why

The prior phase `submodule-skills-and-entity-devtools-expansion` shipped 8 separate OpenSpec changes whose code edits all landed in the same `prometheus-skill-system` repo (via the symlink at `~/.claude/skills/kbd-process-orchestrator → skills/process/kbd-process-orchestrator`). That repo's working tree was deliberately **not** committed at the end of the prior phase — design D8 across multiple of those changes called for a single coherent topic-branch PR. Right now, 10 paths are dirty:

```
 M  skills/process/kbd-process-orchestrator/SKILL.md
 M  skills/process/kbd-process-orchestrator/hooks/hooks.json
 M  skills/process/kbd-process-orchestrator/references/schemas/hooks-config.schema.json
 M  skills/process/kbd-process-orchestrator/references/schemas/project.template.json
 M  skills/process/kbd-process-orchestrator/skills/kbd-status/SKILL.md
 M  skills/process/kbd-process-orchestrator/skills/kbd-assess/SKILL.md
 M  skills/process/kbd-process-orchestrator/skills/kbd-execute/SKILL.md
 M  skills/process/kbd-process-orchestrator/skills/kbd-next-phase/SKILL.md
 M  skills/process/kbd-process-orchestrator/skills/kbd-plan/SKILL.md
 M  skills/process/kbd-process-orchestrator/skills/kbd-reflect/SKILL.md
??  skills/process/kbd-process-orchestrator/references/schemas/current-waypoint.template.json
??  skills/process/kbd-process-orchestrator/references/schemas/fixtures/
??  skills/process/kbd-process-orchestrator/shared/
??  skills/process/kbd-process-orchestrator/skills/kbd-new-phase/
??  skills/process/kbd-process-orchestrator/skills/kbd-new-child/
??  skills/process/kbd-process-orchestrator/skills/kbd-next-child/
??  skills/process/kbd-process-orchestrator/skills/kbd-memory-recall/
??  skills/process/kbd-process-orchestrator/skills/kbd-inject-agent-rules/
??  skills/react/prometheus-entity-skills/entity-realtime-surreal-live/
??  skills/react/prometheus-entity-skills/entity-graph-optimize/SKILL.md (edited block)
```

Until those edits exist on `origin/main` (or a merged PR target branch), the entity-management work later in this phase can't reliably reference them. Specifically:

- Change 4 of this phase (`seim-em-surreal-live-adapter-impl`) and the `entity-realtime-surreal-live` skill's wiring instructions need the shipped skill SKILL.md to be discoverable by other tools (Cursor, Roo, Codex) that re-sync their skill registries from origin.
- Change 8 of this phase (`seim-em-explorer-panel-components`) needs the "Dev-mode entity explorer" subsection in `entity-graph-optimize/SKILL.md` to be the official reference, not a local-only edit.
- Changes 6 and 10's pre-flight UI/UX research depends on `/kbd-memory-recall` being available — which requires `skills/kbd-memory-recall/` to be on origin so the surreal-memory mirror hook can find the recall skill it auto-invokes on `assess:before`.

This change is the git operation that unblocks the rest of W2 onward. It produces no code artifact in *this* repo; its deliverable is a merged PR in a *different* repo plus the merge-SHA back-reference in every prior-phase archived `tasks.md`.

## What Changes

### Topic branch + commit in `prometheus-skill-system`

- Branch name: `feat/kbd-orchestrator-w1-w3-2026-05-27` (date suffix prevents collision with hypothetical re-runs).
- All 10 modified + untracked paths staged in one commit.
- Commit message (HEREDOC body):
  ```
  feat(kbd-process-orchestrator): nested phases + hooks + new-phase + child-phase + memory + rule-injector + uiux routing

  Composite landing of 8 OpenSpec changes from the
  universal-agent-runtime phase
  submodule-skills-and-entity-devtools-expansion:

  - ssed-kbd-nested-phase-schema       — parentPhase / childPhases /
    childPointer + worktreeRoot fields, current-waypoint.template.json
  - ssed-kbd-process-hooks             — hooks.sh dispatcher,
    augment/override semantics, default report-progress reporter,
    JSONL audit log
  - ssed-kbd-new-phase-skill           — /kbd-new-phase first writer
    of phase:before
  - ssed-kbd-child-phase-skills        — /kbd-new-child + /kbd-next-child
  - ssed-kbd-memory-first-execution    — surreal-memory mirror hook +
    /kbd-memory-recall skill
  - ssed-kbd-agent-rules-injector      — Karpathy + Boris Cherny rule
    injection via fenced regions
  - ssed-uar-uiux-skill-routing        — --pack flag on the injector
    (skill-system side only; UAR-side render lives in the UAR repo)
  - ssed-entity-surreal-live-adapter   — companion skill SKILL.md
    (TS adapter implementation tracked separately)
  - ssed-entity-explorer-fab-panel     — "Dev-mode entity explorer"
    subsection added to entity-graph-optimize SKILL.md (React UI
    implementation tracked separately)

  73/73 smoke-test assertions pass live (six pure-bash+jq test scripts
  under skills/process/kbd-process-orchestrator/shared/lib/tests/).
  All KBD skills now fire <kind>:before/<kind>:after hooks at the
  documented lifecycle boundaries; the default report-progress hook
  emits 'starting/ending <kind> <name> [<i>/<n>]' to stderr on every
  fire.

  Cross-references:
  - UAR archived changes: openspec/changes/archive/2026-05-27-ssed-*
  - Each prior-phase tasks.md §9 / §10 will be updated post-merge with
    this commit's SHA + the PR URL.
  ```

### PR open + review path

- Title: `feat(kbd-process-orchestrator): nested phases + hooks + new-phase + child-phase + memory + rule-injector + uiux routing`
- Body references the same 8 changes; links each one's archived OpenSpec proposal/spec/design/tasks in this UAR repo.
- Reviewers: whoever ownership of the orchestrator skill currently belongs to (TBD by operator).
- CI: the orchestrator skill repo's existing CI runs the bash test scripts; this commit lights them up for the first time.

### Post-merge updates back in this UAR repo

For each of the 8 archived prior-phase changes that carry `<fill in after merge>` placeholders:

- Update `openspec/changes/archive/2026-05-27-<change-id>/tasks.md` § "Cross-repo commit + verification": fill in the merged SHA and PR URL.
- Commit those updates locally in this UAR repo in a single follow-up commit titled `chore(kbd): record skill-system merge SHA in archived tasks`.

### What this change does NOT include

- **No new code** in either repo. The skill-system code already exists in the working tree; we're committing it.
- **No new spec capabilities** — the capabilities were already specified by the 8 prior changes (each promoted its own spec into `openspec/specs/`).
- **No entity-management work** — that's W2 onward.
- **No skill-system CI changes** — if CI fails on the existing test scripts, that's a separate bug to fix in its own change.

## Capabilities

### New Capabilities

- None. This change is a git operation that lands previously-specified capabilities into a new commit. The capabilities themselves (`kbd-nested-phase-schema`, `kbd-status-worktree-awareness`, `kbd-process-hooks`, `kbd-new-phase-skill`, `kbd-child-phase-skills`, `kbd-memory-first-execution`, `kbd-agent-rules-injector`, `uar-uiux-skill-routing` partially, `entity-surreal-live-adapter` skill-side, `entity-explorer-fab-panel` skill-side) were all specified and archived in the prior phase.

### Modified Capabilities

- None.

## Impact

- **Risk**: Low. The diff being committed has been smoke-tested live (73/73). The PR may surface code-review feedback that requires follow-up changes, but those are independent issues.
- **Affected files**:
  - **`prometheus-skill-system`** (target of the commit): 10 paths listed in §Why.
  - **`universal-agent-runtime`** (this repo, post-merge follow-up): 8 archived tasks.md files updated with merge SHA + PR URL.
- **Cross-repo**: Yes — the entire point of this change.
- **Reversibility**: PR-revert if necessary; the UAR-side post-merge update is a one-line per-file edit, trivially reverted.
- **Unblocks**: every entity-management change in W2 onward of this phase. Also makes the orchestrator improvements available to every other Prometheus-AGS project consuming the skill set.

### Sequencing note

This change has **no spec capabilities** so the spec-driven artifact sequence is shorter than usual: `proposal.md` → `design.md` → `tasks.md`. No `specs/<capability>/spec.md` files needed. `/opsx:verify` should accept "no specs declared, no specs required" as valid; if it doesn't, treat as a `--skip-qa` candidate.
