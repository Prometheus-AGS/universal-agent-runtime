## Why

Change 17 (`a2ui-uar-renderer-on-webcore`) delivered `EntityCard` as one of
7 planned `Entity*` components and deliberately deferred the other 6 —
`EntityDiff`, `EntityStream`, `EntityApproval`, `EntityToolProvider`,
`EntityChat`, `EntityCopilot`. The phase plan frames this change as
"migrating" those 6 from `prometheus-entity-management`.

**That premise is factually wrong.** Auditing before writing any code
found no such source exists:

- The plan's exact claimed path,
  `prometheus-skill-system/skills/imported/prometheus-entity-management/packages/a2ui-react/src/`,
  does not exist in this repo.
- The actual `prometheus-entity-management` submodule
  (`frontend/packages/prometheus-entity-management`, pinned at
  `v1.2.0-rc-26`) has no `a2ui-react` package and none of the 6 named
  files (`entity-stream.tsx`, `entity-approval.tsx`, `entity-chat.tsx`,
  `entity-copilot.tsx`, `entity-diff.tsx`, `entity-tool-provider.tsx`)
  anywhere in its tree.
- Its full git history (`git log --oneline --all | grep -i a2ui`) has no
  match, so this isn't a case of the files existing at an older tag.
- No sibling repo has an `a2ui-react` directory with these files either.

So there is nothing to migrate. This change instead builds a genuine
subset of the remaining `Entity*` catalog as **new** components on
Change 17's established `web_core`/`GenericBinder` foundation, following
the same naming-alignment convention `EntityCard` set with
`prometheus-entity-management`'s domain model (`EntityType`, `EntityId`,
`EntitySyncMetadata`).

## What Changes

- **`EntityDiff`** (new, bound component via
  `createUarComponentImplementation`, same pattern as `EntityCard`):
  renders a before/after field comparison for an entity, with changed
  rows visually distinguished from unchanged ones.
- **`EntityStream`** (new, binderless component via
  `createBinderlessUarComponentImplementation` — infrastructure Change 17
  already built but didn't exercise): subscribes directly to a
  data-model path via `context.dataContext.subscribeDynamicValue` and
  renders each item reactively as the underlying array changes, without
  going through `GenericBinder`'s single-value resolution.
- Both registered in `uarEntityCatalogComponents`
  (`urn:uar:a2ui:catalog:1+entities`).
- 5 new tests (2 `EntityDiff`, 3 `EntityStream` — including a live
  reactive-update test that pushes a `updateDataModel` message after
  mount and asserts the rendered list picks up the new item).

## Capabilities

### New Capabilities

- `a2ui-entity-components`: `EntityDiff` and `EntityStream` join
  `EntityCard` under the `urn:uar:a2ui:catalog:1+entities` catalog.

## Impact

- **No dependency-graph impact.** Purely additive to
  `@prometheus-ags/a2ui-uar`; no existing component's behavior changes.
- **No new runtime dependency.** Both components are built on
  `@prometheus-ags/a2ui-core`'s existing exports (`AccessibilityAttributesSchema`,
  `DynamicStringSchema`, `ComponentContext`), already a dependency since
  Change 17.
- **`prometheus-entity-management` is untouched.** The plan's "the
  submodule reduces to the upstream package" done-condition doesn't apply
  — there was never any UAR-specific code inside it to remove.

## Out of scope

- **`EntityApproval`, `EntityToolProvider`, `EntityChat`, `EntityCopilot`.**
  Each is closer to a mini-application (an approval workflow, a tool-call
  authorization gate, a full chat surface, a copilot session) than a
  single presentational component. Building these well — with their own
  interaction/state models, not just a props schema — is substantial,
  separate scope. Deferred, same reasoning Change 17 already gave for
  deferring all 6 originally.
- **A `use-entity-*` hooks migration.** The plan's done-condition
  ("New `use-entity-*` hooks migrated in parallel") assumed the same
  nonexistent source tree as the component files. No such hooks were
  found to migrate either. If entity-specific data-fetching hooks are
  needed later (e.g. for `EntityStream` to source real backend data
  instead of a caller-supplied data-model path), that's follow-up work
  once a concrete consumer needs it.
- **Full cross-testing against `@a2ui/react`** for the new components
  (per Change 17's precedent, only `EntityCard` and the 9 protocol
  components got real cross-tests — `@a2ui/react` has no `Entity*`
  components at all, since those are UAR-specific extensions with no
  Google reference implementation to cross-test against).
