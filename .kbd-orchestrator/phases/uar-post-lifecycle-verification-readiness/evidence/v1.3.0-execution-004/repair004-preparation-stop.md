# Repair004 preparation stopped: signed dispatch limit exceeded

Current phase Execute uar-post-lifecycle-verification-readiness. Verification CLOSED. A4 adopted digest f99c0e44b0a1994560453211c6528b247d6ed88030602d0f0a94470c644eea72 remains unchanged. Its240-host-dispatch limit is exhausted. Source-only cumulative root/descendant log audit found243unique emitted calls before stopping the two preparation workers. Subsequent stop/receipt dispatches additionally accrue. The previous220-call stop cutoff was not a fresh allowance. Preparation overran the original limit; no reset or retroactive authorization is asserted. Both current workers were interrupted. No test/build/lint/format/check/probe/signing/adoption/publication ran in this preparation turn.

## Diagnoses retained from source inspection

- Restart readiness: LIAB-02 restart poll does not opt in to startupTransportRetry and the driver restricts retry to setup. Exact fix must allow the existing bounded owned-child health poll following restart, retaining attempt limits and child-exit failure; never retry arbitrary scenario requests.
- CAS production classification: src/uar/persistence/providers/agui_runs/surreal.rs unconditionally maps typed TransactionConflict to Ok(false). The proposed correction must distinguish a changed target from an unrelated conflict and preserve the latter as an error. Implementation and race semantics still require complete source tracing.
- CAS fixture ordering: scheduler launch labels do not guarantee transaction winners; both packet004 directions returned a single valid winner and exact readback. Preserve coverage of actual distinct winner directions and every discriminator assertion; do not weaken the oracle to any winner or claim ordering proved.
- COUNT: the validated wire and emitted bytes differed. Inspection found the Liter driver sends typed chat_req while validated_wire is separate. Exact downstream transport cause and a safe correction remain unresolved. Keep SHA equality strict.
- HISTORY/KB returned404. Missing workspace headers are a candidate cause, not yet proven against namespace middleware. KB router explicitly uses a trailing slash. Do not patch paths by guess. Their early failure prevents the third expected main launch; retain expected3.
- Teardown HTTP404 may follow absent fixture objects; retain honest cleanup failures until ownership/existence semantics are traced. Owned process exits,12 exact rows and Docker cleanup already succeeded in packet004.

## State and inventory limitation

No completed repair004 source inventory, signed amendment or runnable packet005 exists. The previous inventory remains historical and does not certify preparation edits. Writers reported no edits at their last handoff; interrupted activity is not a substitute for a final source diff inspection. Before authoring can be declared complete, inspect their working state, finish every proven fix, reconcile canonical ledger, collect source-only handoffs and freeze all phase inputs. Independent acceptance reviews remain deferred by the closed gate.

All14 obligations, original epoch2026-10-24T13:38:17.685447Z,240active minutes, no-progress3, spend0, actual consumption and historical UNKNOWN remain. All3signed repairs are consumed. No fourth repair or retry is authorized by the existing signature. No acceptance task completion, archive or reflection occurred.

## Exact next approval: preparation allowance only

Proposed decision, not attributed approval:

> Approve an explicit amendment preparation allowance increasing the cumulative original-epoch host-dispatch ceiling from240to320, retaining all actual consumption including the recorded overrun and subsequent stop/bookkeeping calls. Authorize completion of the packet004 diagnosis and exact repair004 source authoring, proposed total repair cap3to4 (four consumed after authoring, no fifth), canonical ledger reconciliation, whole-phase inventory refresh and the unsigned A5/F5/V5 packet. Preserve240active minutes, original epoch, no-progress3, spend0, all14obligations and historicalUNKNOWN. This approval does not authorize verification, signing/adoption, retry or publication. Present the final source-bound amendment and exact commands/fixture effects for a separate approval before verification.

Why this is required: the currently adopted signed contract's budgets.toolCalls=240 and the operator approval explicitly required stopping on exhausted/unknown allowance. Repair-cap increase alone cannot renew exhausted calls. The source inventory and final contract digest cannot honestly be frozen while COUNT and the404 cause remain unresolved. A future A5/F5/V5 approval must explicitly sign the cap and call-limit changes, bind the completed exact source inventory, preserve prior evidence and stop at first failure. This document is a proposal and stopped-state receipt, not executable authority.
