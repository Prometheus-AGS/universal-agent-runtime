## Why

Providers, LLM models, agents, skills, and settings are the entities referenced across the most surfaces in the SPA — toolbars, chat header, agent dropdowns, settings panels, model pickers. They are also the ones where stale data is most disruptive (e.g. user edits a provider's base URL, but the chat header still shows the old model badge). This change extends the entity-mgmt migration from the Knowledge pilot to every shared-state Admin surface.

## What Changes

For each of `provider`, `model`, `agent`, `skill`, `setting`:

- Add `fetchEntity` + `fetchList` to `frontend/src/services/entities/`.
- Replace bespoke fetchers / Zustand store fetch logic in the matching Admin page with `useEntity` / `useEntityList`.
- Register the topic in the realtime adapter (if not already from the bridge change).
- Identify and rewrite **every** consumer outside the matching Admin page that reads the same entity:
  - Chat header model/agent badge → `useEntity("model", id)` / `useEntity("agent", id)`.
  - Agent selector dropdown → `useEntityList("agent")`.
  - Capability toggles (model picker) → `useEntityList("model")`.
  - Session config panel → `useEntity("setting", key)`.

Audit log: produce a single document at `docs/migration-stale-data-audit.md` listing every render path that used to fetch one of these entities and confirming the migration.

## Acceptance

- Editing a provider in Admin changes the chat-header model badge within 200 ms in another tab, no refresh.
- Toggling an agent's `enabled` flag updates the agent picker dropdown immediately.
- Grep `git grep "fetch\\(" frontend/src` shows zero direct REST fetches for these entity types outside `services/entities/`.
