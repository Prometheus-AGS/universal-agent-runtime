# AG-UI saved-agent streamed prompt — Mac PASS

Command: `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/agui.cucumber.mjs`

Observed output: **1 scenario (1 passed), 4 steps (4 passed), 0m 7.15s**. First attempt passed.

Saved mathematics agent `math-specialist-db64c2a8` used its unchanged `liter/MiniMax-M3` binding. AG-UI returned **16 content fragments** whose assembled answer **2173** matched independent arithmetic and contained the identity from saved instructions. Assistant-message start/content/end ordering and matching message identifiers passed; run/thread identities matched and RUN_FINISHED reported success.

[Pass receipt](agui-01-macos-pass.json) · [Raw evidence](agui/aa061402-829b-475d-887d-d8176243c5c6.json) · [Cucumber report](agui-001-macos.cucumber.json)

Files added: `scripts/behaviors/agui.cucumber.mjs`, `scripts/behaviors/features/agui-math.feature`, `scripts/behaviors/steps/agui-math.steps.mjs`. README and the existing OpenSpec delta describe this behavior; ledger, runtime pointer and append-only memory record the result. No production changes or unsolicited features. Assertions enforce the requested stream behavior and final answer; a bounded request deadline prevents an indefinite live call.

No verified scenarios reran. No test credentials, agent/model edits, restart, commit or push. Browser rendering, A2UI, reconnect/replay, cancellation, other platforms and full protocol conformance remain unverified by this scenario. UAR remains running; stopped for operator direction.
