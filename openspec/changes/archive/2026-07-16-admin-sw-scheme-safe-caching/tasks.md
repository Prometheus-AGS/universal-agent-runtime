## 1. Guard the cache-first branch

- [x] 1.1 In `frontend/public/sw.js`, wrapped the `cache.put()` call at
      line 64 in a scheme check: `response.ok && url.protocol.startsWith
      ('http')`, reusing the existing `url` const from line 37 (`new URL
      (event.request.url)`) rather than constructing a second `URL`
      object, per design.md's D2.
- [x] 1.2 Confirmed `return response;` still executes unconditionally —
      only the `cache.put()` call is inside the added condition.

## 2. Guard the network-first branch

- [x] 2.1 Applied the identical guard to the `cache.put()` call at line
      79 (network-first HTML branch).
- [x] 2.2 Confirmed `return response;` and the `.catch()` offline-fallback
      path are both unaffected — neither was touched.

## 3. Keep both service worker copies in sync

- [x] 3.1 Applied the identical two guards to `static/sw.js`.
- [x] 3.2 Ran `diff frontend/public/sw.js static/sw.js` — no output,
      files remain byte-identical.

## 4. Verification

- [x] 4.1 `pnpm -C frontend lint` (`eslint .`) passes clean, exit 0.
- [x] 4.2 Live browser verification (not against the separately-installed
      `~/.uar/static` production instance behind uar-jwt-proxy:8088,
      which is a different deployment unrelated to this checkout's edits
      — instead served this checkout's own `static/` directory via a
      throwaway local static server to test the actual edited file):
      registered the service worker, reloaded, and confirmed via
      `caches.open('uar-v1').then(c => c.keys())` that all ordinary
      http(s) assets (JS/CSS/wasm bundles, index.html, manifest.json,
      cross-origin Google Fonts woff2) were still cached exactly as
      before, and `read_console_messages` showed zero console errors.
      Baseline caching behavior is unchanged.
- [x] 4.3 A non-http(s)-scheme trigger (e.g. a `chrome-extension://`
      fetch from an active browser extension) was **not independently
      reproduced** — no such extension was active in the test browser
      profile used for 4.2. Per this task's own allowance, the fix's
      correctness for that path rests on: (a) `URL.protocol` for a
      `chrome-extension://...` request is the standard, well-defined
      string `"chrome-extension:"`, which `.startsWith('http')` correctly
      excludes; (b) the guard is a pure boolean addition to an existing
      `if` condition with no other logic changed, so its only possible
      effect is skipping `cache.put()` for non-matching schemes, which is
      exactly the target behavior.
