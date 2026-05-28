## ADDED Requirements

### Requirement: Skill Surface
The `kbd-process-orchestrator` skill set SHALL ship a `/kbd-new-phase` skill at `skills/process/kbd-process-orchestrator/skills/kbd-new-phase/` containing a `SKILL.md` and an executable `kbd-new-phase.sh`.

#### Scenario: Skill directory exists
- **WHEN** an operator inspects the orchestrator skill set after this change lands
- **THEN** `skills/process/kbd-process-orchestrator/skills/kbd-new-phase/` MUST exist and MUST contain a non-empty `SKILL.md` and a non-empty executable `kbd-new-phase.sh`.

#### Scenario: SKILL.md front matter
- **WHEN** a tool reads `kbd-new-phase/SKILL.md`
- **THEN** the file MUST begin with a YAML front-matter block declaring `name: kbd-new-phase`, a `license`, and a one-line `description`, matching the convention used by `kbd-next-phase/SKILL.md`.

#### Scenario: Orchestrator documentation reflects the implementation
- **WHEN** `kbd-process-orchestrator/SKILL.md` is read after this change lands
- **THEN** any references to `/kbd-new-phase` MUST treat it as an implemented skill (no "referenced but not implemented" wording, no TODO markers); the per-skill list MUST include `/kbd-new-phase` alongside `/kbd-next-phase`.

### Requirement: Argument Parsing and Validation
The skill SHALL accept arguments in the form `<name> [goal-1] [goal-2] [goal-n…]` and SHALL reject invalid names with a clear error.

#### Scenario: Valid name and one or more goals
- **WHEN** invoked as `/kbd-new-phase my-phase "Goal one" "Goal two"`
- **THEN** the skill MUST treat `my-phase` as the phase name and the remaining arguments as bulleted goals.

#### Scenario: Valid name with no goals
- **WHEN** invoked as `/kbd-new-phase my-phase`
- **THEN** the skill MUST proceed and write a `goals.md` containing exactly one stub line marked `TBD` so the file is never empty.

#### Scenario: Missing name argument
- **WHEN** invoked with zero arguments
- **THEN** the skill MUST exit non-zero with a usage error referencing the canonical form `kbd-new-phase <name> [goals...]`.

#### Scenario: Name fails kebab-case validation
- **WHEN** the name contains uppercase letters, spaces, slashes, parent-directory traversal (`..`), or a leading hyphen/dot
- **THEN** the skill MUST exit non-zero with a message naming the offending character class and the canonical pattern `^[a-z0-9][a-z0-9._-]*$`.

#### Scenario: Name collides with an existing phase
- **WHEN** the validated name resolves to a directory that already exists at `.kbd-orchestrator/phases/<name>/`
- **THEN** the skill MUST exit non-zero without modifying any file, MUST suggest running `/kbd-next-phase` (if a reflection exists) or choosing a different name, and MUST NOT overwrite the existing phase.

### Requirement: Phase Directory Initialisation
The skill SHALL create `.kbd-orchestrator/phases/<name>/` and seed exactly two files inside it: `goals.md` and `progress.json`.

#### Scenario: Directory tree created
- **WHEN** the skill runs to completion
- **THEN** `.kbd-orchestrator/phases/<name>/` MUST exist and MUST contain at least `goals.md` and `progress.json` (no other files).

#### Scenario: goals.md content from CLI arguments
- **WHEN** `<goals>` are supplied
- **THEN** `goals.md` MUST contain a top-level `# Goals` heading followed by one bullet per supplied goal, in argument order, plus a trailing blank line.

#### Scenario: goals.md TBD stub
- **WHEN** no `<goals>` are supplied
- **THEN** `goals.md` MUST contain a top-level `# Goals` heading followed by a single line: `<!-- TBD: enumerate goals before /kbd-assess -->`.

#### Scenario: progress.json field set
- **WHEN** `progress.json` is written
- **THEN** the JSON object MUST contain the keys `phase` (set to `<name>`), `parentPhase` (null), `childPhases` (empty array), `childPointer` (null), `assessment_complete` / `plan_complete` / `execute_complete` / `reflect_complete` (all false), `changes_total` (0), `changes_completed` (0), `completed_changes` (empty array), `active_change` (null), `blocked_changes` (empty array), `sourceTool` (the value from the prior `current-waypoint.json` or the literal `"unknown"`), `createdBy` (the literal `"kbd-new-phase"`), and `updatedAt` (an ISO-8601 UTC timestamp).

#### Scenario: Atomic write
- **WHEN** the skill writes `progress.json` or `goals.md`
- **THEN** each file MUST be written whole (write to a temp file in the same directory, then rename) so a partial file is never observable.

### Requirement: Waypoint Flip
The skill SHALL update `.kbd-orchestrator/current-waypoint.json` to point at the new phase, preserving the previous phase as `previousPhase`.

