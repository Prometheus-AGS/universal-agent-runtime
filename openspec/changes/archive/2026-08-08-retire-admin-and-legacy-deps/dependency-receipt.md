# C-14c dependency retirement receipt

## Direct-source and manifest proof

- `frontend/src/**/*.{ts,tsx}` contains zero direct `@radix-ui/*`, `@tanstack/react-query`, or `highlight.js` imports after the shell retirement.
- The live pre-removal manifest contained 26 direct `@radix-ui/*` declarations, not the stale plan count of 27.
- `@tanstack/react-query` had one provider-only call site in `App.tsx`; `highlight.js` had no surviving product-source call site after Shiki landed.
- The post-removal manifest contains zero direct declarations for all three groups.

## Retained transitive ownership

`pnpm -C frontend why` confirms that Radix remains only through retained supported dependencies:

- `@radix-ui/react-dialog@1.1.15`: `cmdk@1.1.1` and `vaul@1.1.2`.
- `@radix-ui/react-dialog@1.1.19`, dropdown menu, popover, and the current slot graph: `radix-ui@1.6.2` through `@assistant-ui/react@0.14.26` (also used by `@assistant-ui/react-markdown`).
- Older slot variants remain through the `cmdk`/`vaul` dialog graph.

The dependency-manager operation reported removing 41 packages from the resolved graph while retaining the transitive packages above. That historical delta is operation-receipt evidence rather than independently reconstructible from the current lockfile; current-state manifest and `pnpm why` checks prove the retirement and retained ownership. No dependency was added to the manifest.
