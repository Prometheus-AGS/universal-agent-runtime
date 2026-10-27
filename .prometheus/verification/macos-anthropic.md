# Anthropic-compatible saved-agent prompt — Mac PASS

Command: `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/anthropic.cucumber.mjs`

Observed output: **1 scenario (1 passed), 4 steps (4 passed), 0m 11.822s**. First attempt passed.

The saved mathematics agent `math-specialist-db64c2a8` answered through `POST /v1/messages` using the existing `liter/MiniMax-M3` model. Its final answer **1400** matched independent arithmetic and carried the saved specialist marker. HTTP200, Anthropic assistant message/text blocks, message identity, model and end_turn were checked. The marker was read from saved instructions and was not supplied in the user prompt.

[Pass receipt](anthropic-01-macos-pass.json) · [Raw evidence](anthropic/e09d47ff-ef92-4fbe-b53e-c7afb39e0f56.json) · [Cucumber report](anthropic-001-macos.cucumber.json)

Files added: `scripts/behaviors/features/anthropic-math.feature`, `scripts/behaviors/steps/anthropic-math.steps.mjs`, `scripts/behaviors/anthropic.cucumber.mjs`. README and existing OpenSpec delta document the behavior; ledger and append-only memory record the result. No production code, unrelated feature or speculative guard was added. Response assertions enforce the requested API/agent/answer contract; only completed final content is accepted.

No previously verified scenario reran. No test credentials, service restart, agent changes, commit or push. The scope is nonstreaming custom-agent Messages on Mac, not Anthropic streaming, all protocol options, another platform or an Anthropic-hosted provider. UAR remains running; stopped for operator direction.
