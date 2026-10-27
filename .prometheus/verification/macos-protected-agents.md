# Protected agents — Mac PASS

Command: `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/protected-agents.cucumber.mjs`

Observed output: **1 scenario (1 passed), 5 steps (5 passed), 0m 0.80s**.

- `DELETE /api/agents/default-agent`: HTTP403, `Agent 'default-agent' is built in and cannot be deleted`.
- `DELETE /api/agents/orchestrator-agent`: HTTP403, `Agent 'orchestrator-agent' is built in and cannot be deleted`.
- Both complete definitions matched their pre-deletion reads afterward.

[Pass receipt](protected-agents-01-macos-pass.json) · [Cucumber report](protected-agents-001-macos.cucumber.json)

Files added: `scripts/behaviors/features/protected-agents.feature`, `scripts/behaviors/steps/protected-agents.steps.mjs`, and `scripts/behaviors/protected-agents.cucumber.mjs`. README and existing OpenSpec delta describe the behavior; ledger and append-only memory log the result. No production code changed or unrelated work was added. Assertions enforce the requested refusal reason and unchanged definitions; an arbitrary authorization error cannot pass.

No provider calls, test credentials, restart, previous scenario rerun, commit or push. Other platforms and other agent lifecycle operations remain unverified by this scenario. UAR remains running. Stop for operator direction.
