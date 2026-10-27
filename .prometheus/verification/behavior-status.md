# Behavior status catalog

Updated 2026-10-26 (America/Chicago), through KB-UNLINK-ONE-01. Maintained manual coverage view; read each receipt for its exact time, artifact and source. No execution was performed to prepare this catalog.

## Totals and meaning

| Catalog | Total | Verified | Partial | Not verified / pending |
|---|---:|---:|---:|---:|
| Manual behavior labels | 33 | 33 | 0 | 0 |
| Manual user journeys already completed | 30 | 30 | — | 0 |
| Original bundled scenario IDs | 97 | 0 full-case claims | 24 | 73 |

These are overlapping views, not additive unique behaviors: **do not report 130 independent behaviors or 33/97 original cases passed**. B05–B08 split one journey into four behaviors. The original manifest expands 97 cases into 582 planned backend/repetition rows (two stores × three repetitions); that historical denominator is not a completed manual result.

**Verified** means the bounded manual behavior was demonstrated in its linked Mac receipt. **Partial** means recorded manual evidence overlaps an original case but does not establish its complete predicates. **Not verified** means no matching manual proof is credited here; it does not mean the feature is broken. No original case is certified across its full legacy contract. Failed attempts remain failed even when a later attempt passed. Historical passes are not a regression pass on later source.

The uncomfortable limit: many important paths still lack behavior evidence, including full SDK parity, owner-isolation/failure controls, team recovery and persistent memory. Manual operator-selected scenarios supersede the old packet/signature/inventory ceremony; retaining original case definitions does not revive that process or put authentication into tests. Identity remains external infrastructure.

## Platform and store scope

| Scope | Current behavior evidence | Outstanding disposition |
|---|---|---|
| macOS | 33 bounded manual behaviors, 30 journeys; genuine configured Liter/MiniMax-M3, durable Surreal, and Chrome for B18 | Every receipt binds its own artifact/configuration; no whole-current-source certification |
| Linux | No manual behavior credit in this catalog | Required platform follow-up; historical Docker timeout is not a functional result |
| iOS | No manual behavior credit in this catalog | Embedded runtime journeys remain unverified; historical library build pass is not runtime proof |
| Windows | No manual behavior credit | **BLOCKED** in recorded platform scope: no execution host; not silently relabeled deferred/pass |
| Android | No manual behavior credit | **DEFERRED** by operator; retain historical target-OpenSSL build failure |
| PostgreSQL + pgvector | No manual behavior credit | Surreal results do not establish adapter parity |

Platform dispositions come from [scope plan](../../.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/four-platform-parking-lot-priority-plan.md) and [implementation status](../../.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/four-platform-implementation-status.md). Their old execution procedures are historical. Preparation/build receipts are separate from behavior acceptance.

## Manual behavior table

