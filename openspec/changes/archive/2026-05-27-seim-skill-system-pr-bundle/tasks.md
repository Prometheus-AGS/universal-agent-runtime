# Implementation Tasks — seim-skill-system-pr-bundle

> Git operation, not a code change. Local + push + PR done in this session.
> Merge + post-merge UAR archive updates remain user-gated.

## 0. Pre-flight sanity check

- [x] 0.1 Smoke tests pass (60/60 across six scripts):
  - [x] 0.1.1 `test-hooks.sh` — 10/10
  - [x] 0.1.2 `test-kbd-new-phase.sh` — 12/12
  - [x] 0.1.3 `test-kbd-child-phase.sh` — 10/10
  - [x] 0.1.4 `test-memory.sh` — 6/6
  - [x] 0.1.5 `test-agent-rules-injector.sh` — 13/13
  - [x] 0.1.6 `references/schemas/fixtures/test.sh` — 9/9
- [x] 0.2 Dirty-path inventory — 11 modified + 9 untracked (proposal's "10+3" understated; reality includes memory/rule-injector/entity skill paths). Adjusted staging accordingly.

## 1. Topic branch

- [x] 1.1 cd into `/Users/gqadonis/Projects/prometheus/prometheus-skill-system`
- [x] 1.2 `git fetch origin`
- [x] 1.3 `git checkout main && git pull --ff-only origin main` — already up to date
- [x] 1.4 `git checkout -b feat/kbd-orchestrator-w1-w3-2026-05-27`

## 2. Stage explicitly

- [x] 2.1 Ran explicit `git add` with 19 paths (covering all 20 dirty entries; some grouped into directories)
- [x] 2.2 `git status --short` confirms 43 staged paths, nothing else
- [x] 2.3 `git diff --cached --stat` clean: **+3327 / −23 lines across 43 files**

## 3. Commit

- [x] 3.1 `git commit -s -m "$(cat <<EOF…EOF)"` — single coherent commit per design D1
- [x] 3.2 Local SHA: **`0d62578ecbbf6950afb8a64da3a47203912ab556`** (pre-squash)
- [x] 3.3 Sign-off present (Travis James)
- [x] 3.4 Commit message body lists all 8 prior changes by ID; corrected smoke-test total to 60/60 (reflection said 73 — minor inaccuracy noted)

## 4. Push and open PR

- [x] 4.1 `git push -u origin feat/kbd-orchestrator-w1-w3-2026-05-27` — new branch pushed
- [x] 4.2 `gh pr create` — **PR #3 opened**
- [x] 4.3 Captured artifacts:
  - **PR URL**: https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3
  - **PR number**: 3
  - **Branch SHA**: `0d62578ecbbf6950afb8a64da3a47203912ab556`

## 5. PR review iteration

- [ ] 5.1 Awaiting reviewer feedback (none yet at session time)
- [ ] 5.2 Re-runs of §0.1 if any code touched in iteration

## 6. Merge — user-gated

- [ ] 6.1 CI green wait
- [ ] 6.2 Squash-merge (D3 preferred)
- [ ] 6.3 Capture **merged squash SHA** here once available

```
prometheus-skill-system squash-merge SHA: <fill in after merge>
PR URL:                                    https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3
```

## 7. Post-merge: UAR archive updates — deferred

Cannot execute until §6 produces the merged squash SHA. The helper script in design.md is ready; the operator runs it after merge.

- [ ] 7.1 cd into this UAR worktree (already here)
- [ ] 7.2 Run the post-merge `sed` loop from design §Implementation Sketch with `$sha` = merged squash SHA, `$url` = `https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3`
- [ ] 7.3 `git status --short` shows exactly 9 archived tasks.md modified
- [ ] 7.4 `grep -l '<fill in after merge>' openspec/changes/archive/2026-05-27-ssed-*/tasks.md` returns zero
- [ ] 7.5 Commit `chore(kbd): record skill-system merge SHA in archived tasks`

## 8. Closeout

- [x] 8.1 Merge artifacts (partial — branch state only; squash SHA pending):

```
prometheus-skill-system commit (pre-squash): 0d62578ecbbf6950afb8a64da3a47203912ab556
prometheus-skill-system squash-merge SHA:    <fill in after merge>
PR URL:                                       https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3
PR number:                                    3
Topic branch:                                 feat/kbd-orchestrator-w1-w3-2026-05-27
```

- [ ] 8.2 Update phase progress.json: changes_completed=1, completed_changes append, active_change → `seim-surreal-live-spec-correction`
- [ ] 8.3 `/opsx:verify` (likely needs `--skip-qa` per proposal §Sequencing note — no capabilities declared)
- [ ] 8.4 `/opsx:archive` after verify
- [ ] 8.5 Refresh waypoint to point at change 2

## Rollback (only if merged PR introduces a regression)

- [ ] R1 In skill-system: `git revert <merge-commit-sha>` + new PR
- [ ] R2 In UAR: revert §7 commit
- [ ] R3 Mark this change BLOCKED, open corrective change
