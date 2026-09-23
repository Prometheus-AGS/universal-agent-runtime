## Why

the-boss keeps three pieces of run context that UAR cannot receive today (decisions D1 and D2, revision 3). `CreateRunRequest` has no working directory, although `RunExecutionRequest` has one that `create_run` never sets (`src/uar/api/routes.rs:38-47, 68-73`; `src/uar/runtime/turn/request.rs:57-58`). It has no reasoning effort: UAR derives reasoning parameters from its own `thinking_budget` config (`src/llm/orchestrator.rs:1555-1566`, `src/llm/prompt_dialect.rs:497-520`). And it cannot take prior conversation history: session context lives in an in-memory store (`src/session/thread.rs:380-432`), so after a sidecar restart every session is empty and the agent forgets the conversation the user still sees. An embedded-only seam for this exists (`SeedMessage`, `manager.rs:5797-5818`, applied at `manager.rs:2410-2429`, from the unarchived change `seed-embedded-session-history`), but the HTTP runs API does not expose it and it drops assistant tool calls.

## What Changes

- `POST /api/uar/runs` accepts `working_directory`: an absolute path that must exist and be a directory. It becomes the run's world-state directory, the default cwd of `terminal_exec`, the base for relative file-tool paths, and the run's file-tool root. It narrows file access and never widens the operator's configured roots, except that on a launch-token-authenticated sidecar with no configured roots the host-supplied directory is the only root.
- `POST /api/uar/runs` accepts `reasoning_effort`: one of `none`, `low`, `medium`, `high`, `max`. It replaces the config-derived reasoning signal for that run and maps to each provider dialect's parameter. Child runs inherit it.
- Both values are recorded in the run record so a test can read them back.
- `POST /api/uar/runs` accepts `history`: the prior conversation for the request's `session_id`, as typed user, assistant (with optional tool calls) and tool messages. It seeds an empty session only. A session that already holds messages keeps its own history, and the response says which happened. History must name the same session as the request, must be well formed (every tool result answers an earlier tool call), and cannot be combined with checkpoint resume.
- Invalid input is rejected with 422 before any run starts: `working_directory_invalid`, `working_directory_not_allowed`, `reasoning_effort_invalid`, `history_invalid`, `history_session_mismatch`, `history_too_large`.

## Capabilities

### New Capabilities
- `run-host-context`: host-supplied working directory and reasoning effort on the runs API: validation, what each controls, inheritance by child runs, and how they are recorded.

### Modified Capabilities
- `conversation-history-integrity`: adds host-supplied history for a session: when it seeds, precedence against in-memory session context and checkpoint resume, session binding, and structural validation.

## Impact

- **Code:** `src/uar/api/routes.rs` (`CreateRunRequest`, `create_run`), `src/uar/runtime/turn/request.rs` (`reasoning_effort`, typed host history), `src/uar/runtime/manager.rs` (history precedence at the seeding site, `manager.rs:2410-2429`; working directory at `manager.rs:2707-2745`; run context recording), `src/llm/prompt_dialect.rs` and `src/llm/orchestrator.rs:1555-1566` (effort mapping), `src/uar/tools/terminal_exec.rs:224-228` (default cwd), `src/uar/tools/file_tools.rs` and `src/uar/runtime/native_skills/mod.rs:76-99` (per-run file roots).
- **APIs:** additive fields on `POST /api/uar/runs`; the create-run response gains `history: "seeded" | "ignored_warm_session" | "none"` and `seeded_messages`.
- **Provider compatibility:** the effort mapping per dialect is new. OpenAI-family models receive no reasoning parameter today; this change adds one, and its exact name through liter-llm must be confirmed at implementation (design Open Questions).
- **Realtime state:** run records gain `host_context: { working_directory, reasoning_effort }`. Seeded history becomes the session's canonical history; later turns append to it in memory as today.
- **Runtime UX:** none for the UAR SPA, which sends none of the fields.
- **Security:** the working directory is a filesystem boundary (A-3). It narrows file roots and is validated before use.
- **Depends on:** `sidecar-session-principal` (session stores are keyed by owner, `src/session/thread.rs:388-432`; without per-session principals every the-boss session shares the owner `anonymous`).
- **KBD workflow state:** yes. This change belongs to the-boss child phase `the-boss-universal-agent-runtime` (step 2); that phase's `progress.json` in `prometheus-skills-mini` must list it. This repository's `.kbd-orchestrator/` is not updated by this authoring pass.
