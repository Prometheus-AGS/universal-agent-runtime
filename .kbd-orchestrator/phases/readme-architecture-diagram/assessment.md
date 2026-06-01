# Assessment — `readme-architecture-diagram`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `ci-frontend-tests` (100%)

---

## 1. Phase goal

The session arc that started with `direct-entity-migration-providers` and just closed with `ci-frontend-tests` produced a substantial new piece of architecture:

- **Realtime entity graph spine** — SurrealDB live queries → tokio broadcast bus → SSE → frontend `useGraphStore` → admin pages
- **Two canonical patterns** — Direct migration (graph-authoritative) and SSE-reconciler (client-authoritative)
- **11 dynamic entities** all on one of those two patterns
- **40/40 contract tests** + CI gates guarding the architectural invariants

None of this is documented in the README. The `docs/migration-stale-data-audit.md` tells the migration story but a new contributor opening the README first won't find the destination architecture. **The phase goal is to bring the README up-to-date with the entity-management story** — minimal, accurate, future-proof.

### Out of scope

- Rewriting unrelated sections (LLM Configuration, Memory System, Tauri Compatibility).
- Removing existing diagrams (the Mermaid graph at line 65 is good for the LLM/config flow; this phase adds, doesn't replace).
- Slide-deck-level architectural docs (those belong elsewhere if needed).

---

## 2. Current state inventory

### 2.1 Existing README architecture coverage (line numbers)

| Section | Lines | Topic | Covers entity graph? |
|---|---|---|---|
| Architecture Overview (with Mermaid) | 65–163 | LLM config + driver + catalog + orchestrator | **No** |
| Core Design Principles | 268–308 | Tools, streaming, event contract, model catalog | **No** |
| Chat API Protocol | 310–344 | Chat SSE protocol | mentions SSE but for chat only |
| MCP | 346–366 | Tool ecosystem | **No** |
| UI Stack | 368–387 | HTMX + Web Components + Admin UI + PGlite | mentions admin pages but not their data path |
| Memory System | 390–397 | Memory scopes | **No** (Memory is itself an entity now) |

### 2.2 What's already strong

- The existing Mermaid graph at line 67 is a clean LLM-layer diagram. **Keep it as-is.**
- The "Tool Server Health" / MCP sections are accurate.
- The PGlite section honestly describes browser-side persistence.

### 2.3 What's missing

| Topic | Status |
|---|---|
| Realtime entity graph spine | **MISSING** — no mention of `useGraphStore`, SSE topics, or how admin pages stay fresh |
| Direct migration playbook | **MISSING** — destination pattern not stated |
| SSE-reconciler pattern | **MISSING** — Thread's client-first / server-supplements path |
| Form-cache pattern | **MISSING** — Setting's dirty/conflict semantics |
| Per-topic enrolment + table mapping | **MISSING** — `EntityTopic` table (10 topics) |
| Architectural invariants enforced by CI | **MISSING** — `useGraphBridge`, `useSettingsStore`, banned fonts, `outline:none` |
| Terminal admin aesthetic | **MISSING** — `data-admin-theme="terminal"` + scoped tokens |

### 2.4 Cross-doc links

The README should link to (without duplicating):
- `docs/migration-stale-data-audit.md` — full migration history + playbook
- `docs/admin-aesthetic-spec.md` — terminal aesthetic contract
- `scripts/ci-grep-gates.sh` — invariant gates

---

## 3. Definition of done

| # | Criterion | Verification |
|---|---|---|
| H1 | A new section "Frontend Architecture — Realtime Entity Graph" lives in the README between "UI Stack" and "Memory System" | grep section heading |
| H2 | A Mermaid diagram shows the data flow: SurrealDB Live Query → LiveQueryBus → SSE → SSE adapter → useGraphStore → admin pages / chat sidebar | mermaid syntax check |
| H3 | A markdown table lists the 12 entities + their topic + their pattern (direct / SSE-reconciler / non-realtime) — derived from `docs/migration-stale-data-audit.md` | manual review |
| H4 | The three canonical patterns are named and one-line described: **Direct migration playbook**, **SSE-reconciler pattern**, **Form-cache pattern** | manual review |
| H5 | Each pattern links to the canonical playbook in the audit doc | link check |
| H6 | A "CI architectural gates" callout lists the 4 grep gates + links to `scripts/ci-grep-gates.sh` | manual review |
| H7 | A "Terminal admin aesthetic" callout names the spec doc + the `data-admin-theme` scoping | manual review |
| H8 | A "Historical: bridge pattern" footnote acknowledges the retired pattern with a date pointer | manual review |
| H9 | Existing sections untouched | diff |
| H10 | `pnpm run ci-gates` still exits 0 (README changes shouldn't break anything) | output |
| H11 | The new section reads under 250 lines (concise summary, not a re-explanation of the audit doc) | wc -l on the inserted block |

---

## 4. Gap analysis

### 4.1 Diagram fidelity

The audit doc has prose; the README needs a diagram. The right shape:

```
[ SurrealDB sessions, providers, agents, ... tables ]
            │  .live() per table
            ▼
   src/uar/realtime/surreal_bus.rs::LiveQueryBus
            │  tokio broadcast per EntityTopic
            ▼
       /api/live/{topic}    (Axum SSE endpoint)
            │  EventSource per topic
            ▼
   frontend/src/lib/realtime/uar-sse-adapter.ts
            │  insert | update | delete
            ▼
   useGraphStore   (Zustand 5 + Immer 11 graph)
        │             │
        ▼             ▼
  Admin pages    Chat sidebar (via use-thread-graph-sync)
  (direct        (SSE-reconciler: graph → registry)
   migration)
```

Mermaid representation needed; one diagram covers the whole spine.

### 4.2 Entity table accuracy

The 12 entities table should be authored once in the README and reference the canonical audit doc for migration history. Avoid duplicating effort by keeping the README table to columns: `Entity | Topic | Pattern | Notes`. ~14 rows including header + non-realtime.

### 4.3 Pattern descriptions

Each pattern needs ≤ 3 sentences:

- **Direct migration playbook** — Graph is the source of truth. Page reads via `useEntity*` hooks; mutations call services directly wrapped in `optimisticUpsert` / `optimisticRemove`. SSE keeps the graph fresh; UI never goes stale.
- **SSE-reconciler pattern** — For client-first entities (Threads). Local store is authoritative; a small hook subscribes to the graph and reconciles server events into the local store. No REST refetch needed.
- **Form-cache pattern** — For pages with `dirty`/`save` semantics (Settings). Per-namespace module-level cache via `useSyncExternalStore`; commits via bulk POST with optimistic graph upsert + rollback.

### 4.4 Bridge retirement note

The bridge pattern was retired 2026-05-27. A two-line "Historical" note links to the audit's "Historical: bridge pattern (PERMANENTLY RETIRED)" appendix. Documenting the retirement helps future contributors understand why no `useGraphBridge` exists.

### 4.5 Risk

- **Documentation drift.** README content drifts faster than code. To minimise: ALL three patterns link to the audit doc (single source of truth); the README block is intentionally a summary + index, not a duplicate.
- **Mermaid rendering on GitHub.** GitHub renders Mermaid blocks natively since 2022. Verify the new block renders by previewing locally or via `gh repo view --web` after push.

---

## 5. Sequencing recommendation

3 changes:

1. **`author-readme-entity-architecture-section`** — write the new section (Mermaid + entity table + pattern descriptions + callouts) and splice into README between "UI Stack" and "Memory System". Locally render-check.
2. **`add-bridge-retirement-footnote`** — single-line acknowledgment near the entity-architecture section or in a Historical-notes subsection at the README end. Cheap.
3. **`link-from-audit-back-to-readme`** — symmetric link: the audit doc's intro mentions "see README for the architectural summary". Closes the doc graph.

Each change verifies: `pnpm run ci-gates` still exits 0 (no architectural regressions); mermaid block parses (manually or via `mmdc --validate` if available).

---

## 6. Decisions (defaults — no questions needed)

| Decision | Choice | Rationale |
|---|---|---|
| Insertion point | Between "UI Stack" and "Memory System" | UI Stack already touches on admin pages; this section deepens that |
| Diagram style | Mermaid (matches existing README convention) | Renders on GitHub natively |
| Entity table | Authored in README, links to audit doc for full history | Quick reference + canonical source |
| Pattern descriptions | ≤ 3 sentences each | Concise; full details in audit playbook |
| Bridge retirement | Footnote-style | Acknowledges history without dragging the section down |
| Section length budget | ≤ 250 lines | Fits the README's existing density |

---

## 7. Progress signal

Assessment complete. Defaults sufficient. Next: `/kbd-plan readme-architecture-diagram` (or proceed straight to execute — plan is small).
