## Context

`prometheus-skill-system` lives at `/Users/gqadonis/Projects/prometheus/prometheus-skill-system`. The relevant subtree `skills/process/kbd-process-orchestrator/` is symlinked into `~/.claude/skills/kbd-process-orchestrator/`, which is why every prior-phase change that "shipped to the orchestrator skill" actually wrote files into the `prometheus-skill-system` working tree without ever touching this UAR repo.

End of the prior phase: 10 dirty paths, `main` branch, no topic branch yet, no commit yet. This change is the git operation that converts that working-tree state into a merged PR.

## Goals / Non-Goals

**Goals**
- Single coherent PR for all 8 prior-phase changes' skill-system edits.
- Commit message that audits cleanly back to the OpenSpec archive (each prior change is named).
- Post-merge: every prior-phase archived `tasks.md` carries the merge SHA + PR URL.
- Smallest possible PR scope — only the prior phase's work; no opportunistic cleanup.

**Non-Goals**
- No splitting into 8 separate PRs. One PR is simpler to review against the matching archive in this UAR repo.
- No skill-system CI changes. Pre-existing CI runs the new bash test scripts; we land them as-is.
- No tag / release. The skill set isn't currently versioned at the package level; that's a separate change.
- No GitHub-action workflow updates. The smoke tests are runnable as-is.

## Decisions

### D1. One commit, not 8

Eight commits would map 1:1 to the prior changes but would also require 8 rebase passes if a single review comment lands on shared file (which is likely — `SKILL.md` is touched by 6 of the 8 changes). One commit with a long body that names all 8 changes individually preserves the audit trail without the rebase tax.

The commit message body lists every prior change by ID so `git log --grep=ssed-kbd-process-hooks` (etc.) still finds the landing.

### D2. Date-suffixed branch name

`feat/kbd-orchestrator-w1-w3-2026-05-27`. Date suffix prevents collisions if (a) the PR needs to be re-opened from a fresh branch after force-revert, or (b) a future phase produces a similarly-shaped bundle and wants to disambiguate. Three lowercase tokens + a date is searchable and unambiguous.

### D3. No `--force-with-lease`, no rewrites

The branch is brand new. We push once. If review demands changes, we land them as additional commits on the same branch (not amend) so reviewers can see the iteration. Squash-on-merge is the GitHub setting that produces the clean history.

### D4. Capture merge artifacts in two places

After merge, the merged SHA + PR URL get written into:

1. **Prior-phase archived `tasks.md` files** in this UAR repo. Each of the 8 changes' tasks.md §9 or §10 has a placeholder block:
   ```
   prometheus-skill-system commit: <fill in after merge>
   PR URL:                         <fill in after merge>
   ```
   We update all 8 in a single follow-up commit on a single new branch in this UAR repo (or directly to the active branch; the operator chooses).

2. **This change's own tasks.md** §"Closeout" — a section that records the merged SHA + PR URL once. Future audits can find it quickly without scanning every archived tasks.md.

### D5. No dry-run requirement before push

The diff has been live-tested (73/73 smoke assertions, two live UAR injections). Re-validating before push offers nothing new; we trust the prior phase's verification.

Operator may still want to `git diff --stat origin/main` locally before push as a sanity check — that's noted in tasks but not gated.

### D6. Reviewer assignment

The orchestrator skill set's ownership is project-internal — the operator (who runs this change) is presumably the maintainer. If a second reviewer is desired, the PR description should `@`-mention them; this change doesn't prescribe an assignee.

### D7. CI behavior

The existing skill-system CI (if any) runs against the PR. The new bash test scripts (`shared/lib/tests/test-*.sh`, `references/schemas/fixtures/test.sh`) will need to be wired into that CI in a *separate* change — they currently exist as files but aren't invoked by any workflow. This PR ships the tests; the CI integration is filed as a follow-up.

### D8. Rollback strategy

If post-merge testing surfaces a serious regression, the rollback is `git revert <merge-commit>` plus a new PR. The 8 changes' archived artifacts in this UAR repo are untouched by that revert — the OpenSpec record persists and a future change can land a corrected version pointing to the same archived spec.

