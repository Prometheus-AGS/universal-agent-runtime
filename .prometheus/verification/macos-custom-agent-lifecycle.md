# Custom agent update and deletion — Mac PASS

Command: `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/agent-lifecycle.cucumber.mjs`

Observed output: **1 scenario (1 passed), 9 steps (9 passed), 0m 6.580s**.

Disposable agent: `disposable-lifecycle-3c2e587a-0482-46d5-bf90-ddb26dcfca61`.

- Original instructions produced answer **177** and the original random marker.
- PATCH and GET confirmed the new instructions were saved.
- The identical user prompt in a fresh conversation produced answer **805** and the new random marker. Neither rule nor marker appeared in the user prompt.
- DELETE returned204; a subsequent GET returned404. The disposable agent is gone.

[Pass receipt](agent-lifecycle-01-macos-pass.json) · [Raw evidence](agent-lifecycle/disposable-lifecycle-3c2e587a-0482-46d5-bf90-ddb26dcfca61.json) · [Cucumber report](agent-lifecycle-002-macos.cucumber.json)

Files added: `scripts/behaviors/features/custom-agent-lifecycle.feature`, `scripts/behaviors/steps/custom-agent-lifecycle.steps.mjs`, `scripts/behaviors/agent-lifecycle.cucumber.mjs`. README, existing OpenSpec delta, ledger and append-only memory document the behavior/result. No production code or unrelated work changed.

The first attempt returned correct JSON in a fence followed by prose; its strict parser failed and its disposable agent was cleaned up. The corrected parser accepts bare JSON or exactly one final JSON fence, retaining exact marker and arithmetic assertions. Source review also corrected owned cleanup recording onHTTP201 before body parsing and bounded two-request steps to their enclosing timeout; these address actual creation-response and asynchronous timeout boundaries. Final source reviews found no remaining actionable findings.

No earlier scenario reran. No test credentials, restart, commit or push. Existing agents and UAR remain intact. This verifies ordinary successful instruction update/deletion on Mac, not concurrent-update conflicts, restart durability or other platforms. Stopped for operator direction.
