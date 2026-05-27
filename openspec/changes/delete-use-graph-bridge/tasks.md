## 1. Tools branch
- [ ] 1.1 If `git grep useGraphBridge frontend/` shows only `tools-discovery-store.ts`, mark the helper `@deprecated` and skip deletion
- [ ] 1.2 If empty (Tools also migrated), proceed to deletion

## 2. Deletion path
- [ ] 2.1 `git rm frontend/src/lib/realtime/use-graph-bridge.ts`
- [ ] 2.2 `git grep useGraphBridge frontend/` returns empty

## 3. Audit doc
- [ ] 3.1 Move "Bridge pattern (interim) vs. direct migration (destination)" section to a "Historical: bridge pattern" appendix at the bottom
- [ ] 3.2 Add a "Direct migration playbook" section as the canonical guide
- [ ] 3.3 Confirm all 6 rows (Model, Skill, KB, Document, Setting, Memory, CompilerSession) show `direct`
- [ ] 3.4 Tools row remains `bridged` with forward link to `tool-mcp-status-push-channels`

## 4. Verification
- [ ] 4.1 36/36; clean build
- [ ] 4.2 No new test regressions
