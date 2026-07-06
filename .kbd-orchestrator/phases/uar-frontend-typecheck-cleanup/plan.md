PLAN: uar-frontend-typecheck-cleanup
Project: universal-agent-runtime
Date: 2026-07-06
OpenSpec available: YES
Changes to implement: 5

## Framing

All 5 changes are independent of each other in principle, but change 1
(the root-script invocation fix) is sequenced first because it's what
makes `bun run typecheck` usable as a real verification loop for
changes 2–5 — without it, verifying the TypeScript fixes requires
remembering to `cd frontend`/`pnpm -C frontend` by hand every time,
which is exactly the kind of silent workaround this phase exists to
eliminate. Changes 2–4 group the 17 TypeScript errors by root cause
(as assessed); change 5 catches the 6 errors that don't share a common
cause. Given this project's standing implementation-first/
test-at-checkpoints preference and the low individual risk of each
change (frontend-only, no Rust/backend surface touched), all 5 can be
implemented in one pass with a single `bun run typecheck` checkpoint
at the end — not 5 separate checkpoints.

## CHANGE LIST (ordered)

1. **fix-root-frontend-script-invocation**: restore the 6 broken
   root-level `package.json` scripts
   - Scope: build tooling (`package.json` only)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: HIGH (every documented dev command — `bun run
     build`/`dev`/`test`/`test:e2e`/`lint`/`typecheck`/`format` — is
     currently non-functional from the repo root; this is what makes
     the rest of this phase's own verification loop actually work)
   - Details: Replace `pnpm --filter ./frontend <cmd>` with `pnpm -C
     frontend <cmd>` in `build`, `dev`, `test`, `test:e2e`, `lint`,
     `typecheck`. For `format` (`pnpm --filter ./frontend exec
     prettier --write src/`), verify `-C` resolves `prettier`
     correctly first — `assessment.md` found `pnpm -C frontend exec
     prettier` fails to find it (prettier is hoisted as a *root*
     devDependency, not present in `frontend/node_modules`). If `-C`
     doesn't resolve it, either invoke prettier via its root-relative
     binary path (`pnpm exec prettier --write frontend/src/`, run from
     root, no `-C`/`--filter` needed at all since `exec` at the root
     naturally finds the root's own installed binary) or add
     `prettier` to `frontend/package.json`'s own devDependencies —
     prefer the former (no new dependency, matches how the binary is
     actually installed today) unless it proves not to work either.
   - **Do not** add a root-level `pnpm-workspace.yaml` `packages:`
     entry for `frontend` — `assessment.md` confirmed this breaks
     `frontend/packages/prometheus-entity-management`'s submodule
     resolution (`ERR_PNPM_WORKSPACE_PKG_NOT_FOUND`). This change is
     scripts-only.
   - Verify: `bun run build`, `bun run typecheck`, `bun run lint`,
     `bun run format` (or its resolved equivalent) all actually run
     (not necessarily error-free yet, for `typecheck` specifically —
     just confirm they *reach* their real tool rather than failing on
     `No projects matched the filters`).

2. **fix-typecheck-base-ui-select-nullability**: 6 errors, `string |
   null` vs. non-nullable state/prop types
   - Scope: `frontend/src/admin/components/agent-editor.tsx`,
     `frontend/src/admin/pages/agents-page.tsx`,
     `frontend/src/admin/pages/models-page.tsx`
   - Depends on: NONE (but verify via change 1's now-working
     `typecheck`)
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: MEDIUM (type-safety correctness; Base UI's
     `Select` value type is genuinely nullable — these are real gaps,
     not false positives)
   - Details: `@base-ui/react`'s `Select` `onValueChange` passes
     `string | null` (`null` = cleared/no selection). Each site
     currently assumes a non-null `string`. Fix at the call site (not
     by casting away the null) — decide per site whether `null` should
     map to an empty string, a sentinel, or whether the surrounding
     state type itself should become nullable to match. Prefer
     widening the state type to `string | null` where the "cleared"
     state is meaningful (`agents-page.tsx`'s `TriValue`/memory-state
     setter), and an explicit `?? ""`/fallback only where a non-null
     invariant is genuinely guaranteed elsewhere.

3. **fix-typecheck-resizable-panels-api-drift**: 4 errors,
   `resizable.tsx`'s wrapper uses stale export names
   - Scope: `frontend/src/components/ui/resizable.tsx`
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: MEDIUM (this file is a thin wrapper around
     `react-resizable-panels`; it's currently referencing exports that
     don't exist in the installed `2.1.9`)
   - Details: The installed `react-resizable-panels@2.1.9` exports
     `Panel`, `PanelGroup`, `PanelResizeHandle` (+ matching `*Props`
     types) — not `Group`/`Separator`. Rename
     `ResizablePrimitive.Group`→`PanelGroup`,
     `ResizablePrimitive.GroupProps`→`PanelGroupProps`,
     `ResizablePrimitive.Separator`→`PanelResizeHandle`,
     `ResizablePrimitive.SeparatorProps`→`PanelResizeHandleProps`
     throughout the file. Mechanical rename, confirmed via direct
     inspection of the installed package's `.d.ts` — not a version
     bump, the wrapper was just written against an older/different
     naming convention.

4. **fix-typecheck-recharts-export-drift**: 1 error, `chart.tsx`
   imports a type recharts no longer exports
   - Scope: `frontend/src/components/ui/chart.tsx`
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: LOW (single type-only import, no runtime impact)
   - Details: `import type { TooltipValueType } from "recharts"` no
     longer resolves against the installed `recharts@2.15.4`. Find
     what `TooltipValueType` was actually used for in this file (a
     `Tooltip` formatter's value-type annotation) and replace with
     `recharts`'s current equivalent export, or inline the type
     locally if `recharts` genuinely no longer exposes an equivalent
     public type for that shape.

5. **fix-typecheck-remaining-errors**: 6 errors not sharing one of the
   3 causes above
   - Scope: `frontend/src/admin/pages/knowledge-page.tsx`,
     `frontend/src/stores/use-thread-graph-sync.ts`
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: LOW-MEDIUM (closes out the remainder of the 17;
     `use-thread-graph-sync.ts`'s unsafe cast is worth a real look, not
     just an `as unknown as X` silence)
   - Details: `knowledge-page.tsx:445` — `undefined` not assignable to
     `number | null` (likely a missing `?? null` at a call site).
     `use-thread-graph-sync.ts:41` — `Record<string, Record<string,
     unknown>>` cast to `Record<string, ServerThreadRow>` fails
     because `ServerThreadRow` requires an `id` field the source type
     doesn't structurally guarantee; either narrow/validate the shape
     before casting or confirm the cast is genuinely safe and route it
     through `as unknown as Record<string, ServerThreadRow>` explicitly
     (TypeScript's own suggested escape hatch) with a one-line comment
     explaining why it's safe here.

## EXECUTION ROUND ORDER

- **Round 1 (sequenced first, unblocks verification)**:
  `fix-root-frontend-script-invocation`
- **Round 2 (parallel, one shared checkpoint)**:
  `fix-typecheck-base-ui-select-nullability`,
  `fix-typecheck-resizable-panels-api-drift`,
  `fix-typecheck-recharts-export-drift`,
  `fix-typecheck-remaining-errors`

Checkpoint after Round 2: `bun run typecheck` (now working, per Round
1) exits 0 with zero errors. If any error can't be cleanly fixed within
this phase's scope (e.g. an upstream type-export genuinely has no
replacement), disclose it explicitly and re-carry it rather than
silencing it with a blanket type assertion.

## COMMANDS TO RUN

```
/opsx:new fix-root-frontend-script-invocation
/opsx:new fix-typecheck-base-ui-select-nullability
/opsx:new fix-typecheck-resizable-panels-api-drift
/opsx:new fix-typecheck-recharts-export-drift
/opsx:new fix-typecheck-remaining-errors
```

Per this project's established practice (no changes pre-scaffolded
during planning), these `openspec/changes/<id>/proposal.md` + `tasks.md`
pairs are written per-change at execute time.

## Sycophancy self-check

- S-02: change 1's `format` script sub-case is flagged as unverified
  (informational finding, not yet tested against `-C`) rather than
  assumed to work the same way as the other 5 scripts.
- S-03: at least 2 explicit trade-offs stated — change 2's per-site
  null-handling decision (widen type vs. fallback) and change 5's cast
  decision (narrow-and-validate vs. explicit `as unknown as` with
  rationale) are both left as judgment calls for execute, not
  pre-decided here without seeing the surrounding code in full.
- S-07: no scope creep — all 5 changes map directly to
  `assessment.md`'s findings; no new work invented beyond what was
  already discovered during assess.

PLAN COMPLETE
