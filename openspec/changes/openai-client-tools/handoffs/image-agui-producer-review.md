# Published 1.2.2 AG-UI producer diagnosis

Source-only independent diagnosis. Owned artifact: this handoff only. No production edits, checks, probes, builds, tests, imports, runtime or network commands performed. Shared checkout edits preserved. Global verification remains CLOSED outside the lead's finite authorized loop.

## Finding

**The existing evidence does not establish a producer-side race for native run `f936321b-4e89-4b15-aafc-3b035e6c1830` / batch `594edbdd-94dd-44ec-b789-f9dbbc15f630`. No producer fix is justified by this inspection.** The actual failed operation inside handoff confirmation remains UNKNOWN. Persistence/CAS analysis is independently assigned and is not superseded by this report.

Evidence inspected: `target/client-tools-image-bundle-v1.2.2/target/client-tools-published-image-001/candidate.log` and `matrix.json`. Candidate log lines 1532–1538 show provider stream completion, two caller functions, then MCP shutdown at 23:12:39.023001. Line 1539 records the coarse `caller_tool_settlement_failed` warning at 23:12:39.026763. There is no matching finalizer cleanup-error or panic message found by the bounded log search. Absence of a log is not proof of success, but these artifacts do not positively support a cleanup failure. The warning is not the `run_settlement_unconfirmed` category.

## Concrete source trace and exclusions

- The run uses `client-tools-local-...`, not orchestrator-agent, so this case does not enter the installed graph/graph-child handoff path. It is an ordinary producer, not an A2A actor mailbox completion case.
- `await_settled_run_status` clones the owned producer slot, serializes ordinary waiters through its mutex, awaits the retained handle, then joins any graph root and reads the run's terminal status. A second ordinary waiter sees the consumed handle only after the first has released the slot. The formerly problematic post-join `try_lock` settlement re-probe is absent. Concurrent public/background AG-UI reconciliation therefore does not, by this path alone, revoke an established join.
- Ordinary execution writes Done/Error/Cancelled before its terminal frame. The outer finalizer can downgrade status to Error on actual cleanup failure; a successful await sees that final status. Such failures have explicit error logging. This report found no corresponding evidence for this run.
- AG-UI has concurrent background and public reconciliation. Both can enter `seal_frontend_handoff`, but neither directly marks producer status failed. Their confirmation includes multiple distinct fallible operations: load scoped batch, validate it, check Pending/source-settled state, join parent, optionally join child, persist source-settled CAS, reload after lost CAS and validate its acknowledgment. All failures except exactly `run_settlement_unconfirmed` collapse to the same warning category and durable `host_cleanup_unconfirmed` terminal.
- A successful observer can race another observer's failure while sealing the durable AG-UI terminal. The current seal CAS chooses the committed winner; it does not reveal which inner confirmation operation produced a failing contender. This is a concrete error-projection amplification mechanism, **not proof that such a contender or a specific CAS race caused this run**.

## Narrow next evidence

The missing discriminant is the operation/category of the underlying `confirm_handoff_for_run` failure for this exact run/batch. A field-free static classification distinguishing scoped lookup/validation/state, producer join/status, child join and source-settled CAS/reload would let the lead localize the observed failure without exposing SQL, bound data or raw provider/persistence chains. This report does not authorize or implement that instrumentation and does not recommend weakening settlement or turning the error into success.

Only source/evidence inspection commands (`rg`, `sed`) and this documentation write ran. Runtime status, exact stored rows and the cause of the published-image rejection remain unverified. Same-family source diagnosis is not cross-model adversarial evidence.
