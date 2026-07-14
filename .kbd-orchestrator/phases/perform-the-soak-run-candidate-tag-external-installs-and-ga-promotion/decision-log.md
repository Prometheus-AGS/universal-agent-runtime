# Decision log: perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion

## D1 — 2026-07-13 — Tenancy boundary for 1.0 (assessment.md O3)

**Decision (operator)**: UAR 1.0 is **multi-tenant**. Fix all three CRITICAL user-isolation
holes before cutting the candidate tag:

- C1: user-scope sessions/threads (`src/session/thread.rs`, persistence providers)
- C2: legacy memory REST IDOR — derive identity from JWT `UserContext`, never from
  client-supplied `user_id` (`src/uar/api/memory.rs`)
- C3: user-scope knowledge bases/documents/chunks; remove the silent all-KB retrieval
  fallback (`src/uar/persistence/providers/postgres.rs:446`, `manager.rs:748-753`)

**Rationale**: operator explicitly required user isolation with no bleed-over in the
assessment scope; the JWT multi-user story is a structural advantage over Mastra (EE-gated
there) and must be real, not paper.

**Consequences**: `fix-user-isolation-sessions-memory-kb` becomes the top implementation
change in the plan; candidate tag `v1.0.0-rc.3` waits for it; isolation regression tests
(cross-user bleed) become part of the certification matrix.

Open questions still pending for /kbd-plan: O1 (conversation-scope toggles real?),
O2 (kb-retrieval BDD vacuous?), O4 (skills/agents/settings shared-admin by design?).
