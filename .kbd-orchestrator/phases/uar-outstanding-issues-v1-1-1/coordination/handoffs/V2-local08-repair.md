# V2 local08 source repair handoff

Phase `uar-outstanding-issues-v1-1-1`, Execute, plan revision63. Inventory11
was recorded before four serial targeted local08 acceptance commands. All
four terminated without an accepted affected-group result. The immutable
receipts are under `evidence/acceptance/20261017-local-08-*/receipt.json`.
No test, check or build has run against the source edits below.
Canonical task R12/8.5 was registered at sequence36 and transitioned through
in-progress to complete as source-only work. L10/8.6 was registered at
sequence37 and is in progress for the renewed whole-phase inventory.

| Group | Observed local08 result | Source-only disposition |
| --- | --- | --- |
| Binding | Catalog timeout/max-retries inheritance assertion failed. Outer pre-semantic stream-open resilience made more attempts than selected `llm.max_retries=0`. | The `binding_local08_triage` owner changed only `src/llm/orchestrator.rs` to cap stream-open attempts at the lesser of the resilience count and model retry budget plus the first attempt. No provider policy denial or post-output stream path changed. |
| Recovery | The legacy fixture asserted one global row in each table but startup had already created an anonymous default knowledge base. The qualified guest proof was not consumed. | The `recovery_local08_triage` owner changed only `scripts/release/uar_1_1_1_recovery.py` to snapshot the seeded legacy owner for initial and cold-readback equality. Exact one-row-per-table assertions remain. |
| Credentials | The first six scenarios passed, then backend outage cleanup blocked: after Docker restart the cached published host port was stale and readiness was not established. The #387 compiler adapter was not reached. | Lead changed only `scripts/release/uar_1_1_1_credentials.py` to reread `docker port` after start and require HTTP 200 from `/health` before continuing. |
| Streams | The native partition, SSE, EOF, invalid and tool-loop cases completed. The graph case emitted `spawn_agent` approval-required, then the observer hit its total deadline without an approval decision. | The `streams_local08_triage` owner changed only `scripts/release/uar_1_1_1_runtime.py`: observe the ordinary stream, read the owner's pending approval, submit the exact approval ID through the existing endpoint, then collect terminal status/events. The graph allow-list and `agui.subagent.started` assertion remain. |

The source changes are not yet bound to a renewed whole-phase inventory or a
new candidate image. Runtime behavior, endpoint response, recovery proof,
credentials remainder including #387, published child programs, release and
issue receipts remain unverified. No acceptance is inferred from source
inspection. The operator verification lock in `AGENTS.md` is CLOSED; present
the complete phase implementation inventory and exact proposed commands to
the operator, then wait for explicit instruction. A time checkpoint grants no
permission. During this cohort, two teammates ran `git diff --check` before
completion; both produced empty output but violated the lock and count for no
acceptance. The lead recorded that incident in phase execution and memory.

Uncomfortable fact: the local08 pass list is partial evidence on a previous
candidate identity, and current production bytes changed. Even the earlier
MCP/combined passes do not certify a new binary.
