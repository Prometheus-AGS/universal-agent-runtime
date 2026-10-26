# Behavior: prompt the default agent

**Given** UAR is running with a real provider/model configured and verified in UAR settings.  
**When** a user sends the default agent a simple arithmetic prompt.  
**Then** it returns HTTP 200 and the correct answer.

Requires Node.js 20 or later. No packages or test runner.

```sh
node scripts/behaviors/prompt.mjs
```

The default UAR address is `http://127.0.0.1:3000`. Set `UAR_BASE_URL` for another address and `UAR_MODEL` to select a configured `provider/model`.

All manual functional scenarios target an externally provisioned identity endpoint. External infrastructure handles authentication; authentication is out of test scope. These scripts must never require, mint, renew, or manage authentication tokens, and they send no Authorization header. `UAR_WORKSPACE_ID` is a nonsecret ownership selector for scenarios that use workspace-scoped records. URLs must not contain credentials.

For example, in Windows PowerShell, with UAR already running:

```powershell
$env:UAR_BASE_URL = 'http://127.0.0.1:3000'
$env:UAR_MODEL = 'liter/MiniMax-M3'
node scripts/behaviors/prompt.mjs
```

The script sends one prompt, makes no automatic retries, and exits nonzero if it fails. Each run writes its actual result to `.prometheus/verification/`; `UAR_RESULT_FILE` can select another new output file. Existing result files are never overwritten. The recorded platform is the machine running the script; a server-platform claim additionally requires running UAR on that platform.

Mac prompting was verified on the native UAR 1.3.0 build using Liter/MiniMax-M3 on 2026-10-26. See `.prometheus/verification/2026-10-26T06-38-40.440Z-prompt-macos.json` and `macos-prompt.md` in that directory. This verifies one prompt on that build, not other workflows or the entire release.

## Behavior: streamed reply (Cucumber)

Uses the Prometheus `bdd-cucumber-js` HTTP-only pattern and the installed `@cucumber/cucumber`. Given a running UAR and `UAR_MODEL`, run from the repository root:

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/cucumber.mjs
```

Only `features/streaming.feature` runs. It checks multiple content deltas, the complete useful answer, successful completion and `[DONE]`. JSON reports are written under `.prometheus/verification/`; `UAR_CUCUMBER_REPORT` can choose a path. This command does not rerun `prompt.mjs`. See `.prometheus/verification/macos-streaming.md` for the Mac result.

## Behavior: saved specialist agent

`features/persisted-agent.feature` creates a Mathematics Specialist, restarts its dedicated UAR and reads/queries the saved definition. `agent.cucumber.mjs` selects this explicit restart scenario; it requires `UAR_SERVER_BIN`, `UAR_SERVER_CONFIG` (JSON), `UAR_BASE_URL`, and `UAR_MODEL`. The child inherits the externally provisioned service environment, with scenario-private `HOME`/`TMPDIR` and `RUST_LOG` overrides. The scenario performs no credential setup. It retains the agent and exports the definition under `.prometheus/verification/agents/`.

`UAR_BASE_URL` selects the client endpoint, which may be an externally provisioned identity proxy; `server.host` and `server.port` in `UAR_SERVER_CONFIG` select the separate owned native listener. Both must use loopback addresses. The scenario reserves the native listener before startup and sends readiness and product requests through the client endpoint. External provisioning must route that endpoint to the owned UAR across its restart.

Routine queries use the running UAR. Set `UAR_AGENT_RECORD` to the saved definition JSON, plus `UAR_BASE_URL` and `UAR_MODEL`, then run:

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/query-agent.cucumber.mjs
```

This query-only scenario leaves UAR running. It checks the final answer after any leading completed provider reasoning block, including the saved specialist marker and correct random arithmetic. It creates no second agent and reruns neither plain prompting nor streaming. Current Mac agent: `math-specialist-db64c2a8`; see `.prometheus/verification/macos-agent.md`.

## Behavior: writing agent selected by orchestrator intent

`writing.cucumber.mjs` selects only `writing-orchestrator.feature`. It saves `writing-specialist` (reuses the retained identical definition on a retry), confirms the writer and mathematics specialist in the live roster, submits a blog-writing request to the orchestrator's official AG-UI endpoint without naming a specialist, and verifies intent selection plus an actual draft bearing the saved writer's byline. The scenario handles the normal single local `spawn_agent` approval through AG-UI resume.

Use `UAR_BASE_URL`, `UAR_MODEL`, and `UAR_WORKSPACE_ID` when required. `UAR_MATH_AGENT_RECORD` and `UAR_WRITING_AGENT_RECORD` may select the retained agent records. The runtime needs its normal orchestration governance policy installed; the Mac instance has narrowly scoped orchestrator spawn/wait permits.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/writing.cucumber.mjs
```

Both agents and UAR remain running afterward. No plain-prompt, streaming or saved-math scenario is included. See `.prometheus/verification/macos-writing-orchestrator.md` for the result and actual article.

## Behavior: grounded mathematics knowledge base

`knowledge.cucumber.mjs` runs only `math-knowledge.feature`. It creates and retains a sourced wheel-graph mathematics KB, uploads a fresh private research profile, waits for real indexing, persists the binding to the existing math specialist, and verifies its answer against independent Lucas arithmetic and the actual run citation carrying the KB/document IDs and private seal.

Set `UAR_BASE_URL`, `UAR_MODEL`, and the same `UAR_WORKSPACE_ID` for knowledge ownership and prompting. `UAR_MATH_AGENT_RECORD` selects the original saved math definition/marker. A genuine embedding backend must already be configured and serving (the Mac instance uses existing local BGE-small-en-v1.5 assets).

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/knowledge.cucumber.mjs
```

This is a retained-data scenario: each run creates a new KB/document, appends that binding to the saved agent and makes one real inference call. It does not delete data or stop UAR. Evidence is under `.prometheus/verification/knowledge/`; the current math definition is saved as `agents/math-specialist-db64c2a8.knowledge.json`. Previously saved arithmetic-only definitions are historical after this binding change. The previous four behaviors are not rerun. See `.prometheus/verification/macos-knowledge.md`.

## Behavior: concurrent math/writing team project

`team.cucumber.mjs` selects only `math-writing-team.feature`. It packages immutable captures of saved math/writer IDs and instructions, creates an actual durable team and explicit three-task project graph, admits calculation and outline separately for concurrent execution, then passes both published output artifacts to the dependent writer article. This tests host-authored task decomposition; it does not claim automatic natural-language project planning. Full ordinary-agent KB/resource parity is outside this case.

Requires `UAR_BASE_URL`, `UAR_MODEL` (admission captures the current configured model settings), `UAR_WORKSPACE_ID`, and a running service with configured nonempty service_instance.instance_id and executable collaboration support. Uses existing saved agents math-specialist-db64c2a8 and writing-specialist.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/team.cucumber.mjs
```

Each run retains package, team, task and evidence records. Never substitute queued task overlap for live execution. Current Mac result is **PASS**: 1 scenario, all 8 steps, 1m 14.458s. See `.prometheus/verification/team-01-macos-pass.json` and `team-01-macos-article.md`. This behavior is complete for the manual sequence; do not rerun it unless the operator selects a regression after relevant changes. Prior failures remain preserved in the evidence directory. No service shutdown hook or automatic retry is installed.
