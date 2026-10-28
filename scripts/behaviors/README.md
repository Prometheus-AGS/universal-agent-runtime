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

## Behavior: conversation continuity with the saved math agent

`continuity.cucumber.mjs` runs only `conversation-continuity.feature`. It asks the saved math agent to add fresh random numbers, then sends only a follow-up in the same session: transform its previous answer and subtract an original operand. The client does not replay either prior message or the original numbers. Both final answers must match independently calculated results and the saved specialist identity.

Use the existing externally provisioned `UAR_BASE_URL`, `UAR_MODEL`, and nonsecret `UAR_WORKSPACE_ID`. The script supplies no authentication credentials, creates a fresh conversation, preserves request/response evidence, and leaves UAR and the saved agent running unchanged.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/continuity.cucumber.mjs
```

This checks warm two-turn conversation continuity only, not restart durability or cross-session long-term memory. No earlier scenario is included.

Mac result: **CONTINUITY-01 PASS**, all6steps in11.440seconds. See `.prometheus/verification/continuity-01-macos-pass.json`. Completed for this manual sequence; do not rerun without operator direction.

## Behavior: protected default and orchestrator agents

`protected-agents.cucumber.mjs` runs only `protected-agents.feature`. It reads both built-in definitions, attempts their deletion, requires HTTP403 with the specific built-in protection message, then reads both again and compares their complete definitions. Generic authorization failures cannot pass. No model calls, credentials, restart, or earlier scenarios are included; UAR stays running.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/protected-agents.cucumber.mjs
```

Use the existing externally provisioned `UAR_BASE_URL` and optional nonsecret `UAR_WORKSPACE_ID`. Requests and responses are retained under `.prometheus/verification/protected-agents/`.

Mac result: **PROTECTED-AGENTS-01 PASS**, all5steps in0.80seconds. See `.prometheus/verification/protected-agents-01-macos-pass.json`. Complete for this manual sequence; no automatic rerun.

## Behavior: update and delete a disposable custom agent

`agent-lifecycle.cucumber.mjs` selects only `custom-agent-lifecycle.feature`. It creates a fresh disposable agent, confirms its original arithmetic rule and marker through genuine inference, patches different instructions and reads them back, then repeats the same user prompt in a fresh session. The second answer must use the updated rule and fresh marker, which are never supplied in the user prompt. Finally DELETE must return204 and GET must return404.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/agent-lifecycle.cucumber.mjs
```

Use the externally provisioned `UAR_BASE_URL`, `UAR_MODEL`, and nonsecret `UAR_WORKSPACE_ID`. Existing agents and UAR stay untouched. Failure cleanup attempts deletion only for the agent created by this invocation; it does not retry a failed deletion. Request/response evidence is retained under `.prometheus/verification/agent-lifecycle/`. No previous scenario is selected and tests supply no authentication credentials.

Mac result: **AGENT-LIFECYCLE-01 PASS**, all9steps in6.580seconds. See `.prometheus/verification/agent-lifecycle-01-macos-pass.json`. Disposable agent deleted; complete for this manual sequence.

## Behavior: saved math agent through Anthropic Messages

`anthropic.cucumber.mjs` runs only `anthropic-math.feature`. It sends one fresh arithmetic prompt to `/v1/messages` with the saved mathematics agent selected, checks the Anthropic message/text-block envelope and `end_turn`, then verifies the saved specialist marker and independently computed answer. The configured provider/model stays unchanged; this tests UAR's Anthropic-compatible interface, not a switch to an Anthropic-hosted model.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/anthropic.cucumber.mjs
```

Use externally provisioned `UAR_BASE_URL`, `UAR_MODEL`, and nonsecret `UAR_WORKSPACE_ID`. No test credentials, service restart, agent changes, or earlier scenarios. Raw requests/responses are retained under `.prometheus/verification/anthropic/`. Scope is one nonstreaming custom-agent prompt, not streaming or complete protocol conformance.

Mac result: **ANTHROPIC-01 PASS**, all4steps in11.822seconds on the first attempt. See `.prometheus/verification/anthropic-01-macos-pass.json`. Complete for this manual sequence.

## Behavior: saved math agent through A2A