If revert is partial (e.g. one of the 8 changes is bad), the path is:
- `git revert <merge-commit>` (full revert)
- Cherry-pick the 7 good changes onto a new branch from the revert point
- New PR

This is slightly fiddly but the alternative (per-change commits) makes the happy path more painful. Bet that the happy path is the common case.

### D9. The skill-system repo's `.gitignore` is not part of this change

Some of the untracked paths (`shared/lib/tests/`, `references/schemas/fixtures/`) might warrant a `.gitignore` review (e.g. should fixture outputs be ignored?). Out of scope here — the assumption is the prior phase already established the intent that these *are* tracked.

## Implementation Sketch

### `git` invocations (in `prometheus-skill-system` working tree)

```sh
cd /Users/gqadonis/Projects/prometheus/prometheus-skill-system
git fetch origin
git checkout main && git pull --ff-only origin main
git checkout -b feat/kbd-orchestrator-w1-w3-2026-05-27

# Stage exactly the 10 paths — explicit list, not `git add -A`, so an
# unrelated edit doesn't sneak into the commit.
git add \
  skills/process/kbd-process-orchestrator/SKILL.md \
  skills/process/kbd-process-orchestrator/hooks/hooks.json \
  skills/process/kbd-process-orchestrator/references/schemas/hooks-config.schema.json \
  skills/process/kbd-process-orchestrator/references/schemas/project.template.json \
  skills/process/kbd-process-orchestrator/references/schemas/current-waypoint.template.json \
  skills/process/kbd-process-orchestrator/references/schemas/fixtures/ \
  skills/process/kbd-process-orchestrator/shared/ \
  skills/process/kbd-process-orchestrator/skills/kbd-status/SKILL.md \
  skills/process/kbd-process-orchestrator/skills/kbd-assess/SKILL.md \
  skills/process/kbd-process-orchestrator/skills/kbd-execute/SKILL.md \
  skills/process/kbd-process-orchestrator/skills/kbd-next-phase/SKILL.md \
  skills/process/kbd-process-orchestrator/skills/kbd-plan/SKILL.md \
  skills/process/kbd-process-orchestrator/skills/kbd-reflect/SKILL.md \
  skills/process/kbd-process-orchestrator/skills/kbd-new-phase/ \
  skills/process/kbd-process-orchestrator/skills/kbd-new-child/ \
  skills/process/kbd-process-orchestrator/skills/kbd-next-child/ \
  skills/process/kbd-process-orchestrator/skills/kbd-memory-recall/ \
  skills/process/kbd-process-orchestrator/skills/kbd-inject-agent-rules/ \
  skills/react/prometheus-entity-skills/entity-realtime-surreal-live/ \
  skills/react/prometheus-entity-skills/entity-graph-optimize/SKILL.md

git status --short                 # operator sanity-check
git diff --cached --stat           # confirm scope

git commit -s -m "$(cat <<'EOF'
feat(kbd-process-orchestrator): nested phases + hooks + new-phase + child-phase + memory + rule-injector + uiux routing

Composite landing of 8 OpenSpec changes from the universal-agent-runtime
phase submodule-skills-and-entity-devtools-expansion:

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
EOF
)"

git push -u origin feat/kbd-orchestrator-w1-w3-2026-05-27

gh pr create --title "feat(kbd-process-orchestrator): nested phases + hooks + new-phase + child-phase + memory + rule-injector + uiux routing" \
  --body "$(cat <<'EOF'
Composite landing of 8 OpenSpec changes from the universal-agent-runtime
phase `submodule-skills-and-entity-devtools-expansion`. See archive at
`universal-agent-runtime/openspec/changes/archive/2026-05-27-ssed-*` for
the full proposal / spec / design / tasks per change.

## Summary
- New skills: `kbd-new-phase`, `kbd-new-child`, `kbd-next-child`,
  `kbd-memory-recall`, `kbd-inject-agent-rules`.
- New shared library: `shared/lib/{waypoint,hooks,memory,memory-log}.sh`
  + smoke tests.
- Hooks system: extended `hooks/hooks.json` + schema; default
  report-progress reporter; augment/override semantics.
- New skill in `skills/react/prometheus-entity-skills/`:
  `entity-realtime-surreal-live`.
- Updated SKILL.md across orchestrator, kbd-status, kbd-assess,
  kbd-plan, kbd-execute, kbd-reflect, kbd-next-phase, plus
  `entity-graph-optimize` subsection.

## Test plan
- [x] `bash shared/lib/tests/test-hooks.sh` (10 assertions)
- [x] `bash shared/lib/tests/test-kbd-new-phase.sh` (12 assertions)
- [x] `bash shared/lib/tests/test-kbd-child-phase.sh` (10 assertions)
- [x] `bash shared/lib/tests/test-memory.sh` (6 assertions)
- [x] `bash shared/lib/tests/test-agent-rules-injector.sh` (13 assertions)
- [x] `bash references/schemas/fixtures/test.sh` (9 fixture assertions)
- [x] live UAR `kbd-new-phase.sh` invocation + clean rollback
- [x] live UAR `kbd-inject-agent-rules` for both packs (agent-rules + uiux-routing) — idempotent

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

### Post-merge UAR-side updates

A small helper in this UAR repo loops over the 8 archived changes and rewrites the placeholders:

```sh
sha="<merged SHA>"
url="<PR URL>"
for change in \
  ssed-kbd-nested-phase-schema \
  ssed-kbd-process-hooks \
  ssed-kbd-new-phase-skill \
  ssed-kbd-child-phase-skills \
  ssed-kbd-memory-first-execution \
  ssed-kbd-agent-rules-injector \
  ssed-uar-uiux-skill-routing \
  ssed-entity-surreal-live-adapter \
  ssed-entity-explorer-fab-panel \
