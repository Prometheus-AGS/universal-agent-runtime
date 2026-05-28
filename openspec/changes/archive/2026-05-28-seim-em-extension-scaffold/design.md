# Design: seim-em-extension-scaffold

## Decision 1 — Minimal scaffold, no build step required for the MV3 non-React files

`background.js`, `content.js`, and `devtools.js` are plain JS files that
don't need compilation. They can be loaded directly by Chrome.

`panel.tsx` requires a TypeScript/React build step. We use `tsup` with a
separate entry point config to produce `extension/panel.js`.

Add to `tsup.config.ts` (or create one if absent):
```ts
export default defineConfig([
  // existing library build
  { entry: { index: "src/index.ts" }, ... },
  // extension panel bundle
  { entry: { "extension/panel": "extension/panel.tsx" },
    format: ["iife"], globalName: "EntityExplorerExtension",
    outDir: "extension", external: [], noExternal: ["react", "react-dom"] }
]);
```

## Decision 2 — DevtoolsEventBus.inject() as a public method

The `_busInjectMap` WeakMap approach (W5) provides the internal inject fn.
For the extension context, we expose it as a public `inject(event: DevtoolsEvent): void`
method on the bus interface. This avoids exposing internal implementation details
through the main public API.

## Decision 3 — EntityExplorerProvider accepts external bus

The extension panel needs to pass a pre-constructed bus (from `createExtensionBus`)
to the provider. Add optional `bus` prop to `EntityExplorerProviderProps`:

```tsx
interface EntityExplorerProviderProps {
  children: ReactNode;
  bus?: DevtoolsEventBus;           // if provided, use this bus; skip internal creation
  busOptions?: ...;
  enableWindowBridge?: boolean;
}
```

## Decision 4 — forceOpen prop on EntityExplorerPanel

In extension mode, the panel is always visible (the DevTools panel IS the entity
explorer). Add `forceOpen?: boolean` to panel so it skips the FAB and renders directly.

## File layout
```
extension/
  manifest.json
  background.js
  content.js
  devtools.html
  devtools.js
  panel.html
src/extension/
  create-extension-bus.ts
```
