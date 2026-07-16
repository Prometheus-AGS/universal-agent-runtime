## Why

`agentLacksModel()` in `frontend/src/admin/pages/agents-page.tsx` (lines
191-198) renders an amber `AlertTriangle` "No model configured" icon next
to any agent whose `policy.provider.default` has an empty `provider` or
`model`. But every agent seeded by `default_agent()`
(`src/uar/defaults.rs`) intentionally ships with `provider`/`model` left
empty, on purpose — the backend comment there explains this defers to the
system-wide `ProviderRegistry` default rather than hardcoding a model name
that would go stale. The frontend has no way to tell "intentionally defers
to a working system default" apart from "no resolution path exists at
all"; both currently render the identical warning icon. This is exactly
the confusion behind the live bug report that led to the earlier
Orchestrator/gpt-5.2 investigation in `uar-grade-a-upgrade-2026-07` — the
icon reads as "broken" for agents that are actually fine.

## What Changes

- Distinguish two states instead of one:
  - **No per-agent override, but a working system default resolves**
    (registry has a `default_id` AND that provider has a `default_model`)
    → neutral indicator, not a warning (e.g. a muted `Info` icon or no
    icon at all — decided in design.md).
  - **No per-agent override AND no working system default** (no
    `default_id`, or the default provider has no `default_model`) → keep
    today's amber `AlertTriangle` warning; this is the genuine "agent has
    no way to resolve a model" case.
- An agent with an explicit per-agent override is unaffected either way
  (today's fully-configured case, no icon).
- Extend `ProviderMetaEntity`
  (`frontend/src/entities/types.ts`) with a `default_model: string | null`
  field, populated in `loadProvidersIntoGraph()`
  (`frontend/src/entities/fetchers/providers.ts`) from the configured
  provider matching `default_id` — the API (`UarProvider.default_model`)
  already returns this value; it's just not stored in the graph yet.

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
(no existing `openspec/specs/` capability currently covers the Admin
Agents list UI; this is a net-new, narrowly-scoped capability)
- `admin-agent-model-status-indicator`: distinguishes "intentionally
  defers to a working system default" from "no model resolution path
  exists" in the Admin Agents list, replacing the single ambiguous
  warning icon with two states.

## Impact

- **Runtime UX**: Admin Agents list — the only surface affected.
- **Provider compatibility**: none — reads existing registry default data,
  no new backend endpoint.
- **Realtime state**: `ProviderMetaEntity` gains one field, hydrated the
  same way `default_id` already is (via `loadProvidersIntoGraph()`, kept
  fresh by the existing realtime SSE bus — no new subscription needed).
- **KBD workflow state**: absorbs supplemental fix #2 from the
  `uar-grade-a-upgrade-2026-07` phase's operator-directed Admin/Agents UI
  assessment; no further action needed there once this change ships.
- **Affected files**: `frontend/src/admin/pages/agents-page.tsx`,
  `frontend/src/entities/types.ts`,
  `frontend/src/entities/fetchers/providers.ts`,
  `frontend/src/entities/hooks/use-provider-default.ts` (or a new
  sibling hook, decided in design.md).
