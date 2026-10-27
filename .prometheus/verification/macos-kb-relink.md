# Agent KB relinking on Mac

KB-RELINK-01 verifies that restoring an agent's original KB link restores grounded answers from its unchanged indexed document.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/kb-relink.cucumber.mjs
```

Use retained UAR_BASE_URL, UAR_WORKSPACE_ID and UAR_MODEL. Setup creates disposable resources and removes the original link through REST, confirming selected/empty policy; it makes no inference call to recertify unlinking. Restore only the original KB ID, require persisted link/policy agreement and unchanged prompt/route/runtime, then make one fresh-session inference request. Require correct arithmetic, exact private seal and matching actual KB/document citation, with no unrelated KB citations. KB/document snapshots must remain unchanged; no reupload or regeneration. Clean up only owned resources afterward.

No credentials, global settings changes, restart, hosted embeddings, production edits or previous scenario reruns. Scope is one explicit relink on this Mac runtime; other policy/platform/backend combinations remain unverified.

Result: PASS on Mac, 2026-10-26T20:13:44.785282-05:00. First attempt: `1 scenario (1 passed); 4 steps (4 passed); 0m 7.925s`, exit0. One fresh answer returned correct411 plus the exact private seal and matching citation from the restoredKB/originaldocument only. Selected policy and saved link agreed; originalKB/document snapshots unchanged. Disposable resources cleaned up; UAR PID21601 remains running. No failures, repairs, rebuild, restart or earlier scenario reruns. [PASS receipt](kb-relink-01-macos-pass.json).
