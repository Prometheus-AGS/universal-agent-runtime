# UAR React 19 UI — Assessment, Recommendations, Architecture, Functional Specification & Implementation Plan

**Date:** 2026-08-01
**Author:** Kimi Work (research + analysis; no code written)
**Scope:** Built-in React 19 frontend of the Universal Agent Runtime (`frontend/`), assessed against the binding KnowMe UI/UX standard and comparable agent-harness interfaces.
**Status:** Assessment complete. Nothing in this document modifies source code; it is the planning input for agent-driven implementation (Claude Code, Codex, Kimi Code) under the prometheus skill pack / OpenSpec / KBD workflow.

---

## 1. Executive summary

The UAR built-in UI is not a blank slate — it is a **partially-converged product with strong bones and three competing identities**:

- **Good bones.** A real layering contract exists (`Component → Hook → Store → Service`, `docs/frontend-architecture.md`), enforced by `scripts/check-frontend-boundaries.mjs`. Assistant UI is genuinely mounted at the chat boundary. Threads persist to PGlite. An 18-section admin console exists with real entity-graph plumbing. The KnowMe token set has been ported 1:1 into `frontend/src/index.css` by deliberate operator decision (`openspec/changes/uar-ui-token-convergence`).
- **The mess is real, and it is structural, not cosmetic.** Three mutually contradictory design standards are simultaneously "authoritative" in the repo: the KnowMe Flat 2.0 standard (applied to the shell and chat), a stale Material-3/purple design doc (`docs/UI_DESIGN.md`), and a Terminal/CRT phosphor-green admin aesthetic (`docs/admin-aesthetic-spec.md`) that **directly violates the KnowMe standard it is supposed to live alongside** (it mandates visible thin lines and an all-monospace type system — both prohibited by Flat 2.0 §3 and typography §4.2).
- **Identity confusion is baked into the running app.** The left rail renders a `KnowMeLogo` + `KnowMeWordmark` with the subtitle "Universal Agent Runtime" (`frontend/src/app/AppShell.tsx:82-91`), while the admin sidebar brands itself "Runtime Console — UAR operations" (`frontend/src/admin/admin-shell.tsx:170-176`). A first-time user cannot tell what product they are in.
- **Compliance debt is measurable.** 622 border-idiom class usages across `frontend/src`, 50 `variant="outline"` buttons, drop shadows in 9 files, `backdrop-blur` in 6 places — all prohibited by Flat 2.0. The CSS saves the visuals by forcing `--border: transparent`, but the *code idioms* remain border-first, so every new component re-imports the old habits.
- **State architecture is mid-migration with dead weight.** ~40 Zustand stores coexist with a half-adopted Prometheus Entity Management (PEM) entity graph, and a **completely unused** TanStack Query `QueryClientProvider` sits at the app root (`frontend/src/App.tsx:12-14,62`) — zero `useQuery`/`useMutation` call sites exist anywhere in `src/`.
- **The chat experience is the weakest product surface relative to its importance.** It works, but it is far below the KnowMe §7 "world-class conversation" bar: no pin/archive, no recency grouping, no per-thread drafts, no branching/regenerate-preserving-history, no unread/running indicators, no local/sync/cloud lane labeling, and a hard full-screen "No Model Configured" gate instead of a local-first default.

**Recommendation in one paragraph:** Make the KnowMe UI/UX standard the *single* design authority for every UAR surface; retire the Terminal/CRT admin theme and the stale Material-3 doc (folding their legitimate "operator instrument" intent into a KnowMe-compliant **high-density console mode**); resolve product identity with a config-driven brand slot (UAR standalone vs. KnowMe-embedded); restructure information architecture around three destinations (Home, Chat, Console) instead of today's Chat/Admin/About; finish the PEM migration and delete the dead TanStack Query root; then run two remediation waves — Wave 1: design-system convergence + Flat 2.0 purge; Wave 2: chat experience elevation to KnowMe §7 — validated by Storybook visual regression and the already-scaffolded screen-by-screen BDD suite. A phased, agent-assignable plan is in §7.

---

## 2. Current-state assessment (evidence-based)

### 2.1 Inventory — what actually exists today

