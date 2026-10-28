# CONCURRENT-CONVERSATION-RESTART-01 — Mac PASS

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/concurrent-conversation-restart.cucumber.mjs
```

Observed on the first attempt: **1 scenario (1 passed), 6 steps (6 passed), 21.43 seconds**. Five behavior steps plus cleanup. Only this selected new scenario ran.

Two genuine `liter/MiniMax-M3` prompts supplied separate random alpha/beta references and quantities in fresh session `85bf68ab-d292-41c7-80b3-b9647641f110`. Both completed successfully with READY. Ordered live server observations showed run A running, run B running, and the same run A still running; all belong to the same session and disposable agent. This establishes actual overlap. The 846.907 ms overlap of client-received streams is supporting evidence, not the concurrency proof.

After both completed, UAR PID47773 exited gracefully and PID52394 started using the same binary/config/environment/working directory and store. The external proxy stayed running; endpoint ownership was confirmed. A follow-up provided only the same session ID and question, with no history or fact replay. It returned both exact references and the correct combined quantity **933**:

```json
{"alpha":"ALPHA-18317935-713f-46cc-8a90-aa5463cdb759","beta":"BETA-8066ecae-8e94-474a-982f-f63c2267d0cf","total":933}
```

## Changes and review

- `scripts/behaviors/concurrent-conversation-restart.cucumber.mjs`: selects only this feature; retry0.
- `scripts/behaviors/features/concurrent-conversation-restart.feature`: one concurrent restart/recall behavior.
- `scripts/behaviors/steps/concurrent-conversation-restart.steps.mjs`: genuine concurrent streams, server-state overlap, all-settled cleanup ordering, exact recall after owned restart.
- README and OpenSpec delta: scenario contract. Ledger, receipt and manual-runtime metadata: observed result.
- `.prometheus/knowledge/karpathy/manual-bdd-evolution-01a16d22/`: append-only public conversation export, capture cutoffs and process analysis for later skill extraction. Append-only project memory records the outcome.

Both source reviewers rejected the first proposed overlap oracle before execution because separate SSE connections can buffer events. The authored test was corrected to require A→B→A live server-state observations. Both reviewers then reported no concrete source blocker. There was no failed live attempt, product edit, rebuild, model switch, test authentication, store reset, publication or earlier scenario rerun.

No unrelated implementation was added. Guards protect actual API/stream boundaries, owned process lifecycle, outstanding-request settlement, resource cleanup and the requested evidence assertions. The disposable agent was deleted and UAR remains running.

## Limits

**The uncomfortable limit:** this is one observed scheduling interleaving. It does not disprove the previously identified possibility of out-of-order whole-session persistence writes or non-atomic cold cache creation, and those paths were not changed. No all-interleavings, stress, crash/reboot, other-backend or other-platform claim is made. The scenario's requested behavior passed; an exhaustive concurrency guarantee remains unverified.

[Receipt](concurrent-conversation-restart-01-macos-pass.json), [raw evidence](concurrent-conversation-restart/85bf68ab-d292-41c7-80b3-b9647641f110.json), [Cucumber report](concurrent-conversation-restart-001-macos/cucumber.json), [Karpathy process log](../knowledge/karpathy/manual-bdd-evolution-01a16d22/process.md).

Complete for this manual sequence. Stopped for operator direction.
