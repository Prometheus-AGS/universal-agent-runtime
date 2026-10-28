# KNOWLEDGE-RESTART-01 — Mac PASS

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/knowledge-restart.cucumber.mjs
```

Observed on the first attempt: **1 scenario (1 passed), 5 steps (5 passed), 22.95 seconds**. Four behavior steps plus evidence hook. Only the new retained-knowledge restart scenario ran.

The saved `math-specialist-db64c2a8` agent retained its binding to knowledge base `58c0e39f-43e0-47a9-b699-7dea5d82417f` and indexed document `35f6bbe7-5804-4253-b0d1-c77642365757`. Their complete API records were unchanged across graceful owned UAR replacement PID62351→73870. The same binary, config, environment, working directory and store were preserved. Active local FastEmbed `bge-small-en-v1.5`,384dimensions, matched the persisted index before and after restart, with the same asset manifest.

After restart, one genuine `liter/MiniMax-M3` completion in a fresh conversation returned the correct specialist, retained profile,11rimvertices, Lucas number199 and sandpile group order39601. It also returned the document-only seal `1a60323f-e62d-4cfb-8a97-591440ca2fdc` and citation `[1]`. The seal was absent from the current agent instructions and outbound prompt. Memory/history replay was disabled. Actual `agui.rag_citations` evidence for the new run matched the retained KB/document, seal-bearing snippet and answer marker. This rules out accepting a plausible mathematical answer alone.

## Files and checks

- `scripts/behaviors/knowledge-restart.cucumber.mjs`: selects only this feature with retry0.
- `scripts/behaviors/features/knowledge-restart.feature`: the retained-KB restart behavior.
- `scripts/behaviors/steps/knowledge-restart.steps.mjs`: before/after resource snapshots, real owned restart, fresh-session inference, independent Lucas arithmetic and citation assertions.
- Behavior README/OpenSpec delta document the scenario. Receipt, ledger and runtime metadata record its PASS. Karpathy process/transcript and append-only project memory preserve the method and result.

Both independent source reviews found no concrete blockers. No live failures, production edits, rebuilds, setup reruns, new KBs, document uploads, binding changes, reindexing, test credentials or previously verified scenario runs occurred. Existing saved agents, KBs, documents and UAR remain available. No hosted embeddings were used: this task adds $0 Cohere/$0 OpenAI embedding spend; prior consumption and separate caps remain.

No unrelated implementation was added. Assertions cover the real API/process boundaries, unchanged retained resource identity, local indexed embedding compatibility, fresh-session no-replay input and actual retrieved-document evidence. No cleanup mutation was required because the test created no agent/knowledge resources.

## Limits

**This does not certify embedding-model changes or regeneration.** It covers the retained index under the same embedding space and one question after a graceful Mac restart. Other backends/platforms, crash recovery and arbitrary-query retrieval quality remain unverified by this scenario.

[PASS receipt](knowledge-restart-01-macos-pass.json), [raw exchanges and citation evidence](knowledge-restart/5f4777cc-f0fe-48c7-a762-1644de605d17.json), [Cucumber report](knowledge-restart-001-macos/cucumber.json), [Karpathy process log](../knowledge/karpathy/manual-bdd-evolution-01a16d22/process.md).

Complete for the manual sequence. UAR running; stopped for operator direction.
