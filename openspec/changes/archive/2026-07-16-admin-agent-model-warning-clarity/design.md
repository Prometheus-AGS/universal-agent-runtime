## Context

`ProviderMetaEntity` (`frontend/src/entities/types.ts`) currently only
stores `default_id: string | null`, hydrated from
`configured.default_id` in `loadProvidersIntoGraph()`
(`frontend/src/entities/fetchers/providers.ts`). The raw API response
(`UarProvider.default_model`, `frontend/src/types/index.ts`) already
carries the default provider's default model — it's just never written
into the graph. `agents-page.tsx`'s `agentLacksModel()` doesn't consult
either value at all; it only inspects the agent's own
`policy.provider.default`.

## Goals / Non-Goals

**Goals:**
- Distinguish, in the Admin Agents list, "no per-agent override, but a
  working system default exists" from "no per-agent override and no
  working system default."
- Keep the existing "fully configured" (has a per-agent override) case
  visually unchanged.

**Non-Goals:**
- No change to how agents actually resolve their provider/model at chat
  time (that resolution logic is unaffected — this change is UI-only).
- No new backend endpoint — `default_model` is already returned by the
  existing `GET /api/uar/providers` response.
- No change to the Agent Editor's provider/model picker UI (a separate,
  larger change per `admin-agent-provider-first-model-picker`).

## Decisions

**D1 — Store `default_model` on `ProviderMetaEntity`, not a new entity
type.**
`ProviderMetaEntity` is already the graph's home for "current system-wide
provider default" (`default_id`); `default_model` is the same concept's
other half, sourced from the same API call, hydrated in the same
function. Adding a field there is the minimal change. Alternative
considered: derive it on-demand in `agents-page.tsx` by looking up the
`Provider` entity matching `default_id` and reading a
`default_model`-equivalent field — rejected because `ProviderEntity`
doesn't currently carry `default_model` either (only
`display_name`/`base_url`/`configured`/etc.), so this would require the
same kind of plumbing addition just in a different, less discoverable
place.

**D2 — Add a new `useHasWorkingSystemDefault()` hook alongside
`useProviderDefault()`, not a breaking signature change.**
`useProviderDefault(): string | null` has an existing caller —
`frontend/src/hooks/use-providers-admin.ts:10`
(`const defaultId = useProviderDefault() ?? undefined;`) — that expects a
plain string, found via `grep -rn "useProviderDefault" frontend/src/`
before finalizing this design (an earlier draft of this decision assumed
no caller existed and would have silently broken that admin providers
hook). Rather than change `useProviderDefault()`'s return shape, add a
new export in the same file,
`useHasWorkingSystemDefault(): boolean`, which reads both `default_id`
and the new `default_model` field off the same `ProviderMetaEntity` and
returns `!!meta?.default_id && !!meta?.default_model`. `agents-page.tsx`
calls this new hook; `use-providers-admin.ts` is untouched.

**D3 — Icon choice for the neutral state.**
Use `Info` from `lucide-react` (already the icon library in use
throughout `agents-page.tsx`), styled with a muted foreground color
(`text-muted-foreground`) rather than the warning amber
(`text-amber-500`), with `aria-label="Using system default"`. This keeps
the same visual language (small icon next to the agent row) while making
the semantic difference immediately scannable: amber triangle = actually
broken, muted info = fine, just implicit.

**D4 — Warning condition becomes three-way, not two-way.**
Replace `agentLacksModel()` with a small classification function
returning one of `'configured' | 'system-default' | 'unresolved'`:
- `'configured'`: `policy.provider.default` has both `provider` and
  `model` set → no icon (unchanged from today).
- `'system-default'`: no per-agent override, but
  `useHasWorkingSystemDefault()` is true → neutral `Info` icon (D3).
- `'unresolved'`: no per-agent override and no working system default →
  today's amber `AlertTriangle`, unchanged.

## Risks / Trade-offs

- **[Risk] `default_model` on the registry's default provider could
  itself be unset even when `default_id` is set** (a provider is marked
  default but has no configured default model) → **Mitigation**: this is
  exactly the case D4's `useHasWorkingSystemDefault()` check catches —
  `!!id && !!model` correctly falls through to `'unresolved'` in that
  scenario, which is the intended, honest behavior (no silent
  false-negative).
- **[Risk] A new hook could drift out of sync with `useProviderDefault()`
  if `ProviderMetaEntity` is ever restructured later** → **Mitigation**:
  both hooks read the same singleton entity in the same file; a future
  restructure touches them together by construction, not two separate
  files that could silently diverge.

## Migration Plan

None — this is a same-session UI change with no persisted data affected;
the graph re-hydrates `ProviderMetaEntity` on every `loadProvidersIntoGraph()`
call (page load / realtime refresh), so the new field populates
automatically with no migration step.

## Open Questions

None.
