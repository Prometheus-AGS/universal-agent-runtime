# turn-assembly-kernel Specification

## Purpose

Define immutable turn and step assembly, staged contributors, shadow parity and evidence-gated defaults.

## Requirements

### Requirement: Each turn and each model step is an immutable snapshot
The runtime SHALL resolve a turn into an immutable record of policy, artifact, environment, credentials, and prompt fragments, and SHALL resolve each model call into an immutable step carrying the settings, projected tool set, token budget, and MCP catalog used for that call.

#### Scenario: Tool set changes between steps
- **WHEN** a skill is activated or a deferred tool is surfaced during step N
- **THEN** step N+1's projected tool set includes it and step N's recorded tool set is unchanged

### Requirement: Turn assembly is composed from staged contributors
The runtime SHALL assemble a turn through a registry of contributors in fixed stages: artifact instructions, effective policy, memory and retrieval, skills, MCP and tools, context, and lifecycle observation. Contributors SHALL return owned data and SHALL NOT broaden the effective run policy or bypass governance.

#### Scenario: Contributor attempts to widen access
- **WHEN** a contributor returns a tool or skill outside the effective policy
- **THEN** assembly rejects it with a typed error and the run does not start

#### Scenario: Memory reaches every entry point
- **WHEN** a run is started through any runtime entry point with memory hits available
- **THEN** the memory contribution is present in the resolved turn

### Requirement: Typed assembly runs in shadow beside the legacy path
The runtime SHALL support `legacy`, `shadow`, and `typed` harness modes, where `legacy` is the run path as it exists after the context, tool, prompt, skill, and resiliency changes have merged. In `shadow` mode it SHALL render both the legacy and typed requests, compare fragment hashes, ordering, tool eligibility, and context counts, classify each difference as intentional (present in a checked-in allowlist that names the change introducing it) or unexpected, record both in the turn manifest, and dispatch only the legacy request. During the initial typed-turn-assembly migration, the default remained `legacy`. The current default is governed by "Typed assembly becomes the default only on recorded evidence" below; explicit `legacy` rollback and opt-in `shadow` remain supported.

#### Scenario: Shadow parity report
- **WHEN** the parity corpus runs in `shadow` mode
- **THEN** a parity report records, per request, the count of unexpected differences and the allowlisted differences observed, and only legacy requests were dispatched

#### Scenario: Intentional delta
- **WHEN** the typed path re-projects the tool set after a mid-run skill activation and the legacy path does not
- **THEN** the difference is classified as intentional under the allowlist entry naming `progressive-skill-runtime`, not as unexpected

#### Scenario: Operator selects typed explicitly
- **WHEN** an operator sets `mode: typed` before the default changes
- **THEN** the typed request is dispatched and the legacy path is not rendered

### Requirement: Typed assembly becomes the default only on recorded evidence
The runtime's default harness mode SHALL change from `legacy` to `typed` only after a checked-in parity report shows zero unexpected differences over the parity corpus and a live smoke run in `shadow` mode shows zero unexpected differences, both recorded in the project decision log with the corpus size and smoke set contents, and `legacy` SHALL remain selectable for one minor release after the change.

#### Scenario: Evidence present
- **WHEN** the parity report and the live smoke record both show zero unexpected differences
- **THEN** a fresh installation uses `typed` by default and `mode: legacy` still selects the legacy path

#### Scenario: Evidence absent
- **WHEN** either record is missing or shows an unexpected difference
- **THEN** the default remains `legacy` and the change is not merged
