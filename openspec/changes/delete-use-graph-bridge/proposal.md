# delete-use-graph-bridge

## Why
After all 6 consumers migrate direct, `useGraphBridge` has zero callers. Delete the helper and move its section in the audit doc to a "historical" appendix.

## What changes
- Verify `git grep useGraphBridge frontend/` is empty (Tools-page is the only allowed exception; Tools migration is deferred).
- If Tools still uses it, keep the file but add a `@deprecated` JSDoc comment with forward link.
- If empty: `git rm frontend/src/lib/realtime/use-graph-bridge.ts`.
- Rewrite `docs/migration-stale-data-audit.md`:
  - All 6 migrated rows reflect `direct` status (already done per-change).
  - "Bridge pattern (interim)" section moved to a "Historical: the bridge pattern" appendix at the bottom.
  - Add a "Direct migration playbook" section as the canonical guide for future entity additions.

## Impact
Clears the interim infrastructure. Future entity migrations follow the direct pattern from day one.