| Area | Evidence | State |
|---|---|---|
| App shell | `frontend/src/app/AppShell.tsx`, `frontend/src/app/navigation.ts` | 3 destinations: Chat (`/threads`), Admin (`/admin/*`), About. Left rail ≥768px, bottom bar below. KnowMe tokens applied. Readiness card with live `/healthz` status. |
| Chat | `frontend/src/pages/chat-page.tsx`, `frontend/src/components/assistant-ui/enhanced-thread.tsx` (821 lines), `frontend/src/features/chat/*` | Assistant UI runtime per thread; agent selector; session-config panel; attachment manager; memory context; tool-approval dialog; context-usage bar; tool-call/memory-chunk/A2UI-artifact block renderers. |
| Thread library | `frontend/src/components/layout/left-sidebar.tsx` | Search (title substring only), new thread, rename, delete-with-confirm, relative timestamps, last-message preview. No pin/archive/grouping/drafts. |
| Persistence | `frontend/src/lib/db.ts`, `frontend/src/stores/thread-registry-store.ts`, `use-thread-graph-sync.ts` | PGlite-backed threads + messages; hydrate-on-mount; ephemeral-then-persisted thread lifecycle. Honest local-first foundation. |
| Admin console | `frontend/src/admin/admin-shell.tsx`, `frontend/src/pages/admin-page.tsx`, 18 sections in 5 nav groups | Cockpit, Runs, Approvals, Protocols, Providers, Credentials, Models, Knowledge, Memory, Agents, Skills, Tools, API Keys, Cost, Settings, MCP Health, Compiler, A2UI Testing. ⌘K command palette. |
| State layer | `frontend/src/stores/` (~40 stores), `frontend/src/entities/`, `frontend/packages/prometheus-entity-management` | Mid-migration: entity graph + PEM package exist; many pages already use "direct entity" reads; legacy stores remain. |
| A2UI investment | `frontend/packages/a2ui-{core,react,lit,svelte,uar,inspector}` | Deep protocol investment — conformance, semantic DOM, v0.8/v0.9 catalogs, multi-renderer. |
| Design tokens | `frontend/src/index.css` (429 lines), `frontend/tailwind.config.ts` | KnowMe dark+light ladders ported exactly; `--border: transparent`; ember/cyan accents; 4-step surface ladder. Terminal admin theme scoped to `[data-admin-theme="terminal"]`. |
| Testing | `frontend/e2e/`, Storybook 10, Vitest 4, `ui-primitives.stories.tsx` | Infrastructure present; BDD screen coverage acknowledged as absent (`openspec/changes/screen-by-screen-validation`). |

### 2.2 The seven structural problems

**P1 — Three conflicting design authorities.**
1. `docs/knowme-ui-ux-standard.md` (KnowMe repo) — binding, Flat 2.0, ember/cyan, ported into `index.css`.
2. `docs/UI_DESIGN.md` — stale "Material 3 Flat 2.0" with a *purple* user-bubble palette (`hsl(262 60% 18%)`) and different surface values. Untouched by the token convergence; still reads as current guidance.
3. `docs/admin-aesthetic-spec.md` — self-declared "authoritative" Terminal/CRT theme: phosphor green `#7fffa1`, amber warnings, **visible thin lines** (`--terminal-line`), all-monospace type, `13px` body. This contradicts KnowMe §3.3 ("No one-pixel rules…"), §4.2 (Inter for UI copy; mono only for metadata), and §4.1 (restrained ember accent) on every axis. It is applied at runtime by `pages/admin-page.tsx:52-60`.

**P2 — Identity/branding incoherence.** KnowMe logo + wordmark in the rail, "Universal Agent Runtime" subtitle, "Runtime Console / UAR operations" in admin, "Threads" vs "Chat" label mismatch between sidebar and destination. No documented decision exists for what the built-in UI *is*: UAR console, KnowMe preview, or both.

**P3 — Flat 2.0 compliance debt.** Measured on 2026-08-01 in `frontend/src`: 622 border-idiom usages (e.g. `admin-shell.tsx:164` `border-b border-border`, `chat-page.tsx:50` `border border-primary/30`), 50 `variant="outline"`, 9 files with `shadow-*`, 6 with `backdrop-blur`/gradient idioms. The transparent `--border` token hides most of it visually, but the terminal theme *deliberately reintroduces* visible lines. New code keeps copying the old idioms.

**P4 — State-layer sprawl.** ~40 Zustand stores; a PEM entity graph mid-adoption (`entities/` + `frontend/packages/prometheus-entity-management`); a dead `QueryClientProvider` (zero query usage); plus hook-level local state for thread rename/delete dialogs in the sidebar. The KnowMe standard §6.2/§12 is explicit: PEM owns entity reactivity, Zustand owns ephemeral UI state, TanStack Query is not part of the architecture. The repo is ~60% of the way there with the dead 40% still load-bearing in places.

**P5 — Chat below the KnowMe §7 bar.** Against §7.2/§7.4/§7.8: no pin/archive/export, no Today/7-Days/Older grouping, title-only search (not message content), no per-thread draft preservation, no branch/regenerate-with-history, no unread/running/failed indicators in the library, no lane labeling (on-device / my server / cloud), no local-first default model path — instead a blocking full-screen "No Model Configured" gate (`chat-page.tsx:80-102`). The standard also requires the library to reflect *background runs continuing* when you switch threads; the current per-thread `useChatRuntime(threadId)` remount makes this unverifiable and likely lossy.

