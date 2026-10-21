# Renewed UAR lifecycle source inventory

Recorded 2026-10-21T00:40:11.332118+00:00. Execute phase:
`uar-outstanding-issues-v1-1-1::function-first-release-certification`.
Waypoint **3158**, plan **70**.
Verification remains **CLOSED**. The canonical aggregate run counter is
137/153; it counts changes across the run, not individual lifecycle tasks.

This inventory replaces the earlier task5.4 snapshot. It records authored
source and collected handoffs, not compilation or acceptance. The companion
JSON records saved file bytes after this documentation update and includes the
current base Git HEAD separately from the newer uncommitted source. No local acceptance runner, identity gate, compiler probe, test, lint,
formatter, typecheck, native build, packaging, policy-validator or independent
review command has run. The separately approved Docker delivery workflow is
running; it is not routine verification or product acceptance.

## Integrated implementation

| Tasks | Source implementation | Collected handoff |
| --- | --- | --- |
| 1.1 / WP1 | Resource ownership graph, generation/state events, operation IDs, on-demand obligations, SDK engine-close evidence | lifecycle-resource-inventory.md; lifecycle-runtime-handoff.md |
| 2.1 / WP2 | Admission lock plus Tokio-util tracking, retained task outcomes, shared finalization, weak controllers, history producer/drain ordering | lifecycle-runtime-handoff.md; lifecycle-sandbox-handoff.md |
| 3.1 / WP3 | Shared endpoint parsing, physical path collision refusal, separate owned/borrowed persistence and memory, scoped closed facades | lifecycle-state-handoff.md |
| 4.1 / WP4 | SurrealDB3.3.0 source overlays and first-party Memory export; explicit SDK drivers, pumps, subscriptions, index/maintenance/engine joins; SQLx0.8.6 pool closure | lifecycle-state-handoff.md; lifecycle-grpc-continuation.md; vendor provenance READMEs |
| 5.1–5.4 / WP5 | Partial-startup unwind and all exit paths, watcher-first joins, inherited process input, actual child reaping, blocking descendants, canceled waiter ownership | lifecycle-startup-handoff.md; lifecycle-entrypoint-handoff.md; lifecycle-watcher-order-handoff.md; lifecycle-producer-*-continuation.md |
| 6.1 / WP6 | Event-driven work notifications, one-shot business scheduling, explicit reconnect, subscription/SSE gap handling, typed frontend ingestion, streaming-only A2A | lifecycle-console-handoff.md; lifecycle-a2a-handoff.md; lifecycle-reconnect-continuation.md |
| 7.1 / WP7 | Independently authorized same-runtime migration joining, atomic effects/receipts, typed conflict/unavailable/unknown outcomes and stable reconciliation identity | lifecycle-trust-handoff.md; lifecycle-producer-trust-continuation.md |
| 8.1 / WP8 | Immutable fixture generations, final PostgreSQL readiness, hermetic inputs, panic-safe fixture cleanup, controlled confinement handshake and per-case receipts | lifecycle-trust-handoff.md; authored sidecar/live fixtures |
| 6.2 | Opt-in strict local FastEmbed pair, real readiness inference, identical space/assets, retained inference/control work, live promotion, reusable backup and generation-checked recovery, actual native-engine cleanup | lifecycle-embedding-runtime-handoff.md |
| 8.2 | Real SurrealKV-backed ingestion/query through generations1→2→3→4→5; distinct real equal-dimension model refusal and compatible re-probe; acquired-prefix panic cleanup; pinned public MiniLM asset metadata and deferred preparation source | lifecycle-embedding-fixture-handoff.md; tests/fixtures/local_embedding_lifecycle/model-preparation.md |
| 8.3 | Boss launcher and immutable candidate source committed and pushed; exact UAR source pin, retained provenance and proposed bilingual design note | Boss lifecycle payload binding handoff; draft PR74 |

All relative handoff names above resolve in `docs/operations/`. Writer handoffs
record source changes and unverified limits. No native inference, model download
or execution was performed for the new fixture. The original source-only
ownership scenarios in `tests/lifecycle_owned_resources.rs` also remain unrun.

The embedding scope is registered before acquisition. Ingestion, runtime work
and HTTP depend on it so consumers drain first. Member retirement waits for
accepted inference and blocking joins. Recovery publishes a proved compatible
primary and retains backup standby for another cycle. The optional configuration
and authenticated control paths are documented in the runtime handoff. Hosted
endpoints, credentials, personal settings changes, periodic probes and timed
success claims are absent from this new local failover path.

## Source publication and remaining image delivery

| Task | Current state |
| --- | --- |
| 8.3 | Source-complete: Boss candidate pin names actual UAR commit `ad557b0d00134c003bf0e87d248d1aef2e9ce4e5`; writer handoff collected and both repositories pushed. |
| 9.3 | UAR draft PR418 and Boss draft PR74 are published. Docker workflow is running; actual image publication/digest remains pending. |