`a2a.cucumber.mjs` runs only `a2a-math.feature`. It sends one fresh arithmetic question using A2A v1 JSON-RPC `SendMessage` to `/a2a/v1/agents/math-specialist-db64c2a8`, with `returnImmediately:false`. The response must correlate to the request, contain a completed task for that agent, and return the saved specialist marker plus the independently computed answer.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/a2a.cucumber.mjs
```

Use the existing externally provisioned `UAR_BASE_URL`, `UAR_MODEL`, and nonsecret `UAR_WORKSPACE_ID`. The agent's saved model binding is used unchanged. No test credentials, service restart, agent edits, or earlier scenarios. Raw evidence is retained under `.prometheus/verification/a2a/`. This covers one nonstreaming A2A v1 prompt, not streaming, task continuation, cancellation, legacy A2A or full protocol conformance.

Mac result: **A2A-01 PASS**, all4steps in4.539seconds on the first attempt. See `.prometheus/verification/a2a-01-macos-pass.json`. Complete for this manual sequence.

## Behavior: saved math agent through ACP

`acp.cucumber.mjs` selects only `acp-math.feature`. It creates an ACP session for the saved math agent, sends one fresh arithmetic question through `/acp/stream` (`runs/create`), and requires final answer content, a successful terminal event and matching completed agent/run metadata through ACP `runs/get`. The saved specialist marker is never supplied in the question.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/acp.cucumber.mjs
```

Use externally provisioned `UAR_BASE_URL`, `UAR_MODEL` and nonsecret `UAR_WORKSPACE_ID`. No test credentials, agent edits, restart or prior scenario reruns. Raw evidence is retained under `.prometheus/verification/acp/`. Scope is one saved-agent ACP prompt, not continuation, cancellation, tools, other platforms or full protocol conformance.

Mac result: **ACP-01 PASS**, all4steps in3.827seconds on the first attempt. See `.prometheus/verification/acp-01-macos-pass.json`. Complete for this manual sequence; no automatic rerun.

## Behavior: saved math agent through AG-UI

`agui.cucumber.mjs` selects only `agui-math.feature`. One fresh arithmetic prompt targets `/api/uar/ag-ui/agents/math-specialist-db64c2a8`. It requires multiple text fragments between assistant-message start/end, matching run/thread/message identifiers, successful completion, and a correct assembled answer containing the marker known only to the saved agent instructions.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/agui.cucumber.mjs
```

Use externally provisioned `UAR_BASE_URL`, `UAR_MODEL` and nonsecret `UAR_WORKSPACE_ID`. Raw exchanges/events are retained under `.prometheus/verification/agui/`. No test authentication, agent/model changes, restarts or prior scenario reruns. This covers one streamed prompt, not browser rendering, A2UI, reconnect/replay, cancellation or full conformance.

Mac result: **AGUI-01 PASS**, all4steps in7.15seconds on the first attempt. See `.prometheus/verification/agui-01-macos-pass.json`. Complete for this manual sequence; no automatic rerun.

## Behavior: published A2UI response through AG-UI

`a2ui.cucumber.mjs` selects only `a2ui-response.feature`. A disposable agent uses the real `a2ui_render` tool to publish a fresh greeting Text surface. `/v1/chat/completions` with `stream_mode: "agui_spec"` must deliver the matching A2UI artifact and ordered surface/component STATE_DELTA events before successful completion. A prose-only response cannot pass. Cleanup deletes only this scenario's disposable agent.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/a2ui.cucumber.mjs
```

Use externally provisioned `UAR_BASE_URL`, `UAR_MODEL`, `UAR_WORKSPACE_ID`. Evidence: `.prometheus/verification/a2ui/`. No test credentials or prior scenario reruns. This is API publication evidence, not browser rendering, interaction or reconnect evidence. The dedicated `/api/uar/ag-ui/agents/{agent_id}` projector currently omits A2UI artifact/state events; this scenario explicitly covers the `agui_spec` stream mode instead.

Mac result: **A2UI-01 PASS**, all4steps in6.824seconds on attempt002 after correcting runtime metadata parsing. Both disposable agents deleted, initial failure preserved. See `.prometheus/verification/a2ui-01-macos-pass.json`. No automatic rerun.

## Behavior: browser-rendered A2UI action reaches its agent

`a2ui-browser.config.mjs` uses Playwright-BDD and selects only `a2ui-browser.feature`. A disposable real-model agent renders a parcel calculator in the shipped `/threads` UI. The test fills a fresh number through the rendered textbox and clicks its button. It requires the browser's actual action POST, a distinct successful continuation run, the independently correct result plus saved marker, and the result visible in the browser. Only the disposable agent is deleted.