**P6 — Admin information overload.** 18 sections including dev-only surfaces (Compiler, A2UI Testing) in the production nav, against a standard that warns "powerful without looking like an infrastructure console" (KnowMe §1). `settings-page.tsx` is 3,336 lines — a monolith that mixes schema-form rendering, cache logic, and dozens of settings domains. Several sections (MCP Health, Cost, Approvals) are operator-grade but presented with the same weight as daily-use surfaces.

**P7 — Process artifacts without follow-through.** `openspec/changes/impeccable-uiux-audit/` and `openspec/changes/uiux-remediation-wave-1/` exist as **empty scaffolds** (no `proposal.md`), so the repo records intent to fix the UI twice without ever executing. `docs/UI_DESIGN.md` was never archived. The KBD waypoint shows UI work was repeatedly preempted by server-side phases.

### 2.3 What is genuinely good and must be preserved

- The **layering contract** and boundary checker — rare and valuable discipline; the remediation must strengthen, not bypass, it.
- **Assistant UI mounted at the real chat boundary** (KnowMe §6.3 calls this out as the thing most projects fake).
- **PGlite thread persistence** with ephemeral-first thread creation (§7.2 "type before persistence finishes" — already the right shape).
- **KnowMe token port** with documented light-mode contrast correction (`index.css:97-100` notes the 7.06:1 vs 2.38:1 muted-foreground fix — exactly the kind of evidence the standard wants).
- **The A2UI/AG-UI protocol investment** — ahead of every comparable open-source harness (see §3.2).
- **Admin ⌘K palette, readiness card, and offline banner** — correct instincts, worth keeping under the new design authority.

---

## 3. Comparative research — what the best comparable interfaces do

### 3.1 Self-hosted chat harnesses (the UAR built-in UI's peer set)

| Product | Stack | Lessons for UAR |
|---|---|---|
| **Open WebUI** (~145k★) | SvelteKit + FastAPI | The polish benchmark for self-hosted. Wins on *default-path UX*: works out of the box against a local model, RAG built in, mobile PWA. UAR's "No Model Configured" gate is the exact anti-pattern Open WebUI avoids. |
| **LobeChat** (~79k★) | Next.js + Lobe-UI | "Model-first" interaction model — fast provider switching, agent marketplace, Artifacts side panel. Best-in-class visual polish; demonstrates that a single design-token system can carry both chat and marketplace density. |
| **LibreChat** (~40k★, ClickHouse-acquired 2025-11) | React + Express + Mongo/Meilisearch | Enterprise benchmark: conversation search via real full-text index, presets, SAML/OIDC, audit. Relevant to UAR's admin side: usage tracking and audit are *first-class destinations*, not buried sections. |
| **Jan.ai** | Tauri + local inference | The local-first default-path benchmark: download-a-model-with-progress onboarding, zero-credential first run. Directly applicable to UAR's embedded/Tauri story (KnowMe §7.3). |
| **AnythingLLM** | React + Node | Workspace/document-centric IA; shows how to make RAG configuration a product surface rather than an admin chore. |

**Consensus patterns across all five:** (1) the thread library is a working surface with search/pin/archive and recency grouping; (2) model/lane identity is always visible in the composer/header; (3) empty states teach; (4) settings are deliberately separated from daily surfaces; (5) exactly one design system per product.

### 3.2 Generative-UI protocol landscape (AG-UI / A2UI / MCP Apps)

The industry has converged on **three generative-UI patterns** (CopilotKit, 2026; Microsoft Agent Framework ships all 7 AG-UI features natively as of 2026-07):

1. **Static generative UI (AG-UI events → pre-built components).** Frontend owns layout; agent picks component + data. Highest trust, lowest flexibility. *This is what UAR's `enhanced-thread.tsx` block renderers already are.*
2. **Declarative generative UI (A2UI / Open-JSON-UI specs).** Agent streams a UI spec (`surfaceUpdate` → `dataModelUpdate` → `beginRendering`); host renders with its own theme. Shared control. *This is what `frontend/packages/a2ui-*` implements — and UAR is unusually far ahead here, with its own core/react/lit/svelte renderers plus an inspector.*
3. **Open-ended generative UI (MCP Apps, sandboxed iframes).** Full UI surfaces across trust boundaries. UAR should *not* chase this in the built-in UI; the A2UI declarative path already covers the safe 90%.

**Implication:** UAR's protocol investments are architecturally correct and ahead of LibreChat/Open WebUI. The gap is not protocol capability — it is that the *product surfaces* rendering these protocols are visually and behaviorally unpolished. The remediation should be surface-level convergence, not protocol churn.

### 3.3 Agent observability consoles (the admin's peer set)

Langfuse and LangSmith define the category grammar the UAR Console should adopt:

