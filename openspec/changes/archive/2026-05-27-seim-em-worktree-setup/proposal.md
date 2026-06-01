## Why

W3 onward of this phase produces substantial TypeScript work in `prometheus-entity-management`: a SurrealDB live-query adapter, an engine-side devtools tap, an event bus + multi-store registry, the 5-tab explorer panel, a tree-shake gate, and a Chrome MV3 extension scaffold. That work needs a **clean, isolated working tree** — not `main` of the entity-management checkout, where unrelated changes might land.

The prior phase shipped a worktree convention (`uar-worktree-convention` capability): worktrees for any Prometheus-AGS repo live under `~/.claude/worktrees/`, never inside the target repo's working tree. That convention applies here. This change provisions the worktree, opens a topic branch, and records the path so subsequent changes (4, 5, 7, 8, 9, 10, 11) know where to write code.

It's a small operation but worth its own audit entry: tracking *when* the worktree was provisioned, against which base commit, and on which branch, makes the phase's commit graph reconstructable from the OpenSpec archive.

## What Changes

### Provision the worktree

In `prometheus-entity-management`:

```sh
cd /Users/gqadonis/Projects/prometheus/prometheus-entity-management
git fetch origin
git checkout main && git pull --ff-only origin main
git worktree add -b feat/seim-entity-management-impl \
  ~/.claude/worktrees/seim-entity-management main
```

The `-b` flag creates the topic branch atomically as the worktree is checked out; no separate `git checkout -b` step.

### Verify the worktree

From inside the new worktree:

- `git rev-parse --show-toplevel` returns `~/.claude/worktrees/seim-entity-management` (the worktree's own path, not the original repo).
- `git rev-parse HEAD` returns the same SHA as `origin/main` at provisioning time.
- `git status --short` is empty.
- `pnpm install` (the project uses pnpm — see `package.json` and `pnpm-workspace.yaml`) completes without errors; the lockfile is committed so the install is deterministic.

### Record worktree state for downstream changes

Add a new file in this phase's directory: `.kbd-orchestrator/phases/submodule-entity-management-implementation/worktrees.json`. Schema:

```jsonc
{
  "worktrees": {
    "prometheus-entity-management": {
      "path": "~/.claude/worktrees/seim-entity-management",
      "branch": "feat/seim-entity-management-impl",
      "baseCommit": "<sha from origin/main at provisioning>",
      "provisionedAt": "<ISO-8601 UTC>",
      "provisionedBy": "seim-em-worktree-setup"
    }
  }
}
```

This is a **new convention** introduced here: phases can carry a `worktrees.json` file alongside `progress.json` to track per-repo worktree provisioning. Subsequent changes (4, 5, 7–11) read this file to discover where to write code. No mutation of `progress.json` itself; `worktrees.json` is its own sidecar so it can grow without bloating progress.

### Topic-branch policy

The single branch `feat/seim-entity-management-impl` carries every entity-management commit in this phase. That keeps the phase's downstream review surface tight: at the end of the phase, one PR or a small number of PRs land all the entity-management work together. Each subsequent change's commits go onto this same branch (no per-change sub-branches); the OpenSpec archive provides the per-change audit trail.

If the branch's diff grows past review-comfortable size mid-phase, the operator may split into multiple PRs along change boundaries — but that's an in-flight decision, not a precondition of this change.

### Documentation note

Update this phase's `execution.md` (the dispatch contract written by `/kbd-execute`) with a section "Worktree provisioning" pointing at the new `worktrees.json` and naming the branch + base commit.

### What this change does NOT include

- **No code in prometheus-entity-management.** The worktree is empty (mirror of `main`); commits land in subsequent changes.
- **No port of `scripts/worktree-new.sh`** from UAR into entity-management. That helper is UAR-specific (its `is_descendant` check resolves against UAR's repo root). A repo-agnostic helper is a possible future change, out of scope here.
- **No skill-system worktree.** The skill-system PR #3 is already merged; future skill-system work happens on a fresh feature branch off `main` as needed, not in a persistent worktree (skill-system has many small contributors; sharing a worktree would create lock-step coordination overhead that isn't justified yet).
- **No edit to UAR's worktree.** This UAR worktree (`adoring-booth-312094`) stays put per the prior phase's non-relocation policy.

## Capabilities

### New Capabilities

- None. The worktree convention itself is the existing `uar-worktree-convention` capability shipped in the prior phase. This change is a provisioning step honoring that capability, not a new capability declaration.

### Modified Capabilities

- None.

## Impact

- **Risk**: Trivial. The provisioning command set is idempotent-ish (re-running it after success errors out cleanly because the worktree already exists; no destructive operations).
- **Affected files**:
  - **Outside any repo**: `~/.claude/worktrees/seim-entity-management/` (a full checkout of prometheus-entity-management at `feat/seim-entity-management-impl`).
  - **Inside prometheus-entity-management**: `.git/worktrees/seim-entity-management/` (git's internal worktree record); no source-tree changes.
  - **Inside this UAR repo**:
    - `.kbd-orchestrator/phases/submodule-entity-management-implementation/worktrees.json` (new sidecar file)
    - `.kbd-orchestrator/phases/submodule-entity-management-implementation/execution.md` (small note added)
- **Cross-repo**: Yes, in the sense that the worktree resides outside any repo's working tree; but no commits to either repo as part of this change.
- **Reversibility**: Trivial — `git worktree remove ~/.claude/worktrees/seim-entity-management` cleans up the on-disk state; delete the sidecar `worktrees.json` and remove the execution.md note to revert UAR state.
- **Unblocks**: every change in W3 onward of this phase. Each one writes code into `~/.claude/worktrees/seim-entity-management/` and commits onto `feat/seim-entity-management-impl`.

### Sequencing note

Same artifact-sequence variance as W0 (`seim-skill-system-pr-bundle`) and W1 (`seim-surreal-live-spec-correction`): zero capabilities, so the spec-driven flow collapses to `proposal → design → tasks`. `/opsx:verify` should accept "no specs" as valid; if it doesn't, `--skip-qa` is the documented fallback. This change is also below the 3-file threshold (2 UAR-side files), making it a doubly-good `--skip-qa` candidate.
