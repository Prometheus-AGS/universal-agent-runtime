<!-- mirror of openspec/changes/context-history-integrity/proposal.md and specs/*/spec.md -->
# context-history-integrity

Rank 1 of the codex-harness-comparative-analysis change set. Source: `.kbd-orchestrator/phases/skills-a2ui-library-and-runtime-observability/children/agui-a2ui-selection-architecture/children/codex-harness-comparative-analysis/{assessment,analysis}.md`, gap G1.

## Why

Two independent context reducers run back to back on every run and neither knows about tool-call pairs or the system message. `trim_count` and `apply_strategy` slice by index (`src/uar/context/strategy.rs:119, 135, 298, 331`), so a window boundary can orphan a `role: tool` result and the provider returns 400. The vector passed to the first reducer has the system message at index 0 (`src/uar/runtime/manager.rs:1471`, `:1517`) and `SlidingWindow` keeps only the tail, so a long conversation drops the agent's identity, the RAG block, and every skill overlay. The second reducer deduplicates identical repeated user messages (`src/uar/runtime/context/manager.rs:193-198`). The two reducers use different tokenizers, `len/4` (`strategy.rs:100-103`) and `cl100k_base` (`src/uar/runtime/context/token_service.rs:9-13`). Tool output enters history unbounded (`src/uar/tools/terminal_exec.rs:75-81`; `src/llm/orchestrator.rs:1010-1021`).

Checkpoint resume is in this change because it is the same defect class, history reconstruction that discards what it was given: `resume_run_from_checkpoint` loads the checkpoint and then starts a new run whose only input is a prose string (`src/uar/api/routes.rs:346-395`), `Checkpoint::restore_state` (`src/uar/runtime/checkpoint.rs:50-57`) has zero callers, and it swallows deserialization failure with `unwrap_or_default`. The assessment recorded this under resiliency; it lands here because the fix touches the same history-seeding seam. Codex has no direct counterpart; its analogue is that compaction and fork both rebuild history from persisted items rather than from prose (`core/src/compact.rs:645-734`).

Codex enforces three history invariants before every request (`core/src/context_manager/history.rs:563-581`; `normalize.rs:21-138`) and truncates tool output middle-out at ingest with a warning header (`history.rs:246-282`; `utils/output-truncation/src/lib.rs:14-31`). Both are provider-neutral and are ported, not depended on (Apache-2.0). Codex paths are outside this repository; verbatim excerpts for the cited lines are in the phase `analysis.md` appendix "verified Codex excerpts".

## What changes

- One `HistoryNormalizer` applied before provider dispatch: every assistant tool call has exactly one tool result, every tool result has a call, missing results become typed `cancelled` or `error` results, dangling outputs are removed.
- The system message is pinned: no reducer may drop or reorder index 0.
- Identical repeated user messages are preserved.
- One `TokenService` keyed by model: `o200k_base` or `cl100k_base` via `tiktoken-rs` `get_bpe_from_model`, `cl100k_base` as the documented fallback; the `len/4` estimator is deleted. The two `ContextStrategy` enums collapse to one; the survivor is the `uar::domain::context` enum because `progressive-summarization` and `per-model-context-strategy` specs bind to it.
- Tool output truncation applied once at ingest for MCP, native, and terminal results, middle-out, with the original token count and line count in a warning header, under a per-tool byte or token policy with a global default.
- Checkpoint resume uses `restore_state`; the endpoint no longer starts a fresh run with a prose input.

## Scope

- `src/uar/context/strategy.rs`
- `src/uar/runtime/context/{manager.rs,token_service.rs,summarizer.rs}`
- `src/uar/domain/context.rs`
- `src/uar/runtime/manager.rs` (the reducer block `:1478-1538` only; the system-message push at `:1471-1477` belongs to deterministic-prompt-assembly, and this change receives the system message as a pinned input)
- `src/llm/orchestrator.rs` (result ingest at `:1010-1023`, `:1272-1276`)
- `src/uar/tools/terminal_exec.rs`
- `src/uar/runtime/checkpoint.rs`, `src/uar/api/routes.rs` (`:346-395`)
- new: `src/uar/runtime/context/normalize.rs`, `src/uar/runtime/context/truncate.rs`
- tests: `tests/context_history_integrity.rs`

Out of scope: prompt fragment ordering (deterministic-prompt-assembly), skill body re-attachment after compaction (progressive-skill-runtime), typed turn assembly.

## Dependencies

None. This change creates the first seam (`normalize` and `truncate` as pure functions with typed inputs) that typed-turn-assembly needs.

## Verification

Tier 0 per edit: `cargo check --locked --no-default-features --features server-full`. Tier 1: the new focused tests. Tier 2 at change boundary: `cargo fmt --all -- --check` and `cargo test --locked --no-default-features --features server-full`.

## The uncomfortable thing

Collapsing the two enums changes the config surface: `ContextStrategy::TruncateMiddle` and `Hierarchical` have persisted settings readers. The change must map them, not delete them, and the spec delta says which names survive.


## Spec delta: conversation-history-integrity

## ADDED Requirements

### Requirement: Every tool call has exactly one tool result before dispatch
The runtime SHALL normalize conversation history before every provider request so that every assistant tool call has exactly one corresponding tool result and every tool result corresponds to an assistant tool call.

#### Scenario: Missing result is synthesized
- **WHEN** an assistant message contains a tool call whose result was never recorded because the run was cancelled or the call failed before completion
- **THEN** the runtime inserts a typed `cancelled` or `error` tool result for that call id immediately after the call before dispatch

#### Scenario: Orphaned result is removed
- **WHEN** history contains a tool result whose call id matches no assistant tool call
- **THEN** the runtime removes that result before dispatch and records a warning

#### Scenario: Reduction cannot sever a pair
- **WHEN** a context reduction window boundary would fall between an assistant tool call and its result
- **THEN** the pair is kept or dropped as a unit

### Requirement: The system message survives every context reduction
Context reducers SHALL treat the system message as pinned and SHALL NOT drop, reorder, or summarize it.

#### Scenario: Long conversation under a sliding window
- **WHEN** history exceeds the configured window
- **THEN** the system message remains at index 0 and only conversation turns are reduced

### Requirement: Identical repeated user messages are preserved
Context reducers SHALL NOT remove a user message because its content equals an earlier user message.

#### Scenario: User repeats "continue"
- **WHEN** two consecutive user turns have identical content
- **THEN** both remain in the reduced history in order

### Requirement: One token service counts every budget
The runtime SHALL count tokens through one model-keyed token service, using `o200k_base` or `cl100k_base` when the model maps to a known encoding and `cl100k_base` as the documented fallback, and SHALL NOT use a character-ratio estimate on any path.

#### Scenario: Known and unknown models
- **WHEN** a budget is computed for a model with a known encoding and for a model without one
- **THEN** the known model uses its encoding and the unknown model uses `cl100k_base`, and both counts come from the same service

### Requirement: Tool output is bounded once at ingest
The runtime SHALL truncate tool output once, when recording the model-visible result, using a middle-out policy expressed in bytes or tokens, and SHALL prefix truncated output with a warning that states the original token count and total line count.

#### Scenario: Oversized terminal output
- **WHEN** a native, MCP, or terminal tool returns output larger than its policy
- **THEN** the recorded result is within the policy, begins with the warning header, and retains the head and tail of the original output

#### Scenario: Output within policy
- **WHEN** a tool returns output within its policy
- **THEN** the result is recorded unchanged

### Requirement: Checkpoint resume restores checkpoint state
Resuming a run from a checkpoint SHALL seed the new run with the checkpoint's stored state and messages.

#### Scenario: Resume from a graph checkpoint
- **WHEN** a client resumes a run from a checkpoint id
- **THEN** the new run's history equals the checkpoint's messages and its graph state equals the checkpoint's state, and a checkpoint that fails to deserialize returns an error rather than an empty state

## Spec delta: tiktoken-estimation

## MODIFIED Requirements

### Requirement: Accurate token estimation for OpenAI models
The token service SHALL use tiktoken-compatible tokenization keyed by model, selecting `o200k_base` or `cl100k_base` when the model maps to a known encoding, and SHALL use `cl100k_base` as the fallback for models without a known tokenizer instead of character-based estimation.

#### Scenario: OpenAI model token count
- **WHEN** estimating tokens for text "Hello, world!" with model `openai/gpt-4o`
- **THEN** the count matches tiktoken's encoding result for that model

#### Scenario: Non-OpenAI model fallback
- **WHEN** estimating tokens for a model without a known tokenizer (e.g., `groq/llama-3`)
- **THEN** the system counts with `cl100k_base` and labels the estimate as a fallback in telemetry

#### Scenario: No character-ratio path remains
- **WHEN** any context reducer or budget check computes a token count
- **THEN** it calls the token service; no `len/4` estimator exists in the runtime