- **Hierarchical run/trace trees** (parent agent span → child tool spans → LLM generations) with per-node latency/token/cost — UAR's Runs section should present exactly this tree over its append-only AG-UI event record (KnowMe §7.5 already demands this "activity/event inspector" *inside chat*; the console view is the same projection at admin depth).
- **Live dashboard first, drill-down second** — LangSmith's pre-built dashboards (error rate, latency distribution, token/cost) map to UAR's Cockpit.
- **Human-in-the-loop queues** — LangSmith annotation queues ≈ UAR Approvals; both deserve first-class placement.
- **Prompt/config versioning with playground** — maps to UAR Agents/Skills sections' AI-builder ambitions.

**Implication:** the admin is not "too developer-y" — it is *insufficiently opinionated*. The fix is not to hide the console but to organize it around the observability grammar (Cockpit → Runs → Approvals → Config → Diagnostics) and give it a compliant high-density presentation.

---

## 4. Recommendations

Ordered by decision dependency. R1–R3 are *decisions for the operator*; R4–R8 follow from them.

### R1 — Single design authority (decision)
Adopt `docs/knowme-ui-ux-standard.md` as the sole binding visual standard for **all** UAR surfaces. Concretely:
- **Retire the Terminal/CRT admin theme** (`admin-aesthetic-spec.md` → archive with a superseded-by note). Preserve its legitimate intent — "an operator instrument, not a SaaS dashboard" — as a **KnowMe-compliant Console density mode**: compact 13px metadata rows, JetBrains Mono for measurements/IDs/timestamps (permitted by KnowMe §4.2), denser tables with row-background separation instead of grid lines, phosphor-green *replaced* by KnowMe cyan for live/streaming signal and ember for actions. Optional: a user-selectable "Phosphor" accent theme *derived from the same token ladder* may be offered later as a theme variant, but only after the base system is compliant.
- **Archive `docs/UI_DESIGN.md`** (stale purple Material-3) with a pointer to the KnowMe standard.
- Add a one-page `docs/ui-design-authority.md` that says: one standard, one token source (`frontend/src/index.css`), one accent policy, and the acceptance checklist (KnowMe §12) applies to every PR.

### R2 — Product identity (decision)
The built-in UI must know what it is. Recommended resolution: **the built-in UI is the UAR Console — a first-party operator+chat surface for the Universal Agent Runtime — skinned in the KnowMe design language because KnowMe is the house standard.** Implementation: a config-driven brand slot (`productName`, `wordmark`, `logo`) defaulting to UAR branding; KnowMe-embedded deployments override it. Remove the hardcoded `KnowMeLogo`/`KnowMeWordmark` from `AppShell` in favor of the brand slot, ending the current both-at-once state. Fix the "Threads" vs "Chat" label mismatch (destination says Chat, sidebar says Threads — pick "Chats"/"Conversations" per KnowMe §7 vocabulary).

### R3 — Information architecture (decision)
Replace Chat/Admin/About with four destinations:

1. **Home** — the KnowMe §9 instrument panel: readiness, active model + lane, recent conversations, capability entry points, cost/health glance. (New; mostly a projection of existing stores.)
2. **Chat** — the conversation workspace (library + thread + composer).
3. **Console** — today's Admin, reorganized from 18 flat sections into 5 groups: *Observe* (Cockpit, Runs, Approvals, Cost), *Configure* (Providers, Credentials, Models, Agents, Skills, Tools, Knowledge, Memory), *Govern* (API Keys, Settings, MCP Health), *Develop* (Compiler, A2UI Testing — hidden behind a dev-mode flag in production builds), plus ⌘K.
4. **About** — unchanged (version, deployment mode, diagnostics copy).

### R4 — State-layer completion
Finish the PEM migration, then delete: the dead `QueryClientProvider` (`App.tsx`), the stores retired by the migrate-* changes, and hook-level persistence duplication. Publish a store-inventory table (store → owner → PEM replacement → retirement change) so parallel agents don't resurrect them. Zustand keeps only: active thread id, composer draft, panel visibility, scroll intent, stream assembly — per KnowMe §6.2.

### R5 — Chat elevation to KnowMe §7
The full §7.2/7.4/7.5/7.8 contract: pin/archive/export, recency grouping, full-text search across titles *and* message bodies (PGlite FTS), per-thread drafts, regenerate-with-history (append variant, never destroy), unread/running/failed badges, lane labeling, background-run continuity across thread switches, jump-to-latest on scroll-up, collapsed-by-default thinking with live expansion, citation source cards, and a local-first default path replacing the blocking no-model gate (offer "download default local model" or "configure provider" as one-action recoveries).

### R6 — Flat 2.0 purge with mechanical enforcement
Codemod the 622 border idioms / 50 outline variants / shadows / blurs; restyle `button.tsx` variants so `outline` becomes a filled muted-surface variant (or remove the variant); then enforce: ESLint `no-restricted-syntax` rules banning `border`, `shadow`, `backdrop-blur`, `divide`, `ring-1` class literals + a Storybook visual-regression gate (the already-planned `docs-storybook-visual-regression-perf-budget` change). This converts the purge from a one-time effort into a permanent gate.

