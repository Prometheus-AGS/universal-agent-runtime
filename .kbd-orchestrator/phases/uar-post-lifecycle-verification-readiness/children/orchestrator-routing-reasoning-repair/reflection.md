# Reflection — orchestrator-routing-reasoning-repair

Date: 2026-10-27. Outcome: **goal met.** C34 passed on Mac on attempt 5 ([receipt](../../../../../.prometheus/verification/c34-live-roster-routing-macos-pass.json)), credited as B37.

## Delta between plan and delivery

| Planned (plan revision 2) | Delivered | Delta |
|---|---|---|
| Fix the router refusal (T1/T2) and the false success on reasoning-only children (T4); prove both with one C34 rerun | T1/T2/T4/T6/T7 implemented as planned. C34 attempt 4 then **failed honestly** with `delegated_answer_missing: reasoning_only` | The plan named this outcome under D2(a) as likely. It took a second change to reach the goal. |
| D2(b), raising the child output allowance, was out of scope and described as a "budget-governance change" | Planned and delivered as its own change, `raise-delegated-child-output-allowance` (d2b plan revision 2, T8–T10). This added `manager.rs` to the write surface | The parent plan's description was wrong. Accounting reservations without a contract come from catalog limits, not the request cap. Superseded in `decisions.md`. |
| Gate: build, `--lib` unit tests, mock-LLM integration tests, validate, restart, C34 | The operator **removed** the unit and mock steps as contrary to the real-path evidence rule. The `--lib` run had already happened: it first failed to compile on two stale test initializers outside this change, then passed 20/20 after an approved test-only repair. It is not counted as evidence. The mock step never ran | **Planning defect:** I put unit and mock tests in a gate that the repo rules say cannot count as completion evidence. That cost two cargo runs, a test-file edit outside scope, and an operator correction. |
| D2(b) gate step 5: read `reserved_output_tokens` from the run record | The API does not expose that field, which I found by reading attempt 4's child record. Replaced by B5(a), one content-free log line | The plan assumed an observable that didn't exist. A source/API read during planning caught it before the gate. |
| Exit: C34 passes with genuine answers | Passed 13/13. Raw evidence reviewed. The log confirms the specialist ran with 8,192 | Met under B4(a), a single pass. |

## What the evidence does and does not show

- **Shown:**
  - The router selects a registered agent when the classifier answers correctly.
  - A reasoning-only child turn fails with a named code instead of reporting success.
  - Reasoning no longer reaches the client through delegated contributions.
  - With 8,192, the specialist answered with its signature.
- **Not shown:**
  - That the old 1,000 cap would have failed on attempt 5. Attempt 1 once passed under it.
  - The classifier's actual failing reply in attempt 3. The router refusal did not recur, so the new refusal diagnostics never fired.
  - The finish reason, which is still not recorded.
  - Effects on `spawn_agent` children and follow-up turns, which also received 8,192. C34 does not exercise them.

## Residuals (not fixed here)

- Inline reasoning still reaches clients on ordinary, non-delegated MiniMax-M3 chats.
- Other `Completed` consumers (team, spawn/wait tools, actors, A2A thread service) keep the false-success behavior on reasoning-only output.
- The provider finish reason is not recorded, so truncation cannot be distinguished from a natural stop.
- Archive order for the three open changes touching `multi-agent-orchestration`: `reengineer-user-outcome-verification`, then `fix-orchestrator-routing-reasoning`, then `raise-delegated-child-output-allowance`.
- Nothing is committed.

## Lessons

1. **Plan a gate around the evidence rule, not habit.** If the repo says only real-path evidence counts, a plan must not list filtered unit tests or mock tests as gate steps or exit criteria.
2. **Verify an observable exists before making it a gate check.** Confirming the cap needed a reading the API did not provide; a one-line, content-free log fixed that.
3. **A cap fallback needs to be traced end to end before it is labelled.** "Budget-governance" was asserted from the variable's name, not from the reservation path.
4. **Keep each honest failure as its own attempt.** Five C34 attempts, four receipts kept, no retries. The attempt 2 override, where the evidence contradicted a Cucumber pass, was the most important call in the phase.
