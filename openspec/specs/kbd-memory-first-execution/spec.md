# kbd-memory-first-execution Specification

## Purpose

Default-on surreal-memory integration that mirrors every KBD hook fire into a queryable entity store, provides `/kbd-memory-recall` for retrieving prior similar work as planning input, and defines the canonical `kbd_lifecycle_event` entity schema + retention policy for cross-project learning. Degrades gracefully when the endpoint is unreachable.

## Requirements

### Requirement: Memory Availability Detection
The orchestrator SHALL ship a `shared/lib/memory.sh` helper exposing `kbd_memory_available()` that returns 0 when a surreal-memory endpoint is reachable and 1 otherwise.

#### Scenario: MCP tool detection
- **WHEN** the calling agent's tool list contains `create_entity` (the surreal-memory MCP tool)
- **THEN** `kbd_memory_available` MUST return 0 without making any network call.

#### Scenario: Environment-variable endpoint
- **WHEN** `UAR_MEMORY_MCP_URL` or `KBD_MEMORY_MCP_URL` is set and a `GET <url>/healthz` probe succeeds with a 2xx response within 2 seconds
- **THEN** `kbd_memory_available` MUST return 0.

#### Scenario: Config-file endpoint
- **WHEN** `.kbd-orchestrator/memory.config.json` exists with a `mcpEndpoint` field and a probe to that endpoint succeeds
- **THEN** `kbd_memory_available` MUST return 0.

#### Scenario: No detection method succeeds
- **WHEN** no detection method succeeds within the probe timeout
- **THEN** `kbd_memory_available` MUST return 1 and SHALL NOT throw or block; callers MUST treat the result as advisory and degrade gracefully.

#### Scenario: Result caching
- **WHEN** `kbd_memory_available` is called multiple times in the same shell process
- **THEN** subsequent calls MUST return the cached result without re-probing.

### Requirement: kbd-memory-log Hook
The orchestrator's built-in `hooks/hooks.json` SHALL register a `kbd-memory-log` augment hook covering `*:*` that mirrors every hook fire into surreal-memory as a structured observation.

#### Scenario: Hook registration
- **WHEN** `hooks/hooks.json` is loaded
- **THEN** it MUST contain exactly one entry with `id: "kbd-memory-log"`, `event: "*:*"`, `mode: "augment"`, `on_failure: "ignore"`.

#### Scenario: Observation shape
- **WHEN** the hook fires
- **THEN** the observation written to surreal-memory MUST be a single entity with `entityType: "kbd_lifecycle_event"`, an `entityId` containing project + phase + kind + edge + index + timestamp, and observations carrying every `KBD_HOOK_*` value supplied by the dispatcher.

#### Scenario: Relations
- **WHEN** the observation is written
- **THEN** it MUST include at least two relations: one labeled `fires-in` pointing to `phase:<phase>`, and one labeled `belongs-to` pointing to `project:<project>`.

#### Scenario: Memory unavailable
- **WHEN** `kbd_memory_available` returns non-zero at the time the hook fires
- **THEN** the hook MUST exit 0 (no-op) without writing anything; the dispatch loop MUST continue.

#### Scenario: No leakage of third-party stderr
- **WHEN** the hook captures payload
- **THEN** it MUST include only structured `KBD_HOOK_*` env values; it MUST NOT include the stderr stream of the dispatched hook command (which may contain secrets from third-party hooks).

### Requirement: /kbd-memory-recall Skill
The orchestrator SHALL ship a `/kbd-memory-recall` skill that queries surreal-memory for prior similar KBD work and writes a digest to the active phase's directory.

#### Scenario: Skill files exist
- **WHEN** the orchestrator skill set is inspected after this change
- **THEN** `skills/process/kbd-process-orchestrator/skills/kbd-memory-recall/SKILL.md` and `skills/kbd-memory-recall/kbd-memory-recall.sh` MUST exist.

#### Scenario: Digest path
- **WHEN** `/kbd-memory-recall <phase>` runs successfully
- **THEN** it MUST write `.kbd-orchestrator/phases/<phase>/prior-context.md` containing a "Prior context" section with 3–5 most-relevant prior entries plus a "Patterns observed" section.

#### Scenario: Defaults to active phase
- **WHEN** the skill is invoked with no argument
- **THEN** it MUST resolve `<phase>` from `current-waypoint.json` and target that phase's directory.

#### Scenario: Memory unavailable graceful path
- **WHEN** `kbd_memory_available` returns non-zero
- **THEN** the skill MUST emit a stderr warning naming the missing endpoint and MUST write `prior-context.md` containing a single line: `<!-- memory endpoint unreachable; no prior context retrieved -->`. The skill MUST exit 0 so it composes with `on_failure: ignore`.

#### Scenario: Auto-invocation on assess:before
- **WHEN** `/kbd-assess` runs and `assess:before` fires
- **THEN** the dispatcher MUST trigger an `auto-memory-recall` augment hook that invokes the recall skill with the active phase name; failure of the recall skill MUST NOT block the assess workflow.

### Requirement: Event Entity Schema
The orchestrator SHALL document a canonical entity schema for KBD lifecycle events in `shared/references/memory-retention.md`.

#### Scenario: Schema documented
- **WHEN** `shared/references/memory-retention.md` is read
- **THEN** it MUST contain a schema definition for `entityType: "kbd_lifecycle_event"` listing every observation key (kind, edge, name, index, total, phasePath, sourceTool, project, ts) with type and meaning.

#### Scenario: Retention policy documented
- **WHEN** the same reference file is read
- **THEN** it MUST state a default retention window (365 days for lifecycle events) and a relevance ordering for recall queries (project > kind > phase-name-pattern).

### Requirement: Default-On Promotion
The orchestrator's `SKILL.md` SHALL reframe surreal-memory integration from "optional" to "default-on when reachable".

#### Scenario: SKILL.md wording
- **WHEN** the "Surreal-Memory Integration" section is read after this change
- **THEN** it MUST state that integration is active by default whenever `kbd_memory_available` returns 0, MUST name the `kbd-memory-log` hook and the `/kbd-memory-recall` skill, and MUST cross-reference `memory-retention.md`.

### Requirement: Graceful Degradation
The integration MUST never block any KBD operation when the memory endpoint is unreachable.

#### Scenario: Slow endpoint
- **WHEN** the memory endpoint takes longer than the documented probe timeout to respond
- **THEN** `kbd_memory_available` MUST return 1 and the dependent hooks/skills MUST behave as in the "memory unavailable" scenarios.

#### Scenario: Endpoint returns error
- **WHEN** the memory endpoint returns a 5xx error during write
- **THEN** the writer MUST log a single stderr warning, MUST NOT retry within the same hook fire, and MUST continue the dispatch loop without aborting the calling skill.
