## Why

`uar-production-ready-uiux-2026-07`'s assessment confirmed chat + agent/provider
configuration are the two load-bearing capabilities of this product, but found
frontend test coverage is thin (12/206 files, ~5.8%) and e2e Playwright specs
parse but are not exercised live in CI. There is no behavior-level, human-readable
record of which chat use cases actually work end-to-end — knowledge base
retrieval influencing a response, skill activation, tool-call surfacing, agent
switching, and provider/model routing are all real runtime features with no
scenario-level proof they function together, only unit/integration coverage of
their individual backend pieces. The user explicitly asked for this suite by
name at plan time, scoped to six concrete scenarios, with video evidence per
scenario (per the project's `bdd-testing` + `bdd-video-proof` skills).

## What Changes

- Add a Cucumber.js + Gherkin BDD suite (new `tests/bdd/` directory, following
  the `bdd-cucumber-js` skill's convention: cucumber-js 13, playwright-bdd for
  browser-driven scenarios, tsx) covering six chat scenarios:
  1. Chat with no knowledge base attached.
  2. Chat with a knowledge base enabled and memory/context retrieval actually
     influencing a response.
  3. Chat with skills activated (a skill visibly selected/applied mid-conversation).
  4. Chat with tool calls (a tool invoked and its result surfaced in the transcript).
  5. Agent selection/switching mid-session.
  6. Provider/model configuration affecting which model actually answers.
- Each scenario gets its own Gherkin `.feature` file — readable scenario
  documentation that doubles as a checked-in "what we support" record — plus
  step definitions driving the real running app (not mocks), per the project's
  `bdd-lifecycle-loop` outside-in convention.
- Capture video-proof evidence per scenario via the `bdd-video-proof` skill
  (Playwright WebM → ffmpeg MP4 remux, SHA-256 manifest keyed to commit,
  local bundle under `docs/certifications/bdd-chat/<sha>/`).
- Add `docs/BDD_SCENARIOS.md` as a checked-in registry listing every scenario,
  its `.feature` file, and its current pass/fail status, so support claims are
  documented rather than only passively tested.
- Wire the suite into CI as an advisory/gated job per the existing
  `bdd-lifecycle-loop` flake-budget pattern (`--retry-tag-filter`), consistent
  with how `comprehensive-tests.yml` and `security-audit.yml` are structured.

**Non-goals**: no new product features — this is pure test-infrastructure
coverage of existing, already-verified chat/agent/provider functionality. No
changes to `runtime-console-*` capabilities (Round 2, already closed).

## Capabilities

### New Capabilities
- `chat-bdd-coverage`: BDD scenario coverage proving the six load-bearing chat
  use cases (no-KB, KB-influenced, skill-activated, tool-call, agent-switch,
  provider/model-routed) function end-to-end against the real running app,
  with video-proof evidence and a checked-in scenario registry.

### Modified Capabilities
(none — this adds new test coverage only; no existing spec's runtime
requirements change)

## Impact

- **New code**: `tests/bdd/` (features, step defs, world/hooks, cucumber
  config), `docs/BDD_SCENARIOS.md`, CI workflow additions (likely a new
  `bdd-chat.yml` or an added job in an existing test workflow).
- **Dependencies**: `@cucumber/cucumber`, `playwright-bdd`, `tsx` (all already
  named as the project's standing BDD convention per `bdd-cucumber-js` /
  `bdd-lifecycle-loop` skills — verify current pinned versions before adding,
  per Rule 22/23).
- **Runtime UX**: none — read-only test coverage, no product surface changes.
- **Provider compatibility**: scenario 6 exercises real provider/model routing
  against whatever provider is configured for the test environment; needs a
  keyless or fixture-backed provider path (consistent with how
  `eval-targeted-suites` used keyless `CompletionProvider` fixtures) to avoid
  requiring live API keys in CI.
- **Realtime state**: scenarios 2-4 exercise real AG-UI/SSE event flow (memory
  retrieval, skill activation, tool-call surfacing) — must drive the actual
  running server, not mocked events, per the project's outside-in BDD
  convention.
- **KBD workflow state**: this change is tracked as Round 3 of
  `uar-production-ready-uiux-2026-07` (`.kbd-orchestrator/phases/uar-production-ready-uiux-2026-07/progress.json`);
  progress.json and the waypoint must be updated to DONE + archived on
  completion, same as Rounds 1-2.
