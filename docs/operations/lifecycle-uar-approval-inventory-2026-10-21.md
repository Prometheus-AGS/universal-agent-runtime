# UAR lifecycle renewed implementation inventory

Latest operator closeout, 2026-10-21: testing was explicitly stopped. The new embedding attempt was canceled during compilation (Cargo SIGTERM/-15, wrapper143); no new success or test failure is claimed. Other targets never started. All further verification is CLOSED. Remaining work is tracked in https://github.com/Prometheus-AGS/universal-agent-runtime/issues/421. Earlier proposal/inventory text below records its historical state, not current authorization.

2026-10-21. UAR only, as directed by the operator. Boss work is outside this
packet and does not block UAR verification. Verification remains CLOSED.

## Authority and exact source

Execute phase: `uar-outstanding-issues-v1-1-1::function-first-release-certification`.
Canonical waypoint3198 / plan88. Both parent and child ledgers were read.
All planned production/fixture/documentation rows are complete. The child's
lifecycle ledger has22 completed rows; three historical Boss binding rows are
outside this packet, leaving19 UAR implementation/delivery rows. Its five
original release-child implementation/documentation rows are also complete.
Writer handoffs are collected in the companion JSON. This means source complete,
not accepted behavior.

The run-wide counter remains138/153; it is not the count of unfinished UAR
source rows and is not rewritten to manufacture completion. Child4.1
preparation,4.2 independent review,4.3 acceptance and5.1 reflection/handoff
remain pending. Parent6.2 acceptance is in progress;7.1 immutable release
publication,7.2 two-architecture acceptance and7.3 receipts/reflection remain
pending.

- Published production: `0f926ed6deee26c353278640e3d7c1f19982bc55`.
- Published documentation HEAD at renewal: `ce1a7a1b5dc95db1e1c79d9949983c1eee1ac116`.
- Earlier embedding repair: `d35234ba132988e5d64b1e19aea688c7fa978690`.
- Current saved-worktree records, canonical rows and handoff hashes:
  `lifecycle-uar-approval-inventory-2026-10-21.json`.

The saved inventory is authored metadata, not an executed acceptance or
source-identity check. Proposed commands exercise this saved worktree.
Pre-existing unrelated edits, including seed-tool changes excluded from scoped
publication, remain; no clean-checkout certification is claimed. This renewal
changes documents/memory and supported KBD metadata only.

## Implementation-complete inventory

| Work | Existing source and dependencies | Source state |
| --- | --- | --- |
| WP1 resource inventory and transitions | runtime/lifecycle.rs, server/lifecycle_graph.rs, resource inventory | Complete: ownership, generations, obligations, close reports |
| WP2 admitted work, retained cleanup, cycles/history | lifecycle supervisor, runtime/manager.rs, schedule/follow-up controllers, existing Tokio-util | Complete: registration before execution, weak facades, producers before writer drain |
| WP3 endpoints and both store owners | Surreal/Postgres providers, scoped persistence/memory, memory service, credential maintenance, server/embedded | Complete: endpoint/path identity, collision rejection, owned/borrowed closure |
| WP4 dependency completion | vendor/patches/surrealdb-lifecycle, pinned first-party memory export, SQLx pool/listeners | Complete source: SurrealDB3.3.0 and SQLx0.8.6 retained; provenance documented |
| WP5 startup and all exits | server.rs, lifecycle graph, embedded.rs, main/service/signals, sidecar | Complete: acquired-prefix unwind, retained dependencies, actual joins and errors |
| Watchers and background descendants | ConfigManager, ingestion/skill watchers, manager/thread/team/MCP/sandbox scopes | Complete: stop/join watchers before dependents; retain accepted blocking/background work |
| WP6 event-driven recovery/consumers | realtime buses, live API, reconnect/snapshot, frontend typed entity SSE adapters, streaming A2A | Complete: EOF/gap handling, explicit reconnect, reconciliation and streaming-only dispatch |
| WP7 migration | credential migration/principal stores/API | Complete: per-caller authorization, joining, atomic effect/receipt, typed conflict/unavailable/unknown |
| WP8 hermetic fixtures and sidecar | release support/credentials scripts, live harness, sidecar process support, inherited input/control | Complete: generation-bound final readiness, owned cleanup, UAR Windows named-pipe endpoint |
| Local embedding failover | config/settings, embeddings/failover/swappable/fastembed, server wiring | Complete: local compatible switching/recovery and space identity enforcement |
| Real incompatible-space fixture | tests/embedding_lifecycle_failover.rs, locked local fixture assets | Complete: actual distinct equal-dimension inference |
| Delivery/compiler/principal repairs | scoped source commits and collected repair handoffs | Complete source; UAR draft PR418; earlier image receipt retained |
| Budgeted Pending API | runtime/lifecycle.rs, embedded.rs, lifecycle spec/API guide | Complete: stable UUID, retained owner, observer-only budget, explicit unavailable inventory; no fresh compile/run |

The table uses source shorthand; exact file mappings and per-producer ownership
remain in
`lifecycle-plan-implementation-evidence-checklist.md`. Its historical Boss
entries are outside current scope. No dependency upgrade is introduced.
One nonfatal SDK unused-import warning remains from the prior embedding run.

## Actual evidence and remaining obligations

MiniLM preparation exited0. Embedding attempt3 exited0:1 passed,0 failed,
0 ignored; compilation40.81s, test4.35s. It exercised real model
generations1→2→3→4→5, principal-bound durable ingestion/query, distinct
384-dimensional inference,409 mismatch with retained state, and fixture
supervisor/engine-close acknowledgements. Results and exact output remain in
`lifecycle-verification-results-2026-10-21.md` and
`lifecycle-verification-result-embedding-2026-10-21-attempt-3.json`.

The pass predates budget API0f926ed6. No new compilation or execution occurred.
The owned-resource and sidecar targets have no fresh result. The three existing
target commands are ready for explicit approval in
`lifecycle-verification-command-proposal.md`, with declared prerequisites and
coverage limits. Assets have prior successful preparation evidence; no new
availability check or preparation rerun occurred.

The uncomfortable limit: none directly exercises the new budget API branches.
They cannot establish expiry/acknowledgement races, repeated IDs, SnapshotPending
or the embedded budget wrapper. No new test/gate hides this gap. Bounded
observation cannot guarantee finite termination of uncooperative work; the host
must keep its executor alive and cleanup owned.

Full release acceptance remains closed for current-source image binding,
same-image preparation and independent review. The earlier Linux AMD64/ARM64
image at32cb3bcf excludes later source edits. Existing deployment dispatch is
listed separately for approval, not executed. Windows, full frozen embedding
requirements and other coverage limits retain their evidence obligations.
Missing historical503 bodies and c9 sidecar traces cannot be reconstructed by
declaring source complete.

No new tests, gates or production guards were added. Existing guards trace to
the operator's outage, migration, shutdown, sidecar and embedding requirements
and the observed principal trust boundary. Only source inspection and
authorized document/metadata authoring occurred.
