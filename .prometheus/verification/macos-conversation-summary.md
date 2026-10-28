# CONVERSATION-SUMMARY-01 — Mac PASS

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/conversation-summary.cucumber.mjs
```

Observed: **1 scenario (1 passed), 5 steps (5 passed), 10.430 seconds**. Four behavior steps plus cleanup. Only this new scenario ran.

The disposable agent acknowledged a long first turn containing a fresh booking reference, adult/child quantities and a required delivery gate. Its answer was only READY, so it did not repeat these facts. A separate follow-up in the same server-held conversation asked for the reference, combined headcount and required entrance without repeating any original fact or replaying the transcript.

UAR's `summarize` strategy (threshold2, summary allowance2048, resolved liter/MiniMax-M3) produced a genuine completed summary call. Its exact input digest matches the independently serialized fact-bearing first turn under the runtime's summary directive. The source and prepared context hashes differ; `summary_generated` and `changed` are true. Estimated context fell from **3,968 to917 tokens**. The summary call used approximately3,417 input tokens and produced317 output tokens. This rules out passing by merely retaining original history or selecting a strategy label without generating a summary.

The final answer correctly preserved booking reference `BOOK-40f57158-05ab-46e2-a9e8-00827cbbac8a`, computed **19 attendees**, and retained **Gate Q** as the required entrance. The disposable agent was deleted, global settings were untouched, and UAR remains running.

## Changes and initial failure

- `scripts/behaviors/conversation-summary.cucumber.mjs`: one isolated feature, zero automatic retries.
- `scripts/behaviors/features/conversation-summary.feature`: four declarative steps.
- `scripts/behaviors/steps/conversation-summary.steps.mjs`: real conversation, actual summary receipts/source binding, exact fact recall, raw evidence and owned cleanup.
- README, OpenSpec scenario delta, receipt, behavior ledger and append-only memory record this behavior.

Attempt001 produced a summary and correct answer but failed a fixture expectation that the receipt would retain model:null. Runtime receipts identify the resolved model. Inspection also established role/content insertion-order serialization for the input hash. Corrected those two fixture expectations; source binding, actual compression and exact answer assertions remain mandatory. Both attempts are retained. Source critic approved the corrected artifacts; the specialized JavaScript reviewer supplied no formal verdict because its workflow demanded unrelated lint. No unrelated verification was run.

Guards are limited to real API/response boundaries, bounded waits, owned cleanup, exact source identity and the requested summary/recall assertions. No production changes, speculative product guards or unrelated additions. No credentials in tests, previous scenario reruns, model switch, global setting change, rebuild, restart or publication.

[Receipt](conversation-summary-01-macos-pass.json), [raw evidence](conversation-summary/c54371b9-13cf-4586-9e2e-d60ae7cb2d9b.json), [passing report](conversation-summary-002-macos/cucumber.json), [initial failed report](conversation-summary-001-macos/cucumber.json).

Scope is one warm summarization and fact-preserving follow-up on Mac. Repeated summarization, arbitrary fact retention, restart persistence, other strategies/platforms and exact tokenizer guarantees remain unverified by this scenario. Stop for operator direction.
