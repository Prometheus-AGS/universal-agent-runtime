## Why

`useChatRuntime` and `useAgentConfig` sit on the busiest call path in the SPA — every chat message render passes through them, every agent switch retunes them, every model change reshapes them. They currently compose state from `useAgentsAdminStore`, `useProvidersAdminStore`, and `useSettingsStore`. After the cross-cutting migration retires those stores, the derived hooks must read directly from the entity graph instead.

This is intentionally the **last** migration in the phase because it touches the most surface and any regression here is the most user-visible.

## What Changes

- `useChatRuntime` reads `useEntity("agent", id)`, `useEntity("model", id)`, and relevant `useEntity("setting", key)` instead of store selectors.
- `useAgentConfig` composes from `useEntity("agent", id)` + `useEntity("setting", key)` results.
- `AgentSelector` and `SessionConfigPanel` are already migrated by the cross-cutting PR; this change tightens their interactions with derived hooks.
- Memoization is preserved via `useMemo` over the entity outputs so per-message renders don't re-run normalization.

Gate behind a Vite env flag (`VITE_ENTITY_MGMT_CHAT_RUNTIME`) for the first release so we can flip back quickly if anything regresses. Remove the flag in the final tests + audit change.

## Acceptance

- Sending a chat message takes ≤ the prior baseline measured in DevTools Performance.
- Switching the active agent updates the model badge + agent label within one frame, no refresh.
- Editing the global system-prompt setting (or whatever setting `useAgentConfig` reads) updates the next outgoing prompt without manual reload.
- The flag-off path (env unset) still works as a rollback escape hatch during the bake-in window.
