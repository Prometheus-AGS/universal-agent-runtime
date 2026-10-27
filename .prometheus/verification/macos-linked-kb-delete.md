# Linked KB deletion on Mac

LINKED-KB-DELETE-01 verifies a disposable agent still answers after its linked KB is deleted, without using the deleted knowledge.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/linked-kb-delete.cucumber.mjs
```

Use the retained UAR_BASE_URL, UAR_WORKSPACE_ID and UAR_MODEL. The scenario uploads a tiny private-seal document and creates a disposable agent linked only to that KB. A real pre-deletion prompt must return the correct sum and private seal with a matching runtime citation. It deletes the KB and confirms the agent definition/reference is unchanged. The identical prompt in a fresh conversation must still calculate correctly but return null for the unavailable seal/citation, with no RAG citations or seal in completion/events. Cleanup removes owned temporary resources.

No test credentials, global settings changes, restart, hosted embeddings, saved-agent/KB changes, or earlier scenario reruns. Configured subscription inference is genuine. Scope does not include purging facts from existing conversations or all backend/platform behavior.

Result: PASS on Mac, 2026-10-26T19:49:08.234441-05:00. First run: `1 scenario (1 passed); 5 steps (5 passed); 0m 14.145s`, exit0. Before deletion the answer contained476 and the actual document seal/citation; afterward the unchanged agent returned476 and null seal/citation in a fresh session. Runtime citations were empty and deleted seal absent from completion/events. Disposable agent/KB cleanup succeeded; UAR PID97354 remains running. No failures or repairs. [PASS receipt](linked-kb-delete-01-macos-pass.json).
