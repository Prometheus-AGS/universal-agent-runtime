## Context

Investigation before writing this design found the repo already has three
overlapping, none-sufficient test layers for chat behavior:

1. `tests/bdd.rs` — a native Rust `cucumber` 0.23 suite (no JS/browser),
   boots the real server via `tests/integration/live/harness.rs` against a
   `StubLlmServer` (`tests/integration/live/stub_llm.rs`), and proves
   API/AG-UI event-vocabulary behavior (`tests/features/librefang_and_agui.feature`).
   Deterministic, keyless, fast — but API-level only, no browser, so it
   cannot observe UI state (a skill badge rendering, an agent selector
   switching, a tool-call block appearing in the transcript).
2. `tests/e2e/*.spec.ts` (root) — plain Playwright (no Gherkin), boots the
   full app via `cargo run` at `127.0.0.1:3001`. `chat.spec.ts` drives a real
   message round-trip; `rag.spec.ts` only asserts a file-upload `<input>` is
   `toBeAttached()` — it does **not** upload a document or verify retrieval
   influenced a response; `tools.spec.ts` asks "what time is it" against
   whatever real provider is configured, with no stub, so it is not
   deterministic in CI.
3. `frontend/e2e/*.spec.ts` — plain Playwright, boots only `bun run dev`
   against a separately-running backend. `chat-agent-selection.spec.ts` and
   `chat-session-config.spec.ts` only assert a selector/button is *visible*
   (both `test.skip()` outright if no model is configured) — neither proves
   switching agents changes which model answers, nor that config changes
   take effect.

None of the six user-requested scenarios (no-KB, KB-influenced, skill
activation, tool calls, agent switching, provider/model routing) are proven
end-to-end today — existing coverage is presence-only or provider-dependent.
None use Gherkin, so there is no human-readable scenario record, and none
capture video evidence.

## Goals / Non-Goals

**Goals:**
- Prove all six scenarios end-to-end against the real running app (real
  browser, real backend, real ingestion/skill/tool-call machinery), not
  mocked at the frontend layer.
- Deterministic, keyless CI execution — no live provider API key required.
- Gherkin `.feature` files as a checked-in, readable "what we support" record.
- Video-proof evidence per scenario via the existing `bdd-video-proof` skill.
- Reuse existing infrastructure (`stub_llm.rs`'s fixture/fingerprint model,
  the `tests/e2e` `cargo run`-boots-everything pattern) rather than
  duplicating it in a new toolchain.

**Non-Goals:**
- Replacing or deleting `tests/bdd.rs`, `tests/e2e/*`, or `frontend/e2e/*` —
  they test different layers (API vocabulary vs. browser UI) and stay.
  Weak assertions found in `rag.spec.ts`/`chat-agent-selection.spec.ts` are
  superseded in spirit by this suite's stronger scenarios but not deleted in
  this change (out of scope; flagged as a follow-up below).
- New product features. Pure test-infrastructure coverage of already-shipped
  behavior.
- Testing runtime-console admin surfaces — those are `runtime-console-ux`'s
  scope (Round 2, already closed).

## Decisions

**D1 — Cucumber.js + playwright-bdd, not Rust cucumber+thirtyfour.**
The project's `bdd-cucumber-rs` convention (`thirtyfour` for browser
scenarios) has zero adoption (`thirtyfour` is not in `Cargo.toml`; `tests/bdd.rs`
is API-only). The project's frontend already has a full, working Playwright
investment (`@playwright/test` in both `package.json` and `frontend/package.json`,
two configs, 9+ existing specs). `playwright-bdd` binds Gherkin/Cucumber.js
directly onto the existing `@playwright/test` runner — no second browser
driver, no new test-runner infrastructure, just a Gherkin layer over what's
already there. This is the smallest true change (Rule 27: no silent/heavy
dependency) and matches the user-confirmed plan (`bdd-testing` /
`bdd-cucumber-js` skill, `bdd-video-proof` skill — both JS-toolchain skills).

