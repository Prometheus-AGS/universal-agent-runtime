# ACP saved-agent prompt — Mac PASS

Command: `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/acp.cucumber.mjs`

Observed output: **1 scenario (1 passed), 4 steps (4 passed), 0m 3.827s**. First attempt passed.

ACP created a session for saved mathematics agent `math-specialist-db64c2a8`, then streamed its response to a fresh arithmetic question using its unchanged `liter/MiniMax-M3` binding. The final answer **733** matched independent arithmetic and included the specialist marker from saved instructions. A successful terminal event and ACP runs/get confirmed the same run completed for the selected agent.

[Pass receipt](acp-01-macos-pass.json) · [Raw evidence](acp/1f97faad-8008-4168-883c-987651e7a0cb.json) · [Cucumber report](acp-001-macos.cucumber.json)

Files added: `scripts/behaviors/acp.cucumber.mjs`, `scripts/behaviors/features/acp-math.feature`, `scripts/behaviors/steps/acp-math.steps.mjs`. README and existing OpenSpec delta describe the selected behavior. Ledger, runtime pointer and append-only memory record the result. No production edits or unsolicited feature changes. Assertions enforce successful ACP completion, selected agent and correct final answer; request deadlines bound this live scenario.

No previously verified scenario reran. No test credentials, agent/model edits, service restart, commit or push. Continuation, cancellation, tools, other platforms and full protocol conformance remain unverified by this scenario. UAR remains running. Stopped for operator direction.
