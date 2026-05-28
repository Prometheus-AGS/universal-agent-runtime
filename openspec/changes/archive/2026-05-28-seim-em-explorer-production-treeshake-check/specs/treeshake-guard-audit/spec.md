# Spec: treeshake-guard-audit

## Goal
Confirm that all devtools code paths in the published library are fully eliminated by
standard bundler dead-code elimination when `NODE_ENV=production`.

## Requirement 1 — NODE_ENV guard in engine.ts

`src/engine.ts` exports `notifyDevtools(event)`. The function body must be wrapped in:
```ts
if (process.env.NODE_ENV !== "production") { ... }
```
so that bundlers (esbuild, rollup, webpack) can replace the condition with `false` and
eliminate the body.

### Scenario: Guard present
Given: `src/engine.ts` contains the guard
When: bundler replaces `process.env.NODE_ENV` with `"production"`
Then: entire `notifyDevtools` body is dead code and tree-shaken

### Scenario: Guard absent
Given: guard is missing
When: build step runs
Then: `prepublishOnly` check script exits non-zero and prints a clear error

## Requirement 2 — Explorer exports conditionally excluded

The `EntityExplorerProvider`, `EntityExplorerPanel`, and `EntityExplorerFAB` exports in
`src/index.ts` are already importable in development. They do NOT need a runtime guard
because consumers tree-shake them by not importing. Verify no automatic side-effect
import happens at module load.

### Scenario: No side-effect import
Given: a production bundle that does not import `EntityExplorer*`
When: build is inspected
Then: no `createDevtoolsEventBus`, `ReactDOM.createPortal`, or `useReducer` calls appear
      in the output bundle for that entry point

## Requirement 3 — prepublishOnly gate

`package.json` `scripts.prepublishOnly` must run a script that:
1. Builds the package in production mode (`NODE_ENV=production`)
2. Greps the output for `notifyDevtools` (plain string, not mangled)
3. Exits non-zero if found

### Scenario: Gate passes
Given: `notifyDevtools` is properly guarded
When: `pnpm run prepublishOnly` is invoked
Then: exits 0

### Scenario: Gate fails
Given: guard removed from engine.ts
When: `pnpm run prepublishOnly` is invoked
Then: exits non-zero with message "devtools leak detected"