| ID | Status | Behavior description | Saved evidence |
|---|---|---|---|
| B01 | Verified — Mac receipt | Prompt configured default agent | [prompt](2026-10-26T06-38-40.440Z-prompt-macos.json) |
| B02 | Verified — Mac receipt | Receive a useful incremental streamed answer | [stream](2026-10-26T06-56-36.601Z-streaming-macos.json) |
| B03 | Verified — Mac receipt | Create, persist and query a custom mathematics agent | [math](agents/math-specialist-db64c2a8.verified.json) |
| B04 | Verified — Mac receipt | Persist a writing specialist and route a writing request to it by intent | [writer](agents/writing-specialist.verified.json) |
| B05 | Verified — Mac receipt | Create a knowledge base | [kb](knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json) |
| B06 | Verified — Mac receipt | Add and genuinely index a document in a knowledge base | [kb](knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json) |
| B07 | Verified — Mac receipt | Persist a knowledge-base reference on an existing agent | [kb](knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json) |
| B08 | Verified — Mac receipt | Answer using retrieved KB content with matching source provenance | [kb](knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json) |
| B09 | Verified — Mac receipt | Execute math and writing team members concurrently, then merge both real outputs into an article | [TEAM-01 PASS](team-01-macos-pass.json) |
| B10 | Verified — Mac receipt | Continue a saved math-agent conversation using server-held history without client transcript replay | [CONTINUITY-01 PASS](continuity-01-macos-pass.json) |
| B11 | Verified — Mac receipt | Prevent deletion of default and orchestrator agents while preserving their definitions | [PROTECTED-AGENTS-01 PASS](protected-agents-01-macos-pass.json) |
| B12 | Verified — Mac receipt | Update custom-agent instructions, observe changed inference behavior, and delete the disposable agent | [AGENT-LIFECYCLE-01 PASS](agent-lifecycle-01-macos-pass.json) |
| B13 | Verified — Mac receipt | Prompt the saved math agent through the Anthropic-compatible Messages API | [ANTHROPIC-01 PASS](anthropic-01-macos-pass.json) |
| B14 | Verified — Mac receipt | Prompt the saved math agent through A2A v1 and receive a completed task with its answer | [A2A-01 PASS](a2a-01-macos-pass.json) |
| B15 | Verified — Mac receipt | Prompt the saved math agent through ACP and receive its correct answer with completed run status | [ACP-01 PASS](acp-01-macos-pass.json) |
| B16 | Verified — Mac receipt | Prompt the saved math agent through AG-UI and assemble a correct streamed answer with correlated successful completion | [AGUI-01 PASS](agui-01-macos-pass.json) |
| B17 | Verified — Mac receipt | Receive a genuinely published A2UI greeting surface and matching state updates through AG-UI agui_spec stream mode | [A2UI-01 PASS](a2ui-01-macos-pass.json) |
| B18 | Verified — Mac receipt | Render A2UI controls in Chrome and deliver a fresh user action to the agent for a correct continuation answer | [A2UI-BROWSER-01 PASS](a2ui-browser-01-macos-pass.json) |
| B19 | Verified — Mac receipt | Implicitly detect and activate the matching keyword skill, then answer using its private recipe and marker | [SKILL-DETECTION-01 PASS](skill-detection-01-macos-pass.json) |
| B20 | Verified — Mac receipt | Select and use the matching skill from a paraphrased prompt with zero trigger matches, using local embeddings at threshold0.65 | [SEMANTIC-SKILL-01 PASS](semantic-skill-01-macos-pass.json) |
| B21 | Verified — Mac receipt | Reduce oversized context within the token allowance while preserving original facts for a follow-up | [CONTEXT-BUDGET-01 PASS](context-budget-01-macos-pass.json) |
| B22 | Verified — Mac receipt | Generate a summary of the fact-bearing conversation turn and preserve its key facts for a follow-up | [CONVERSATION-SUMMARY-01 PASS](conversation-summary-01-macos-pass.json) |
| B23 | Verified — Mac receipt | Preserve a conversation across a graceful UAR restart and recall earlier facts without replay | [CONVERSATION-RESTART-01 PASS](conversation-restart-01-macos-pass.json) |
| B24 | Verified — Mac receipt | Retain both overlapping same-conversation turns across a graceful restart and recall their facts | [CONCURRENT-CONVERSATION-RESTART-01 PASS](concurrent-conversation-restart-01-macos-pass.json) |
| B25 | Verified — Mac receipt | Update a user setting through REST and retain it across a graceful UAR restart | [REST-SETTING-RESTART-01 PASS](rest-setting-restart-01-macos-pass.json) |
| B26 | Verified — Mac receipt | Use the saved math agent’s retained knowledge base after UAR restart with matching retrieval evidence | [KNOWLEDGE-RESTART-01 PASS](knowledge-restart-01-macos-pass.json) |
| B27 | Verified — Mac receipt | Change embedding model/dimension, reject the incompatible index, regenerate the retained document and retrieve its original private fact | [KNOWLEDGE-REGENERATION-01 PASS](knowledge-regeneration-01-macos-pass.json) |
| B28 | Verified — Mac receipt | Delete an indexed KB document so it disappears from retrieval while another document in the same KB remains searchable | [KNOWLEDGE-DOCUMENT-DELETE-01 PASS](knowledge-document-delete-01-macos-pass.json) |
| B29 | Verified — Mac receipt | Delete a linked KB while its unchanged agent still answers in a fresh conversation without the deleted knowledge or citations | [LINKED-KB-DELETE-01 PASS](linked-kb-delete-01-macos-pass.json) |
| B30 | Verified — Mac receipt | Unlink a KB so the agent stops using it in a fresh conversation while the unchanged KB remains directly searchable | [KB-UNLINK-01 PASS](kb-unlink-01-macos-pass.json) |
| B31 | Verified — Mac receipt | Relink an existing KB to an agent and restore grounded use of its unchanged document in a fresh conversation | [KB-RELINK-01 PASS](kb-relink-01-macos-pass.json) |
| B32 | Verified — Mac receipt | Link two KBs to an agent and combine private facts from both with matching source citations | [TWO-KB-ANSWER-01 PASS](two-kb-answer-01-macos-pass.json) |
| B33 | Verified — Mac receipt | Unlink one of two agent KBs; a fresh answer uses only the remaining KB, while the removed KB remains intact and directly searchable. | [KB-UNLINK-ONE-01 PASS](kb-unlink-one-01-macos-pass.json) |

