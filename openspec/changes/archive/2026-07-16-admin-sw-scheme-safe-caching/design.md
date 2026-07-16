## Context

`frontend/public/sw.js` / `static/sw.js` (byte-identical) call
`cache.put(event.request, clone)` at two sites — the cache-first asset
branch (line 64) and the network-first HTML branch (line 79) — without
checking the request's URL scheme first. The Cache API rejects non-
`http(s)` requests, throwing uncaught inside the `.then()` chain.

## Goals / Non-Goals

**Goals:**
- Eliminate the uncaught `TypeError` for non-http(s) request schemes at
  both call sites.
- Preserve identical caching behavior for the normal http(s) path.

**Non-Goals:**
- No change to the existing skip-list at lines 40-47 (method/path
  filtering) — this is a separate, already-correct guard for a different
  concern (skipping API/streaming routes entirely, before either cache
  branch runs).
- No change to cache invalidation, versioning, or the offline-fallback
  behavior in the `.catch()` block.

## Decisions

**D1 — Guard at the point of `cache.put()`, not earlier in the fetch
handler.**
The existing lines 40-47 skip block runs once, before the branch split,
and is about *which requests this service worker handles at all*
(method/path). The scheme check is a different, narrower concern — *of
the requests already being handled, which are cacheable* — and only
`cache.put()` actually requires an http(s) scheme (the `fetch()` call and
`caches.match()` lookups are unaffected). Guarding right before each
`cache.put()` call keeps the check next to the code it protects and
avoids conflating two different filtering concerns into one block.
Alternative considered: add the scheme check to the top-level skip block
— rejected, since a non-http(s) request that reaches this SW should still
be fetched and returned to the caller (per the current line 66/81
`return response`), just not cached; folding it into the early-return
block would incorrectly skip serving the response entirely.

**D2 — Check `event.request.url` scheme via `new URL(...).protocol`.**
Reuses the same `URL` parsing pattern already established at line 37
(`const url = new URL(event.request.url)`) rather than a substring/regex
check, for consistency with the rest of the file.

## Risks / Trade-offs

- **[Risk] `frontend/public/sw.js` and `static/sw.js` drift if only one
  is edited** → **Mitigation**: this change edits both files explicitly
  and identically; tasks.md includes a byte-diff verification step.

## Migration Plan

None — this is a same-file edit with no data migration, deployed via the
normal frontend build/asset pipeline. Browsers pick up the new service
worker on their normal update-check cadence (no forced re-registration
needed).

## Open Questions

None.
