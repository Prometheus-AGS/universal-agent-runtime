# fix-typecheck-base-ui-select-nullability

## Why

`@base-ui/react`'s `Select` and `ToggleGroup` components pass `null` to
`onValueChange` to represent "cleared"/no-selection — genuinely part of
their type contract, not a false positive. 6 call sites across
`agent-editor.tsx` (4), `agents-page.tsx` (2), and `models-page.tsx` (2)
assumed a non-nullable `string` and were rejected by `tsc`.

## What changed

All 6 sites are enum-style pickers (`status`, `skill_selection_method`,
`memory_scope`, `tool_approval`, provider/model dropdowns, and a
tri-state toggle) backed by a fixed, always-non-empty option list — the
user can never actually trigger a `null` value through the UI as these
components are used here. Rather than widening each backing state
field to `string | null` (which would be semantically wrong for these
required-value fields and would push null-handling into every other
place they're read), guarded each call site: `(v) => v !== null &&
update(...)` / `if (v !== null) { ... }`.

`agents-page.tsx`'s `TriToggle` wrapper had a second, distinct issue:
it passed `type="single"` and a bare string to `ToggleGroup`, mimicking
Radix UI's API — but Base UI's `ToggleGroup` has no `single`/`multiple`
discriminated `type` prop at all; it's unconditionally array-based
(`value?: readonly Value[]`, `onValueChange?: (groupValue: Value[],
...) => void`), with a separate `multiple` boolean (default `false`)
controlling whether more than one item can be pressed. Fixed by passing
`value={[encodeTriValue(value)]}` and reading `vals[0]` in
`onValueChange`, and removing the nonexistent `type="single"` prop.

## Verification

- `bun run typecheck`: these 6 errors resolved (17 → 11 after this
  change alone, confirmed incrementally before the other 3 changes in
  this round landed).
- `bun run lint`: 215 problems before and after (confirmed via `git
  stash`/`stash pop` A-B comparison) — no new lint issues introduced.
- `bun run build`: succeeds.
- No behavior change for actual users — every affected picker already
  only ever produces non-null values in practice; this only makes the
  types honest about the underlying library's real contract.
