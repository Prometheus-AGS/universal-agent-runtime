## Why

The service worker's fetch handler (`frontend/public/sw.js` /
`static/sw.js`, byte-identical) calls `cache.put(event.request, clone)`
unconditionally for any successful same-origin-looking fetch response
(lines 64 and 79). The Cache API only accepts `http(s)` request schemes —
calling `put()` with a request whose scheme is something else (e.g. a
`chrome-extension://` resource a browser extension's content script
triggers a fetch for, which can bubble through this page's active service
worker) throws `TypeError: Failed to execute 'put' on 'Cache': Request
scheme 'chrome-extension' is unsupported`, uncaught, surfacing as a
console error the operator observed live while using the Admin/Agents UI.
This is cosmetic-but-noisy today (the response is still returned to the
caller either way — line 64's throw is caught nowhere but doesn't block
`return response`), but it pollutes the console during every affected
request and should be fixed at the source rather than filtered out by
habit.

## What Changes

- Guard both `cache.put()` call sites (lines 64 and 79 of `sw.js`) so they
  only execute when `event.request.url` has an `http:` or `https:`
  scheme, matching the existing skip-list style already used at lines
  40-47 for method/path filtering.
- No behavior change for the normal (http/https) path: caching continues
  exactly as before.
- `frontend/public/sw.js` and `static/sw.js` are byte-identical today and
  must remain so after this change (the frontend build presumably copies
  or symlinks one to the other, or a build step keeps them in sync — this
  change edits both explicitly rather than assuming a build step handles
  it, to avoid silently reintroducing drift).

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `pwa-offline`: the existing "Service worker for offline asset caching"
  requirement gains a new scenario covering non-http(s) request schemes,
  where the worker must not attempt to cache the response (and therefore
  must not throw).

## Impact

- **Runtime UX**: removes a recurring, user-visible console error during
  normal Admin/Agents UI usage; no visible functional change otherwise.
- **Provider compatibility**: none.
- **Realtime state**: none.
- **KBD workflow state**: this change absorbs supplemental fix #1 from
  the `uar-grade-a-upgrade-2026-07` phase's operator-directed Admin/Agents
  UI assessment (see that phase's `assessment.md`); no further action
  needed there once this change ships.
- **Affected files**: `frontend/public/sw.js`, `static/sw.js`.