**D2 — New `tests/bdd/` directory, sibling to `tests/e2e/`, own Playwright
config extending the root's `cargo run`-boots-everything pattern.**
Alternative considered: extend `frontend/e2e/`'s split dev-server config.
Rejected — that config boots only the frontend against *whatever* backend is
already running, which is why its existing specs fall back to `test.skip()`
when no model is configured. The root `tests/e2e/` pattern (single `cargo run`
serving the built frontend + API) is self-contained and already proven by
`chat.spec.ts`. `tests/bdd/playwright.config.ts` copies that shape but adds a
second `webServer` entry (see D3) and `video: 'on'`.

**D3 — Deterministic stub LLM, reusing `stub_llm.rs`'s existing model, exposed
as a standalone boot target.**
`tests/integration/live/stub_llm.rs` already implements fixture-keyed,
request-fingerprinted stub responses — proven by `tests/bdd.rs`. Rather than
reinventing this in TypeScript, add a thin `src/bin/stub-llm.rs` binary
(reuses the existing module, adds a `main()` that reads a JSON fixture file
path from `argv`/env and serves on a fixed port) so Playwright's `webServer`
array can boot it (`cargo run --bin stub-llm -- fixtures/bdd-chat.json`)
*before* `cargo run` (main app), which is pointed at it via
`UAR_LLM__BASE_URL` / `UAR_LLM__API_KEY=test-stub-key` env vars — the same
override mechanism `proxy-integration-gate`'s `UAR_LIVE_INTEGRATION_BACKEND`
already established. Two logically distinct model names route to the same
stub process with different canned fixtures, so scenario 6 (provider/model
routing) can assert *which* fixture answered, proving routing actually
happened rather than merely that a response arrived.

**D4 — RAG/retrieval proof via request introspection, not a "needle"
fixture (REVISED after implementation-time research).**
Original plan was a needle fixture keyed on exact request match. Research
during task 2 (see Findings below) established that agent-scoped knowledge-
base search results are appended to the **system prompt**
(`src/uar/runtime/manager.rs`, `system_prompt.push_str("\n\n[RELEVANT
KNOWLEDGE]\n...")`), while `RequestFingerprint` (the stub's response-routing
key) only inspects `model` + last user message + tool flags — it never sees
the system prompt. A pure needle-in-user-message fixture would therefore
prove nothing about KB retrieval specifically. Instead, added a small
introspection surface to the shared stub server
(`tests/integration/live/stub_llm.rs`): `GET /_stub/requests` returns every
raw request body received so far (including its `messages[0]` system
prompt), `POST /_stub/requests/reset` clears the log. Scenario 2's step
definition ingests a fixture document containing a distinctive phrase,
sends a question, then asserts via `/_stub/requests` that the *actual
outgoing system prompt* contains that phrase — a direct proof retrieval
reached the LLM call, independent of which canned response routed back.
This same introspection endpoint is reused where useful (e.g. confirming a
skill's system-prompt overlay was actually injected, not just that the
frontend rendered a badge).

**Findings from implementation-time research (recorded here per this
project's disclosure convention, not silently fixed — out of this change's
non-goals):**
- Two distinct "memory" mechanisms exist and must not be conflated: (a)
  agent-scoped knowledge-base search → system prompt (`manager.rs`, what
  scenario 2 tests); (b) session/user/agent/global memory-service context →
  **prepended directly to the user message** (`src/uar/memory/context_builder.rs`
  `build_context_with_hits`, called from `src/server.rs` before
  `start_run`), despite that code's own doc comment claiming system-prompt
  injection. Scenario 2 is scoped to (a) per the user's original "knowledge
  base enabled" wording; (b) is a distinct, untested mechanism, flagged as a
  gap in `docs/BDD_SCENARIOS.md`.
