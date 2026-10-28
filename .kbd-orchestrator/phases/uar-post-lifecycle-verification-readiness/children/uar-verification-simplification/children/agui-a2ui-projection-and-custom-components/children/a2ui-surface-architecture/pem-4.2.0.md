# prometheus-entity-management 4.2.0 — A2UI rendering findings

Read-only, 2026-10-28. Source: `/Users/gqadonis/Projects/prometheus/prometheus-entity-management`
at commit `28a500d6`. Nothing was built or tested. Registry versions were read
with `npm view <pkg> dist-tags` and the pub.dev API.

Operator instruction (2026-10-28): UI/UX testing must take PEM 4.2.0 into
account. The embedded React application should support it, and it handles both
React and Flutter A2UI rendering.

## Published versions

| Package | Workspace | Registry (2026-10-28) |
|---|---|---|
| `@prometheus-ags/a2ui-react` | 4.2.0 | npm `latest` 4.2.0 |
| `@prometheus-ags/entity-graph-core` | 4.2.0 | npm `latest` 4.2.0 |
| `@prometheus-ags/prometheus-entity-management` | 4.2.0 | npm `latest` 4.2.0 (compatibility alias for `entity-graph-react`, README package table) |
| `entity_graph_flutter` | 4.0.0 (`packages/entity_graph_flutter/pubspec.yaml:6`) | pub.dev 4.0.0 |

`release/npm-registry-status.json` still says
`"releaseStatus": "candidate-prepared-unpublished"` and lists 4.1.1 as the
latest (verified 2026-10-08). The registry query above supersedes it, so that
file is stale.

## The React A2UI runtime (`packages/a2ui-react`)

Sources: `packages/a2ui-react/README.md`, `src/official/runtime.ts`,
`release/a2ui-protocol-bridge.md`.

- **Built on the official stack.** `@a2ui/react` 0.12.0, `@a2ui/web_core`
  0.12.0 and `@a2ui/markdown-it` 0.2.0. Peer `@ag-ui/core >=0.0.59 <0.1.0`,
  React 19 (`package.json`).
- **Accepted input.** A2UI v1.0-RC and v0.9.1. A v1.0 `createSurface` with
  inline `components` and `dataModel` is decomposed into the official v0.9
  processor. The package "does not implement a second renderer, data model, or
  surface store" (README, "Pinned version axes").
- **One long-lived runtime, owned by the application.**
  `createPrometheusA2uiRuntime()` holds a single official `MessageProcessor`
  across messages, and `dispose()` ends it. This differs from UAR's renderer,
  which builds a fresh processor per artifact
  (`frontend/src/features/chat/components/a2ui-artifact-block.tsx:271-273`).
- **AG-UI activity transport.** `processAgUiEvent()` accepts `ACTIVITY_SNAPSHOT`
  with `activityType:"a2ui-surface"` and `content.a2ui_operations`
  (`runtime.ts:272-316`). The behavior per activity `messageId`:
  - `replace:false` for a known `messageId` is ignored (`:289-291`);
  - otherwise the surfaces that this `messageId` created before are deleted,
    and the operations are replayed (`:306-315`).

  So a cumulative `replace:true` snapshot resets those surfaces, including
  their data model. A new activity message that carries only
  `updateComponents`/`updateDataModel` for an existing surface deletes nothing,
  because the prior set for a new `messageId` is empty. That is the
  incremental path. It was inferred from the code and has not been run.
- **Atomic batches.** `processMessages()` preflights the whole batch against a
  shadow official processor seeded from the current surfaces, so a later error
  cannot partially commit an earlier message (README; `runtime.ts:438-494`).
- **State-to-messages reconstruction exists.** The preflight rebuilds each live
  surface as messages: `createSurface`, then every component, then
  `updateDataModel` at `/` (`runtime.ts:445-486`). This is the compaction step
  that `web_core` itself lacks. It is roughly 40 lines of real code that
  depends only on official `web_core` state.
- **Catalogs.** `createPrometheusA2uiCatalog()` derives implementations from
  the official React basic catalog through explicit component and function
  allowlists. `openUrl` is excluded by default. The latest commit `34a21156`
  ("let applications supply catalog component implementations") lets an
  application add its own component implementations.
