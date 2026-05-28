## ADDED Requirements

### Requirement: Canonical Event Form
The hooks subsystem SHALL accept events declared in the form `<kind>:<edge>`, where `kind` is one of `phase | child | plan | execute | reflect | task | assess | *` and `edge` is one of `before | after | *`.

#### Scenario: Hook declares a typed event
- **WHEN** a hook entry's `event` field is exactly `"phase:before"`, `"task:after"`, `"reflect:*"`, or any other valid `<kind>:<edge>` string
- **THEN** the dispatcher MUST recognize the entry as a typed hook and route it to the matching dispatch point.

#### Scenario: Hook declares a wildcard event
- **WHEN** a hook entry's `event` field is exactly `"*:*"`
- **THEN** the dispatcher MUST fire that hook on every dispatch point.

#### Scenario: Invalid event
- **WHEN** a hook entry's `event` field does not match `<known-kind>:<edge>` or a known legacy alias (see "Legacy Alias Compatibility")
- **THEN** the dispatcher MUST log a warning naming the offending hook id and event string, MUST NOT register the hook, and MUST NOT abort the dispatch.

### Requirement: Legacy Alias Compatibility
The hooks subsystem SHALL keep the existing snake_case event names working as aliases for the canonical form, so projects that ship pre-existing `hooks-config.json` files continue to function unchanged.

#### Scenario: on_phase_complete alias
- **WHEN** a hook entry's `event` is `"on_phase_complete"`
- **THEN** the dispatcher MUST treat it as if it were `"phase:after"`.

#### Scenario: on_plan_complete alias
- **WHEN** a hook entry's `event` is `"on_plan_complete"`
- **THEN** the dispatcher MUST treat it as if it were `"plan:after"`.

#### Scenario: on_reflection_complete alias
- **WHEN** a hook entry's `event` is `"on_reflection_complete"`
- **THEN** the dispatcher MUST treat it as if it were `"reflect:after"`.

#### Scenario: on_assessment_complete alias
- **WHEN** a hook entry's `event` is `"on_assessment_complete"`
- **THEN** the dispatcher MUST treat it as if it were `"assess:after"`.

#### Scenario: on_change_complete alias
- **WHEN** a hook entry's `event` is `"on_change_complete"`
- **THEN** the dispatcher MUST treat it as if it were the *final* `"task:after"` fire of the change's execute loop (i.e. it fires once per archived change, after that change's last task).

#### Scenario: Situational events keep their names
- **WHEN** a hook entry's `event` is `"on_blocker_detected"` or `"on_cross_tool_handoff"`
- **THEN** the dispatcher MUST keep those event names unchanged; they are not lifecycle boundaries and are not mapped to a `<kind>:<edge>` form.

### Requirement: Discovery Order
The hooks subsystem SHALL load hooks from three layers, in this order, and concatenate them into a single registration list.

#### Scenario: Built-in layer
- **WHEN** the dispatcher initializes
- **THEN** it MUST first read `~/.claude/skills/kbd-process-orchestrator/hooks/hooks.json`; entries are tagged with `layer: "builtin"`.

#### Scenario: User layer
- **WHEN** the dispatcher initializes
- **THEN** it MUST next read `~/.claude/skills/kbd-process-orchestrator/hooks/user.json` if the file exists; entries are tagged with `layer: "user"`.

#### Scenario: Project layer
- **WHEN** the dispatcher initializes inside a project containing `.kbd-orchestrator/hooks-config.json`
- **THEN** it MUST last read that file; entries are tagged with `layer: "project"`.

#### Scenario: Missing layers are silent
- **WHEN** any of the three layer files does not exist
- **THEN** the dispatcher MUST treat the missing file as the empty list `[]` and MUST NOT warn.

### Requirement: Augment vs Override Modes
Each hook entry SHALL declare an optional `mode` field with values `augment` (default) or `override`, and the dispatcher SHALL resolve overrides using a documented precedence rule.

#### Scenario: Default mode is augment
- **WHEN** a hook entry omits the `mode` field
- **THEN** the dispatcher MUST treat the entry as `mode: "augment"`.