- Skill activation's `record_skill_activation` (`src/uar/telemetry/metrics.rs`)
  is pure telemetry with zero effect on the outgoing request. The actual
  LLM-visible effects (system-prompt overlay, optional MCP tool merge,
  optional `preferred_model` override) happen separately in
  `manager.rs` (~884-1021). Scenario 3's fixture must trigger a skill whose
  `execution_config` has no `preferred_model` override, so the scenario
  isolates skill-overlay behavior from model routing (which scenario 6
  covers separately).
- **Real dead-facade bug found**: `session-config-panel.tsx`'s "Save"
  button POSTs `model_override` to `POST /api/uar/sessions/{id}/agent-config`
  (persisted `AgentSessionConfig.model`), but the actual per-turn request
  builder (`use-chat-runtime.ts`) only ever sends `agentConfig.model` (the
  *agent's* configured default, from `agent-selector.tsx`'s
  `extractAgentConfig`) — the session-level override is written but never
  read back on the request path. This mirrors the exact pattern
  `uar-production-ready-uiux-2026-07`'s goals.md calls out (features that
  render and appear to work but don't reach task completion). **Scenario 6
  is redesigned** to prove model routing via the mechanism that actually
  works today — switching to an agent with a distinctly-configured model —
  rather than the session config panel's non-functional override field.
  The dead override field itself is disclosed as a known gap in
  `docs/BDD_SCENARIOS.md` and this phase's carry-over notes, not silently
  fixed (out of this test-infrastructure-only change's scope).
- Model strings sent to the provider are always `"{provider_id}/{model_id}"`
  (`src/llm/registry.rs`, `src/server.rs:4143`), confirming the
  `openai/gpt-5.4-mini`-style fixture keys already used by `tests/bdd.rs`.

**D5 — Video-proof reuses Playwright's native video capture, no new capture
path.** `playwright.config.ts`'s `use: { video: 'on' }` already produces WebM
per test; `bdd-video-proof`'s existing ffmpeg-remux + SHA-256-manifest step
runs unmodified against `tests/bdd/test-results/**/*.webm`.

## Risks / Trade-offs

- [Risk] Booting two Rust binaries (`stub-llm` + main `cargo run`) in
  Playwright's `webServer` array adds ~10-20s to suite startup and a new
  failure mode (stub port collision). → Mitigation: fixed, documented port
  with a preflight check (mirrors `scripts/live-integration.sh`'s existing
  health-check+remediation pattern from `proxy-integration-gate`).
- [Risk] Skill-activation and tool-call scenarios depend on the frontend
  actually rendering distinguishable UI state (a skill badge, a tool-call
  block) for the stub-driven response — if the stub's canned response
  doesn't trigger the same code path a real tool call would, the assertion
  is hollow. → Mitigation: the stub fixture for the tool-call scenario must
  emit a real tool-call completion chunk (matching `stub_llm.rs`'s existing
  tool-call fixture shape, already exercised by `tests/bdd.rs`'s
  `native_echo` scenario), so the frontend exercises its real tool-call
  rendering path, not a hardcoded UI state.
- [Risk] `docs/BDD_SCENARIOS.md` can drift from the actual `.feature` files
  over time. → Mitigation: keep the registry a thin index (scenario name,
  `.feature` path, one-line description) generated by hand at suite-build
  time; add a CI check only if drift is observed in practice (not
  pre-built — avoid speculative tooling per Rule 2).
- [Trade-off] `rag.spec.ts` and `chat-agent-selection.spec.ts`'s weak
  assertions are not fixed/deleted in this change (Non-Goal) — flagged as a
  named follow-up in `docs/BDD_SCENARIOS.md` and this phase's carry-over
  notes so it isn't silently forgotten.

## Confirmed Findings From The Real Suite Run (task 14)

Running the suite for real against the live app (not just authoring it)
surfaced several genuine issues, fixed or disclosed per this design's own
"disclose, don't weaken" rule:

- **Fixed as part of this change (user-approved scope expansion via
  AskUserQuestion):** the agent-selector popover's list was permanently
  broken — `loadAgentsIntoGraph()` only calls the entity-management
  library's `upsertEntity()` (writes entity *data*), never populates
  `graph.lists[baseKey]` (the id-index `useAgents()`'s `useEntityView()`
  call actually reads from). Every agent list showed "Loading agents..."
  forever, for every agent, not just test-created ones — a real,
  previously-undiscovered dead facade blocking the load-bearing "configure
  and chat with agents" capability's switching feature. Fixed by rewriting
  `frontend/src/entities/hooks/use-agents.ts` to use the same
  `useGraphStore`-selector pattern already working for `useModels()` /
  `useAgentsByStatus()` in this codebase, dropping the deprecated
  `useEntityView` entirely. Confirmed fixed: `chat-agent-switching` and
  `chat-model-routing` scenarios now pass, proving real UI-driven agent
  switching works end-to-end.
- **Confirmed (not just suspected) product bug, deliberately NOT
  fixed — out of this test-infra-only change's scope, left as a real,
  failing scenario:** `chat-kb-retrieval` fails because knowledge-base
  search returns zero matches even for an exact-phrase query against a
  freshly-ingested, successfully-indexed document (`status: "indexed"`,
  confirmed via direct `POST /api/knowledge/{id}/search` call, bypassing
  the chat/agent layer entirely — `{"results":[]}` for a query that should
  obviously match). This is the previously-flagged `task_188b4179`
  (`VectorMatcher::embed_batch` returns placeholder zero-vector embeddings,
  `model.forward()` still commented out) now empirically confirmed to
  break the KB search path in addition to whatever it already broke.
  `chat-kb-retrieval.feature` stays red — the assertion was not weakened —
  and this is called out in `docs/BDD_SCENARIOS.md` and the phase
  carry-over notes as a real, separate bug fix needed.
- Two dead-CLI-passthrough bugs found and worked around (not fixed —
  narrow config-loading quirks, not user-facing): `Cli::port`
  (`#[arg(long, env = "PORT")]`) and `Cli::jwt_required`
  (`env = "JWT_REQUIRED"`) are both parsed by clap but never applied to
  the config builder anywhere in `config.rs`/`main.rs`. The suite's
  `playwright.config.ts` uses the config-rs `Environment`-source
  equivalents (`UAR_SERVER__PORT`, `UAR_SECURITY__JWT_REQUIRED`) instead,
  which do work.
- The outgoing LLM request's `model` field format is inconsistent —
  sometimes bare (`"gpt-5.4-mini"`), sometimes `"provider/model"` —
  depending on which resolution branch a given run takes
  (`registry.rs`'s `has_explicit_base_url` check vs. a global-config
  fallback). Not chased further (implementation-internal formatting, not
  a routing-correctness question); fixtures and assertions accept either
  form.
- Root `tests/e2e/playwright.config.ts`'s bare `cargo run` webServer
  command is latently ambiguous now that 4 binaries exist in `Cargo.toml`
  with no `default-run` key set — not fixed (pre-existing, out of scope),
  `tests/bdd/playwright.config.ts` uses `cargo run --bin
  universal-agent-runtime` explicitly.

## Migration Plan

Purely additive — no existing test file is modified or deleted. New
`tests/bdd/` directory, new `src/bin/stub-llm.rs`, new `docs/BDD_SCENARIOS.md`,
new CI job. Rollback is `git revert` of the single commit/PR; no data
migration, no runtime behavior change.

## Open Questions

- Should the new CI job be advisory (like `comprehensive-tests.yml`'s
  current state) or blocking from day one? Recommend advisory first run,
  same pattern as `eval-nightly`'s `--require-baseline` gate — flag to user
  at CI-wiring task time rather than deciding here.
- Should `rag.spec.ts` / `chat-agent-selection.spec.ts` be deleted once this
  suite supersedes them, or kept as fast presence-only smoke checks
  alongside the fuller BDD scenarios? Deferred to a follow-up change per
  this design's Non-Goals.
