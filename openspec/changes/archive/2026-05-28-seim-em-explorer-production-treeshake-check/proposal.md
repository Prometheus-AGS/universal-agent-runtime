# Proposal: Production Tree-shake Verification (W7a)

## Why
The Entity Explorer panel and DevtoolsEventBus use `notifyDevtools` in `src/engine.ts`
which is guarded by `if (process.env.NODE_ENV !== "production")`. Without a verified
build check, the devtools code path could ship in production bundles, increasing bundle
size and exposing internal state APIs.

## What Changes
1. Verify that `src/engine.ts` `notifyDevtools` path has a correct `process.env.NODE_ENV`
   guard, and that the UI explorer exports are gated so bundlers can statically eliminate
   them in production.
2. Verify the `prepublishOnly` script fails if the tree-shake gate is missing.
3. Run `pnpm build` (or the equivalent tsc/rollup step) in production mode and confirm
   no `DevtoolsEventBus` or `EntityExplorer*` class names appear in the output.

## Capabilities
- treeshake-guard-audit

## Impact
- No runtime changes; purely a verification + documentation artifact.
- Ensures the library is safe to publish without accidentally shipping devtools overhead.
- Confirms the `treeshake_gate: "hard fail in prepublishOnly"` default decision.
