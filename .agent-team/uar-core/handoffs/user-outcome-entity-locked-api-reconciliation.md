# Locked entity API source reconciliation

Phase: `uar-post-lifecycle-verification-readiness`, implementation. Gate CLOSED.
Bounded source correction and this handoff authored. No package installation.

## Outcome

No missing export or signature incompatibility was identified between the new
A2UI action bootstrap/hooks/domain/stream ingestion calls and the official locked
umbrella/core 3.0.3 packages. Installed local 4.2.0 is not used as API authority.
The installed dependency closure still needs separately approved installation
against the locks. This conclusion is source compatibility, not a build or
runtime PASS.

A concrete semantic limitation was identified and reported separately to root:
locked core `startLocalFirstGraph.ready` resolves after a malformed persisted
snapshot produces `{ok:false,error}` from hydration. It subsequently sets phase
ready/offline while retaining the error. Current bootstrap awaits ready and
then enables action persistence without inspecting `runtime.getStatus().error`.
Therefore ready resolution alone does not establish successful restoration of
old consumed action receipts. Missing storage returns ok:false too, but does not
set error, so checking persistent error can distinguish fresh storage from the
observed malformed-snapshot branch. Minimal proposed production scope, if root
accepts this repair, is `frontend/src/entities/bootstrap.ts` and its existing
bootstrap source-only tests. Root authorized this minimal correction after the exact finding was reported.
This is a hydration contract concern, not an invented 3.x missing API.

## Skills applied

Read `.agents/skills/prometheus-entity-skills/entity-graph-setup/SKILL.md` and
`.agents/skills/prometheus-entity-skills/entity-graph-realtime/SKILL.md`.
Task-specific guidance: normalized entities remain canonical; components consume
typed platform hooks; domain actions sequence mutations; transport owns I/O;
primitive selectors constrain rerenders; hydration precedes live sync; explicit
subscription cleanup matters. Skills' executable detection/verification steps
were not run because the operator verification lock takes precedence. No UI
change or design acceptance is claimed.

## Official source references

Firecrawl developer CLI unavailability was previously recorded by root; official
npm metadata and package tarball declarations/bundled source were read directly:

- https://registry.npmjs.org/@prometheus-ags%2fprometheus-entity-management/3.0.3
- https://registry.npmjs.org/@prometheus-ags/prometheus-entity-management/-/prometheus-entity-management-3.0.3.tgz
- https://registry.npmjs.org/@prometheus-ags%2fentity-graph-core/3.0.3
- https://registry.npmjs.org/@prometheus-ags/entity-graph-core/-/entity-graph-core-3.0.3.tgz

Tarballs were read in memory with standard-library HTTP/tar readers. Package
code was never imported/executed. Published core integrity metadata is
`sha512-fYI+1siZMQh9PoBIV1Cbevh/qQpaveIkX6OA5oC1nDgVk4EBwCU+mkdFLqDYVUzBdWvXxokRL55Bsa96WslVRA==`.
Umbrella metadata reports
`sha512-MHHh69c82pkLKP58KyZPxpWc3zzaMkeTx0U2ZwAizHZ5qYAnUJqQbh/3IRuSuXc0QEW3ESM5mZxyoAc3dIySPg==`.
These identify registry metadata, not an executed integrity check.

## Exact source contract comparison

Published umbrella `dist/index.d.ts` re-exports core `persistGraphToStorage`,
`graphStore`, `startLocalFirstGraph`, `createPGlitePersistenceAdapter`, persistence
interfaces and retry policy. Its React `useGraphStore` is a callable selector
hook intersected with GraphStore; bundled source assigns vanilla graphStore
methods to that hook. This supports platform primitive selectors and existing
runtime-ingest `useGraphStore.getState()` calls.

Published core `dist/index.d.ts` declares:

- `persistGraphToStorage({storage,key,store?,pendingActions?})` returning a Promise
  of `{ok:true,key,bytes,persistedAt}`. The bootstrap uses the required storage
  and key; omitted store selects the same singleton used by action domain.
