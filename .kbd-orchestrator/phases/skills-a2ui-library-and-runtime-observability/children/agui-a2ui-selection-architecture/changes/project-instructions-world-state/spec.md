<!-- mirror of openspec/changes/project-instructions-world-state/proposal.md and specs/*/spec.md -->
# project-instructions-world-state

Rank 9 of the codex-harness-comparative-analysis change set. Source: gap G9 in the phase `analysis.md`.

## Why

There are zero references to `AGENTS.md`, `CLAUDE.md`, cwd, time, or environment injection anywhere in `src/`. The operator's goal includes code generation and agentic development, and every surveyed harness layers project instruction files by ancestor walk and injects an environment block. Without it, an agent working in a repository does not know the repository's rules or its own working directory.

Codex discovers `AGENTS.md` by walking up to a project-root marker, collecting root-to-cwd, preferring an override file, and skipping untrusted projects (`core/src/agents_md.rs:1-64`), and represents environment, time, and permissions as world-state sections with stable ids that are diffed with RFC 7386 merge patches so only changes are re-sent (`core/src/context/world_state/mod.rs:228-348`). `walkdir` 2.5 is already pinned; `json-patch` has a merge apply but no generator, so the generator is built in-house. Codex paths are outside this repository; verbatim excerpts for the cited lines are in the phase `analysis.md` appendix "verified Codex excerpts".

## What changes

- Project-instruction discovery: configurable file names (`AGENTS.md` default; `CLAUDE.md`, `GEMINI.md` optional), root marker list (`.git` default), root-to-cwd concatenation with a separator, `AGENTS.override.md` preferred, untrusted projects skipped, subtree files loaded on first file read in that subtree. Rendered as `Host` fragments.
- World-state sections with stable ids: environment (cwd, workspace roots, platform), current time, permissions summary, active project instructions. Full render on the first turn or after any history rewrite; merge-patch diff thereafter so only changed sections re-enter history, with replacement and removal text per section.
- Trust boundary: project instructions are `Host` authority and cannot override `System` or `Policy` fragments; an operator flag marks a workspace trusted.

## Scope

- new: `src/uar/runtime/world_state/{mod.rs,sections.rs,merge_patch.rs}`, `src/uar/runtime/project_instructions.rs`
- `src/uar/runtime/turn/contributors.rs` (a world-state contributor)
- `src/config.rs` (file names, root markers, trusted workspaces)
- tests: `tests/project_instructions.rs`, `tests/world_state_diff.rs`

Out of scope: skill discovery paths (already covered by standard-agent-skill-discovery), remote instruction URLs.

## Dependencies

deterministic-prompt-assembly (fragments), typed-turn-assembly (contributor stage).

## Verification

Tier 0 per edit; Tier 1 the new tests; Tier 2 at the boundary.

## The uncomfortable thing

Reading instruction files from a working directory is an injection surface. Untrusted-workspace skipping and `Host` authority are the controls; the change must not ship with a default that trusts every directory it is pointed at.


## Spec delta: project-instructions-world-state

## ADDED Requirements

### Requirement: Project instruction files are discovered from root to cwd
The runtime SHALL discover project instruction files by walking up from the working directory to a configured root marker, SHALL concatenate every matching file from the root down to the working directory in that order, SHALL prefer an override file when present, SHALL load subtree files on first file access in that subtree, and SHALL NOT read above the project root.

#### Scenario: Nested instruction files
- **WHEN** the repository root and a subdirectory both contain an instruction file and the working directory is the subdirectory
- **THEN** the rendered host instructions contain the root file followed by the subdirectory file

#### Scenario: Untrusted workspace
- **WHEN** the workspace is not marked trusted
- **THEN** no project instruction file is read or rendered

### Requirement: Project instructions carry host authority
Project instructions SHALL be rendered as host-authority fragments and SHALL NOT override system or policy fragments.

#### Scenario: Instruction file imitates policy
- **WHEN** an instruction file contains text formatted as a policy directive
- **THEN** it renders inside host markers and the policy fragment is unchanged

### Requirement: World state is rendered in full once and diffed thereafter
The runtime SHALL maintain world-state sections with stable ids for environment, current time, permissions, and active project instructions, SHALL render every section in full on the first turn and after any history rewrite, and SHALL otherwise render only a merge-patch diff of changed sections with replacement or removal text. The current-time section SHALL be compared at a configured granularity (default one minute) from a clock the runtime can substitute in tests, so a turn inside the same granularity bucket counts as unchanged.

#### Scenario: Working directory changes
- **WHEN** the working directory changes between turns
- **THEN** only the environment section is re-sent, stating that it replaces the previous environment

#### Scenario: Nothing changed within the time granularity
- **WHEN** no section changed since the last turn and the current time is in the same granularity bucket as the last render
- **THEN** no world-state fragment is added to history

#### Scenario: Time bucket rolls over
- **WHEN** only the current time has crossed into a new granularity bucket since the last render
- **THEN** only the current-time section is re-sent