```sh
node_modules/.bin/bddgen --config scripts/behaviors/a2ui-browser.config.mjs
node_modules/.bin/playwright test --config scripts/behaviors/a2ui-browser.config.mjs
```

Set externally provisioned `UAR_BASE_URL`, `UAR_MODEL`, nonsecret `UAR_WORKSPACE_ID`, and fresh absolute `UAR_BROWSER_EVIDENCE_DIR`. Uses installed Chrome; no test credentials, mock responses, server startup or earlier scenarios. Evidence includes raw streams/action, screenshots, video, trace and reports. This tests one real renderer/control/continuation workflow, not all components, replay, browser engines or platforms.

The browser fixture uses an explicit valid component template with `urn:uar:a2ui:catalog:1`; the model still invokes the real renderer, and the fresh number is generated only after the form renders. Native-tool schema now advertises this browser-supported catalog alongside its existing basic-catalog URL. The owned local UAR must serve the current built frontend; the installed global `~/.uar/static` bundle can be older. This scenario does not substitute raw JSON for rendered controls.

Mac result: **A2UI-BROWSER-01 PASS**, all5Gherkin steps; selected test25.4seconds, Playwright28.4seconds, attempt007. Fresh606 button action produced625 in a distinct agent continuation. See `.prometheus/verification/a2ui-browser-01-macos-pass.json`; prior attempts retained. Completed for this manual sequence; no automatic rerun.

## Behavior: keyword skill detection and actual use

`skill-detection.cucumber.mjs` selects only `skill-detection.feature`. A disposable agent receives a parcel quote request without a skill ID or explicit attachment. Two disposable skills offer distinct parcel/gardening triggers. The current service/keyword strategy must select and activate only the parcel skill; the answer must contain its fresh private marker and correctly apply its randomly generated pricing recipe. Neither the recipe nor marker appears in the agent prompt or user request. Run inspection and streamed activation events are required alongside genuine inference.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/skill-detection.cucumber.mjs
```

Set externally provisioned `UAR_BASE_URL`, `UAR_MODEL`, nonsecret `UAR_WORKSPACE_ID`, and optional `UAR_CUCUMBER_REPORT`. No authentication in tests, global matching changes, forced skill preference/attachment, prior scenario reruns or restart. Only acknowledged disposable resources are removed, including after failure; raw exchanges remain under `.prometheus/verification/skill-detection/`. Scope is the configured keyword strategy, not semantic/LLM/embedding strategy parity or every skill.

Mac result: **SKILL-DETECTION-01 PASS**, 1scenario/5reportedsteps (four behavior steps plus cleanup), 5.655seconds on attempt002. Selected/implicitly activated parcel skill produced280 and its private marker. Initial404 setup failure retained. See `.prometheus/verification/skill-detection-01-macos-pass.json`. Completed for this manual sequence; no automatic rerun.

## Behavior: semantic skill selection without keyword matches

`semantic-skill.cucumber.mjs` selects only `semantic-skill.feature`. The disposable agent makes delivery-pricing and gardening skills equally eligible, without preference or attachment. A paraphrased request contains none of their exact trigger keywords and has a derived keyword score of zero. The BDD temporarily selects `local_embedding` with an explicit0.65 acceptance threshold, requires runtime evidence for that method, exact accepted/activated delivery skill and an answer applying its private random recipe/marker. The original matching configuration is saved before mutation and restored/read back in cleanup; disposable resources alone are deleted.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/semantic-skill.cucumber.mjs
```

Set `UAR_BASE_URL`, `UAR_MODEL`, nonsecret `UAR_WORKSPACE_ID`, and optional `UAR_CUCUMBER_REPORT`. Run serially against the owned manual UAR. No test authentication, paid embedding calls, earlier scenario reruns or restart. Evidence is retained under `.prometheus/verification/semantic-skill/`. Scope is one local embedding selection and genuine skill use, not every semantic strategy or paraphrase. A failed restoration is a test failure.

Mac result: **SEMANTIC-SKILL-01 PASS**, 1 scenario / 5 reported steps, 17.395 seconds on attempt002. Local embeddings at explicit threshold0.65 selected only delivery with zero trigger matches; private recipe/marker produced360. Initial0.5 false-positive failure retained. Original configuration restored; disposable resources deleted. See `.prometheus/verification/semantic-skill-01-macos-pass.json`. No automatic rerun.

## Behavior: bounded context with remembered follow-up facts

