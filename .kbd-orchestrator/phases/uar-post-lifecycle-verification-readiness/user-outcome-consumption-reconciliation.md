# Historical consumption reconciliation — current balance unknown

Phase: user-outcome implementation. Gate CLOSED. This is read-only historical
receipt reconciliation, not a fresh usage measurement, budget seed, billing
receipt, signature verification, or authorization to spend. No provider/service
request ran. Root/budget writer owns current cumulative reconciliation.

## Available historical floors

All paths below are relative to this phase directory. These are cumulative
checkpoints of the same original epoch `2026-10-24T13:38:17.685447Z` and must NOT
be summed. They preserve earlier overruns and unknown pre-epoch effort.

| Existing receipt | Recorded cutoff | Unique emitted host dispatches | Recorded active-minute upper bound | Repairs recorded |
|---|---|---:|---:|---:|
| evidence/v1.3.0-execution-002/budget-at-stop.json | 2026-10-24T16:12:39.957638Z | 140 | 67.48775318333334 | 1 |
| evidence/v1.3.0-execution-003/budget-at-stop.json | 2026-10-24T16:35:19.165215Z | 184 | 90.14121279999999 | 2 |
| evidence/v1.3.0-execution-004/budget-at-stop.json | 2026-10-24T17:14:21.470474Z | 220 | 129.17963378333332 | 3 |
| evidence/v1.3.0-execution-005/budget-at-stop.json | 2026-10-24T18:27:31.357673Z | 307 | 202.3444204333333 | 4 |
| evidence/v1.3.0-execution-006/budget-at-stop.json | 2026-10-24T19:07:07.711354Z | 381 | 147.17569844999997 under adopted A6 exclusions | 5 |

Latest read checkpoint is a historical call floor of381 and repair floor5, not
current consumption. Time values are reported upper-bound estimates under their
own signed exclusion treatment, not monotonic measured active-time lower bounds.
The 006 receipt preserves its conservative pre-additional-exclusion estimate
241.95031511666667minutes and wall time328.83376511666665minutes. Do not replace
all history with the lower A6 number or infer new idle intervals.

006's receipt names effective limits480 cumulative calls,240 active minutes,
5 repairs, no-progress3, spend0, with0 repairs remaining at that cutoff. The
corresponding existing amendment is
`replacement-exit-contract.v1.3.0.repair-005.amendment.json` with companion `.sig`;
its referenced proposed contract digest is
`8b2557856e30e257dd91e82c6fcb6fa659c663c0e603a8754e36140c7231552d`.
`evidence/v1.3.0-execution-006/adoption-result.json` records ADOPTED binding8 /
planRevision114. This task READ those receipts and signature-file presence; it
did not cryptographically verify a signature, call adoption, or assert current
canonical authority. The adopted reason preserves all prior consumption and the
240-call overrun, adds no sixth repair, and does not authorize an automatic retry.

`budget-accounting-adoption-clarification.json` explains inherited proposed fields
in 006 launch metadata: actual adopted A6 excludes original5213.007 seconds plus
nine specific additional intervals totaling5686.477 seconds. Use 006 stop's
cleaned accounting description; never silently turn preparation labels into
fresh adoption. Pre-epoch effort is explicitly UNKNOWN in the inspected records.
The canonical historical resolution artifact exists but no fresh canonical-floor
query was performed here; canonical budget state remains a required input.

## Current amount and room are unresolved

Do not compute “99 calls remaining” or subtract the historical147.1757-minute
estimate from240 as today's balance. Many later source/delegation calls accrued;
this turn itself contributes under any applicable host-call authority. No fresh
root/transitive-session call-ID audit or approved idle reconciliation was run.
Historical repair ceilings and prior exhausted allowance are not automatically
reset by this new implementation task or its source snapshot.

Fresh resource authority must explicitly reconcile original epoch and parent
scope, applicable cumulative ceilings, actual current deduplicated root/descendant
calls, later elapsed/active intervals, historical/unknown effort and pending
reservations. Preserve historical floors and overruns in the seed; resolve unknown
reservations explicitly, without refunds or zero defaults. A new epoch needs
explicit adopted disposition of predecessor consumption; renaming an epoch is
not a reset. Budget writer/lead must bind this to actual signed/adopted authority
and exact packet before execution.

## Paid model exposure is separate from host effort

The source billing evidence records operator-reported Liter subscriptions with no
incremental charge separately from pay-as-you-go embeddings. It later records
at mostUSD10 Cohere plusUSD10 OpenAI embeddings, independent nontransferable caps
(combined maximumUSD20). These ceilings are not targets, current balances,
billed totals, verified account credit, or evidence of provider calls.

No exact current tariff or independently established conservative binding-specific
charge bound was established in this bounded task. Rate/rounding/unit/provider
binding must be frozen with request input/output/retry reservations before a
charge is admitted. A UAR→gateway request is not every gateway-internal attempt;
existing gateway retry3 must be represented honestly in full-chain exposure.
Unsettled reservations remain held; no implied refund. Source inspection makes
no spend assertion for historical execution beyond what each old receipt says.

The uncomfortable thing: a historical signed limit can look like fresh room when
only its stop receipt is read. Current balance is UNKNOWN here. Root must obtain
current authority/consumption reconciliation from the assigned budget writer,
not present this historical table as permission. No executable accounting probe,
provider discovery request or verification gate was run; only source/receipt
reads and the two requested phase documents were authored, with login:false.


## Refreshed source-log cutoff

Original root plus44 available transitive sessions were read from local rollout source logs at 2026-10-25T08:33:26.499928+00:00. The authoring receipt user-outcome-accounting-001.json records 2113 unique emitted host dispatches since the original epoch and a conservative active upper bound 953.48884135 minutes after retaining only the signed10900.484-second idle exclusions. No new idle exclusion or historical reset is applied. 1 prior recorded call IDs are absent from the new source-log set; 380 overlap. This is a cutoff, not a current remaining balance; later calls/time accrue. Every source snapshot has exact log byte length/SHA and no malformed JSON line was observed. Pre-epoch effort remains UNKNOWN. A fresh cumulative ceiling must exceed this floor; the earlier unsigned1600-call proposal is already insufficient. Proposed replacement ceilings4000calls/2400active minutes retain original epoch, all consumption/overruns,5consumed repairs/no automatic repair, no-progress3. These values require an explicit new signed amendment. No product command was executed.

Retain the one previously recorded dispatch absent from the current source set as historical evidence. The conservative union floor is2114, not2113; do not discard that call or silently infer its cause. The updated signed proposal must bind both source-log receipt and old381-call receipt. Additional calls after08:33:26Z accrue normally.
