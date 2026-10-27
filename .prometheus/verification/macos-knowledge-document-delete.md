# Knowledge document deletion on Mac

KNOWLEDGE-DOCUMENT-DELETE-01 verifies that deleting one indexed document removes its content from retrieval while another document in the same knowledge base remains searchable.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/knowledge-document-delete.cucumber.mjs
```

Use the retained UAR_BASE_URL and UAR_WORKSPACE_ID. The scenario creates a disposable KB with two tiny notes and fresh private seals. It requires both in the same query before deletion, DELETE204 for the target, then no target ID/seal in that unchanged query while the retained control is returned. Target GET404, a one-document list, unchanged control record and surviving KB reinforce the result. Cleanup removes the disposable KB. No credentials, host provisioning, provider spending, settings changes or restarts; local embedding only. No prior scenario reruns.

Scope is sequential deletion of an indexed document on this Mac runtime. Concurrent ingestion/deletion, restart durability and other backends/platforms remain unverified.

Result: PASS on Mac, 2026-10-26T19:44:56.458617-05:00. First run: `1 scenario (1 passed); 5 steps (5 passed); 0m 2.925s`, exit0. Both documents were retrieved before deletion; only the unchanged control remained afterward. Target GET404 and KB document_count1 confirmed. Cleanup succeeded, UAR PID97354 remains running. No failures, repairs or previous scenario reruns. [PASS receipt](knowledge-document-delete-01-macos-pass.json).
