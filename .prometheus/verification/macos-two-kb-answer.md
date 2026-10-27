# Two linked knowledge bases on Mac

TWO-KB-ANSWER-01 verifies that one agent combines private facts from two separate linked KBs in a grounded answer.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/two-kb-answer.cucumber.mjs
```

Use retained UAR_BASE_URL, UAR_WORKSPACE_ID and UAR_MODEL. Create two disposable KBs containing separate inventory quantities and unique seals, then link both to a disposable agent. A fresh question supplies neither quantity nor seal. Require both source quantities, exact seals, correct total, and answer citations matching actual runtime retrieval from each distinct KB/document. No citations from unrelated KBs; original KB/document snapshots remain unchanged. Clean up the agent and both KBs after assertions.

Local embeddings, genuine configured subscription inference, no test credentials, global settings changes, restart, build, production edits or previous scenario reruns. Scope is one small two-source synthesis on this Mac runtime, not arbitrary-corpus recall or all backends/platforms.

Result: PASS on Mac, 2026-10-26T20:29:35.070003-05:00. First run: `1 scenario (1 passed); 5 steps (5 passed); 0m 8.488s`, exit0. The answer combined 233 + 864 = 1097, returned both private seals and cited the matching document from each KB. Both KB/document snapshots stayed unchanged; all citations belonged to the two selected KBs. Disposable agent and KBs cleaned up; UAR PID21601 remains running. No failures, repairs, rebuild, restart or earlier scenario reruns. [PASS receipt](two-kb-answer-01-macos-pass.json).
