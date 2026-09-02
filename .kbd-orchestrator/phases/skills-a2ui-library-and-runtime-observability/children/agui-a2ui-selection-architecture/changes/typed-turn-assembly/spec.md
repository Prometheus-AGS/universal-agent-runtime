<!-- mirror of openspec/changes/typed-turn-assembly/proposal.md and specs/*/spec.md -->
# typed-turn-assembly

Rank 6 of the codex-harness-comparative-analysis change set; first of the structural changes. Source: gap G6 in the phase `analysis.md`.

## Why

`start_run_with_policy_and_history` is one function of about 1,510 lines (`src/uar/runtime/manager.rs:1094-2604`) that sequences prompt, RAG, skills, MCP, policy, context, credentials, failover, and approval imperatively. The tool list is frozen before the loop (`src/llm/orchestrator.rs:498-512`, `:601`), skills are matched once against the first input (`manager.rs:1334`, `:1360`), and no extension surface can add a prompt section, observe a turn, or change sequencing (`src/uar/runtime/wasm/plugin_loader.rs:108-109` has no implementors). Memory is assembled outside the function entirely (`src/server.rs:4941-4947`).

Codex freezes a `TurnContext` per turn and a `StepContext` per sampling request so that context, advertised tools, and executed tools share one view (`core/src/session/turn_context.rs:194-246`; `core/src/session/step_context.rs:15-34`; `core/src/session/turn.rs:344-367`), and lets extensions contribute through typed traits (`ext/extension-api/src/contributors.rs:77-380`). The design is ported; the code is not. Codex paths are outside this repository; verbatim excerpts for the cited lines are in the phase `analysis.md` appendix "verified Codex excerpts" and in `evidence/codex-*.md` under the phase directory.

Changes 1 through 5 each extract a pure function from the monolith. This change composes them.

## What changes

- `RunExecutionRequest` replaces the positional entry point internally; `start_run*` methods and `POST /api/uar/runs` decoding become adapters.
- `ContributorRegistry` with fixed stages: artifact instructions, effective policy, memory and RAG, skills, MCP and tools, context, lifecycle observation. Contributors are internal Rust traits, return owned data, and cannot broaden `EffectiveRunPolicy` or bypass Cedar.
- `TurnAssemblyPlan` (pre-I/O decisions), immutable `ResolvedTurn` (policy, artifact, environment, credentials, fragments) and per-model-call `ResolvedStep` (settings, projected tool set, token budget, MCP catalog). The tool set and active skills are re-projected per step, so `activate_skill` and deferred tools take effect on the next call.
- `HarnessConfig.mode: legacy | shadow | typed`. "Legacy" means the run path as it exists after changes 1 through 5 have merged, which already has deterministic ordering, validated tools, and a single history path; this change does not compare against the pre-series code. Shadow renders both paths, compares fragment hashes, ordering, tool eligibility, and context counts, records every difference in the turn manifest, and sends only the legacy request. Differences that the typed path introduces on purpose (per-step re-projection of tools after `activate_skill`, per-step MCP catalog capture) are declared in an intentional-delta allowlist checked into the parity corpus; parity means zero differences outside that allowlist. The default stays `legacy` in this change. Flipping the default to `typed` is the separate change `typed-turn-default-flip`, gated on recorded parity evidence.
- Memory contribution moves inside the assembler; a direct `start_run` caller gets memory.

## Scope

- `src/uar/runtime/manager.rs` (the whole entry function, refactored around the extracted functions)
- `src/llm/orchestrator.rs` (per-step tool set input)
- `src/server.rs` (memory prepend at `:4892-4947` removed in favor of the contributor)
- `src/config.rs` (`HarnessConfig`)
- new: `src/uar/runtime/turn/{request.rs,contributors.rs,plan.rs,resolved.rs,shadow.rs}`
- tests: `tests/typed_turn_assembly.rs`, `tests/turn_shadow_parity.rs`

Out of scope: a public plugin ABI (WIT work stays separate); MCP lifecycle (projected-mcp-runtime); subagent threads (thread-native-subagents).

## Dependencies

context-history-integrity, fail-closed-tool-arguments, deterministic-prompt-assembly, progressive-skill-runtime, model-path-resiliency. Each supplies a pure function this change composes; without them there is nothing to shadow.

## Verification

Tier 0 per edit; Tier 1 the new tests; Tier 2 at the boundary, plus the shadow-parity suite producing a parity report over the corpus. The report is this change's output; acting on it belongs to `typed-turn-default-flip`.

## The uncomfortable thing

Shadow mode doubles assembly cost per turn for as long as it is on, and its parity report is only as good as the test corpus it runs over. The intentional-delta allowlist is a place where a real regression could be waved through as "intentional"; every allowlist entry must name the dependency change that introduces it.


## Spec delta: turn-assembly-kernel

## ADDED Requirements

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
The runtime SHALL support `legacy`, `shadow`, and `typed` harness modes, where `legacy` is the run path as it exists after the context, tool, prompt, skill, and resiliency changes have merged. In `shadow` mode it SHALL render both the legacy and typed requests, compare fragment hashes, ordering, tool eligibility, and context counts, classify each difference as intentional (present in a checked-in allowlist that names the change introducing it) or unexpected, record both in the turn manifest, and dispatch only the legacy request. The default mode SHALL remain `legacy` in this change.

#### Scenario: Shadow parity report
- **WHEN** the parity corpus runs in `shadow` mode
- **THEN** a parity report records, per request, the count of unexpected differences and the allowlisted differences observed, and only legacy requests were dispatched

#### Scenario: Intentional delta
- **WHEN** the typed path re-projects the tool set after a mid-run skill activation and the legacy path does not
- **THEN** the difference is classified as intentional under the allowlist entry naming `progressive-skill-runtime`, not as unexpected

#### Scenario: Operator selects typed explicitly
- **WHEN** an operator sets `mode: typed` before the default changes
- **THEN** the typed request is dispatched and the legacy path is not rendered