### R7 — Protocol surfaces keep their investment
Keep AG-UI/A2UI renderers as-is; the work is (a) retheming A2UI surfaces to KnowMe tokens (the planned `a2ui-world-class-theming-a11y-i18n` change), (b) mounting the activity/event inspector projection inside chat per KnowMe §7.5, (c) retiring A2UI Testing from production nav (planned `retire-a2ui-testing-page-from-prod`).

### R8 — Validation as product evidence
Execute `screen-by-screen-validation` (already specified, 20-screen BDD with video proof) as the *acceptance gate* for each remediation wave rather than a separate phase — every UI change ships with its screen's BDD evidence.

---

## 5. Target architecture

### 5.1 Layering (unchanged contract, completed implementation)

```text
React 19 component (pure render + hooks)
  → feature hook (presentation composition only)
    → PEM 3.x entity domain  (durable entities, queries, mutations, realtime reconciliation)
    → Zustand store          (ephemeral: selection, drafts, panels, stream assembly)
      → service (typed fetch/SSE/upload adapters)
        → Axum API / AG-UI SSE / PGlite (lib/db.ts worker)
```

Enforcement: `check-frontend-boundaries.mjs` continues; add the R6 style gates; delete the TanStack Query root. `Component → Service` or `Hook → Service` imports remain hard errors with an empty production allowlist as the exit criterion (`close-react-boundary-gate`).

### 5.2 Ownership map (aligns with KnowMe §6.3, UAR-adapted)

| Concern | Owner | Note |
|---|---|---|
| General primitives | Shadcn (restyled, Flat 2.0) | `components/ui/*` — restyle variants, don't wrap |
| Chat thread/composer/streaming | Assistant UI | Already mounted; extend with §7 renderers |
| Rich content blocks | Typed renderers in `features/chat/components` + `@prometheus-ags/gen-ui-react` when shared | Exhaustive over the ContentBlock union; compile-time exhaustiveness |
| Declarative agent UI | `packages/a2ui-react` (themed to KnowMe) | Retheme, don't rewrite |
| Durable entities | PEM 3.x + PGlite | threads, messages, blocks, citations, attachments, drafts, agui_events |
| Ephemeral UI state | Zustand | ≤ ~10 stores after R4 |
| Server state / admin entities | PEM over REST/SSE services | Complete the migrate-* wave |
| Brand identity | Config-driven brand slot | R2 |

### 5.3 Design-token pipeline

Single source: `frontend/src/index.css` (HSL channel mechanism retained per `uar-ui-token-convergence`). Add: (a) a token-hash parity check against the KnowMe source CSS in CI; (b) Console density tokens (`--density-compact` spacing/row-height scale) instead of a parallel theme; (c) Storybook stories per primitive in both themes × both densities.

### 5.4 Chat runtime continuity model

Fix the per-thread remount problem: hoist stream assembly into a run-scoped (not thread-view-scoped) store keyed by `runId`, subscribed by the thread view. Switching threads cancels only view subscriptions; the SSE adapter keeps ingesting into PGlite + the run store; the library shows running/failed badges from run state. This is the single most important behavioral fix in the plan and is the frontend half of the `uar-scoped-chat-control-plane` work.

### 5.5 Screen inventory (target)

| Destination | Screens | Primary sources today |
|---|---|---|
| Home | Instrument panel | new (projects health/models/threads/cost stores) |
| Chat | Library, Thread, Composer, Activity inspector, Artifact side panel | chat-page, enhanced-thread, left-sidebar |
| Console / Observe | Cockpit, Runs (trace tree), Approvals queue, Cost | runtime-console-page, cost-dashboard-page |
| Console / Configure | Providers, Credentials, Models, Agents (+AI builder), Skills, Tools, Knowledge, Memory | existing admin pages, redesigned |
| Console / Govern | API Keys, Settings (split the 3,336-line monolith by domain), MCP Health | auth-page, settings-page, McpHealthPage |
| Console / Develop (flag-gated) | Compiler, A2UI Testing | compiler-page, A2uiTestingPage |
| About | About + diagnostics | about-page |

---

## 6. Functional specification (selected, normative)

Format: requirement ID — statement — acceptance evidence. Full per-screen detail belongs in each OpenSpec change; these are the load-bearing requirements.

