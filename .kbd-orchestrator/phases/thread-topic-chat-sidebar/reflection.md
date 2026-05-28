# Reflection — `thread-topic-chat-sidebar`

**Date:** 2026-05-27
**Status:** reflect_complete

---

## 1. Goal achievement

**Phase goal:** wire the last `pending` entity (`Thread`) so server-side session SSE events reconcile into the client-first thread registry.

| # | DoD criterion | Verdict |
|---|---|---|
| F1 | `useThreadGraphSync()` mounted in `App.tsx`; reconciles graph → registry | **MET** |
| F2 | Server update to known thread → `setTitle` / `markPersisted` applied | **MET** (test 2 passes) |
| F3 | Server insert of unknown thread → registry creates persisted entry | **MET** (test 1 passes) |
| F4 | Server delete → `removeThread` called | **MET** (test 3 passes) |
| F5 | Local-only ephemeral threads untouched by absence-events | **MET** by design — diff-by-keyset only acts on mutated keys |
| F6 | Audit row flipped from `pending` → `direct (SSE-driven sync)` | **MET** |
| F7 | Tests ≥ 37/37 (target 40/40) | **EXCEEDED** — 40/40 (37 + 3 new contract tests) |
| F8 | Build clean | **MET** |

**Goal achievement: 100%.** All 8 criteria MET.

---

## 2. Delivered changes

| # | Change | Status |
|---|---|---|
| 1 | `add-thread-graph-sync-hook` | DONE — `stores/use-thread-graph-sync.ts` (80 LOC) + 3 contract tests + 2-line mount in `App.tsx` |
| 2 | `flip-thread-audit-row-to-direct` | DONE — audit row updated; **SSE-reconciler pattern** added as a sibling to the Direct migration playbook |

---

## 3. Code shape

| Metric | Value |
|---|---|
| New files | 2 (hook + test) |
| Files edited | 2 (App.tsx + audit doc) |
| LOC added | ~155 (hook + test + doc) |
| LOC deleted | 0 (additive only — `thread-registry-store` retained) |
| Contract tests | 40/40 (+3 this phase) |

---

## 4. Technical debt status

**No new debt.** The SSE-reconciler pattern is now a documented sibling to the Direct migration playbook in the audit doc — future client-first entities (e.g. local message drafts, attachment placeholders) can adopt the same shape.

Residual carry-overs (unchanged):
- Browser smoke walkthrough still owed
- Knowledge page aesthetic redesign still deferred
- Skill plugin installs are user-interactive
- Playwright screenshots need live dev server
- `ci-frontend-tests` not yet wired

---

## 5. Lessons captured

1. **Not every entity belongs in the graph as authoritative.** Client-first entities (Threads here, potentially message drafts in the future) have legitimate reasons to live in a local store. The graph becomes a *signal channel*, not a *source of truth*. The audit playbook now formalises this with the **SSE-reconciler pattern** distinct from the **Direct migration playbook**.

2. **Diff-by-keyset is cheap and correct.** The old `useGraphBridge` did the same key-count snapshot trick but called bulk `refetch`. The reconciler hook does the same diff but applies per-key actions to the local store. Result: ~80 LOC, no race against in-flight requests, no double-fetch.

3. **Initial snapshot must NOT fire reconciliation.** The hook captures the prior keyset at mount without running the diff loop — otherwise every locally-persisted thread already in the graph at mount-time would re-trigger `registerThread` and clobber the local copy. Same lesson as the `useGraphBridge` `last=""` bug from the contract-test phase.

4. **PGlite mock pattern is reusable.** `vi.mock("@/lib/db", () => ({ getDbInstance: () => stubMethods }))` lets entity sync tests run in happy-dom without needing the WASM PGlite bundle. Pin this for future store-level contract tests.

5. **Live-only sync is OK when there's no REST list endpoint.** No SSE history replay is fine for an entity created client-first; the user's own actions are the dominant input. Server-side metadata catches up live or stays stale until reconnect.

---

## 6. Cross-phase status — entity migration project COMPLETE

| Entity | Final status |
|---|---|
| Provider | direct ✅ |
| Agent | direct ✅ |
| Model | direct ✅ |
| Skill | direct ✅ |
| Memory | direct ✅ |
| CompilerSession | direct ✅ |
| KnowledgeBase + Document | direct ✅ |
| Setting | direct ✅ |
| Tool | direct ✅ |
| McpStatus | direct ✅ |
| **Thread** | **direct (SSE-reconciler) ✅** |
| ApiKey | non-realtime (intentional — secrets) |

**There are no more `pending` entities.** The "no stale data anywhere" project is structurally complete. ApiKey is correctly classified non-realtime; every other entity is on either the Direct migration playbook (admin entities) or the SSE-reconciler pattern (Thread).

Remaining carry-overs are all NON-data:
- Browser smoke walkthrough
- Knowledge aesthetic pass
- CI integration of gates
- Playwright screenshot infrastructure

---

## 7. Recommended next phase

The waypoint's next seeds are now non-data:

1. **`ci-frontend-tests`** — wire `pnpm test ≥ 40/40` + `pnpm build` + all the architectural greps (`useGraphBridge`, `useSettingsStore`, banned fonts) into CI. **Highest leverage** — guards every migration done so far.
2. **Browser smoke walkthrough** — still owed; now covers 8+ migrated pages.
3. **`readme-architecture-diagram`** — document the now-stable architecture for new contributors.
4. **`knowledge-page-aesthetic-pass`** — visual-only follow-up.

---

## 8. Progress signal

Reflection complete. The entity migration arc that started with `direct-entity-migration-providers` is closed. **Bridge pattern retired**, **all 10 dynamic entities are on direct or SSE-reconciler patterns**, **40/40 tests across 11 contract suites**, **clean build**. Three sequential phases this session (`settings-store-retirement` → `add-push-channels-backend` → `thread-topic-chat-sidebar`) all hit 100% goal achievement.