#### Scenario: Augment hooks all fire
- **WHEN** zero or more `mode: "augment"` entries match a dispatch point
- **THEN** the dispatcher MUST fire every matching augment entry in registration order.

#### Scenario: Single override replaces built-in default
- **WHEN** exactly one `mode: "override"` entry matches a dispatch point
- **THEN** the dispatcher MUST suppress the built-in default reporter for that dispatch point and fire the override entry instead; augment entries still fire.

#### Scenario: Multiple overrides resolve by layer precedence
- **WHEN** more than one `mode: "override"` entry matches a single dispatch point
- **THEN** the dispatcher MUST fire only the entry from the highest layer (project > user > builtin), MUST emit a single warning naming both override sources, and MUST NOT fire the suppressed overrides.

### Requirement: Hook Context Payload
The dispatcher SHALL pass a uniform context payload to every hook invocation, exposed as environment variables, in addition to the existing substitution variables documented in the hooks schema.

#### Scenario: KBD_HOOK_* variables present on every fire
- **WHEN** the dispatcher invokes a hook command
- **THEN** the spawned process environment MUST contain the variables `KBD_HOOK_KIND`, `KBD_HOOK_EDGE`, `KBD_HOOK_NAME`, `KBD_HOOK_INDEX`, `KBD_HOOK_TOTAL`, `KBD_HOOK_PHASE_PATH`, `KBD_HOOK_CHILD_PATH`, `KBD_HOOK_SOURCE_TOOL`, and `KBD_HOOK_STARTED_AT` with values matching this fire's context.

#### Scenario: Index and total default to 1
- **WHEN** a dispatch point has no containing loop (e.g. a `phase:before` fire that owns no parent loop)
- **THEN** the dispatcher MUST set `KBD_HOOK_INDEX=1` and `KBD_HOOK_TOTAL=1`.

#### Scenario: phasePath uses the chain separator
- **WHEN** the active waypoint has populated `parentPhase` and/or `childPointer`
- **THEN** `KBD_HOOK_PHASE_PATH` MUST be rendered using the same `chain_separator` helper used by `kbd-status` (`›` by default, ` > ` under `LC_ALL=POSIX`).

#### Scenario: Source tool propagation
- **WHEN** the waypoint declares a `sourceTool` value
- **THEN** `KBD_HOOK_SOURCE_TOOL` MUST be that value; if absent, it MUST be the literal string `"unknown"`.

### Requirement: Default report-progress Reporter
The built-in `hooks.json` SHALL register exactly one default reporter, identified by `id: "report-progress"`, matching `event: "*:*"` and `mode: "augment"`, that emits a uniform progress line to stderr.

#### Scenario: Before edge format
- **WHEN** the default reporter fires on a `<kind>:before` dispatch
- **THEN** it MUST write exactly one line to stderr in the form `starting <kind> <name> [<index>/<total>]` followed by a newline.

#### Scenario: After edge format
- **WHEN** the default reporter fires on a `<kind>:after` dispatch
- **THEN** it MUST write exactly one line to stderr in the form `ending <kind> <name> [<index>/<total>]` followed by a newline.

#### Scenario: Reporter does not block
- **WHEN** the default reporter is invoked
- **THEN** the invocation MUST complete in less than the schema's default `timeout` (15 s) under normal conditions; if it does exceed, `on_failure` semantics apply but the dispatch loop MUST continue.

#### Scenario: Override suppresses default
- **WHEN** a project supplies a `mode: "override"` entry covering the same dispatch point as the default reporter
- **THEN** the default reporter MUST NOT fire for that dispatch point; the override is solely responsible for any progress output.

### Requirement: Per-Skill Wiring
Every KBD skill that owns a lifecycle loop SHALL emit `<kind>:before` at the start of its work and `<kind>:after` at the end, using the dispatcher helper, before emitting its existing `Progress Signals (MANDATORY)` lines.

#### Scenario: kbd-assess emits assess events
- **WHEN** `/kbd-assess` is invoked
- **THEN** it MUST fire `assess:before` before any other action and `assess:after` after writing `assessment.md`.

#### Scenario: kbd-plan emits plan events
- **WHEN** `/kbd-plan` is invoked
- **THEN** it MUST fire `plan:before` before reading the assessment and `plan:after` after writing `plan.md`.