`context-budget.cucumber.mjs` selects only `context-budget.feature`. A disposable agent acknowledges a fresh reference and two quantities without repeating them. In the same server-held conversation, three separate acknowledged turns supply large irrelevant appendices, followed by a separate question that does not replay those facts. The keep-first-and-latest strategy must reduce genuinely over-budget context, retain the initial facts, and produce the correct reference/sum. Assertions cover reducer counts, final serialized request estimates and reported provider input/output usage.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/context-budget.cucumber.mjs
```

Set `UAR_BASE_URL`, `UAR_MODEL`, nonsecret `UAR_WORKSPACE_ID`, and optional `UAR_CUCUMBER_REPORT`. Serial execution temporarily changes only the chosen model's context_window to8192; each request reserves2048 output tokens. The API preserves existing credentials when api_key is omitted. The original nonsecret provider view is saved before the change and restored/read back afterward; the provider's operation receipt is refreshed after each configuration change. Cleanup deletes only the disposable agent. No test credentials, prior scenarios, restart or paid embedding calls. Evidence under `.prometheus/verification/context-budget/`. Token estimators are identified honestly; this does not establish exact-tokenizer guarantees or other context strategies.

Mac result: **CONTEXT-BUDGET-01 PASS**, 1 scenario / 6 reported steps, 18.413 seconds on attempt002. History12,869→4,830 within5,879 allowance; final request estimate5,339 and provider input4,689 fit6,144 budget. Correct retained reference/sum906. Initial fixture failure preserved, original model settings restored, disposable deleted. See `.prometheus/verification/context-budget-01-macos-pass.json`. Completed; no automatic rerun.

## Behavior: actual conversation summarization preserves facts

`conversation-summary.cucumber.mjs` selects only `conversation-summary.feature`. A disposable agent uses `summarize` with threshold2 and a2048-token summary allowance. A long first turn contains random booking facts; the assistant acknowledges without repeating them. The follow-up supplies no facts or client transcript replay. The BDD requires actual compressed context, completed summary-model calls, a summary-input digest binding the fact-bearing first turn, and the correct reference/headcount/delivery gate in the answer.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/conversation-summary.cucumber.mjs
```

Set `UAR_BASE_URL`, `UAR_MODEL`, nonsecret `UAR_WORKSPACE_ID`, and optional `UAR_CUCUMBER_REPORT`. No global settings change, test credentials, previous scenario reruns or restart. Only the disposable agent is deleted. Raw evidence under `.prometheus/verification/conversation-summary/` retains model streams and body-free summary receipts. A correct answer with unchanged history cannot pass; a selected summary strategy without a completed summary also cannot pass. Scope is this warm summarization workflow, not all retention strategies or long-term persistence.

Mac result: **CONVERSATION-SUMMARY-01 PASS**, 1 scenario / 5 reported steps, 10.430seconds on attempt002. Actual summary source binding and3,968→917estimated-token compression, correct booking reference/19attendees/GateQ. Initial fixture mismatch preserved; disposable deleted. Receipt `.prometheus/verification/conversation-summary-01-macos-pass.json`. Completed; no automatic rerun.

## Behavior: durable conversation recall after restart

`conversation-restart.cucumber.mjs` selects only `conversation-restart.feature`. A disposable agent acknowledges a fresh reference and quantities in one session without repeating them. The Mac host adapter gracefully stops only the process recorded in `target/prompt-macos-launch.json`, confirms exit, and launches the same binary/config/environment/working directory. It preserves the store and external identity proxy. After startup, the agent definition must be unchanged and a fresh follow-up using the original session ID must recover the exact reference and independently computed sum without transcript replay.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/conversation-restart.cucumber.mjs
```

Set `UAR_BASE_URL`, `UAR_MODEL`, nonsecret `UAR_WORKSPACE_ID`, and optional `UAR_CUCUMBER_REPORT`. The lifecycle adapter is deliberately Mac-only and uses the existing `python3` standard-library `ctypes` to read NUL-delimited process environment bytes from macOS. It binds the client endpoint to the owned proxy and UAR listeners before creating the agent and after restart. Port that adapter before running this scenario on Windows. Existing host environment values are carried opaquely in memory and never saved in evidence. No test authentication, force-kill, rebuild, store reset or previously verified scenario rerun. Evidence is saved before shutdown and after completion under `.prometheus/verification/conversation-restart/`. Cleanup deletes only the disposable agent and leaves UAR running. Scope is graceful process restart with the existing disk store, not crash recovery or reboot durability.

Mac result: **CONVERSATION-RESTART-01 PASS**, all 6 reported steps in 17.442 seconds on attempt002 after correcting normal-chat durable save/load. Prior failed attempt retained. See `.prometheus/verification/conversation-restart-01-macos-pass.json`. No previous scenario reruns; UAR stays running. Complete for this manual sequence.

## Concurrent conversation restart

`concurrent-conversation-restart.cucumber.mjs` selects one new scenario. Two genuine prompts give separate random references and quantities in one fresh session. Live SSE observations must show both RUN_STARTED events before either RUN_FINISHED. After both successful turns, the owned Mac UAR restarts with its store intact. A follow-up without history replay must recover both references and their correct total. Only the disposable agent is removed; UAR remains running. Uses the same nonsecret environment inputs and owned Mac lifecycle adapter as the prior restart case; no test credentials.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/concurrent-conversation-restart.cucumber.mjs
```

