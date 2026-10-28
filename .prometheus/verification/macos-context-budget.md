# CONTEXT-BUDGET-01 — Mac PASS

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/context-budget.cucumber.mjs
```

Observed: **1 scenario (1 passed), 6 steps (6 passed), 18.413 seconds**. Five behavior steps plus cleanup. Only this new scenario ran.

A disposable agent received a fresh reference and two quantities in a new session. Its first answer was only READY, so it did not copy the facts into a later assistant response. Three separate acknowledged turns then added irrelevant archival text. The follow-up supplied no original reference or quantities and no replayed transcript. Genuine Liter/MiniMax-M3 inference returned the original random reference and correct sum **906** from server-held conversation history.

| Budget evidence | Observed |
|---|---:|
| Configured context window | 8,192 tokens |
| Explicit output reservation | 2,048 tokens |
| History input allowance after fixed overhead | 5,879 tokens |
| Original history estimate | 12,869 tokens / 22 messages |
| Reduced history estimate | 4,830 tokens / 14 messages |
| Final serialized request estimate | 5,339 tokens |
| Total input allowance before fixed overhead | 6,144 tokens |
| Provider-reported input usage | 4,689 tokens |
| Provider-reported output usage | 94 tokens |

Both source/final history digests and counts changed. Requested strategy was `truncate_middle` with keep_first1/keep_last1000, so the large message-count limit did not explain away the token-pressure test. Actual token-budget reduction preserved the initial fact-bearing turn. UAR's final request receipt explicitly reports approximate counting and fit_guarantee false; this result is corroborated by provider usage, not a claim of exact tokenizer certification.

## Changes and initial failure

- `context-budget.cucumber.mjs`: isolates one feature with zero automatic retries.
- `features/context-budget.feature`: five behavior steps.
- `steps/context-budget.steps.mjs`: reversible context-window setup, genuine conversation, oversized history, budget/recall assertions, evidence and cleanup.
- README, OpenSpec delta, receipt, ledger and append-only memory document this behavior.

Attempt001 bundled multiple user messages in one chat request. The endpoint admitted only the last user message, so filler never entered history and the source-over-budget assertion failed. The corrected fixture sends each appendix as a separate acknowledged turn; assertions remain intact. Both attempts are retained. Source critic confirmed the correction; the specialized JavaScript reviewer supplied no formal verdict because its workflow required unrelated checks. No unrelated checks were run.

The original provider configuration was saved without credentials, only the selected model context_window changed temporarily, and cleanup restored the original view exactly. The existing model-operation receipt was refreshed after each configuration change so the runtime remains usable. Credentials were neither read nor supplied by the test. The disposable agent was deleted; existing agents and store remained intact. No production edits, rebuild, restart, previously verified scenario rerun, publication or unrelated additions.

Guard rationale: HTTP boundaries and bounded waits; owned agent cleanup; comparison to the temporary provider view before restoration to avoid overwriting a concurrent edit; explicit independent budget and recall assertions. No speculative production guard was added.

[Receipt](context-budget-01-macos-pass.json), [raw evidence](context-budget/6e963cf7-8709-4610-b8e0-59abbac9aaf1.json), [passing report](context-budget-002-macos/cucumber.json), [initial failed report](context-budget-001-macos/cucumber.json).

Scope is keep-first/latest token reduction and warm follow-up recall on Mac. Summarization, hierarchical strategies, restart persistence, protected tool-record pressure, exact tokenizers, other platforms and repeat reliability remain unverified by this scenario. UAR remains running; stop for operator direction.
