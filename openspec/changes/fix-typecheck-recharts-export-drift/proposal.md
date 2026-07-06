# fix-typecheck-recharts-export-drift

## Why

`chart.tsx` imported `TooltipValueType` from `recharts`'s top level,
but the installed `recharts@2.15.4` doesn't re-export it there —
confirmed via direct inspection of
`recharts/types/index.d.ts`, which only re-exports
`DefaultTooltipContent`/`DefaultTooltipContentProps` from
`component/DefaultTooltipContent.d.ts`, not that module's own
`ValueType`/`NameType` generic-constraint types.

This is consistent with the file's own existing convention: the
adjacent `TooltipNameType = number | string` local alias (already
present before this change) exists for exactly this reason —
`NameType` was never importable from `recharts`'s top level either.

## What changed

Removed the broken `import type { TooltipValueType } from "recharts"`
and added a local alias next to the existing `TooltipNameType`,
matching `recharts`'s own `ValueType` definition
(`component/DefaultTooltipContent.d.ts`): `type TooltipValueType =
number | string | Array<number | string>`.

## Verification

- `bun run typecheck`: this error resolved.
- `bun run build`: succeeds.
- Type-only change (no runtime code touched); the local alias is
  structurally identical to `recharts`'s own internal `ValueType`.
