# Implementation Tasks — seim-em-worktree-setup

> Provisioning change. Result: working worktree + UAR sidecar files.
> Surfaced a real bash-cwd-tracking lesson — captured in §3 below.

## 0. Pre-flight

- [x] 0.1 entity-mgmt checkout exists at `/Users/gqadonis/Projects/prometheus/prometheus-entity-management`
- [x] 0.2 Target path `~/.claude/worktrees/seim-entity-management` empty before start
- [x] 0.3 Parent dir `~/.claude/worktrees/` ensured

## 1. Provision worktree

- [x] 1.1 `cd /Users/gqadonis/Projects/prometheus/prometheus-entity-management`
- [x] 1.2 `git fetch origin` — picked up 3 new commits on main (`9c76030..1abae4f`)
- [x] 1.3 `git checkout main` (note: branch was behind by 3 commits; the worktree-add used `origin/main` directly so the local main staying behind is acceptable)
- [x] 1.4 Captured `base_sha=1abae4f74b3c3e2b22a0c2f7ef18e931a89a81fd`
- [x] 1.5 `git worktree add -b feat/seim-entity-management-impl ~/.claude/worktrees/seim-entity-management origin/main` succeeded
- [x] 1.6 `git worktree list` shows both checkouts at `1abae4f`

## 2. Verify the new worktree

- [x] 2.1 cd into the new worktree
- [x] 2.2 `git rev-parse --show-toplevel` = `/Users/gqadonis/.claude/worktrees/seim-entity-management` ✓
- [x] 2.3 `git rev-parse HEAD` = `$base_sha` ✓
- [x] 2.4 `git status --short` empty ✓
- [x] 2.5 `pnpm install --frozen-lockfile` — **done in 3.6s** (warm pnpm store)
- [x] 2.6 `pnpm typecheck` — clean (`tsc --noEmit` returned 0; no setup-related failures)

## 3. Record worktree state — `worktrees.json` sidecar

- [x] 3.1 cd back to UAR worktree (**lesson learned in §3.x**: the Bash tool's shell cwd did NOT auto-reset after §2.5/§2.6's `cd`, so the first sidecar write landed in the entity-mgmt worktree instead of UAR. Recovered by `mv`-ing the two files back to UAR and `rmdir`-ing the empty entity-mgmt subdir. Entity-mgmt is clean again. Worth recording as a known gotcha for future agent runs.)
- [x] 3.2 `worktrees.json` written at `.kbd-orchestrator/phases/submodule-entity-management-implementation/worktrees.json` in **the UAR worktree** (after relocation)
- [x] 3.3 `jq .` parses cleanly

## 4. Append `execution.md` note

- [x] 4.1 `## Worktree provisioning` section appended to **UAR-side** `execution.md` (after relocation per §3.x lesson)
- [x] 4.2 `grep '^## Worktree provisioning' execution.md` returns 1 match

## 5. Verification cross-checks

- [x] 5.1 new worktree branch: `feat/seim-entity-management-impl`
- [x] 5.2 new worktree status: clean (after the §3.x cleanup)
- [x] 5.3 original entity-mgmt main checkout: branch `main`, 0 status entries — untouched
- [x] 5.4 `git worktree list` shows both checkouts at `1abae4f`

## 6. Documentation

- [x] 6.1 No README/index update needed (`worktrees.json` self-describes via `$comment`)
- [ ] 6.2 **Follow-up**: orchestrator `SKILL.md` doesn't yet document `worktrees.json` as a phase-sidecar convention. File for a future change titled `seim-skill-orchestrator-phase-sidecar-doc` or similar.

## 7. Cross-repo commit

- [x] 7.1 **Zero commits to entity-mgmt** in this change (worktree carries an empty branch).
- [ ] 7.2 UAR commit pending: `chore(kbd): provision prometheus-entity-management worktree (seim-em-worktree-setup)` — staged at `/opsx:archive` time as part of the consolidated archive operation. **Pending user-driven** (or batched with later changes in this phase's UAR commits).

## 8. Closeout

- [x] 8.1 `progress.json` will update: `changes_completed: 3`, `active_change: seim-em-surreal-live-adapter-impl` (or `seim-em-engine-devtools-tap` — W3 has two parallel items)
- [ ] 8.2 `/opsx:verify seim-em-worktree-setup` — recommend `--skip-qa` (no capabilities, doc/git-op only)
- [ ] 8.3 `/opsx:archive seim-em-worktree-setup` — no spec promotion
- [x] 8.4 `current-waypoint.json` will refresh to W3 start

## §3.x Known gotcha — recorded for the orchestrator's future memory

When the Bash tool runs without an explicit `cd`, the shell starts at *whatever directory the previous bash invocation ended in*. The "Shell cwd was reset" line shown in some bash outputs only appears on certain transitions (apparently not after `cd` followed by a multi-line script without trailing `cd ..`). Lesson: **for cross-worktree writes, every bash invocation that writes files should start with an explicit `cd <abs-path>`** to avoid wrong-directory bugs.

This change recovered cleanly; future changes should adopt the discipline preemptively. The auto-memory-recall hook will surface this when the next phase starts, so future agent runs benefit.

## Rollback (not invoked — verification passed)

- Available paths documented in design §D4. Not needed.
