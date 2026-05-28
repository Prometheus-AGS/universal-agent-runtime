## Why

The contract tests (`vitest-contract-test-suite`) lock the code paths the Provider + Agent migrations rely on, but they don't prove the **integration**: real SSE from SurrealDB → graph → page render → user perceives fresh data. This change is the manual walk-through that closes that gap.

The 8 scenarios are the headline value props of the two migrations. Any failure here means the migration shipped a regression that the contract tests don't catch — a clear signal that the test suite needs a new scenario.

## What Changes

No code edits. Create `.kbd-orchestrator/phases/browser-smoke-providers-and-agents/smoke-log.md` and walk these 8 scenarios in order in two Chrome windows pointed at `http://127.0.0.1:8088/`:

- **P1** Configure provider → cross-tab propagation
- **P2** Set default provider → optimistic flip (instant) + SSE reconcile
- **P3** Remove provider → cross-tab removal
- **A1** Edit agent memory toggle in Admin → AgentSelector dropdown in another tab reflects (latent-bug regression guard)
- **A2** Delete agent → both Admin list and AgentSelector drop the row
- **A3** Switch active agent in chat sidebar → header model badge updates + next message uses new agent's policy
- **R1** Force `setDefault` rejection → optimistic flip rolls back
- **R2** Force `patchAgent` rejection → optimistic merge rolls back

For each scenario record `Observed:` and a Pass/Fail verdict. After all 8, batch-triage any failures into new tasks before reflect.

## Acceptance

- `smoke-log.md` exists with 8 filled-in entries.
- ≥6 of 8 Pass (75% baseline); below that, escalate before phase reflect.
- Every Fail filed as a `TaskCreate` with phase + entity + scenario tag.
