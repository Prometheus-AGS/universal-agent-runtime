# Goals — perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion > fix-embedded-sse-offline-reconnect

1. Make the embedded SurrealDB SSE client consume the named `entity.change`
   event and payload emitted by `/api/uar/sync/stream`.
2. Restore the registered embedded adapter after a detected stream error without
   reloading the page, opening parallel streams, or leaving retries after
   unsubscribe.
3. Prove delivery and reconnection through the application adapter with focused
   unit controls and one live browser scenario that changes visible product
   state exactly once.
4. Return truthful evidence to `screen-by-screen-validation` so its local-first
   tasks can resume; do not alter parent certification evidence inside this
   child.
5. Correct the source `prometheus-entity-management` projection defect, advance
   UAR to the tested source/compatibility commit, and deliver its rc.2 version
   through the separate canonical Changesets PR instead of adding a
   screen-local refresh workaround.

## Boundaries

- No backend endpoint changes unless a new observed failure proves the existing
  endpoint cannot support the selected client contract.
- No changes to the shared `/api/live` adapter, dependency declarations,
  frontend manifest, lockfile, unrelated realtime transports, or UI design.
  The root BDD preparation script may build the upstream React package through
  its declared dependency graph. The approved upstream React package
  implementation/test, canonical version PR, and UAR submodule pin are in
  scope.
- No phase-wide screen recertification, release certification, tag, push, or
  publication from this child.
