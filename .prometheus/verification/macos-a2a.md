# A2A saved-agent prompt — Mac PASS

Command: `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/a2a.cucumber.mjs`

Observed output: **1 scenario (1 passed), 4 steps (4 passed), 0m 4.539s**. First attempt passed.

A2A v1 `SendMessage` targeted saved mathematics agent `math-specialist-db64c2a8` using its unchanged configured `liter/MiniMax-M3` binding. The completed task returned answer **898**, matching independent arithmetic, and the specialist marker from saved instructions. JSON-RPC correlation, selected-agent metadata, completed state and message task/context identities were verified.

[Pass receipt](a2a-01-macos-pass.json) · [Raw evidence](a2a/f5c001b0-9d15-4548-af63-345f8211f98b.json) · [Cucumber report](a2a-001-macos.cucumber.json)

Files added: `scripts/behaviors/features/a2a-math.feature`, `scripts/behaviors/steps/a2a-math.steps.mjs`, `scripts/behaviors/a2a.cucumber.mjs`. README and the existing OpenSpec delta describe the behavior; ledger and append-only memory record the pass. No production code, unrelated feature or speculative guard was added. Assertions enforce the selected successful A2A task, saved agent and correct final answer.

No verified scenarios reran. No test credentials, model/agent changes, restart, commit or push. Streaming, task continuation/cancellation, legacy A2A, other platforms and full protocol conformance are unverified by this scenario. UAR remains running; stopped for operator direction.
