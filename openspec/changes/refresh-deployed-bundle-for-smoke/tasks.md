## 1. Build

- [x] 1.1 `pnpm --filter ./frontend install --frozen-lockfile` (idempotent — no changes).
- [x] 1.2 `pnpm --filter ./frontend run build` — emitted `index-ChbheD4z.js`.

## 2. Deploy

- [x] 2.1 `cp -R static/* ~/.uar/static/`.
- [x] 2.2 `~/.uar/static/index.html` now references `index-ChbheD4z.js` (≠ stale `Bg0JK_oV`).

## 3. Restart

- [x] 3.1 Killed prior PID 29699.
- [x] 3.2 Relaunched UAR with `UAR_BUILTIN_SKILLS_DIR=…/crates/prometheus-skill-system/skills` from `~`; new PID 70040.
- [x] 3.3 `curl /health` → 200.

## 4. Pre-smoke check

- [ ] 4.1 Hard-reload browser tabs (Cmd+Shift+R) — pending human walkthrough.
- [ ] 4.2 DevTools → Network → filter "live" → confirm 10 `EventSource` connections — pending.
