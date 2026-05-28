## ADDED Requirements

### Requirement: Skill Surface
The orchestrator SHALL ship two skills, `/kbd-new-child` and `/kbd-next-child`, each as a directory under `skills/process/kbd-process-orchestrator/skills/` containing a `SKILL.md` and an executable helper script.

#### Scenario: kbd-new-child files
- **WHEN** the skill set is inspected after this change lands
- **THEN** `skills/kbd-new-child/SKILL.md` and `skills/kbd-new-child/kbd-new-child.sh` MUST exist; the `.sh` MUST be executable.

#### Scenario: kbd-next-child files
- **WHEN** the skill set is inspected after this change lands
- **THEN** `skills/kbd-next-child/SKILL.md` and `skills/kbd-next-child/kbd-next-child.sh` MUST exist; the `.sh` MUST be executable.

#### Scenario: Orchestrator documentation
- **WHEN** `kbd-process-orchestrator/SKILL.md` "Quick Start Commands" / per-skill list is read after this change
- **THEN** it MUST list both `/kbd-new-child` and `/kbd-next-child` alongside `/kbd-new-phase` and `/kbd-next-phase`.

### Requirement: kbd-new-child Behavior
`/kbd-new-child <child-name> [goals…]` SHALL create a child phase owned by the currently-active top-level phase.

#### Scenario: Happy path
- **WHEN** invoked as `/kbd-new-child my-child "goal 1" "goal 2"` with an active top-level phase named `<parent>`
- **THEN** the skill MUST create `.kbd-orchestrator/phases/<parent>/children/my-child/` containing `goals.md` (with the supplied goals as bullets, or a TBD stub when none) and `progress.json` (canonical field set with `parentPhase: "<parent>"`, `childPhases: []`, `childPointer: null`).

#### Scenario: Waypoint update
- **WHEN** the child has been written to disk
- **THEN** the skill MUST atomically update `.kbd-orchestrator/current-waypoint.json` to append the child's name to `childPhases[]` (dedup-safe), set `childPointer` to the new child's name, set `currentTask` to `"run kbd-assess for <parent>/<child>"`, and set `exactNextCommand` to `/kbd-assess` scoped to the child.

#### Scenario: No active phase
- **WHEN** invoked while `current-waypoint.json` is missing or `phase` is empty
- **THEN** the skill MUST exit non-zero with a message naming the precondition and suggesting `/kbd-new-phase` first.

#### Scenario: Child name validation
- **WHEN** invoked with a name that contains uppercase letters, parent traversal (`..`), slashes, or fails the kebab-case regex
- **THEN** the skill MUST exit non-zero without modifying any file, naming the offending character class.

#### Scenario: Duplicate child name
- **WHEN** invoked with a name already present in `childPhases[]`
- **THEN** the skill MUST exit non-zero, name the existing entry, and suggest `/kbd-next-child <name>` to jump to it instead.

#### Scenario: Hook fire
- **WHEN** the skill runs to completion
- **THEN** the hooks dispatcher MUST observe exactly one `child:before` fire with `KBD_HOOK_NAME = <child-name>`, `KBD_HOOK_INDEX = <1-based child index>`, `KBD_HOOK_TOTAL = <total children after add>`.

### Requirement: kbd-next-child Behavior
`/kbd-next-child [<child-name>]` SHALL advance `childPointer` to a later child in `childPhases[]`.

#### Scenario: Implicit advance
- **WHEN** invoked with no argument while `childPointer` references a non-final child
- **THEN** the skill MUST set `childPointer` to the entry immediately after the current pointer in `childPhases[]`.

#### Scenario: Explicit jump
- **WHEN** invoked as `/kbd-next-child <child-name>` and `<child-name>` is present in `childPhases[]`
- **THEN** the skill MUST set `childPointer` to that name regardless of current position.

#### Scenario: Hook fires
- **WHEN** the pointer transitions from a non-null prior value `A` to a new value `B`
- **THEN** the dispatcher MUST observe `child:after` for `A` followed by `child:before` for `B`, both with the correct `index/total` derived from `childPhases[]`.

#### Scenario: No children defined
- **WHEN** invoked while `childPhases` is empty
- **THEN** the skill MUST exit non-zero with a hint to run `/kbd-new-child` first; no state changes.

#### Scenario: Already at last child
- **WHEN** invoked with no argument while `childPointer` references the final entry of `childPhases[]`
- **THEN** the skill MUST exit non-zero with a hint to run `/kbd-reflect` then `/kbd-next-phase`; no state changes.

#### Scenario: Explicit jump to unknown name
- **WHEN** invoked as `/kbd-next-child <name>` and `<name>` is not in `childPhases[]`
- **THEN** the skill MUST exit non-zero, name the unknown value, and list the available children; no state changes.

### Requirement: Cross-Field Invariants Enforced on Write
Both skills SHALL refuse to write a waypoint that violates the invariants documented in `kbd-nested-phase-schema`.

#### Scenario: childPointer always in childPhases
- **WHEN** either skill is about to write `current-waypoint.json`
- **THEN** the proposed `childPointer`, if non-null, MUST be a member of the proposed `childPhases[]` array; otherwise the skill MUST exit non-zero before writing.

#### Scenario: childPhases never has duplicates
- **WHEN** either skill is about to write `current-waypoint.json`
- **THEN** the proposed `childPhases[]` array MUST have no duplicate string members; otherwise the skill MUST exit non-zero.

### Requirement: Atomic Writes
Both skills SHALL write `goals.md`, `progress.json`, and the waypoint JSON via temp-file + `mv` so partial writes are never observable.

#### Scenario: Atomic file production
- **WHEN** either skill writes any of those files
- **THEN** the write MUST land via a same-directory temp file followed by `mv -f`; an interrupted skill MUST leave the original file intact.

### Requirement: Progress Signals
Each skill SHALL emit canonical Progress Signal lines.

#### Scenario: kbd-new-child signals
- **WHEN** `/kbd-new-child` runs
- **THEN** it MUST emit `Starting kbd-new-child — <parent>/<child>` at start and `Completed kbd-new-child — <parent>/<child> ready for /kbd-assess` on success.

#### Scenario: kbd-next-child signals
- **WHEN** `/kbd-next-child` runs
- **THEN** it MUST emit `Starting kbd-next-child — <parent>/<from> → <to>` at start and `Completed kbd-next-child — now on <parent>/<to>` on success; the `<from>` slot is `(none)` when no prior child was active.
