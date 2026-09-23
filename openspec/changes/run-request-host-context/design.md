## Context

See proposal.md for why. The code this design builds on:

- **Working directory.** `RunExecutionRequest.working_directory` exists and is documented as "never grants workspace trust or file permissions" (`src/uar/runtime/turn/request.rs:57-58`). The manager uses it for `WorldStateRuntime` (project instructions) and for delegation bindings, falling back to the process cwd (`manager.rs:2707-2745`, `3434`). It is compared with the single process-wide MCP environment directory when an MCP capture exists (`manager.rs:2250-2256`).
- **File tools** are off by default (`src/config.rs:2347-2348`). When on, they are registered once per process with the global `native_tools.file_allowed_paths` and a `DelegatedFileRoots` captured from them as `cap_std` directory handles (`src/uar/runtime/native_skills/mod.rs:76-99`, `src/uar/tools/file_tools.rs:89-127`). Relative configured paths resolve against the process cwd (`file_tools.rs:95-100`).
- **Terminal** (`terminal_exec`, off by default, `config.rs:2355`) takes an optional model-supplied `working_dir` and otherwise inherits the process cwd (`src/uar/tools/terminal_exec.rs:224-228`).
- **Reasoning.** Each model step builds `DialectRequest { wants_reasoning, multi_turn, hard }` from `LlmConfig.thinking_budget` (`src/llm/orchestrator.rs:1555-1566`). `request_params` maps it per dialect; OpenAI-family models get nothing (`src/llm/prompt_dialect.rs:456-520`). `LlmConfig` already carries runtime-only metadata as `#[serde(skip)]` (`resolved_provider_id`, `config.rs:1638-1644`), and children reuse the root's `LlmConfig` through `RunModelBindings` (`src/uar/runtime/turn/bindings.rs:131-141`).
- **Sessions** are keyed by `(owner_id, session_id)` (`src/session/thread.rs:388-432`). A run resolves its session with `get_or_create_for_user` (`manager.rs:2402-2407`). `seed_history` (`Vec<SeedMessage>`, text only) is replayed into an empty session when no checkpoint or inherited history is present (`manager.rs:2410-2429`). `Session` already has `add_assistant_with_tool_calls` and `add_tool_result` (`thread.rs:224-249`). Checkpoint resume carries protected history (`request.rs:9-17`, `src/uar/runtime/checkpoint.rs:103-140`) and wins over everything (`manager.rs:2105-2115`).
- **Reporting precedent.** `create_run` already reads a value the manager left in `run.context` (`activation_failures`, `routes.rs:75-80`).

## Goals / Non-Goals

**Goals:**
- A host can set the directory and reasoning level of each run and read them back from the run record.
- A restarted UAR continues a session from host history, with a defined winner whenever history sources overlap.
- A history never lands in a session other than the one it names.

**Non-Goals:**
- Durable session storage inside UAR. The host stays canonical (D2).
- Sandboxing `terminal_exec` to the working directory. A shell can change directory; the working directory is a default, not a boundary, for terminal commands. File tools are the boundary.
- Merging host history into a warm session.

## Decisions

### D1 — Resolve the working directory once, at the route
`create_run` canonicalizes `working_directory` (`std::fs::canonicalize`), requires an absolute input, an existing directory, and a non-root result, then sets `RunExecutionRequest.working_directory` to the resolved path. Children already inherit it through `InheritedRunBindings.working_directory` (`bindings.rs:32`).
- **Alternative rejected:** validating inside the manager. A 422 before a run id is issued is cleaner for the host than a run that starts and immediately errors.

### D2 — Per-run file roots from the working directory
When file tools are enabled and a run has a working directory, the manager registers run-local `file_read`, `file_write` and `file_patch` instances in the run's captured native registry (the `filtered(None)` capture at `manager.rs:3424`), each built with `DelegatedFileRoots::capture(&[working_directory])`. They replace the process-wide instances for that run only. Root selection:
- Operator roots configured: the working directory must be inside one of them (checked on canonical paths), else 422 `working_directory_not_allowed`. The run's roots are the working directory only, so access narrows.
- No operator roots, and the request passed the sidecar launch-token guard (a request extension set by `sidecar-launch-security`): the working directory is the run's only root. The authenticated host is the party granting it.
- No operator roots, any other request: file tools behave as configured today; the working directory is still used for world state and terminal defaults.

This keeps the promise in `request.rs:57` for standalone UAR and meets coordinator inventory item 6 for the sidecar.
- **Alternative rejected:** one process-wide root set containing every session's directory. Any session could then read another session's workspace.

