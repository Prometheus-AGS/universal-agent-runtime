# Existing Dot task reconciliation

Date: 2026-10-12. Stage: Plan. This records scheduling against the parent `dot-feature-support/plan.md` draft and the full-phase verification boundary in `AGENTS.md`. It does not register tasks, change the canonical KBD cursor, or claim product acceptance.

All 17 existing Dot OpenSpec changes are active and have an existing `tasks.md`; no matching archived Dot task file was found. Dotted labels are display labels. Existing canonical backend task IDs remain unchanged; Dot 17 adds backend task 3 for runner implementation under the lead's sequencing decision. Before that registration, the 17 changes contained 35 listed tasks: 13 checked and 22 unchecked. With Dot 17 task 3, they list 36 tasks, 13 checked and 23 unchecked. Proposed changes 18–20 have no registered task IDs yet.

| Change | Current backend task IDs and category | Current state and scheduling |
|---|---|---|
| `dot-01-release-provenance` | 1 implementation; 2 verification | Both checked; historical receipts for the earlier source only. Final-source provenance and artifact acceptance remain in Dot 17. |
| `dot-02-principal-credentials` | 1 implementation; 2 verification | Both checked; historical receipts only. |
| `dot-03-durable-api-keys` | 1 implementation; 2 verification | Both checked; historical receipts only. |
| `dot-04-truthful-tool-outcomes` | 1 implementation; 2 verification | Both checked; historical receipts only. |
| `dot-05-stream-terminal-contract` | 1 implementation; 2 verification | Both checked; historical receipts only. |
| `dot-06-fresh-process-resume` | 1 implementation; 2 verification | Both checked; historical receipts only. |
| `dot-07-approval-recovery` | 1 implementation; 2 verification | 1 checked; 2 remains unchecked and is deferred to the full-phase gate. The interrupted earlier gate is not acceptance. |
| `dot-08-durable-ingestion` | 1 implementation; 2 verification | Both unchecked; author tests in 1, execute 2 only after the full-phase inventory. |
| `dot-09-idempotent-settlement` | 1 implementation; 2 verification | Both unchecked; same boundary. |
| `dot-10-durable-budget-admission` | 1 implementation; 2 verification | Both unchecked; same boundary. |
| `dot-11-supervisor-fencing` | 1 implementation; 2 verification | Both unchecked; same boundary. |
| `dot-12-memory-isolation` | 1 implementation; 2 verification | Both unchecked; same boundary. |
| `dot-13-memory-controls` | 1 implementation; 2 verification | Both unchecked; same boundary. |
| `dot-14-durable-schedules` | 1 source/docs timing decision; 2 implementation; 3 verification | All unchecked. Task 1 inspects the exact version without a probe; task 2 implements and authors tests; task 3 runs executable fit and real-path acceptance only after the full-phase inventory. |
| `dot-15-proactive-followups` | 1 implementation; 2 verification | Both unchecked; same boundary. |
| `dot-16-assistant-capability-surface` | 1 implementation; 2 verification | Both unchecked; same boundary. |
| `dot-17-assistant-release-acceptance` | 3 runner/fixtures/docs implementation; 1 full-profile product acceptance; 2 published-artifact acceptance | All unchecked. Execute backend task 3 first and finish it before the full-phase inventory. Run backend task 1 only after the inventory and all 18–20 production contracts; run backend task 2 after task 1 and the customer build. |

The current scheduling note in each active task file supersedes older per-edit Tier 0 and completed-change wording without changing a checkbox or historical receipt. Future executable verification, policy validators, QA detectors and product review gates remain closed until the lead records every planned production behavior, persistence contract, migration, operational setting, required document, conformance fixture and Dot 17 runner complete across the expanded phase. After that boundary, the lead runs the serial production integration and customer-build gate, then installed-artifact conformance and deployment health. A later plan revision expands the same inventory. If canonical change-complete prerequisites require an early verification task to be checked before a successor can begin, the lead must revise the canonical sequencing through supported KBD operations; checking or running that verification early is prohibited.

The uncomfortable constraint is that the 01–06 checked records cannot be reused as final-source acceptance. The immutable existing Dot 17 backend task 1 retains final profile execution/evidence; new backend task 3 carries source-only runner implementation and is ordered before 1 and 2 by the lead's sequencing manifest. Only the KBD lead can register task 3 and reconcile the canonical task cursor. This document does not alter existing task IDs or the registered 17-change total.

## Lead registration receipt

Canonical registration completed: changes18–20, eight associated tasks, and Dot17 runner task3. Existing verification remains unpassed; Dot07 task2 is blocked by the full-phase boundary. The immutable sequencing decision references `../../implementation-sequencing.json`; exact commands and observed exit codes are in `canonical-reconciliation-receipt.json`. OpenSpec semantic validation remains deferred; registration is not product verification.
