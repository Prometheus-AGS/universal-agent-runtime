## 1. Stub LLM boot target

- [x] 1.1 Add `src/bin/stub-llm.rs`: thin binary wrapping the existing
      `tests/integration/live/stub_llm.rs` fixture/fingerprint server logic
      (move shared logic into a reusable module if needed to avoid
      `tests/`-only visibility), reading a fixture JSON path from argv/env
      and serving on a fixed, documented port.
- [x] 1.2 Create `tests/bdd/fixtures/bdd-chat.json`: stub fixtures for all
      six scenarios — plain response, needle-gated RAG response with the
      "missing context" marker, skill-trigger response, tool-call response
      (real tool-call completion chunk shape matching `stub_llm.rs`'s
      existing `native_echo` fixture), and two distinct model-identity
      fixtures for agent-switch / provider-routing scenarios.
- [x] 1.3 Verify `cargo run --bin stub-llm -- tests/bdd/fixtures/bdd-chat.json`
      boots and serves fixture responses (manual curl check).

## 2. BDD suite scaffold

- [x] 2.1 Add `playwright-bdd` and `@cucumber/cucumber` to root `package.json`
      devDependencies (not `frontend/package.json` — `tests/bdd/` is a sibling
      of `tests/e2e/`, which already gets its `@playwright/test` from root) —
      verified current versions: `playwright-bdd@9.2.0` (peer `@playwright/test
      >=1.44`, satisfied by root's `^1.57.0`), `@cucumber/cucumber@13.0.0`.
- [x] 2.2 Create `tests/bdd/playwright.config.ts`: extends the root
      `tests/e2e` pattern (`webServer` boots `stub-llm` then `cargo run`,
      pointed at the stub via `UAR_LLM__BASE_URL`/`UAR_LLM__API_KEY`),
      `use: { video: 'on' }`, `testDir: defineBddConfig(...)` output dir.
- [x] 2.3 Create `tests/bdd/support/world.ts` and shared step helpers
      (new-conversation, wait-for-response, DB-ready wait) reusing patterns
      already established in `tests/e2e/fixtures.ts` and `frontend/e2e/*.spec.ts`.
- [x] 2.4 Add `test:bdd` script to root `package.json` running
      `bddgen && playwright test -c tests/bdd/playwright.config.ts`.

## 3. Scenario feature files + step definitions

- [x] 3.1 `tests/bdd/features/chat-no-kb.feature` + step defs (Requirement:
      No-KB Chat Scenario Coverage).
- [x] 3.2 `tests/bdd/features/chat-kb-retrieval.feature` + step defs,
      including the fixture-document ingestion step (Requirement:
      Knowledge-Base-Influenced Chat Scenario Coverage).
- [x] 3.3 `tests/bdd/features/chat-skill-activation.feature` + step defs
      (Requirement: Skill Activation Chat Scenario Coverage).
- [x] 3.4 `tests/bdd/features/chat-tool-call.feature` + step defs
      (Requirement: Tool Call Chat Scenario Coverage).
- [x] 3.5 `tests/bdd/features/chat-agent-switching.feature` + step defs
      (Requirement: Agent Switching Chat Scenario Coverage).
- [x] 3.6 `tests/bdd/features/chat-model-routing.feature` + step defs
      (Requirement: Provider/Model Routing Chat Scenario Coverage).

## 4. Run, verify, evidence

- [x] 4.1 Run the full suite locally; fix any step definitions against real
      selectors (do not adjust scenarios to match broken behavior — if a
      scenario fails because a feature is actually broken, disclose it
      rather than weakening the assertion).
- [x] 4.2 Apply the `bdd-video-proof` skill against the suite's video output:
      ffmpeg WebM→MP4 remux, SHA-256 manifest keyed to the current commit,
      local bundle under `docs/certifications/bdd-chat/<sha>/`.
- [x] 4.3 Create `docs/BDD_SCENARIOS.md`: registry of all six scenarios,
      their `.feature` paths, and pass/fail status from the 4.1 run
      (Requirement: Scenario Registry Documentation).

## 5. CI wiring

- [x] 5.1 Add a CI job (new `bdd-chat.yml` or a job in an existing test
      workflow) running `test:bdd`, advisory (non-blocking) on first run per
      this change's design.md Open Questions.
- [x] 5.2 Dispatch the workflow for real (not just a dry run) and confirm it
      executes on actual GitHub Actions infrastructure.

## 6. KBD + phase bookkeeping

- [x] 6.1 Update `.kbd-orchestrator/phases/uar-production-ready-uiux-2026-07/progress.json`
      and `current-waypoint.json`: mark `bdd-chat-scenario-suite` DONE,
      advance `next_change` to `bootstrap-docusaurus-site`.
- [x] 6.2 Run `openspec archive bdd-chat-scenario-suite` (or `/opsx:archive`)
      once all tasks are verified complete.
