# Lifecycle producer continuation — lead integration

Active phase: `uar-outstanding-issues-v1-1-1::function-first-release-certification`.
Implementation continuation: `deterministic-lifecycle-management` task 5.4.
Verification remains **CLOSED**. This document records authored source and its
limits; it is not a test result or an independent review.

## Changes by source

| Source | Change and reason |
| --- | --- |
| `src/uar/runtime/lifecycle.rs` | Admitted tasks restore a task-local `ResourceScope`. Blocking work moves its operation lease into the actual closure, retaining dependency access after a waiter disappears. Blocking panics are recorded before resuming unwinding. Supervisor shutdown registers its single finalizer under the registry lock, preventing a second caller from closing admission before that registration. |
| `src/uar/file_processing/kreuzberg.rs` | Extraction uses the admitted blocking helper. Explicit budget expiry requests cancellation and then awaits the actual worker; it cannot release ownership on elapsed time. |
| `src/uar/tools/file_tools.rs`, `web_fetch.rs` | Confined file-open and DNS blocking operations retain their inherited host scope through actual closure completion. |
| `src/uar/tools/terminal_process.rs`, `terminal_exec.rs` | Standalone terminal calls use the owned process supervisor, retained cleanup and actual child reaping. A failed signal still leads to an awaited child result; a signal alone is never a reap receipt. |
| `src/uar/runtime/native_skill.rs`, `src/llm/orchestrator.rs` | Direct native calls move an `Arc` skill and owned inputs into an admitted retained task. Dropping its reply waiter cannot drop the tool operation or canonical receipt processing. Existing governance, credential binding and final-wire preparation remain unchanged. |
| `src/llm/health.rs` | The legacy periodic sweep wakes from changed health state or a one-shot business cooldown expiry. Its returned task handle still belongs to its caller. No production caller of this helper was found in this continuation. |
| `src/server.rs`, `src/server/lifecycle_graph.rs` | Startup, request handlers and response-body reads restore lifecycle scope for descendants. Startup descendants keep acquired services alive. Ingestion readiness awaits actual native watcher registration. Configuration and ingestion watchers remain the first joined resources after external admission closes. |
| `src/embedded.rs` | Embedded startup owns an admitted startup scope with dependencies on acquired services, including descendants surviving a canceled initialization waiter. |
| SDK `conn/driver_scope.rs` | Read-only pumps can be canceled locally and joined. Pump admission is atomic with closure, including per-query pumps whose subscribe reply races kill. Unary operations retain their own acknowledged completion contract. |
| SDK `engine/any/{native,wasm}.rs` and HTTP transport files | `Any` reuses typed retained HTTP/WebSocket constructors. HTTP export pumps are tracked; stream errors remain visible. |
| SDK core index builder/admission/module source | Actual builder tasks remain tracked after map removal. Rollback and shutdown join builders before durable deletion or engine release. Cleanup contention is reported after one attempt rather than retried on a timer. The existing test source was adjusted to assert actual completion instead of a spent deadline; it was not run. |
| SDK and core manifests plus root `Cargo.lock` | Enable existing Tokio-util tracking and the optional direct Hyper executor dependency from the existing locked resolution. No dependency version upgrade is intended. |
| OpenSpec runtime-resource and realtime-recovery deltas | Document retained canceled operations, blocking completion, one shutdown owner, builder joins and ordered transport/subscription recovery. |

The state, runtime and trust writers own their separate continuation handoffs.
The trust writer also integrated the explicit reconnect controller into the
server and control protocol before returning shared-file ownership to the lead.
The Boss writer integrated the correlated reconnect-error schema separately.

## Resource order

External admission closes atomically. Configuration and filesystem watchers
then stop and join before downstream mutable services finalize. Their immutable
configuration remains readable. Startup and admitted producer descendants retain
their required dependencies. Runs, team work, observers, native tools and MCP
operations settle before accepted history writes drain. Subscription and
transport scopes close before the persistence and memory owners close their
engines or release borrowed access. Process-signal observation remains until
the other registered resources have acknowledged closure.

The graph is a dependency contract, not a cancellation deadline. A permanently
silent admitted operation can hold it open. Its owner, operation identity and
unknown durable effects must remain inspectable.

## Evidence and remaining limits

Only source reads/searches, authorized implementation/documentation writing,
dependency documentation retrieval and supported metadata mutations were used.
No tests, builds, lint, formatting, typechecks, compiler probes, policy
validators, acceptance scripts, packaging or review gates ran.

All changed source remains uncompiled. Native/WASM/Windows compatibility,
physical engine release, immediate reopen, migration contention, reconnect
races and real child reaping remain unverified. The historical concurrent 503
response body and individual sidecar failure traces are still absent.

The model-stream source trace found owned pull streams rather than detached
adapter tasks. Local read abandonment cannot acknowledge remote provider work;
durable accounting preserves pending liability. The separate model handoff
records provenance fields not supplied by its current accounting record. This
continuation does not invent a remote cancellation API or expand lifecycle work
into an unrelated provider-receipt redesign.

No unsolicited feature, unrelated upgrade, source commit, push, publication or
deployment was added by this continuation. The separately recorded local
first-party Memory commits remain unpublished. The renewed source inventory
must be authored only after all current writer handoffs and metadata updates.
