## 1. Contract test

- [ ] 1.1 Vitest + RTL "two-views/one-SSE-event/both-rerender" — **deferred**. Vitest isn't wired into npm scripts in this repo (only Playwright is). Adding a contract-test scaffold + Vitest config is its own task. The bridge pattern is small enough to validate manually with a two-tab smoke today; a permanent regression test belongs alongside the first direct-`useEntity` migration that retires a bridge.

## 2. Audit doc

- [x] 2.1 [`docs/migration-stale-data-audit.md`](../../../docs/migration-stale-data-audit.md) — full inventory of every entity, hook, bridge, and status. Updated as bridges shipped.
- [ ] 2.2 Final `git grep "fetch\\(" frontend/src/` sweep — deferred until direct-`useEntity` migrations retire the bridge layer entirely.

## 3. AGENTS.md / CLAUDE.md

- [x] 3.1 New "Realtime freshness contract — no stale data anywhere" section added to [`AGENTS.md`](../../../AGENTS.md) describing topic enrollment, bridge pattern, and optimistic-mutation rule.
- [ ] 3.2 README architecture diagram — deferred.
- [ ] 3.3 `CLAUDE.md` invariants update — deferred (AGENTS.md captures the rules for now).

## 4. Cleanup

- [ ] 4.1 Remove dev diagnostic listener — N/A (none added).
- [ ] 4.2 Flip `VITE_ENTITY_MGMT_CHAT_RUNTIME` — N/A (change 7 needed no flag).
- [ ] 4.3 Verify no retired-store references — N/A; bridge pattern preserves stores.

## Status

**DONE for docs + audit.** Vitest contract test + README diagram remain as follow-ups; the audit doc + AGENTS.md update give future contributors the rules they need to extend the pattern correctly.