; do
  f="openspec/changes/archive/2026-05-27-$change/tasks.md"
  [[ -f "$f" ]] || continue
  sed -i.bak "s|prometheus-skill-system commit: <fill in after merge>|prometheus-skill-system commit: $sha|" "$f"
  sed -i.bak "s|PR URL:                              <fill in after merge>|PR URL:                              $url|" "$f"
  sed -i.bak "s|PR URL:                         <fill in after merge>|PR URL:                         $url|" "$f"
  rm -f "$f.bak"
done

git add openspec/changes/archive/2026-05-27-ssed-*/tasks.md
git commit -m "chore(kbd): record skill-system merge SHA in archived tasks"
```

(macOS `sed` quirks: the `-i.bak` form is the portable spelling; we delete the `.bak` afterwards.)

## Risks

1. **Reviewer pushback on commit shape.** Reviewers may prefer 8 separate commits. Mitigation: D1 explains the rationale; if the reviewer insists, splitting after the fact is `git reset --soft` + per-prior-change `git commit -p` — annoying but mechanical.
2. **Conflict on `main` after rebase.** If `main` advances between local pull and push, a rebase is needed. Standard git flow; not unique to this change.
3. **Untracked dirs containing unintended files.** D2-style explicit `git add` paths reduce this risk; the operator's `git status --short` sanity-check before commit catches the rest.
4. **DCO sign-off requirement.** `-s` is included in the commit invocation. If the repo doesn't require DCO, the sign-off is harmless; if it does, we're compliant.
5. **`gh` CLI unavailable.** Fall back to opening the PR in the GitHub web UI; the title and body templates above port directly.

## Alternatives Considered

- **8 separate PRs.** Rejected per D1.
- **Squash all 8 prior-phase changes into one OpenSpec change in this phase too.** Rejected — the prior phase's archive integrity matters; we don't rewrite it.
- **Wait for entity-management work to land before opening this PR.** Rejected — that creates a dependency cycle (entity-management imports the skill versions; the skill versions need to land first).
- **Skip the post-merge UAR-side updates.** Rejected — without the merge SHA back-references, future audits can't cross-reference the OpenSpec archive with the actual commit history.
