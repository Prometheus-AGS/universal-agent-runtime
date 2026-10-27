# Agent KB unlinking on Mac

KB-UNLINK-01 verifies that removing an agent's KB link stops its retrieval while the KB and document remain available.

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/kb-unlink.cucumber.mjs
```

Use retained UAR_BASE_URL, UAR_WORKSPACE_ID and UAR_MODEL. Create disposable resources; establish the agent's real use of a private document seal with a matching citation. PATCH only the knowledge-base ID list to empty, leaving KB support enabled and other agent behavior unchanged. A fresh-session identical prompt must still calculate correctly while returning no seal/citation or RAG citations. The KB/document snapshots must remain unchanged and direct KB search must still retrieve the private fact. Delete the disposable resources only after these assertions.

No test credentials, global configuration changes, restart, hosted embeddings or prior scenario reruns. This does not erase old conversation history or certify all backend/platform combinations.

Attempt001 failed: the empty saved link list still produced the private fact and citations from that KB and unrelated KBs. Failure evidence: [raw attempt001](kb-unlink/2c639dd3-f8b7-4272-8355-caab308cbb5a.json). Cleanup succeeded.

The production repair records explicit KB selection intent when links/enabled state are patched in `src/uar/domain/agent_store.rs`. An empty explicit selection no longer falls back to automatic access. Untouched legacy/default agents retain their existing behavior; explicit policy in the same patch wins and unrelated policy fields remain preserved. The initial broader mapping change was discarded after source review, before compilation. BDD assertions are unchanged.

Result: PASS on Mac, 2026-10-26T20:07:08.758403-05:00. Attempt004: `1 scenario (1 passed); 5 steps (5 passed); 0m 21.190s`, exit0. Before unlink:679/private seal with matching citation. After link-only PATCH:679/nullseal/nullcitation and no RAG citations in a fresh session. KB/document snapshots unchanged; direct KB retrieval still returned the original seal at score0.9160574. KB support stayed enabled; persisted explicit selection was empty. Disposable resources removed only after availability assertions; UAR PID21601 remains running.

Attempt001 exposed the product selection defect. Attempts002/003 exposed overly strict fixture JSON framing; both failures remain recorded. The final parser accepts exactly one JSON block outside reasoning, allowing explanatory prose, while preserving all functional assertions. Mac `cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime` with CARGO_TARGET_DIR=target/uar-team-concurrency-repair passed; three UAR and two vendor warnings came from unchanged code. Only the owned runtime was restarted, with config/store preserved. No earlier scenarios were rerun. Source review preserved legacy-default behavior; it and other policy-edit combinations were not runtime-retested. [PASS receipt](kb-unlink-01-macos-pass.json).