- **Action authority.** Actions go through a default-deny policy:
  - official schema;
  - exact registered name;
  - Zod context schema;
  - allowlisted entity type, action and fields;
  - the application's `authorize` callback (tenant and scope);
  - approval for `replace` and `remove`;
  - only then a graph write.

  The built-in action names are `prometheus.entity.upsert`, `replace`,
  `remove`, `patch`, `unpatch` and `clear-patch`. "Protocol validity never
  grants application authority" (`release/a2ui-protocol-bridge.md`, "Action
  authorization").
- **Layering.** Components go through Prometheus A2UI hooks and provider, then
  runtime policy, then the graph store. Renderer components never touch the
  store.
- **Legacy surface.** `@prometheus-ags/a2ui-react/ag-ui` keeps the pre-3.0
  AG-UI chat and state components (`EntityChat`, `EntityStream`, and others).
  The package says they are not official A2UI.
- **Stated limits** (`release/a2ui-protocol-bridge.md`, "Limits"). The bridge
  does not prove:
  - "Flutter or native A2UI rendering";
  - A2A conformance;
  - the complete agentic example's streaming and cancellation flows.

## Flutter

- **The package version is 4.0.0** on pub.dev.
- **A2UI rendering is in the showcase, not the package.** The `flutter-riverpod`
  example accepts strict A2UI 1.0-RC surfaces and normalizes them to GenUI
  0.10.2 only after catalog, component, function, action, tenant and entity
  validation (`packages/entity_graph_flutter/README.md:368-370`;
  `examples/flutter-riverpod/pubspec.yaml:21`; `examples/flutter-riverpod/lib/a2ui/*`).
- **The rendering evidence is example-level.** Coverage includes Flutter
  3.44.8 goldens and Android emulator and iOS simulator smoke runs. Physical
  devices are not claimed.

## UAR today

- **Old pins.** `frontend/package.json:46-47` pins
  `@prometheus-ags/entity-graph-core` and
  `@prometheus-ags/prometheus-entity-management` at **3.0.3**, two major
  versions behind.
- **A name collision.** UAR has a local workspace package named
  `@prometheus-ags/a2ui-react` at version 0.10.1, a "reference implementation
  only" re-export of Google's `@a2ui/react`. `frontend/packages/a2ui-uar`
  depends on it through `workspace:*`. PEM 4.2.0 publishes a different package
  under the same name.
- **A vendored copy.** `frontend/packages/prometheus-entity-management/` holds a
  copy of the PEM workspace (`entity-graph-workspace` 0.0.0). Which PEM
  revision it was copied from was not determined.
- **UAR's own renderer.** `@prometheus-ags/a2ui-uar` is built directly on
  `web_core` with shadcn and react-aria. It reduces each artifact with a fresh
  processor, and the console uses a separate hand-written reducer
  (`uar-surface-consumers.md` §4, §6).

## What this means for the architecture analysis (revision 2 inputs)

1. **The client side of C2 and W6 has an existing answer.** A long-lived,
   application-owned PEM runtime per client keeps one processor per surface.
   Incremental activity messages update a surface without resetting its data
   model. Snapshot restatement still resets it. So morphs must be incremental,
   and only reconnects use full snapshots.
2. **Compaction is feasible on official `web_core` state.** PEM's preflight
   shows how. A non-authoritative server mirror running `web_core` (option D,
   now concrete) could compact a long-lived surface into a v1.0 inline
   `createSurface`. That removes the 128-operation ceiling for reconnects, and
   it validates exactly as the client does (C3).
3. **The authorization model differs.** PEM's action policy runs in the client
   application, with the application's `authorize` callback. UAR's rule is that
   mutating actions are gated in the trusted host. So client policy is a
   first filter, and the server's check (offered, current, permitted) remains
   authoritative.
4. **Versions to settle.**
   - AG-UI: PEM targets `@ag-ui/core` 0.0.59. The upstream research read the
     AG-UI 1.0 spec at `dab9c95f`.
   - A2UI: PEM accepts v1.0-RC and v0.9.1. UAR admits v0.9 and v0.9.1.
5. **UI/UX testing.** The same surfaces should render through:
   - the React path: UAR's frontend on PEM 4.2.0 `a2ui-react`;
   - the Flutter path: GenUI 0.10.2 through `entity_graph_flutter`.

   Shared message fixtures and the same action-policy outcomes would cover
   both. The Flutter A2UI path is at example level today, not package level.
6. **Adoption cost in UAR.**
   - upgrading from 3.0.3 to 4.2.0;
   - resolving the `@prometheus-ags/a2ui-react` name collision;
   - deciding whether `a2ui-uar` is replaced by PEM's runtime or rebuilt on it.
