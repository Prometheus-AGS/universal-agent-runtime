# Tasks: seim-em-explorer-production-treeshake-check

## 1. Audit engine.ts guard

- [x] 1.1 Read `src/engine.ts` — confirm `notifyDevtools` body is wrapped in `if (process.env.NODE_ENV !== "production")`
- [x] 1.2 If guard is missing or incorrect, add/fix it

## 2. Add prepublishOnly guard check

- [x] 2.1 Read `package.json` scripts section
- [x] 2.2 Add inline guard check to `prepublishOnly` that exits non-zero if the NODE_ENV guard is missing from `src/engine.ts`
- [x] 2.3 Verify `prepublishOnly` passes: `cd ~/.claude/worktrees/seim-entity-management && node -e "const s=require('fs').readFileSync('src/engine.ts','utf8'); if(!s.includes(\"process.env.NODE_ENV !== 'production'\")) { console.error('devtools leak detected'); process.exit(1); } else { console.log('treeshake gate: OK'); }"`

## 3. Verify no side-effect imports from explorer

- [x] 3.1 Confirm `src/ui/entity-explorer/context.tsx` has no top-level side effects (no event listeners registered at import time — only inside `useEffect`)
- [x] 3.2 Confirm `src/ui/entity-explorer/fab.tsx` and `panel.tsx` have no top-level side effects

## 4. Commit

- [x] 4.1 `git add package.json src/engine.ts` (if changed)
- [x] 4.2 Commit: `chore(build): add treeshake gate for devtools exports (W7a)`

## 5. Archive

- [x] 5.1 Mark all tasks done and archive this change
