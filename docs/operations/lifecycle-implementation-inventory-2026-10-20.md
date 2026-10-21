# Renewed UAR lifecycle source inventory

Recorded 2026-10-21T03:23:28.173821+00:00. Execute phase:
`uar-outstanding-issues-v1-1-1::function-first-release-certification`.
Waypoint **3179**, plan **78**. Verification **CLOSED**.
The canonical aggregate run counter is 138/153; it counts run changes, not individual lifecycle tasks.

All planned lifecycle production and fixture source has collected writer handoffs. Scoped implementation and required documentation are published in both draft PRs. The approved Docker delivery completed successfully for both Linux architectures at production source `32cb3bcfc9691212ec0b0359ce22474968bc6850`. This records source authoring and artifact delivery, not runtime acceptance or whole-phase completion.

The JSON companion records saved workspace bytes, including pre-existing unrelated work, separately from the published production commit and image. It is an authored inventory, not an executed source-identity acceptance gate. No local acceptance runner, compiler probe, test, lint, formatter, typecheck, native build, policy validator, packaging check or independent review ran. Only the explicitly authorized Docker delivery workflow supplied compilation and publication evidence. No further embedding design decision is pending; execution evidence remains required for the full preserved contract.

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
| 8.3 / 8.4 | Boss launcher and immutable candidate binding are published and renewed after delivery repairs; exact source pin, provenance and proposed bilingual note | Boss payload binding handoff; draft PR74 |
| 9.5 / 8.5 | Awaited startup logging values are resolved before tracing macros; retained server ownership unchanged; Boss pin and proposed notes renewed to actual source | lifecycle-delivery-startup-send-repair.md; Boss binding handoff |
| 9.4 | All19 observed delivery compiler diagnostics addressed in source: actual tuple APIs, retained IDs, borrower copies, live stream variants/closure, sealed owned proof and atomic recovery CAS | lifecycle-delivery-runtime-repair.md; lifecycle-delivery-state-repair.md; lifecycle-delivery-mcp-repair.md |

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

## Published source and image delivery

| Task | Current state |
| --- | --- |
| 8.3 | Source-complete: Boss candidate pin names actual UAR commit `32cb3bcfc9691212ec0b0359ce22474968bc6850`; writer handoff collected and both repositories pushed. |
| 9.3 | Complete for requested delivery: both drafts published; approved run39532243053 succeeded and published the combined Linux image `sha256:8e6fb6f453323fa921d956adb78d15b4ab4ae6137acb3cc5827a83bcd9bc1fc8`. Routine verification remains CLOSED. |

The operator explicitly approved source-only hook exceptions. UAR hooks were
skipped only for authorized scoped commits; no stored hook setting was changed.
No validator, lint/typecheck or Commitlint result is claimed. See
`lifecycle-publication-approval.md` for authorization and actual delivery.
The scoped implementation paths are recorded in
`lifecycle-publication-scope-2026-10-20.txt`; the bounded delivery repair is recorded in its own source commit; publication follow-ups contain required documentation and canonical coordination updates. Unrelated dirty files remain.

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
https://github.com/Prometheus-AGS/the-boss/pull/74. Current immutable UAR
production source/candidate pin: `32cb3bcfc9691212ec0b0359ce22474968bc6850`. Current Boss source commit:
`a609869079bf6d51af196944965e6f86ee6c0758`, SSH-signed and DCO-signed off; GitHub reports signature
`verified: true`, reason `valid`. Existing registered signing key and account
noreply identity were selected per command; no account or stored identity changed.

Delivery history:

- Run https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39523808909 at `ad557b0d00134c003bf0e87d248d1aef2e9ce4e5`: both architectures failed19 compiler diagnostics; manifest publication skipped, no image.
- Run https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39528865081 at `6d50cb8a05fb2af33e0928dc8d7ac3ff0c5f3e46`: both architectures failed the same two non-Send diagnostics from one await inside startup tracing. Manifest publication was skipped; no image was published. Actual job logs and terminal outcome are retained in the failure receipt.
- Run https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39532243053 at `32cb3bcfc9691212ec0b0359ce22474968bc6850`: completed success. ARM64 job119000651255, AMD64 job119000614670 and manifest job119019008132 succeeded. The merged index is `sha256:8e6fb6f453323fa921d956adb78d15b4ab4ae6137acb3cc5827a83bcd9bc1fc8` and the published tag is `sha-32cb3bcfc9691212ec0b0359ce22474968bc6850`. Its workflow inspection reports linux/arm64 and linux/amd64. The exact receipt is `lifecycle-image-delivery-39532243053.md`. No runtime acceptance is claimed.

Use the immutable image reference:

```text
ghcr.io/prometheus-ags/universal-agent-runtime@sha256:8e6fb6f453323fa921d956adb78d15b4ab4ae6137acb3cc5827a83bcd9bc1fc8
```


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

No local or native integration has run. Both Linux release compilations and combined image publication succeeded in the authorized delivery workflow. Inference, lifecycle races,
engine release/reopen, migration contention and lost replies, remote transports,
Windows named pipes, WASM and customer packages remain unverified. The historical503 response
body and individual c9 sidecar traces remain missing; stronger source contracts
do not manufacture their causes. Physical engine-close fault injection has no
public deterministic hook. A permanently silent operation may remain pending;
local release does not resolve unknown remote effects. Per-attempt provider,
credential and final-wire provenance remains a separately recorded limitation.

No tests or guards were added outside the requested lifecycle/local-embedding/
payload scope. Pre-existing unrelated changes were not reverted. Source-only
inspection, authorized writing and supported KBD/team metadata were used. No routine
verification result, whole-phase completion or release acceptance is claimed.
All planned lifecycle production and fixture source has writer handoffs, and requested image delivery is complete. Canonical preparation, review, functional acceptance and reflection remain unfinished.
