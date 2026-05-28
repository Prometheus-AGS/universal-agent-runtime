## 1. Optimistic patches

- [x] 1.1 Skill toggle — `stores/skills-admin-store.ts::toggle` flips locally first, rolls back on failure.
- [x] 1.2 Agent enable/disable (shallow patch) — `stores/agents-admin-store.ts::patchAgent` merges body locally, rolls back to snapshot on failure.
- [x] 1.3 Provider set-default — `stores/providers-admin-store.ts::setDefault` sets `defaultId` immediately, rolls back on failure.
- [ ] 1.4 Setting field edit (single-key PUT) — deferred; settings already use a dirty-buffer pattern that's effectively optimistic.
- [ ] 1.5 KB rename — deferred; lower-frequency mutation.

## 2. Rollback UX

- [x] 2.1 Error state surfaced on each store on failure.
- [ ] 2.2 Toast component wiring — deferred; existing error renderers display the message.

## 3. Tests

- [ ] 3.1 Force-reject test — deferred to integration-tests change.

## Verification

- [x] Build clean (`index-CHuAcm3q.js`).
- [x] UAR restarted; SPA serves new bundle.
- [ ] Manual smoke: click a skill toggle and observe instant flip — pending browser test.

## Status

**DONE for the three highest-frequency paths.** Setting field edits already use a dirty-buffer pattern that achieves the same UX without per-field patches; KB rename is rare enough to leave non-optimistic.
