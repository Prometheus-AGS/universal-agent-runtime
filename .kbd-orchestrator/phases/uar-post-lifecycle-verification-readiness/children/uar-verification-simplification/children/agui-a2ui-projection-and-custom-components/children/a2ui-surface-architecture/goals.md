# Goals — uar-post-lifecycle-verification-readiness› uar-verification-simplification› a2ui-surface-architecture

- Decide who owns A2UI surface state and how long it lives, so agent-generated UI is live, recoverable, interactive across turns, extensible with custom components, and safe on a shared server
- Assess: how A2UI and AG-UI upstream carry A2UI over AG-UI, with citations
- Assess: every consumer of A2UI surface state in UAR, with source locations
- Analyze: compare options A-E against the five objectives and recommend one
- Stop before Plan for the operator's architecture decision

## Scope decision — operator, 2026-10-28

The architecture must support all three uses at once:

1. Chat-based A2UI: streamed, agent-generated surfaces in a conversation.
2. Full applications built only from A2UI components, interacting only through AG-UI, sufficient to implement a specification like the San Saba workspace (`/Users/gqadonis/Projects/sansaba/sansaba-workspace`).
3. Surfaces that morph under highly personalized, real-time conditions.
