# Archive Readiness Verification

Date: 2026-08-07
Phase change: C-00 (`archive-completed-ui-changes`)
Historical implementation: `base-ui-foundation`

## Current-tree evidence

- `verify-archive-readiness.sh` is the executable gate for the assertions below.
- At verification time, the working tree's base commit was
  `a91cd5b50b9061c0ef439003142e994c46b8d013`.
- `git hash-object frontend/package.json`:
  `0a70007a38c1bd88545df8d55a5d706111b54cde`.
- `git hash-object frontend/components.json`:
  `3005b7532e1c7c6db88b508168940f853fd006c5`.
- Parsed current values: `@base-ui/react=1.6.0`, `style=base-vega`,
  `baseColor=neutral`.
- The recorded gate run reports 34 Base UI-backed wrapper files and zero direct production
  `@radix-ui/*` imports.
- All 24 historical task checkboxes are complete.
- `openspec validate base-ui-foundation --strict` passes.
- `openspec status --change base-ui-foundation --json` reports proposal, design,
  specs, and tasks complete.
- `verification-output.txt` records the successful command transcript used for this
  archive decision.

## Immutable implementation evidence

Commit `e92670e248d7e02fed764edc16a7fabcf9d84dca` exists as a Git commit with
subject `feat: swap radix-ui for @base-ui/react and regenerate shadcn components`.
Its scoped diff changes `frontend/package.json`, `frontend/components.json`, and 56
primitive files (58 files total; 5,133 insertions and 3,184 deletions). At that commit:

- the manifest declared `@base-ui/react=^1.5.0`, `@floating-ui/react=^0.27.19`,
  and retained only the deliberately deferred `@radix-ui/react-icons` entry;
- `components.json` resolved `style=base-vega` and `baseColor=neutral`; and
- `button.tsx` and `dialog.tsx` imported `@base-ui/react/button` and
  `@base-ui/react/dialog`, respectively.

These values are independently reproducible with `git show e92670e:<path>`; the
executable gate checks both the immutable commit identity and the current tree.

## Deliberate follow-up boundary

`frontend/package.json` still declares unused Radix packages. This verification does
not misreport those declarations as removed. The phase plan assigns their transitive
audit and final prune to C-14c, after `cmdk`, assistant-ui, Storybook, and the migrated
admin surfaces can be evaluated together.

## Verdict

PASS — the implemented production primitive foundation and its durable specification
are ready to synchronize and archive. Later composition, icon, declaration-pruning,
and full UI certification obligations remain open in their named phase changes.