This observes one real concurrent interleaving, not an exhaustive race or crash-durability proof.

Mac result: **CONCURRENT-CONVERSATION-RESTART-01 PASS**, all 6 reported steps in 21.43 seconds on the first attempt. Both live runs overlapped; both fact sets and a total of 933 survived restart. Receipt `.prometheus/verification/concurrent-conversation-restart-01-macos-pass.json`. No previous scenario reruns or production edits; UAR stays running. Complete for this manual sequence.

## REST setting persistence through restart

`rest-setting-restart.cucumber.mjs` selects one new scenario: GET the current `/api/uar/user/settings` record, PUT only a different `prompt_caching_enabled` value, confirm it, restart the owned Mac UAR with the same store/config/binary, and compare the complete saved record after restart. Cleanup restores the original nullable preference and verifies readback. No provider calls or test credentials. This verifies the per-user REST preference's persistence, not caching effectiveness or all runtime settings.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/rest-setting-restart.cucumber.mjs
```

Set `UAR_BASE_URL`, `UAR_WORKSPACE_ID`, and optional `UAR_CUCUMBER_REPORT` using the retained local fixture. Raw exchanges and before-update/before-restart records are retained under `.prometheus/verification/rest-setting-restart/`. Only this scenario runs; UAR remains running afterward.

Mac result: **REST-SETTING-RESTART-01 PASS**, all 5 reported steps in 13.834 seconds on the first attempt. Changed nullable preference survived restart; original value restored. Receipt `.prometheus/verification/rest-setting-restart-01-macos-pass.json`. No prior scenario reruns; UAR stays running. Complete for this manual sequence.

## Retained math knowledge after restart

`knowledge-restart.cucumber.mjs` selects only the new retained-KB restart scenario. It uses the saved math agent, wheel-graph KB and indexed document recorded in the existing knowledge receipt. It snapshots the binding/index and real local embedding space, restarts the owned Mac UAR, verifies unchanged resources, then asks one fresh-session research-profile question. Acceptance requires the correct Lucas-number result, exact document-only seal and an actual RAG citation matching the retained KB/document. No KB creation, upload, rebinding, reindexing, test credentials or prior scenario rerun.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/knowledge-restart.cucumber.mjs
```

Use `UAR_BASE_URL`, `UAR_WORKSPACE_ID`, `UAR_MODEL` and optional `UAR_CUCUMBER_REPORT`. The local embedder must still match the retained index. One configured model inference occurs; no hosted embedding spend. Existing resources and UAR remain running. Scope excludes embedding-model regeneration and backend/platform parity.

Mac result: **KNOWLEDGE-RESTART-01 PASS**, all 5 reported steps in 22.95 seconds, first attempt. Correct retained-document mathematics and seal with matching retrieval citation after restart; existing agent/KB/document unchanged. Receipt `.prometheus/verification/knowledge-restart-01-macos-pass.json`. No previously verified scenario reruns; UAR stays running. Complete for this manual sequence.

## Selective KB unlink (Mac verified)

`node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/kb-unlink-one.cucumber.mjs` exercises only removal of one of two KB references. It uses a disposable agent/two KBs, fresh inference and matching source citations, then cleans up. Credentials stay outside the test. The current pass and retained first fixture failure are indexed in `.prometheus/verification/kb-unlink-one-01-macos-pass.json`. See `.prometheus/verification/behavior-status.md` for the maintained complete status catalog. Run only when explicitly selected; do not automatically replay the manual sequence.