### 6.1 Design system
- **DS-1** Every surface renders from the KnowMe token ladder; zero visible borders, dividers, layout shadows, blurs, or gradients. Evidence: ESLint style gate green; visual-regression baseline; grep audit = 0 outside allowlisted primitives internals.
- **DS-2** `docs/UI_DESIGN.md` and `admin-aesthetic-spec.md` are archived with superseded-by pointers; `docs/ui-design-authority.md` exists and is linked from AGENTS.md.
- **DS-3** Light and dark themes pass WCAG 2.2 AA on all text/status combinations; evidence: Storybook a11y addon + contrast CI check.
- **DS-4** Console density mode: compact spacing scale + mono metadata, same tokens. Evidence: density stories for Cockpit/Runs/Settings.
- **DS-5** Brand slot: `productName`/logo/wordmark from runtime config; default UAR; evidence: shell renders either brand from config without code change.

### 6.2 Home (new)
- **HM-1** Answers in ≤2s of paint: Is the runtime ready? Which model/lane is active? What did I do last? Evidence: readiness card, active-model chip, recent-conversations strip, capability entries (New chat, Console, Knowledge, Models).
- **HM-2** All widgets project existing PEM entities; no new backend endpoints unless already exposed.

### 6.3 Chat
- **CH-1** Library: search across titles *and* message bodies; groups Today / Previous 7 days / Older; item actions rename, pin, archive, export, delete (recoverable within 30s undo toast); badges for running/failed/unread. Evidence: BDD scenarios + PGlite FTS migration.
- **CH-2** Per-thread draft preserved across switches and reloads. Evidence: `drafts` entity + reload BDD.
- **CH-3** Regenerate appends a variant; prior answer remains navigable. Branch-from-message creates a child conversation recording `parent/branch` metadata. Evidence: schema + thread BDD.
- **CH-4** Background continuity: switching threads never cancels a run; returning restores thread, expansion state, draft, and scroll anchor. Evidence: run-store unit tests + BDD with two concurrent runs.
- **CH-5** Streaming: echo user message immediately; one assistant message updated incrementally; auto-scroll only while near bottom with jump-to-latest otherwise; stop/retry/copy/feedback actions; events persisted incrementally (crash-safe). Evidence: stream-store tests; fault-injection BDD.
- **CH-6** ContentBlock exhaustiveness: text, thinking (collapsed default, cyan surface, duration/token metadata), code (lang label + copy), citation (inline markers + source cards), memory (read/proposed/written/updated/rejected + inspect), tool use/result (state + inspectable input), skill, artifact (preview + download), image, divider (spacing only), AG-UI state delta, confirmation dialog (safe default). Compile-time exhaustiveness failure on new variants. Evidence: type test + per-block stories.
- **CH-7** Lane labeling: every thread header shows On-device / My server / Cloud provider with text+icon, never color alone. Evidence: header stories.
- **CH-8** First-run: if no model is configured, chat offers one-action recovery (configure provider; download default local model where the host supports it) — never a dead-end full-screen gate. Evidence: onboarding BDD.
- **CH-9** Errors render on a dedicated error surface with plain-language recovery; raw transport/Rust/JS errors never appear as assistant content. Evidence: fault-injection BDD.

### 6.4 Console
- **CO-1** IA regrouped per R3; dev sections flag-gated off in production builds. Evidence: nav snapshot tests.
- **CO-2** Runs presents the hierarchical trace tree (agent span → tool spans → LLM generations) with per-node latency/tokens/cost, filterable event families, sanitized payloads, and replay from the append-only event record without re-contacting the model. Evidence: replay BDD against recorded fixtures (`runtime-replay-fixtures.ts` already exists).
- **CO-3** Approvals is a queue (pending first, bulk-safe defaults, full consequence disclosure), wired to the reconciled tool-approval policy (`governance-tool-approval-reconciliation`).
- **CO-4** Settings monolith split by domain (LLM, RAG, Memory, Security, Realtime, UI prefs) with shared schema-form machinery; no page > ~600 lines. Evidence: file-size + boundary checks.
- **CO-5** Every console page: intentional empty/loading/degraded/error states on KnowMe surfaces. Evidence: state stories per page (the existing `admin-states.tsx` is the seed).

### 6.5 Cross-cutting
- **XC-1** Layering boundary allowlist trends to empty; no new entries without an ADR.
- **XC-2** TanStack Query dependency and provider removed; PEM owns all server/entity reactivity.
- **XC-3** Store inventory published; each surviving store has a one-line charter (ephemeral-only).
- **XC-4** Accessibility: keyboard-only completion of new-chat → send → approve-tool → inspect-run; skip-nav, landmarks, polite live regions for streaming; reduced-motion honored. Evidence: Playwright axe scans + manual SR pass.
- **XC-5** Responsive: 320/768/1024/1440 review in both themes; bottom-bar nav <768px; composer above safe areas.

---

## 7. Implementation plan (agent-executable)

Sequenced for the prometheus skill-pack workflow: each item is an OpenSpec change with a spec delta, executed by the assigned harness (Claude Code / Codex / Kimi Code), committed via the repo's conventional-commit flow, gated by the listed evidence. Wave gates are hard: a wave does not start until the prior wave's gates are green.

