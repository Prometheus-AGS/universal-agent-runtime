## Discovery

Auditing the chat hot path reveals the original concern is already addressed by the bridge migration shipped in change 5:

- `useChatRuntime` (`features/chat/use-chat-runtime.ts`) reads from chat-message, chat-intent, thread-registry, thread-title stores. **None** of these are REST caches — they're client-side streaming/UI state and were explicitly out of scope.
- `useAgentConfig` (`features/chat/agent-config-context.ts`) is a React context. The provider value originates in `AgentSelector` which reads from `useAgentsAdmin`. That hook now subscribes to the entity-graph bridge for the `Agent` topic, so any SSE-delivered Agent mutation propagates into chat runtime automatically — no rewrite needed.

## 1. useAgentConfig

- [x] 1.1 Verified: receives data via context from `AgentSelector` → `useAgentsAdmin` (bridged).
- [x] 1.2 No code changes required.

## 2. useChatRuntime

- [x] 2.1 Verified: depends only on client-side streaming/UI stores (chat-message-store, chat-intent-store, thread-registry-store, thread-title-store) — all explicitly out-of-scope.
- [x] 2.2 No code changes required.

## 3. Feature flag

- [x] 3.1 `VITE_ENTITY_MGMT_CHAT_RUNTIME` flag — N/A; no code path to gate.

## 4. Verification

- [x] 4.1 Chat hot path unchanged — bridge propagation is transparent to it.
- [ ] 4.2 Manual: switch agent → header updates ≤1 frame — pending browser smoke.

## Status

**DONE — vacuously.** The bridge migration (change 5) already gives the chat runtime fresh data via context propagation. A future direct `useEntity` migration of `AgentSelector` would replace the bridge for the `Agent` entity, but the chat hot path doesn't change either way.
