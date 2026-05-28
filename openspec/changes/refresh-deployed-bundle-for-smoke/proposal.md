## Why

The browser smoke session needs to validate the **post-fix** behaviour of the `useGraphBridge` helper. The deployed bundle in `~/.uar/static/` (`index-Bg0JK_oV.js`, built 2026-05-27 04:09) was produced before the bridge bug fix landed in `vitest-contract-test-suite`. Running the smoke against the stale bundle would either pass spuriously (the bug only fired on first-listener tick) or test the wrong code path.

## What Changes

No source edits. Mechanical redeploy:

1. `pnpm install --frozen-lockfile` in `frontend/` (idempotent guard).
2. `pnpm run build` — emits a new bundle hash with the bridge fix.
3. `cp -R static/* ~/.uar/static/`.
4. Restart UAR (kill PID listening on 1906, relaunch with the same env vars currently in use).
5. Verify `/health` returns 200 and the new bundle hash differs from `index-Bg0JK_oV.js`.

## Acceptance

- `grep -oE 'index-[A-Za-z0-9_-]+\.js' ~/.uar/static/index.html` differs from `index-Bg0JK_oV.js`.
- `curl http://127.0.0.1:1906/health` returns 200.
- UAR startup log shows the live-query bus reconnecting cleanly (the `does not exist` topics still park at max backoff as expected).
