## Why

Agents, Providers, Models, Skills, and Settings are referenced from many places in the SPA — toolbars, dropdowns, header chips, capability toggles, session config panel. Migrating any one of them in isolation leaves the other consumers reading from a deleted Zustand store and causes runtime errors. Each entity needs to migrate **everywhere** in one PR.

## What Changes

One PR per entity, each rewriting **every** consumer of that entity in a single sweep:

### Agents
- `admin/pages/agents-page.tsx`
- `features/chat/agent-selector.tsx`
- `useAgentConfig` hook (in `features/chat/`)
- Chat header agent label
- Default-agent fallback in `chat-page.tsx`
- Delete `stores/agents-admin-store.ts`

### Providers
- `admin/pages/providers-page.tsx`
- `features/chat/session-config-panel.tsx`
- Any header provider chip
- Delete `stores/providers-admin-store.ts`

### Models
- `admin/pages/models-page.tsx`
- Model selector in header
- `capability-toggles.tsx` model picker
- `useProviderModels` derived hook
- Delete `stores/models-browse-store.ts`

### Skills
- `admin/pages/skills-page.tsx` (preserves the Built-in badge from `builtin-skills-ui-affordance`)
- `capability-toggles.tsx` skills section
- Agent → skills binding screen
- Delete `stores/skills-admin-store.ts`

### Settings
- `admin/pages/settings-page.tsx`
- `features/chat/session-config-panel.tsx`
- Global feature toggles (RAG config, sycophancy, etc.) wherever they read settings
- Delete `stores/settings-store.ts`

Each PR is gated by: `git grep -E "use<Entity>AdminStore|use<entity>Browse"` returns zero matches before merge.

## Acceptance

- Editing a provider in Admin causes the chat-header model badge to refresh in another open tab within 200 ms.
- Toggling an agent's enabled flag updates the agent selector dropdown without refresh.
- Renaming a setting under the Settings page reflects in `SessionConfigPanel` without refresh.
- Zero `git grep` hits for the retired stores.