## Original scenario table — all 97 IDs

IDs and order are preserved from [original manifest](../../scripts/acceptance/fixtures/scenarios.json). Descriptions follow the [coverage matrix](../../.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-coverage-matrix.md) and original case source. Every U05 instance includes an input-dependent first answer, same-context follow-up, intended agent/model and protocol terminal/correlation; custom instances also require saved instructions and cleanup. A single response does not close that instance. HTTP, SDK and gRPC entries remain separate.

Evidence references B01–B33 resolve to the manual table above. B16 exercises the dedicated AG-UI agent endpoint, not the native run/stream endpoint; B17 uses OpenAI chat with agui_spec, not every AG-UI variant. Neither is spread across unrelated interface rows. B18's A2UI browser interaction does not establish all U24 browser management/history workflows. B19/B20 do not establish filesystem discovery, every policy, or the same-prompt lexical/semantic disagreement case.

| Original ID | Behavior description | Coverage status | Manual overlap and remaining work |
|---|---|---|---|
| `U01.configured-prompt` | Configure a genuine model and obtain a correct input-dependent answer. | Partial — not fully verified | B01: Configured model produces useful answer; complete configuration/correlation predicates remain. |
| `U02.session-continuation` | Recall prior turns before and after restart with consistent durable session history. | Partial — not fully verified | B10, B23–B24: Warm and graceful-restart recall observed; complete canonical-message/run-catalog durability contract remains. |
| `U03.custom-agent` | Create/read/list/update, restart, use and delete a saved custom agent. | Partial — not fully verified | B03–B04, B07, B12: Saved specialist, reload, instruction update and deletion observed across separate journeys; combined revision/cold lifecycle remains. |
| `U04.protected-agents` | Refuse built-in agent deletion across management paths while keeping both usable. | Partial — not fully verified | B11: Both built-in DELETE refusals and unchanged definitions observed; all management paths, post-refusal prompting and restart remain. |
| `U20.effective-configuration` | Apply REST configuration precedence, preserve valid state on failure and reload after restart. | Partial — not fully verified | B25, B27; original KB setup: One user preference survives restart; embedding settings take effect. Chat-model precedence and failed-write controls remain. |
| `U21.startup-maintenance` | Seed once, preserve operator settings and run durable idempotent startup/maintenance. | Partial — not fully verified | B03, B23–B26: Retained records survive warm restart; empty-store seed uniqueness and named maintenance/migrations remain. |
| `U06.live-roster-intent` | Route to the eligible live specialist roster, including changes and unmatched intent. | Partial — not fully verified | B04: Writer selected over math and executed; roster changes/removal and ambiguous/unmatched routing remain. |
| `U07.official-agui-selection-replay` | Deliver/render selected-agent events with follow-up and reconnect without duplicate work. | Partial — not fully verified | B04; B16 related: Selection-before-answer observed; selected-agent browser indicator and reconnect/replay remain. Direct B16 prompt is not orchestrator replay. |
| `U05.chat.json.default` | Native chat endpoint: nonstream JSON, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.chat.json.custom` | Native chat endpoint: nonstream JSON, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.chat.openai.default` | Native chat endpoint: OpenAI SSE, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.chat.openai.custom` | Native chat endpoint: OpenAI SSE, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.chat.agui.default` | Native chat endpoint: AG-UI stream, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.chat.agui.custom` | Native chat endpoint: AG-UI stream, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.chat.dual.default` | Native chat endpoint: dual stream, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.chat.dual.custom` | Native chat endpoint: dual stream, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.chat.agui_spec.default` | Native chat endpoint: agui_spec stream, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.chat.agui_spec.custom` | Native chat endpoint: agui_spec stream, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.openai.json.default` | OpenAI-compatible chat endpoint: nonstream JSON, default agent; two-turn parity. | Partial — not fully verified | B01: Default nonstream answer observed; complete two-turn interface parity remains. |
| `U05.openai.json.custom` | OpenAI-compatible chat endpoint: nonstream JSON, custom agent; two-turn parity. | Partial — not fully verified | B03, B08, B10, B12: Saved-agent answers, warm continuity and lifecycle observed; complete original correlated parity task remains. |
| `U05.openai.openai.default` | OpenAI-compatible chat endpoint: OpenAI SSE, default agent; two-turn parity. | Partial — not fully verified | B02: Useful streamed chunks, stop and DONE observed; follow-up continuity remains. |
| `U05.openai.openai.custom` | OpenAI-compatible chat endpoint: OpenAI SSE, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.openai.agui.default` | OpenAI-compatible chat endpoint: AG-UI stream, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.openai.agui.custom` | OpenAI-compatible chat endpoint: AG-UI stream, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.openai.dual.default` | OpenAI-compatible chat endpoint: dual stream, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.openai.dual.custom` | OpenAI-compatible chat endpoint: dual stream, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.openai.agui_spec.default` | OpenAI-compatible chat endpoint: agui_spec stream, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.openai.agui_spec.custom` | OpenAI-compatible chat endpoint: agui_spec stream, custom agent; two-turn parity. | Partial — not fully verified | B17: Custom-agent A2UI publication through chat/completions agui_spec observed; original two-turn parity task remains. |
| `U05.native.agui.default` | Native run/stream endpoints: AG-UI stream, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.native.agui.custom` | Native run/stream endpoints: AG-UI stream, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.native.agui_spec.default` | Native run/stream endpoints: agui_spec stream, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.native.agui_spec.custom` | Native run/stream endpoints: agui_spec stream, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.anthropic.json.default` | Anthropic-compatible Messages: nonstream JSON, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.anthropic.json.custom` | Anthropic-compatible Messages: nonstream JSON, custom agent; two-turn parity. | Partial — not fully verified | B13: Saved-agent Messages answer and terminal observed; follow-up context and complete parity remain. |
| `U05.anthropic.stream.default` | Anthropic-compatible Messages: streaming, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.anthropic.stream.custom` | Anthropic-compatible Messages: streaming, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.a2a_legacy.json.default` | Legacy A2A: nonstream JSON, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.a2a_legacy.json.custom` | Legacy A2A: nonstream JSON, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.a2a_v1.json.default` | A2A v1: nonstream JSON, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.a2a_v1.json.custom` | A2A v1: nonstream JSON, custom agent; two-turn parity. | Partial — not fully verified | B14: Saved-agent completed A2A task observed; task continuation and complete parity remain. |
| `U05.a2a_v1.stream.default` | A2A v1: streaming, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.a2a_v1.stream.custom` | A2A v1: streaming, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.acp.json.default` | ACP: nonstream JSON, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.acp.json.custom` | ACP: nonstream JSON, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.acp.stream.default` | ACP: streaming, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.acp.stream.custom` | ACP: streaming, custom agent; two-turn parity. | Partial — not fully verified | B15: Saved-agent ACP streamed answer/Done observed; original adapter envelope and follow-up parity remain. |
| `U05.anthropic.json.implicit` | Anthropic-compatible Messages: nonstream JSON, implicit default-agent selection; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.anthropic.stream.implicit` | Anthropic-compatible Messages: streaming, implicit default-agent selection; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.python_sdk.json.default` | Shipped Python SDK: nonstream JSON, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.python_sdk.json.custom` | Shipped Python SDK: nonstream JSON, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.python_sdk.stream.default` | Shipped Python SDK: streaming, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.python_sdk.stream.custom` | Shipped Python SDK: streaming, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.typescript_sdk.json.default` | Shipped TypeScript SDK: nonstream JSON, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.typescript_sdk.json.custom` | Shipped TypeScript SDK: nonstream JSON, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.typescript_sdk.stream.default` | Shipped TypeScript SDK: streaming, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.typescript_sdk.stream.custom` | Shipped TypeScript SDK: streaming, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.rust_sdk.json.default` | Shipped Rust SDK: nonstream JSON, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.rust_sdk.json.custom` | Shipped Rust SDK: nonstream JSON, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.rust_sdk.stream.default` | Shipped Rust SDK: streaming, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.rust_sdk.stream.custom` | Shipped Rust SDK: streaming, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.grpc_legacy.json.default` | Legacy gRPC: nonstream JSON, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.grpc_legacy.json.custom` | Legacy gRPC: nonstream JSON, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.grpc_legacy.stream.default` | Legacy gRPC: streaming, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.grpc_legacy.stream.custom` | Legacy gRPC: streaming, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.grpc_v1.json.default` | gRPC v1: nonstream JSON, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.grpc_v1.json.custom` | gRPC v1: nonstream JSON, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.grpc_v1.stream.default` | gRPC v1: streaming, default agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U05.grpc_v1.stream.custom` | gRPC v1: streaming, custom agent; two-turn parity. | Not verified | No matching manual result credited; full described behavior remains. |
| `U08.installed-standard-a2ui-action` | Render A2UI, accept fresh input and deliver a bound action to genuine continuation. | Partial — not fully verified | B17–B18: Published surface, Chrome render/action and genuine continuation observed; complete original installed-client predicates remain. |
| `U24.shipped-ui.default` | Use shipped browser configuration, prompting, streaming and saved history with default agent. | Not verified | No matching manual result credited; full described behavior remains. |
| `U24.shipped-ui.custom` | Use shipped browser configuration, prompting, streaming and saved history with custom agent. | Not verified | No matching manual result credited; full described behavior remains. |
| `U24.shipped-ui.orchestrator` | Use shipped browser configuration, routing, streaming and history with orchestrator. | Not verified | No matching manual result credited; full described behavior remains. |
| `U09.skill-discovery-scope` | Discover top-level/nested/allowed-symlink skills once with correct scope and refresh behavior. | Not verified | No matching manual result credited; full described behavior remains. |
| `U10.skill-matching-strategies` | Exercise every exposed skill-matching strategy with discriminating candidates and real use. | Partial — not fully verified | B19–B20: Keyword and calibrated local-embedding examples observed; every other exposed matching strategy/ranking contract remains. |
| `U11.skill-activation-governance` | Enforce skill policy modes, catalog/overlay admission, scope and activation budgets. | Partial — not fully verified | B19–B20: Matching skill implicitly activated and used; policy modes, scope/overlay combinations and budget denial remain. |
| `U11.bounded-skill-reattachment` | Retain/re-attach selected skill context within governed bounds across later turns. | Not verified | No matching manual result credited; full described behavior remains. |
| `U12.context-strategy-effects` | Exercise all structural context strategies and thresholds without losing protected groups. | Partial — not fully verified | B21–B22: Context reduction and fact-bearing summary observed; all structural strategies/thresholds and protected groups remain. |
| `U13.whole-request-budget` | Enforce whole-request limits including tools, retrieval, context and output reservation. | Partial — not fully verified | B21: Useful reduced request stays within input allowance; exact-boundary/refusal and full tool/retrieval/output accounting cases remain. |
| `U14.knowledge-lifecycle` | Create/index/bind/query/update/delete knowledge with current citations and retained-fact controls. | Partial — not fully verified | B05–B08, B26, B28–B32: Create/index/grounding/restart/delete/link/relink/two-KB use observed; document-update/replacement and complete lifecycle predicates remain. |
| `U15.embedding-regeneration` | Rebuild retained sources across same/different-dimension model changes and interrupted jobs. | Partial — not fully verified | B27: Different-dimension model transition rejects old index and regenerates retained document; same-dimension change and interrupted rebuild/resume remain. |
| `U16.knowledge-adapter-contract` | Verify the knowledge persistence/adapter contract on both declared durable stores. | Not verified | No matching manual result credited; full described behavior remains. |
| `U17.team-dependencies` | Run independent team work concurrently and enforce dependent joins using genuine outputs. | Partial — not fully verified | B09: Overlapping specialists and merged real outputs observed; complete dependency scheduling/attempt/duplicate-effect predicates remain. |
| `U18.team-multicall-context` | Preserve authorized child context across rounds/attempts with provenance and team isolation. | Not verified | No matching manual result credited; full described behavior remains. |
| `U19.team-cancel-recovery` | Cancel/recover team execution with honest outcomes and no duplicate uncertain effects. | Not verified | No matching manual result credited; full described behavior remains. |
| `U22.native-tool-approval` | Discover/call native tools with approval/denial, paired results and cancellation. | Partial — not fully verified | B04, B17–B18: spawn_agent approve/resume and native A2UI action observed; denial/nonexecution, cancellation and uncertain effects remain. |
| `U22.mcp-tool-approval` | Discover/call actual MCP tools with approval/denial and correctly consumed results. | Not verified | No matching manual result credited; full described behavior remains. |
| `U23.owner-isolation` | Prevent cross-owner reads, continuation and cancellation of protected records/runs. | Not verified | No matching manual result credited; full described behavior remains. |
| `U23.failure-honesty` | Report provider disconnect/denial and durable-write failure without false success. | Not verified | No matching manual result credited; full described behavior remains. |
| `U23.unknown-tool-effect` | Keep unknown tool effects unresolved instead of silently repeating them. | Not verified | No matching manual result credited; full described behavior remains. |
| `U25.memory-lifecycle` | Capture/recall/update/delete durable scoped memory across sessions and restart. | Not verified | No matching manual result credited; full described behavior remains. |
| `U25.memory-consent` | Honor owner consent, opt-out and retention without late capture after restart. | Not verified | No matching manual result credited; full described behavior remains. |
| `U25.formats-vision` | Observe real image content and supported structured output through advertised routes. | Not verified | No matching manual result credited; full described behavior remains. |
| `U25.routing-capabilities` | Route to an actually eligible provider/model under capability and cooldown rules. | Not verified | No matching manual result credited; full described behavior remains. |
| `U25.lexical-semantic-disagreement` | Use one discriminating prompt to prove lexical/semantic scoring selects different winners. | Not verified | No matching manual result credited; full described behavior remains. |
| `U25.context-threshold-restart` | Keep below-threshold history unchanged and recall it after restart for all six strategies. | Not verified | No matching manual result credited; full described behavior remains. |
| `U22.skill-activation-count-limit` | Reject activation beyond the configured skill-count limit without extra effects. | Not verified | No matching manual result credited; full described behavior remains. |
| `U25.protected-multimodal-context` | Preserve required multimodal/tool context groups through reduction and durability. | Not verified | No matching manual result credited; full described behavior remains. |

## Maintenance and provenance

After each selected journey, update its B-row with the immutable receipt, bounded claim and result; update totals and only the original cases whose predicates it actually covers. Add a B-row for a newly tracked behavior rather than renumbering earlier rows. Preserve failed attempts and source/platform limits. Do not infer complete original-case acceptance from topical similarity or simply adding narrower passes together.

[Mac ledger](macos-behavior-ledger.md) is the chronology behind this view. Its “11 case IDs / 86 remaining” paragraph and [initial machine ledger](macos-behavior-ledger-2026-10-26.json) describe only the first eight behaviors/five journeys, not today's totals. Later dated entries add the results reflected here. The original full-case count remains zero; current overlap is 24 partial cases and 73 without credited matching manual proof. Independent diagnostic/build results remain in their original [sweep report](../../.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-independent-sweep-001-results.md), outside these behavior totals.

This change writes only this catalog. No product code, guards, canonical waypoint or other authors' files changed. Source/document inspection and documentation writing only; no tests, review gate, lint, build, service/API call or verification command ran. Runtime behavior remains unverified beyond the saved receipts; this documentation reconciliation claims no additional execution.
