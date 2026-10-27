# A2UI locked API reconciliation

Phase: user-outcome implementation in `uar-post-lifecycle-verification-readiness`.
Gate CLOSED. Source research and minimum implementation only; no acceptance claim.
This supersedes the installed-0.10.4 Catalog/parser guidance in
`user-outcome-a2ui-action-production.md`; that earlier artifact remains historical.

## Observed drift and source

The wrapper manifest and both pnpm lockfiles select `@a2ui/web_core` 0.12.0.
The actual wrapper node_modules link points to root's installed 0.10.4 directory.
Earlier Catalog constructor repairs and parser normalization used those installed
0.10.4 declarations. They do not describe the locked target.

Firecrawl developer CLI unavailability was already recorded by the lead. This
bounded reconciliation read official npm metadata and the published source tarball
with Python standard-library HTTP/tar readers, without installation, importing
package code, extracting into node_modules or executing upstream JavaScript:

- https://registry.npmjs.org/@a2ui%2fweb_core/0.12.0
- https://registry.npmjs.org/@a2ui/web_core/-/web_core-0.12.0.tgz
- Package repository metadata points to https://github.com/a2ui-project/a2ui/tree/main/typescript/web_core

Registry metadata reports integrity
`sha512-wEg8954eDM4tx70O+qoUQYSCJbCywEIZEiqOpNXrmQUAKGsMNM47uns/7AE0W0nMfFbmJWE/huBwUVFaq+pDCA==`,
also present in the frontend lockfile. This is metadata/source agreement, not an
executed package integrity or product check.

## Declared 0.12.0 API and minimum edits

Published `src/catalog/types.d.ts` declares
`Catalog(id, protocolVersion, components?, functions?, themeSchema?, instructions?)`.
Both `uar-basic-catalog.ts` and `uar-entity-catalog.ts` now pass explicit
`"v0.9.1"` as the required second argument, followed by their actual components.

Published `src/v0_9/schema/server-to-client.js` explicitly accepts both `v0.9`
and `v0.9.1` in all four message schemas. The exported `A2uiMessageSchema` is
re-exported by `src/v0_9/schema/index.d.ts`, then `src/v0_9/index.d.ts`, then our
thin wrapper. `a2ui-artifact-block.tsx` now parses the project-validated canonical
v0.9.1 message directly; the obsolete forced downgrade/comment was removed.
The project profile and protocol validation are unchanged.

Other source calls align with the published declarations:

- `MessageProcessor<T>(catalogs, actionHandler?, options?)`; current call supplies
  catalogs only. `model.surfacesMap` remains a readonly map and `processMessages`
  accepts the parsed message list.
- `SurfaceModel.onAction` is `EventSource<ActionPayload>`;
  `subscribe(listener)` returns `Subscription.unsubscribe()`. Its payload carries
  name, surfaceId, sourceComponentId, timestamp and context. Async listeners are
  allowed. The surface listener remains the action path.
- `surface.dataModel.get<T>(path)` remains available for the detached root
  snapshot. `ComponentContext(surface,id,basePath)` and `GenericBinder(context,
  schema)` plus subscribe/snapshot/dispose remain declared and re-exported.
- `SurfaceModel.catalog` remains a deprecated but present alias of
  `defaultCatalog`, so existing rendering lookup needs no migration here.
- Adapter source `DEFAULT_CATALOG_COMPATIBILITY` explicitly maps 0.9.1 to both
  0.9.1 and 0.9 catalogs. No unsupported-version workaround was introduced.

Only these three production files changed in this bounded task:
`frontend/packages/a2ui-uar/src/catalog/uar-basic-catalog.ts`,
`frontend/packages/a2ui-uar/src/catalog/uar-entity-catalog.ts`, and
`frontend/src/features/chat/components/a2ui-artifact-block.tsx`.
Root's action-hook effect/status/hydration fixes and stream store were untouched.
No new guard, dependency, UI redesign or unrelated change was added.

## Entity-management reconciliation: unresolved installed drift

Both locks and `frontend/package.json` select the umbrella package
`@prometheus-ags/prometheus-entity-management` 3.0.3 (core also 3.0.3).
However `frontend/node_modules/@prometheus-ags/prometheus-entity-management`
resolves to local `frontend/packages/prometheus-entity-management/packages/entity-graph-react`.
Its actual package declares `@prometheus-ags/entity-graph-react` 4.2.0. Thus the
installed source previously consulted is NOT established as the locked 3.0.3
source. This was explicitly reported to root; no entity dependency, source,
lockfile or installed link was changed by this task.

Official 3.0.3 metadata/source was also read:
https://registry.npmjs.org/@prometheus-ags%2fprometheus-entity-management/3.0.3
and https://registry.npmjs.org/@prometheus-ags/prometheus-entity-management/-/prometheus-entity-management-3.0.3.tgz .
The 3.0.3 umbrella declarations re-export core APIs; any further locked-core
persistence reconciliation is separate root-owned integration, not assumed
compatible from the installed 4.2.0 package. Root's current bootstrap uses its
own confirmed `persistGraphToStorage` path and was preserved.

## Execution and handoff status

Only source reads (`cat`, `sed`, `rg`, `readlink`, standard-library HTTP/tar source
inspection) and authorized source/document writes ran, with login:false. Some
exploratory source paths were absent; no missing path was fabricated into an
implementation. No npm/pnpm install, dependency import, test, compiler/typecheck,
build, lint, format, browser, model, service, review verification or other product
gate ran. No product PASS, build compatibility, UI functionality or package
installation claim is made. Snapshot001 remains historical; the lead must renew
source inventory because three production files changed.

The uncomfortable limitation is that installed dependencies are still divergent
from the locks. This source targets official locked 0.12.0; running against the
old installed 0.10.4 tree would not establish locked-version behavior. Entity
installed-version drift remains an explicit lead integration item. Ownership is
released after this handoff.