#### Scenario: Waypoint fields updated
- **WHEN** the skill runs to completion against an existing waypoint
- **THEN** `current-waypoint.json` MUST be updated with `previousPhase` set to the prior value of `phase`, `phase` set to `<name>`, `change` set to null, `status` set to `"assessment_ready"`, `currentTask` set to the literal `"run kbd-assess for <name>"`, `nextPendingChange` set to null, `exactNextCommand` set to `/kbd-assess <name>`, `parentPhase` to null, `childPhases` to the empty array, `childPointer` to null, and `updatedAt` set to the current ISO-8601 UTC timestamp.

#### Scenario: First waypoint write (file did not previously exist)
- **WHEN** `current-waypoint.json` does not exist at invocation time
- **THEN** the skill MUST create the file with the same field set as above; `previousPhase` MUST be null.

#### Scenario: Existing waypoint is malformed JSON
- **WHEN** `current-waypoint.json` exists but is not valid JSON
- **THEN** the skill MUST exit non-zero without modifying any other file, naming the offending path; the operator must repair the waypoint by hand before retrying.

### Requirement: project.json activePhase Flip
The skill SHALL update `.kbd-orchestrator/project.json` to set `activePhase` to the new phase name.

#### Scenario: project.json updated when present
- **WHEN** `.kbd-orchestrator/project.json` exists
- **THEN** the skill MUST set `activePhase` to `<name>` and `updatedAt` to the current ISO-8601 UTC timestamp; no other fields MUST change.

#### Scenario: project.json absent
- **WHEN** `.kbd-orchestrator/project.json` does not exist (fresh project before `/kbd-init`)
- **THEN** the skill MUST proceed without error and MUST emit a warning advising the operator to run `/kbd-init`.

### Requirement: Hook Fire
The skill SHALL fire exactly one `phase:before` event for the new phase via the shared `kbd_hooks_fire` helper.

#### Scenario: phase:before fires once
- **WHEN** the skill runs to completion
- **THEN** the hook dispatcher MUST observe exactly one `phase:before` fire with `KBD_HOOK_NAME = <name>`, `KBD_HOOK_INDEX = 1`, `KBD_HOOK_TOTAL = 1`.

#### Scenario: Hook fire ordering
- **WHEN** the skill writes the new phase
- **THEN** the `phase:before` fire MUST occur after both the waypoint flip and the project.json flip, and before the skill's `Completed kbd-new-phase` Progress Signal — so that any hook reading state sees the new phase as authoritative.

#### Scenario: Hook subsystem unavailable
- **WHEN** `shared/lib/hooks.sh` cannot be sourced (e.g. the file is missing or `KBD_ORCHESTRATOR_ROOT` is unset)
- **THEN** the skill MUST proceed without firing the hook, MUST emit a single stderr warning naming the failure, and MUST NOT abort the phase creation; the new phase MUST exist on disk.

### Requirement: Progress Signals
The skill SHALL emit the canonical Progress Signal lines per the orchestrator's "Progress Signals (MANDATORY)" convention.

#### Scenario: Start signal
- **WHEN** the skill begins execution
- **THEN** it MUST emit exactly one line: `Starting kbd-new-phase — <name>` to plain response text (no tool call).

#### Scenario: Completion signal
- **WHEN** the skill completes successfully
- **THEN** it MUST emit exactly one line: `Completed kbd-new-phase — <name> ready for /kbd-assess` to plain response text.

#### Scenario: Failure (refused name, collision, malformed waypoint)
- **WHEN** the skill exits non-zero for any documented failure reason
- **THEN** it MUST NOT emit the completion signal; it MUST emit a single stderr line describing the failure and the suggested remediation.

### Requirement: Confirmation Banner
The skill SHALL emit a human-readable confirmation banner naming the phase, the path of `goals.md`, and the exact next command.

#### Scenario: Banner contents
- **WHEN** the skill completes successfully
- **THEN** the banner emitted to plain response text MUST contain the phase name, the absolute or repo-relative path of `goals.md`, and the literal next-step instruction `Next: /kbd-assess <name>`.

### Requirement: Idempotency Boundary
The skill SHALL NOT modify any existing phase directory and SHALL NOT roll back changes on partial failure.

#### Scenario: No retroactive cleanup on hook failure
- **WHEN** the phase directory and waypoint flip have completed but the hook fire fails
- **THEN** the skill MUST leave the new phase in place; the operator can re-trigger the hook later via `/kbd-status` or similar without re-creating the phase.

#### Scenario: No silent re-init
- **WHEN** the skill is invoked with a name that matches an existing phase directory
- **THEN** the skill MUST refuse (see "Argument Parsing and Validation" → name-collision scenario); it MUST NOT regenerate `goals.md` or reset `progress.json` for an existing phase.
