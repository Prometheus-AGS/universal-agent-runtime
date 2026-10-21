# V2 verification lock handoff

Phase `uar-outstanding-issues-v1-1-1`, Execute, plan revision63. The operator
requires a closed verification gate until **every** planned implementation
item across the entire phase is complete, a renewed whole-phase inventory
with exact identity is presented with the named proposed commands, and the
operator explicitly instructs the lead to run those commands. The standing
instruction is in `AGENTS.md` under `Operator verification lock`. A task,
repair cohort, inventory, prior gate, or 120-minute tick cannot reopen it.

## Current source and evidence state

Local08 targeted receipts exist for binding, recovery, credentials and streams.
They are terminal failures/blocks, not acceptance. The resulting repair cohort
has edits in `src/llm/orchestrator.rs`,
`scripts/release/uar_1_1_1_recovery.py` and
`scripts/release/uar_1_1_1_credentials.py`. Runtime has now handed off a
bounded streams fixture repair in `scripts/release/uar_1_1_1_runtime.py`:
the graph scenario observes the ordinary stream while the owner explicitly
resolves the pending `spawn_agent` approval through the existing endpoint.
No renewed inventory binds these edits. Two teammates ran
`git diff --check` during this unfinished cohort; both produced no output and
both violated the boundary. Do not credit either command.

The authoritative waypoint remains Execute/task 6.2 at revision3044. A
source-only KBD trace found that 137/151 is run-wide change completion, not
14 missing implementation tasks in this phase. Canonical phase tasks show
31/35 complete; R11/8.3 and L9/8.4 are complete, while V2/6.2, P1/7.1,
P2/7.2 and P3/7.3 remain open verification/publication tasks. Inventory11
covered 22 production items before the local08 edits. The waypoint's
`exactNextCommand` and ledger summary retain stale R11/L9 prose. Do not edit
generated projections to compensate. A new whole-phase inventory must bind
current source before the lead presents any gate. V2, published child
programs, release and issue receipts remain unverified.

## Team packet and checkpoint rule

Every `uar-core` task packet must quote the closed lock and assign one owned
source area or read-only question. Teammates may inspect source and author
assigned implementation, documentation and memory. They must not run tests,
checks (`git diff --check` included), builds, compiler/import probes, product
gates, or delegated verification. Their handoffs must name changed files,
unresolved decisions, current source state and what remains unverified.

At each 120-minute checkpoint and lead context change, read `AGENTS.md`, the
waypoint, complete phase plan/task ledger, this handoff and the latest
inventory. Reconcile active agents and commands since the previous checkpoint.
Record `gate=CLOSED` unless an exact operator instruction has followed a
consistent whole-phase completion record and proposed command list. Report
violations immediately. The heartbeat
`uar-phase-verification-lock-audit` is active every two hours on this thread
for a read-only compliance audit; it does not grant permission. Primary scoped
memory record: `memory:e78788af-cb74-444a-9ce9-fa86a8d7cfee`. Append-only
local memory: `.prometheus/decisions.md`, `gotchas.md` and `session-log.md`.

Uncomfortable fact: inventory11 said the implementation boundary was open,
but local08 exposed more code work. The next gate is blocked by current
source changes without a renewed inventory and by the operator's unissued
named gate instruction, even if the clock reaches another checkpoint.