- `GraphPersistenceAdapter.set(key,value): Promise<void>|void`. Bundled
  `dist/index.mjs` persistence serializes a cloned singleton snapshot, directly
  awaits `storage.set`, and does not catch rejection. The PGlite adapter's set
  directly awaits its INSERT/ON CONFLICT query. Current awaited receipt writer
  therefore has a real confirmed write/error path; no persistNow swallow is
  assumed. Locked 3.0.3 persistNow also awaits direct persistence, but current
  bootstrap does not depend on that method.
- `startLocalFirstGraph` accepts storage/key/replayPendingActions/retryPolicy;
  runtime has ready/dispose/persistNow/hydrate/getStatus. Current bootstrap's
  arguments and typed dependency abstraction match those declarations.
- `readEntity<T>(type,id):T|null`, `replaceEntity(type,id,Record<string,unknown>)`
  and `registerSchema({type,relations?,globalListKeys?})` support the current
  receipt domain and registration. Source replacement publishes actual entity
  state synchronously; no fetched-list loop was introduced.
- Vanilla store subscribe accepts listener or selector/listener and returns an
  unsubscribe function. React hook accepts `<T>(state=>T):T`; current hooks
  select status or continuation ID directly. No custom subscription API is
  fabricated.
- Existing runtime event ingestion uses declared `upsertEntity(type,id,data)`.
  `upsertEntities` and atomic `ingestFetchedList` also exist, but the new action
  integration does not ingest a fetched list and requires no API migration.

Source inspected: current `frontend/src/entities/bootstrap.ts`, platform index,
A2UI action index/contracts/hooks/domain/durability, chat-stream-store references
and `entities/runtime-ingest.ts`. Exploration encountered absent guessed register
filenames; actual registration is in the read A2UI action index. No code was
written using those nonexistent paths.

## Remaining risk and explicit execution status

The uncomfortable scenario is corrupt persisted receipt state: correct current
write acknowledgement does not prove earlier consumed actions were restored.
Root received the exact semantic finding and authorized the bounded files above.
The new guard traces to corrupt stored receipts enabling action admission before
trustworthy hydration; it does not silently broaden production scope.

Commands performed only source inspection and authorized source/document writes, all
with login:false. No install/import/check/typecheck/build/test/lint/format,
review certification, browser, database/server/provider call or acceptance run
was performed. Signature/source agreement is not compiled interoperability,
PGlite durability, hydration behavior or successful packet acceptance. Installed
4.2.0 links still differ from locked 3.0.3, and resolving that installed closure
requires future explicit authorization. Ownership of this research packet is
released to root; no unrelated additions were made.


## Root-authorized correction delivered

- `frontend/src/entities/bootstrap.ts`: after awaited runtime readiness, reads
  actual `getStatus()` and rejects non-null error or a phase other than ready or
  offline. The error is content-free. Existing catch disposes the runtime;
  sync and action-persistence binding occur only after this guard. A fresh
  database with no old snapshot remains allowed because locked source leaves
  error null in that branch. Offline trustworthy hydration remains allowed.
- `frontend/src/entities/bootstrap.test.ts`: success fakes now supply the
  declared complete status shape. Authored regression models the exact locked
  resolved-ready/retained-error branch and asserts disposal, no sync and no
  action-persistence writer binding. The regression has not run.
- This handoff records reconciliation and correction. No other files changed.

Exact upstream members: official core 3.0.3 tarball URL above,
`package/dist/index.mjs`: `hydrateGraphFromStorage`, `startLocalFirstGraph`'s
`ready` async closure, `persistGraphToStorage`, and
`createPGlitePersistenceAdapter`; declarations are in
`package/dist/index.d.ts` (`GraphSyncStatus`, `LocalFirstGraphRuntime`,
`PersistGraphToStorageOptions`). Official umbrella `package/dist/index.d.ts`
re-exports these core contracts. No installed-local4.2.0 implementation was used
as evidence for this repair. Both implementation and authored regression remain
unverified under CLOSED; parent must renew source inventory for the two edits.
The installed-lock drift still requires future approved installation; changing
application code does not repair node_modules links.