#### Scenario: kbd-execute emits execute events
- **WHEN** `/kbd-execute` is invoked
- **THEN** it MUST fire `execute:before` before selecting a backend and `execute:after` after writing `execution.md`.

#### Scenario: kbd-reflect emits reflect events and closes the phase
- **WHEN** `/kbd-reflect` completes
- **THEN** it MUST fire `reflect:after` and immediately after MUST fire `phase:after` for the closing phase.

#### Scenario: Per-task fire inside execute
- **WHEN** `/kbd-execute` (or `/opsx:apply`) advances from one OpenSpec task to the next
- **THEN** it MUST fire `task:after` for the just-finished task and `task:before` for the new task; `KBD_HOOK_INDEX` MUST be the task number within the change's `tasks.md`, and `KBD_HOOK_TOTAL` MUST be the total task count of the change.

#### Scenario: Phase bracket fires
- **WHEN** a phase is created by `/kbd-new-phase` or `/kbd-next-phase`
- **THEN** the creating skill MUST fire `phase:before` for the new phase exactly once; `phase:after` is the responsibility of `/kbd-reflect`.

#### Scenario: Child bracket fires (forward-compatible)
- **WHEN** the future `/kbd-new-child` or `/kbd-next-child` skill creates or advances a child phase
- **THEN** the creating skill MUST fire `child:before` for the new child and `child:after` for the closing child; this requirement is forward-looking and MUST be satisfied by the implementation of those skills (changes 5).

### Requirement: Hook Log
The dispatcher SHALL persist every hook fire to a JSONL log inside the active phase directory.

#### Scenario: Log path
- **WHEN** a hook fires in the context of an active phase `<phase>`
- **THEN** the dispatcher MUST append one JSON object as a single line to `.kbd-orchestrator/phases/<phase>/hooks.log.jsonl`.

#### Scenario: Log entry shape
- **WHEN** a log entry is appended
- **THEN** the entry MUST be a single JSON object containing the keys `ts` (ISO-8601 UTC), `kind`, `edge`, `name`, `index`, `total`, `phasePath`, `sourceTool`, `hookId`, `layer`, `mode`, and `status` (exit code of the hook command, integer).

#### Scenario: Failure capture
- **WHEN** a hook command exits non-zero
- **THEN** the log entry MUST include a `stderrSnippet` field containing the captured stderr truncated to 200 characters.

#### Scenario: No active phase
- **WHEN** a hook fires before any phase directory exists (e.g. during `/kbd-init`)
- **THEN** the log entry MUST be written to `.kbd-orchestrator/hooks.log.jsonl` at the project root instead.

### Requirement: Schema Extensions
The `references/schemas/hooks-config.schema.json` file SHALL document the new `mode` field and the new event kinds.

#### Scenario: mode field declared
- **WHEN** the schema is loaded
- **THEN** the per-entry `properties` object MUST include `mode` with `enum: ["augment", "override"]` and `default: "augment"`.

#### Scenario: Event kinds documented
- **WHEN** the schema is loaded
- **THEN** the `event` field's description MUST enumerate the kinds `phase | child | plan | execute | reflect | task | assess | *` and the edges `before | after | *`, and MUST note that legacy `on_*` snake_case names are accepted aliases.

### Requirement: Coexistence with Progress Signals
The hooks subsystem SHALL NOT remove or replace the existing `Progress Signals (MANDATORY)` lines emitted by each KBD skill; hook output is complementary to those lines.

#### Scenario: Skill emits both
- **WHEN** any KBD skill runs end-to-end
- **THEN** the run MUST emit both the skill's existing `Starting kbd-<skill> — <phase>` / `Completed kbd-<skill> — <phase>` lines (their canonical agent-facing format) and the new `starting/ending <kind> <name> [<i>/<n>]` reporter lines; neither set is allowed to suppress the other.

#### Scenario: Output streams are distinct
- **WHEN** both line sets are emitted
- **THEN** the existing `Starting/Completed` lines MUST continue to be emitted to the same stream they use today (plain response text), and the hook reporter lines MUST be emitted to stderr — they are independently captureable.