### D3 — Terminal defaults to the working directory
`terminal_exec` receives the run's working directory through its execution context. With no `working_dir` argument it runs there; a relative `working_dir` resolves against it. An absolute `working_dir` is honored as today, because the terminal is not a confinement boundary (Non-Goals).

### D4 — Reasoning effort as run metadata on `LlmConfig`
Add `ReasoningEffort { None, Low, Medium, High, Max }`, parsed at the route. Store it on the run's `LlmConfig` in a new `#[serde(skip)]` field, so it never reaches a config file and children inherit it with the model bindings. `DialectRequest` gains `effort: Option<ReasoningEffort>`; when set, it replaces the `thinking_budget` signal. Proposed mapping (numbers to be confirmed against provider limits):

| Dialect | none | low | medium | high | max |
|---|---|---|---|---|---|
| Anthropic `thinking.budget_tokens` | omitted | 1024 | 2048 | 8192 | 16384 |
| GLM `reasoning_effort` | omitted | `high` | `high` | `high` | `max` |
| Kimi / Qwen thinking toggles | omitted | enabled | enabled | enabled | enabled |
| OpenAI-family `reasoning_effort` | omitted | `low` | `medium` | `high` | `high` |
| Generic | omitted | omitted | omitted | omitted | omitted |

The mapping is monotonic per dialect, which is what the spec requires; exact numbers are tuning.
- **Alternative rejected:** a free-form string passed through to the provider. It would accept values a provider rejects at run time and give the host no contract.

### D5 — Typed host history, validated at the route, seeded atomically
`CreateRunRequest.history: Option<HostHistory { session_id, messages }>`. The route validates structure (roles, tool-call pairing, duplicate ids, `system` rejected, size limits) and session equality, then converts to typed `llm::Message` values on a `pub(crate)` request field. At the existing seeding site (`manager.rs:2410-2429`) a new `Session::seed_if_empty(messages) -> SeedOutcome` takes the session's message write lock, checks emptiness and appends under that one lock, so two concurrent first turns cannot both seed. The embedded `SeedMessage` path calls the same method. The outcome goes into `run.context["history"]` and `create_run` returns it, following the `activation_failures` precedent.

Precedence, highest first:
1. Checkpoint resume history (protected; history is refused on resume routes).
2. Inherited child history (internal only).
3. Messages already in the in-memory session (`ignored_warm_session`).
4. Host history (`seeded`).

- **Why reject malformed history instead of repairing it:** the runtime already repairs orphaned tool results by removing them (`conversation-history-integrity`). For host input that would silently drop turns the user can see. A 422 tells the host its serializer is wrong.
- **Why in-memory wins:** the in-memory session holds tool calls, compaction summaries and turns the host's view may render differently. Replacing it with a host projection on every turn would lose that and double-count turns. Host history exists for the cold case.

### D6 — Session binding
Cross-session binding is prevented by three facts together: `history.session_id` must equal the request `session_id`; the session is resolved under the request's verified owner (`thread.rs:422-432`); and, with `sidecar-session-principal`, each the-boss session has its own owner. Without that change every the-boss session shares owner `anonymous`, so a wrong `session_id` from the host would still land in another the-boss session. That is why this change depends on it.

## Risks / Trade-offs

- [Warm-session precedence hides host edits] If the user edits or deletes an earlier message in the-boss, a warm UAR session keeps the old turn → the-boss must start a new UAR session id after destructive edits. Named in the create-run response (`ignored_warm_session`) so the host can detect it.
- [History size] Large histories cost tokens before compaction runs → limits `runs.host_history.max_messages` and `runs.host_history.max_bytes` (proposed keys, proposed defaults 1000 and 4 MiB); context reduction applies to seeded history like any other.
- [Sessions are never evicted] `SessionStore::cleanup_expired` is `#[allow(dead_code)]` (`thread.rs:462-470`). Host history makes eviction safe later, but this change does not add it.
- [The unarchived `seed-embedded-session-history` change] targets a capability `embedded-admin-surface` that does not exist under `openspec/specs/`. This change reuses its code path and does not depend on its spec.
- [OpenAI reasoning parameter name through liter-llm] is unconfirmed; a wrong name is either ignored or rejected by the provider → contract test 1.6 asserts the parameter on the recorded request, and the name is confirmed before implementation.

## Migration Plan

Additive. Existing callers send none of the fields and see `history: "none"` in the response. Rollback: revert.

## Open Questions

- Exact Anthropic budget numbers and whether `max` should track the model's output limit. Tuning only; the spec requires monotonicity.
