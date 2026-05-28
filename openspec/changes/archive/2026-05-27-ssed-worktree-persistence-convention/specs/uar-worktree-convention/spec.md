## ADDED Requirements

### Requirement: Worktree Root Location
The repository SHALL treat `~/.claude/worktrees/` as the canonical root for every git worktree of this repo, and SHALL NOT permit new worktrees to be created inside the repository's working tree.

#### Scenario: New worktree created via helper script
- **WHEN** a developer runs `scripts/worktree-new.sh <name>`
- **THEN** the script MUST create the worktree at `${HOME}/.claude/worktrees/<name>`, expanding `${HOME}` from the current environment.

#### Scenario: Helper script invoked with a name that would land inside the repo tree
- **WHEN** a developer runs `scripts/worktree-new.sh` with a name whose resolved path falls under the repository working tree (e.g. via `..` traversal or absolute override)
- **THEN** the script MUST exit non-zero with a message naming `~/.claude/worktrees/` as the only permitted root.

#### Scenario: Helper script invoked with an already-used name
- **WHEN** a developer runs `scripts/worktree-new.sh <name>` and `${HOME}/.claude/worktrees/<name>` already exists
- **THEN** the script MUST exit non-zero without modifying the existing directory and suggest running `scripts/worktree-list.sh`.

#### Scenario: Parent directory missing
- **WHEN** `${HOME}/.claude/worktrees/` does not yet exist at script invocation time
- **THEN** the script MUST create it (mode 0755) before calling `git worktree add`.

### Requirement: Worktree Helper Script Surface
The repository SHALL ship three helper scripts under `scripts/` that wrap `git worktree` and confine all paths to `~/.claude/worktrees/`.

#### Scenario: Creating a worktree
- **WHEN** `scripts/worktree-new.sh <name> [--base <ref>]` is invoked with a valid name
- **THEN** the script MUST run `git worktree add ~/.claude/worktrees/<name> <base-or-HEAD>` and seed the new worktree's `.claude/settings.local.json` from the current repo's copy when one exists.

#### Scenario: Listing worktrees
- **WHEN** `scripts/worktree-list.sh` is invoked
- **THEN** the script MUST run `git worktree list --porcelain` and filter output to entries whose path starts with `${HOME}/.claude/worktrees/`.

#### Scenario: Removing a worktree
- **WHEN** `scripts/worktree-rm.sh <name>` is invoked
- **THEN** the script MUST refuse if the resolved path is not under `${HOME}/.claude/worktrees/`, otherwise run `git worktree remove ${HOME}/.claude/worktrees/<name>`.

### Requirement: Repository Documentation
The repository's top-level `CLAUDE.md` and `AGENTS.md` SHALL document the worktree convention so every AI tool reading these files (Roo, Cursor, Codex, OpenCode, Claude Code) receives the same instruction.

#### Scenario: CLAUDE.md read by an AI tool
- **WHEN** any AI tool loads `CLAUDE.md`
- **THEN** the file MUST contain a "Worktree convention" section stating that worktrees are always created under `~/.claude/worktrees/` and never inside the repo tree, and pointing to `scripts/worktree-new.sh`.

#### Scenario: AGENTS.md read by an AI tool
- **WHEN** any AI tool loads `AGENTS.md`
- **THEN** the file MUST contain the same "Worktree convention" section as `CLAUDE.md`, kept in sync.

### Requirement: Defense-in-Depth Against In-Repo Worktrees
The repository SHALL guard against worktrees accidentally appearing inside the working tree.

#### Scenario: Stray worktree appears under .claude/worktrees/
- **WHEN** any path under `.claude/worktrees/` exists in the working tree
- **THEN** `.gitignore` MUST contain a `.claude/worktrees/` rule so that such paths are not tracked.

#### Scenario: Stray worktree appears in git index
- **WHEN** a developer attempts to commit a file whose path is under `.claude/worktrees/`
- **THEN** an advisory pre-commit hook MUST emit a warning that names the offending path and references `scripts/worktree-new.sh`; the hook MUST NOT block the commit (the warning is advisory only).

### Requirement: KBD Orchestrator Awareness
The KBD orchestrator MUST surface the worktree convention so any tool reading `.kbd-orchestrator/project.json` or running `/kbd-status` is aware of the configured root.

#### Scenario: project.json declares worktree root
- **WHEN** `.kbd-orchestrator/project.json` is loaded
- **THEN** it MAY include an optional `worktreeRoot` field whose default value is `${HOME}/.claude/worktrees`; consumers MUST treat the field as authoritative when present.

#### Scenario: kbd-status invoked inside a worktree
- **WHEN** a developer runs `/kbd-status` from any directory
- **THEN** the output MUST include the current `git rev-parse --show-toplevel` value and a warning if that path is not a descendant of the configured `worktreeRoot`.

### Requirement: Non-Disruption of Existing Worktrees
This change SHALL NOT relocate, rename, or otherwise touch any worktree that exists at the time of adoption.

#### Scenario: Existing in-repo worktree at adoption time
- **WHEN** this change is applied while `.claude/worktrees/adoring-booth-312094` (or any other in-repo worktree) exists
- **THEN** the change MUST leave that worktree's files and `git worktree` registration untouched; the convention applies only to worktrees created after adoption.
