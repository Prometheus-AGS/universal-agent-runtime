## Why

Today this repo's git worktrees are being created **inside** the working tree at `.claude/worktrees/<name>/`. That directory collides with the checked-in `.claude/` configuration (settings, commands, skills) — the same `.claude/` that Roo, Cursor, Codex, OpenCode, and Claude Code all read. The collision means a worktree can shadow or be polluted by the repo's own `.claude/` files, and any "clean the worktrees" sweep risks deleting tool config that belongs in the tree.

The user's preferred location — `~/.claude/worktrees/` — is already partially in use (two siblings, `confident-wilbur-c27abe` and `musing-sinoussi-09cea6`, exist there today) but there is no convention, tooling, or guard rail to make that the default. This change closes the gap before the rest of `submodule-skills-and-entity-devtools-expansion` lands, because every subsequent change in that phase will be developed in a worktree and we don't want to deepen the bad pattern.

## What Changes

### Tooling
- Add `scripts/worktree-new.sh <name> [--base <ref>]` that:
  - Computes `~/.claude/worktrees/<name>` as the destination (creating `~/.claude/worktrees/` if missing).
  - Runs `git worktree add` against that path with optional base ref.
  - Seeds the new worktree's local `.claude/settings.local.json` from the repo's current one so per-tool permissions follow the developer.
  - Refuses to run if `<name>` already exists under `~/.claude/worktrees/` or if the destination would land inside the repo tree.
- Add `scripts/worktree-list.sh` and `scripts/worktree-rm.sh <name>` as thin wrappers around `git worktree list` / `git worktree remove` scoped to `~/.claude/worktrees/`.

### Documentation
- Update this repo's `CLAUDE.md` and `AGENTS.md` with a "Worktree convention" section: **always** create worktrees under `~/.claude/worktrees/`; **never** inside the repo tree; use `scripts/worktree-new.sh`.
- Note the rationale (collision with checked-in `.claude/` config) so future maintainers don't reintroduce the bad path.

### Guard rails
- Add `.claude/worktrees/` to `.gitignore` (defense in depth — the directory exists today and we don't want it tracked even if a stray worktree is ever created there).
- Add a `pre-commit` advisory hook entry (non-blocking warning) that flags any path under `.claude/worktrees/` showing up in the index — gives the developer a chance to back out before pushing.

### KBD orchestrator integration
- `.kbd-orchestrator/project.json` gains an optional `worktreeRoot` field defaulting to `~/.claude/worktrees/`. The orchestrator's `kbd-status` skill will surface the active worktree's path so it's obvious when a developer is inside the wrong tree.

### Non-changes
- The **currently active** worktree (`.claude/worktrees/adoring-booth-312094`) is **not relocated** mid-phase — the convention applies to the *next* worktree created. Relocating live work risks losing uncommitted state and violates the phase plan's risk note.

## Capabilities

### New Capabilities
- `uar-worktree-convention`: A documented, tool-enforced convention that all git worktrees for this repo live under `~/.claude/worktrees/` and are created via `scripts/worktree-new.sh`, with KBD orchestrator awareness and `.gitignore` defense in depth.

### Modified Capabilities
- None.

## Impact

- **Risk**: Low. Tooling is additive; the convention is doc + advisory hook; no existing worktree is touched.
- **Affected files**: `scripts/worktree-new.sh` (new), `scripts/worktree-list.sh` (new), `scripts/worktree-rm.sh` (new), `CLAUDE.md`, `AGENTS.md`, `.gitignore`, `.kbd-orchestrator/project.json`.
- **Cross-tool**: Roo, Cursor, Codex, OpenCode, Claude Code all benefit equally — none requires changes.
- **Reversibility**: Trivial — drop the scripts, revert the docs.
- **Unblocks**: Every other change in `submodule-skills-and-entity-devtools-expansion` (each will be developed in a properly-located worktree).
