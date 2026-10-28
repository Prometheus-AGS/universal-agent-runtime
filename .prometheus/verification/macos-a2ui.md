# A2UI response through AG-UI — Mac PASS

Command: `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/a2ui.cucumber.mjs`

Observed output: **1 scenario (1 passed), 4 steps (4 passed), 0m 6.824s** on attempt002.

The configured `liter/MiniMax-M3` model called the real `a2ui_render` tool. Its host-published A2UI v0.9 artifact and ordered create-surface/component STATE_DELTA events matched the requested Text root greeting **Welcome, visitor ee4cdb66!**. Run identity, successful tool result, publication metadata and successful completion passed. Both attempts' disposable agents were deleted; existing agents and services remain intact.

[Pass receipt](a2ui-01-macos-pass.json) · [Published surface](a2ui-01-macos-surface.json) · [Raw evidence](a2ui/disposable-a2ui-574fb622-a5d8-447e-831a-254a18835dbe.json) · [Cucumber report](a2ui-002-macos.cucumber.json)

Attempt001 published the surface but the new test parser incorrectly treated the multiplexed runtime.run completion metadata as a second AG-UI event after completion. The only correction separated runtime metadata from AG-UI frames and requires matching run identity/completed status for the trailing runtime.run receipt. All A2UI publication assertions remained. [Original failure](a2ui-001-macos.cucumber.json) is retained.

Files added: `scripts/behaviors/a2ui.cucumber.mjs`, `scripts/behaviors/features/a2ui-response.feature`, `scripts/behaviors/steps/a2ui-response.steps.mjs`. README/OpenSpec describe the behavior; ledger/runtime pointer/append-only memories record results. No production edits or unrelated features. Assertions enforce genuine host publication and expected surface content; cleanup is limited to confirmed scenario-created agents, and the metadata distinction addresses the observed first failure.

Scope matters: this uses `/v1/chat/completions` with `stream_mode: "agui_spec"`. The dedicated `/api/uar/ag-ui/agents/{agent_id}` projector currently omits A2UI artifact/state events in source and is not certified for A2UI here. Browser rendering, user actions, reconnect and other platforms remain unverified. No prose-only or synthetic test-trigger response counts as publication.

No prior verified scenario reran. No test credentials, service restart, commit or push. UAR remains running; stopped for operator direction.
