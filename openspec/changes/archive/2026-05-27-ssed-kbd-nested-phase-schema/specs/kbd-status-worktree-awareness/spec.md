## ADDED Requirements

### Requirement: Worktree Path Rendering
The `kbd-status` skill SHALL render the active worktree's top-level path on every invocation, so that a developer always knows which checkout a given status report describes.

#### Scenario: kbd-status invoked inside any worktree
- **WHEN** a developer runs `/kbd-status` from any directory inside a git checkout
- **THEN** the output MUST include a `worktree: <path>` line where `<path>` is the result of `git rev-parse --show-toplevel` evaluated from the current working directory.

#### Scenario: kbd-status invoked outside a git checkout
- **WHEN** `/kbd-status` is invoked from a directory that is not inside a git checkout
- **THEN** the output MUST include `worktree: (none — not inside a git checkout)` and MUST continue rendering the remainder of the status report (phase, change, waypoint) using only `.kbd-orchestrator/` state.

### Requirement: worktreeRoot Resolution
The `kbd-status` skill SHALL resolve the active `worktreeRoot` from `project.json`, falling back to a documented default, before evaluating the worktree's location.

#### Scenario: project.json declares worktreeRoot
- **WHEN** `.kbd-orchestrator/project.json` is present and contains a non-empty `worktreeRoot` string
- **THEN** the skill MUST treat that value as authoritative and expand any `${HOME}` (or other documented variables) using the current environment.

#### Scenario: project.json omits worktreeRoot
- **WHEN** `.kbd-orchestrator/project.json` is present but lacks a `worktreeRoot` field
- **THEN** the skill MUST use the literal default `${HOME}/.claude/worktrees`, expanded against the current environment.

#### Scenario: project.json absent
- **WHEN** `.kbd-orchestrator/project.json` does not exist at all (e.g. a fresh project before `/kbd-init`)
- **THEN** the skill MUST still render the worktree line using the documented default and MUST NOT fail the invocation.

### Requirement: Outside-worktreeRoot Warning
The `kbd-status` skill SHALL emit a visible warning whenever the active worktree's top-level path is not a descendant of the resolved `worktreeRoot`, so a developer who lands in the wrong tree sees it immediately.

#### Scenario: Checkout inside the configured root
- **WHEN** `git rev-parse --show-toplevel` returns a path that is a descendant of the resolved `worktreeRoot`
- **THEN** the rendered line MUST be exactly `worktree: <path>` with no warning suffix.

#### Scenario: Checkout outside the configured root
- **WHEN** `git rev-parse --show-toplevel` returns a path that is NOT a descendant of the resolved `worktreeRoot`
- **THEN** the rendered line MUST be `worktree: <path>  ⚠ outside worktreeRoot (<resolved-root>)` with the resolved root included for clarity, and the warning MUST appear in every status report until the developer changes directory.

#### Scenario: Checkout exactly equals worktreeRoot
- **WHEN** the top-level path equals the resolved `worktreeRoot` exactly (the developer is at the root itself, not in a child worktree)
- **THEN** the warning MUST appear (a worktree root is not itself a worktree).

### Requirement: Phase Chain Rendering
The `kbd-status` skill SHALL render the active phase as a chain that reflects the nested-phase fields introduced by the `kbd-nested-phase-schema` capability.

#### Scenario: Top-level phase, no children
- **WHEN** the waypoint's `parentPhase` is `null` and `childPhases` is empty
- **THEN** the rendered line MUST be `phase: <phase-name>` with no separator characters.

#### Scenario: Parent phase with at least one child
- **WHEN** the waypoint declares `childPhases` with one or more entries and `childPointer` selects one of them
- **THEN** the rendered line MUST be `phase: <parent> › <childPointer>` using the U+203A `›` separator, and a second line `children: <i>/<n>` MUST report the active child's 1-based index and the total number of children.

#### Scenario: Child waypoint with parentPhase set
- **WHEN** the waypoint itself has `parentPhase` set (the row represents a child phase)
- **THEN** the rendered chain MUST be `phase: <parentPhase> › <phase>` and the skill MUST still render any deeper `childPointer` chain when present (grand-children).

#### Scenario: childPointer absent while childPhases non-empty
- **WHEN** `childPhases` has entries but `childPointer` is `null` (no child currently active)
- **THEN** the rendered line MUST be `phase: <phase-name>` with a trailing `(children defined, none active)` annotation.

### Requirement: Backward Compatibility for Pre-Schema Waypoints
The `kbd-status` skill SHALL operate identically against waypoint files that predate the nested-phase fields.

#### Scenario: Pre-schema waypoint file
- **WHEN** the waypoint file contains none of `parentPhase`, `childPhases`, or `childPointer`
- **THEN** the skill MUST render `phase: <phase-name>` exactly as it did before this change, with no warnings about missing fields and no degradation.

### Requirement: Graceful Degradation
The new rendering MUST never block or fail the status report; each new piece of information is best-effort.

#### Scenario: git not available
- **WHEN** the `git` binary is not on PATH
- **THEN** the skill MUST render `worktree: (none — git not available)` and continue with the remainder of the status report.

#### Scenario: Cannot read project.json
- **WHEN** `.kbd-orchestrator/project.json` exists but is unreadable (permissions, corrupt JSON)
- **THEN** the skill MUST render `worktree: <path>  ⚠ project.json unreadable, using default root` and continue.