### Wave 0 — Decisions & authority (operator + any agent, 1–2 days)
| # | Change | Harness | Gate |
|---|---|---|---|
| 0.1 | `ui-design-single-authority` — archive stale docs, write `docs/ui-design-authority.md`, record R1/R2/R3 decisions as ADR-0xx | Kimi Code (docs) | Docs merged; links from AGENTS.md + README |
| 0.2 | `brand-slot-config` — runtime brand config + shell rendering; remove hardcoded KnowMe wordmark | Codex | Brand switch demo, shell tests |

### Wave 1 — Design-system convergence (2–3 weeks)
| # | Change | Harness | Gate |
|---|---|---|---|
| 1.1 | Fill the empty `impeccable-uiux-audit` scaffold: full screen inventory + violation census (mechanical counts per file) | Kimi Code | Audit report merged (this document is the seed) |
| 1.2 | `flat2-codemod` — purge border/outline/shadow/blur idioms; restyle `button.tsx` and sibling variants; admin-shell + chat-page first | Claude Code | Grep census → 0 outside allowlist; visual review |
| 1.3 | `retire-terminal-admin-theme` — delete `[data-admin-theme="terminal"]`, port its density intent to Console density tokens | Claude Code | Admin renders on KnowMe ladder; density stories |
| 1.4 | `style-gate-eslint` — no-restricted-syntax class bans + CI wiring | Codex | Intentional violation fails CI |
| 1.5 | Execute `docs-storybook-visual-regression-perf-budget` (already planned) — stories for all primitives ×2 themes ×2 densities + VR gate | Codex | VR gate green on main |
| 1.6 | `console-ia-regroup` — 5-group nav, dev flag, label fixes (Chat/Threads) | Codex | Nav snapshot tests; screen-by-screen BDD updated |

### Wave 2 — State-layer completion (2–3 weeks, parallel-eligible with late Wave 1)
| # | Change | Harness | Gate |
|---|---|---|---|
| 2.1 | `remove-tanstack-query` — delete provider + dep | Codex | Zero references; bundle diff |
| 2.2 | Complete migrate-* wave (settings reads/mutations, knowledge, memory, models, providers, cross-cutting pages) — the already-scaffolded `migrate-*` changes, batched per page | Claude Code (large pages) / Codex (mechanical) | Per-page BDD; store retired in same PR |
| 2.3 | `store-inventory-and-retirement` — publish charter table; delete dead stores | Kimi Code | Inventory doc; store count ≤ charter |
| 2.4 | `settings-page-split` — domain split of the 3,336-line monolith | Claude Code | CO-4 gate |

### Wave 3 — Chat elevation (3–4 weeks)
| # | Change | Harness | Gate |
|---|---|---|---|
| 3.1 | `chat-run-scoped-runtime` — run-store keyed by runId; background continuity (CH-4, CH-5) | Claude Code | Concurrent-runs BDD |
| 3.2 | `chat-library-upgrade` — FTS search, grouping, pin/archive/export, badges, undo-delete (CH-1) + PGlite FTS migration | Codex + Claude Code (schema) | CH-1 BDD |
| 3.3 | `chat-drafts-and-regenerate` — drafts entity, variant append, branching (CH-2, CH-3) | Claude Code | CH-2/CH-3 BDD |
| 3.4 | `chat-contentblock-exhaustiveness` — typed renderers for every block + inspector projection (CH-6; KnowMe §7.5) | Claude Code | Compile-time exhaustiveness test; per-block stories |
| 3.5 | `chat-lane-and-first-run` — lane labels (CH-7), first-run recovery replacing the no-model gate (CH-8), error surfaces (CH-9) | Codex | Onboarding + fault-injection BDD |
| 3.6 | Absorb `uar-scoped-chat-control-plane` UI bindings — effective-policy chip in thread header/session panel | Claude Code | Policy chip shows resolved global→agent→conversation policy |

### Wave 4 — Console elevation & Home (2 weeks)
| # | Change | Harness | Gate |
|---|---|---|---|
| 4.1 | `runs-trace-tree` — hierarchical run inspector + replay (CO-2) | Claude Code | Replay BDD on fixtures |
| 4.2 | `approvals-queue` — queue UX (CO-3) with `governance-tool-approval-reconciliation` | Codex | Approvals BDD |
| 4.3 | `home-instrument-panel` — Home destination (HM-1/2) | Codex | Home BDD + stories |
| 4.4 | `a2ui-knowme-theming` — execute planned `a2ui-world-class-theming-a11y-i18n`; retire A2UI Testing to dev flag (`retire-a2ui-testing-page-from-prod`) | Codex | A2UI conformance + themed stories |

