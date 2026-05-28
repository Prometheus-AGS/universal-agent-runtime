## 1. Helper scripts

- [x] 1.1 Create `scripts/worktree-new.sh` per the §Implementation Sketch in `design.md`:
  - [x] 1.1.1 Shebang `#!/usr/bin/env bash`; `set -euo pipefail`
  - [x] 1.1.2 Argument parsing: `<name>` required; optional `--base <ref>` (default `HEAD`)
  - [x] 1.1.3 Resolve `root="${HOME:?HOME not set}/.claude/worktrees"`, `target="$root/$name"`
  - [x] 1.1.4 Refuse if `$target` exists (clear error → suggest `scripts/worktree-list.sh`)
  - [x] 1.1.5 Refuse if canonicalized `$target` is a descendant of `git rev-parse --show-toplevel`
  - [x] 1.1.6 `mkdir -p "$root"`; run `git worktree add "$target" "$base"`
  - [x] 1.1.7 Seed `.claude/settings.local.json` into the new tree if source exists
  - [x] 1.1.8 Final stdout: `created: $target`
- [x] 1.2 Create `scripts/worktree-list.sh`:
  - [x] 1.2.1 `git worktree list --porcelain` filtered to entries under `${HOME}/.claude/worktrees/`
  - [x] 1.2.2 Pretty-print as `<path>\t<branch>\t<HEAD-short>`
- [x] 1.3 Create `scripts/worktree-rm.sh`:
  - [x] 1.3.1 Refuse names whose resolved path is not under `${HOME}/.claude/worktrees/`
  - [x] 1.3.2 Pass through to `git worktree remove "${HOME}/.claude/worktrees/$name"`
  - [x] 1.3.3 Surface `--force` flag to the underlying git invocation
- [x] 1.4 `chmod +x scripts/worktree-*.sh`
- [x] 1.5 Manual smoke test on macOS (Bash 3.2): create → list → rm; verified — see §6 below

## 2. Documentation — CLAUDE.md and AGENTS.md

- [x] 2.1 Append a "Worktree convention" section to `CLAUDE.md` with:
  - [x] 2.1.1 Single-sentence rule
  - [x] 2.1.2 Reason (collision with checked-in `.claude/` config)
  - [x] 2.1.3 Usage line: `scripts/worktree-new.sh <name>`
  - [x] 2.1.4 Note the non-relocation of existing in-repo worktrees
- [x] 2.2 Append the identical section verbatim to `AGENTS.md`
- [x] 2.3 Cross-link from `CONTRIBUTING.md` "Local setup" → the new section

## 3. Guard rails

- [x] 3.1 Add `/.claude/worktrees/` to `.gitignore` (anchored)
- [x] 3.2 Verify `git status` reports no files inside any in-repo worktree (manual: see Verification §6)
- [x] 3.3 Document the advisory pre-commit one-liner in `CONTRIBUTING.md`

## 4. KBD orchestrator integration

- [x] 4.1 Add `"worktreeRoot": "${HOME}/.claude/worktrees"` to `.kbd-orchestrator/project.json`
- [ ] 4.2 Update `kbd-status` skill (in `~/.claude/skills/kbd-process-orchestrator/skills/kbd-status/`) to:
  - [ ] 4.2.1 Read `worktreeRoot` from `project.json` (default `${HOME}/.claude/worktrees`)
  - [ ] 4.2.2 Compute `$(git rev-parse --show-toplevel)`
  - [ ] 4.2.3 Render `worktree: <path>`; if path is not a descendant of `worktreeRoot`, render `worktree: <path>  ⚠ outside worktreeRoot`
- [ ] 4.3 Document the new field in `references/schemas/project.template.json`

> **Deferred — cross-repo dependency.** Tasks 4.2 and 4.3 modify the
> `kbd-process-orchestrator` skill, which lives in the separate
> `prometheus-skill-system` repository at
> `/Users/gqadonis/Projects/prometheus/prometheus-skill-system`. This worktree
> is checked out from `universal-agent-runtime` only. The schema field is in
> place here (4.1) so the orchestrator change is purely additive and can land
> independently in its own PR. Filed as a follow-up; will be picked up under
> change `ssed-kbd-nested-phase-schema` (W1), which already needs to modify
> the same skill set.

## 5. Tests

- [x] 5.1 Add `scripts/tests/worktree-new.bats` covering:
  - [x] 5.1.1 Happy path: name + default base
  - [x] 5.1.2 Happy path: name + `--base feature/x`
  - [x] 5.1.3 Refusal: pre-existing target
  - [x] 5.1.4 Refusal: target inside repo (via crafted `$HOME`)
  - [x] 5.1.5 Refusal: `$HOME` unset
  - [x] 5.1.6 Seed: source `settings.local.json` exists → copied
  - [x] 5.1.7 Seed: source absent → no error
- [x] 5.2 Add `scripts/tests/worktree-rm.bats` covering path-prefix refusal
- [x] 5.3 Document bats install in `CONTRIBUTING.md`; gate is by-availability (no mandatory dep)

## 6. Verification

- [x] 6.1 `bash scripts/worktree-new.sh ssed-smoke-001` → created `~/.claude/worktrees/ssed-smoke-001` with seeded `.claude/settings.local.json` ✓
- [x] 6.2 `git rev-parse --show-toplevel` from the new worktree returns that worktree ✓ (verified during smoke test)
- [x] 6.3 `bash scripts/worktree-list.sh` showed the new worktree alongside `musing-sinoussi-09cea6`; `bash scripts/worktree-rm.sh ssed-smoke-001 --force` removed it cleanly; post state shows only the two pre-existing siblings ✓
- [x] 6.4 `CLAUDE.md` and `AGENTS.md` "Worktree convention" sections are byte-identical ✓
- [ ] 6.5 `/kbd-status` rendering — **deferred** with task 4.2 (cross-repo)
- [x] 6.6 OpenSpec artifact set complete: proposal, spec, design, tasks ✓

## 7. Closeout

- [x] 7.1 Update `.kbd-orchestrator/phases/submodule-skills-and-entity-devtools-expansion/progress.json` → `changes_completed += 1`, append change to `completed_changes[]`
- [ ] 7.2 Run `/refine-validate ssed-worktree-persistence-convention` (QA gate required: ≥3 files, not doc-only) — **pending** (user-driven)
- [ ] 7.3 On QA pass: `/opsx:verify ssed-worktree-persistence-convention` → `/opsx:archive ssed-worktree-persistence-convention` — **pending**
- [x] 7.4 Refresh `current-waypoint.json` to point at change 2: `ssed-kbd-nested-phase-schema`