The operator explicitly approved source-only hook exceptions. UAR hooks were
skipped only for authorized scoped commits; no stored hook setting was changed.
No validator, lint/typecheck or Commitlint result is claimed. See
`lifecycle-publication-approval.md` for authorization and actual delivery.
The scoped implementation paths are recorded in
`lifecycle-publication-scope-2026-10-20.txt`; later commits contain only required
documentation and canonical coordination updates. Unrelated dirty files remain.

The new standalone producer exports exact committed root/gitlink objects and
builds with a fresh target directory. Boss verifies the selected commit/tree,
platform, features, private protocol, executable/log/archive hashes and inner
manifest. Unrelated dirty worktree files cannot enter that candidate. Neither
the producer nor the consumer has been executed. Public and other-platform
payloads retain their existing identities. Boss source changes are limited to
`UarSidecarService.ts`, local payload source/loader/preparer, the new binding
helper, source pin and the three lifecycle handoffs and proposed bilingual payload design note.

Published drafts:
https://github.com/Prometheus-AGS/universal-agent-runtime/pull/418 and
https://github.com/Prometheus-AGS/the-boss/pull/74. The immutable UAR production
source/candidate pin is `ad557b0d00134c003bf0e87d248d1aef2e9ce4e5`. Subsequent source-inventory and coordination
commits do not relabel that candidate. Deployment run: https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39523808909.
It is running at the pinned source. No available image or published digest is
claimed yet. The workflow request is authorized delivery, not acceptance.

Boss published commit: `7e7ff9f696e7f32f69c223a9aaa49b815134a23b`.
GitHub reports its SSH signature `verified: true`, reason `valid`. The commit
retains a DCO signoff. The initial configured key was not registered for signing;
the final commit uses the existing registered Boss release signing key and
account noreply address through per-command Git options. No key registration
or persistent Git identity setting was changed. Signature metadata is source
publication evidence, not product verification.

## Canonical phase obligations and preserved requirements

Active child release tasks4.1/4.2/4.3/5.1 remain pending: fresh same-image
preparation, independent completed-artifact review, local functional acceptance
and reflection/handoff. Parent6.2/7.1/7.2/7.3 remain accountable for acceptance,
immutable release publication, both-architecture evidence and release receipts.
Lifecycle9.1/9.2 remain pending. Neither this document nor completed source
checkboxes can override those canonical ledgers or open the gate.

The full embedding exit contract remains intact. New source provides live
failover and a real incompatible-space fixture; it does not turn either into
passing evidence. Process-restart binding/hydration, invalid settings mutation,
and every other frozen condition still need actual authorized receipts. Old
mirror A→B→A restart evidence is not live failover. The new fixture covers the
RAG/KB path; MemoryService's separate provider configuration is not certified by
it. The nested child stays NOT READY until every frozen condition is evidenced.

The first-party memory export stays pinned to
`9e36b1cd8b83a657366eb1e3ef8cd837178336de`; its local upstream commits have not
been pushed. SurrealDB remains3.3.0, SQLx0.8.6 and existing Tokio/Tokio-util.
No unrelated dependency upgrade or unsolicited feature is intended. Guards
trace to the observed admission/cancellation/shutdown, credential contention,
stale generation, readiness/configuration and incompatible-space failures, or
the actual authenticated peer/private-input boundaries.

## Exact commands and uncomfortable limits

`lifecycle-verification-command-proposal.md` gives the exact deferred model
preparation, real-model integration, owned-resource, sidecar, immutable native
candidate, local/public Boss package and release-runner commands. Each requires
its own explicit operator approval and stated prerequisites. No hidden
preflight is implied. The full release manifest is stale for these bytes;
fresh source/image/supervisor preparation must be bound before it can run.

No local or native integration has run. The image build is pending; inference, lifecycle races,
engine release/reopen, migration contention and lost replies, remote transports,
Windows named pipes, WASM and customer packages. The historical503 response
body and individual c9 sidecar traces remain missing; stronger source contracts
do not manufacture their causes. Physical engine-close fault injection has no
public deterministic hook. A permanently silent operation may remain pending;
local release does not resolve unknown remote effects. Per-attempt provider,
credential and final-wire provenance remains a separately recorded limitation.

No tests or guards were added outside the requested lifecycle/local-embedding/
payload scope. Pre-existing unrelated changes were not reverted. Source-only
inspection, authorized writing and supported KBD/team metadata were used. No routine
verification result, whole-phase completion or release acceptance is claimed.
All planned production and fixture source has writer handoffs; image delivery
and the canonical evidence obligations remain unfinished.
