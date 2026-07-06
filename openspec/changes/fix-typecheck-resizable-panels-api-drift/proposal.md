# fix-typecheck-resizable-panels-api-drift

## Why

`resizable.tsx`'s wrapper referenced `ResizablePrimitive.Group`/
`GroupProps`/`Separator`/`SeparatorProps` — none of which exist on the
installed `react-resizable-panels@2.1.9`. Confirmed via direct
inspection of the installed package's `.d.ts` (`dist/declarations/src/index.d.ts`):
the actual exports are `Panel`, `PanelGroup`, `PanelResizeHandle` (+
matching `*Props` types). This is a naming-convention mismatch, not a
version incompatibility — no dependency bump needed.

## What changed

Mechanical rename throughout `resizable.tsx`:
`Group`→`PanelGroup`, `GroupProps`→`PanelGroupProps`,
`Separator`→`PanelResizeHandle`, `SeparatorProps`→`PanelResizeHandleProps`
(both the JSX element usages and the destructured prop types). `Panel`/
`PanelProps` were already correct and untouched.

## Verification

- `bun run typecheck`: these 4 errors resolved.
- `bun run build`: succeeds — confirms the renamed JSX elements resolve
  to real, working components at runtime, not just type-level.
- No visual/behavioral change — purely a type-level naming fix; the
  underlying rendered DOM is unaffected.