### Wave 5 — Certification (1 week)
| # | Change | Harness | Gate |
|---|---|---|---|
| 5.1 | Execute `screen-by-screen-validation` across the 20-screen inventory with video-proof bundles | Kimi Code | Evidence bundle per screen |
| 5.2 | `close-react-boundary-gate` — empty production allowlist | Codex | Boundary gate at zero |
| 5.3 | Accessibility certification (XC-4) + responsive sweep (XC-5) | Claude Code | axe clean; manual SR log |

### Dependency & concurrency notes
- Wave 1.2/1.3 (Claude) and Wave 2.1/2.2 (Codex) touch disjoint surfaces (styles vs stores) and may run concurrently in **separate worktrees** per the repo worktree convention (`~/.claude/worktrees/`) — the waypoint already documents cross-contamination from shared-tree concurrency.
- 3.1 blocks 3.2–3.5 (run-store is the substrate). 2.2 blocks 2.3 (retirement follows migration). 1.2 blocks 1.5 (VR baseline after the purge, not before).
- Every change carries a spec delta per the repo's OpenSpec rule; the empty `impeccable-uiux-audit` / `uiux-remediation-wave-1` scaffolds are either filled (1.1) or deleted to keep the change list honest.

---

## 8. Risks & open questions

1. **Terminal-theme retirement is a taste decision.** The operator commissioned that aesthetic deliberately. Mitigation: present the Console density mode side-by-side in Storybook before deletion; keep the phosphor palette as an optional accent theme derived from the same ladder if attachment persists. *Operator sign-off required (R1).*
2. **Brand-slot default.** If UAR's standalone open-source identity should stay pure (no KnowMe anywhere), the default brand slot ships UAR assets only; KnowMe branding arrives via embed config. Confirm intent (R2).
3. **Local-first default model (CH-8)** depends on the host story (Tauri sidecar conversion, embedded provider contract in `uar-scoped-chat-control-plane`). The web build cannot offer "download local model" until WebLLM/WebGPU support lands; interim: one-action "configure provider" + honest messaging.
4. **PGlite FTS migration (CH-1)** may need a schema version bump shared with pglite-oxide on desktop — coordinate with `desktop-data-layer-pglite-oxide` so web/desktop schemas stay logically equivalent (KnowMe §8).
5. **Assistant UI version pinning.** `0.14.x` is pre-1.0; the block-renderer work (3.4) should pin and wrap its API surface so upgrades are deliberate.
6. **Scope discipline.** This plan intentionally excludes the A2A/gRPC, WASM skills, and provider-routing backend workstreams; UI changes that need new backend fields are flagged inline (CH-1 FTS, CO-2 replay) and must land as small, UI-motivated backend deltas, not backend redesigns.

---

## Appendix A — Measurement snapshot (2026-08-01)

| Metric | Value | Method |
|---|---|---|
| TS/TSX files in `frontend/src` | 288 | `find` |
| Border-idiom class usages | 622 | grep `border(-*)?` over `*.tsx` |
| `variant="outline"` usages | 50 | grep |
| Files with `shadow-*` | 9 | grep |
| `backdrop-blur`/gradient idioms | 6 | grep |
| Zustand stores with `create<` | 31 (+ hook-coupled modules) | grep |
| TanStack Query call sites | 0 (provider only) | grep `useQuery\|useMutation` |
| Largest file | `admin/pages/settings-page.tsx` — 3,336 lines | `wc -l` |
| Admin sections in production nav | 18 | `admin-shell.tsx` |
| Empty UI OpenSpec scaffolds | 2 (`impeccable-uiux-audit`, `uiux-remediation-wave-1`) | `ls` |

## Appendix B — Sources

- KnowMe UI/UX Standard: `/Users/gqadonis/Projects/know-me/know-me-system/docs/knowme-ui-ux-standard.md` (binding standard, 2026-07-17)
- UAR frontend architecture contract: `docs/frontend-architecture.md`; ADR-007
- UAR design artifacts: `docs/UI_DESIGN.md` (stale), `docs/admin-aesthetic-spec.md` (conflicting), `frontend/src/index.css`, `frontend/tailwind.config.ts`
- KBD waypoint: `.kbd-orchestrator/current-waypoint.json`; OpenSpec change inventory under `openspec/changes/`
- Comparative chat UIs: Open WebUI / LobeChat / LibreChat comparisons (local-llm.net 2026-04, agentlist.top 2026-06, elest.io 2025-12, slashdot/software 2026); Jan.ai, AnythingLLM (naileru.com 2026)
- Generative UI protocols: CopilotKit "Developer's Guide to Generative UI in 2026" (2026-01), CopilotKit/generative-ui repo (S), Microsoft Agent Framework AG-UI docs (S, 2026-07), A2UI guide (cnblogs, B, 2026-01)
- Observability consoles: Zenml Langfuse vs LangSmith (B, 2025-11), inference.net (2026-06), Langflow observability tour (2025-06)
