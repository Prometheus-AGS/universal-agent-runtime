# React implementation plan for the UAR operator shell

Status: **plan, no code changed yet**. Date: 2026-10-05.

This plan turns the HTML prototypes in this folder into React 19 code in `frontend/`, following the
repository's own rules: `docs/frontend-architecture.md`, `docs/adr/ADR-007-react-first-frontend.md`,
`.claude/rules/typescript.md`, the binding `docs/knowme-ui-ux-standard.md`, and the gate scripts
`scripts/check-frontend-boundaries.mjs`, `scripts/check-flat2-style.mjs` and
`scripts/check-settings-ui-decomposition.mjs`.

## 0. Decisions (recorded 2026-10-05)

| # | Question | Decision | Consequence |
|---|---|---|---|
| D1 | Body font: the UI/UX standard names Inter; the shipped frontend loads Geist | **Inter** | `tokens.css` `--font-sans/--font-body/--font-ui` become `"Inter Variable", "Inter", system-ui, sans-serif`; `@fontsource-variable/geist` is replaced by `@fontsource-variable/inter` in `main.tsx`. Space Grotesk (display) and JetBrains Mono (identifiers) are bundled through `@fontsource-variable/*` too, and the Google Fonts `<link>` tags are removed (offline and CSP). The design prototypes already use Inter. `--font-reading: Roboto` is out of scope and stays as it is |
| D2 | Where the component library lives | **New workspace package `frontend/packages/uar-ui`** (`@prometheus-ags/uar-ui`) | Presentational code cannot reach stores, services or the entity graph, by construction. Lint and boundary gates must be extended to `packages/**` in the same PR (§2.3) |
| D3 | `scripts/frontend-boundary-allowlist.txt` is cited by the architecture doc but missing | **Create it empty in the first PR** | Done (§12). `check-frontend-boundaries.mjs` reads it and fails on any entry missing from the merge base with `origin/main` (`boundary-allowlist-growth`), on stale entries and on malformed lines. The script's older inline exception sets stay where they are as recorded debt |

## 1. Rules applied

The repository asks for a short statement of the rules in force before any code:

- **Data direction.** Data flows Component → Hook → Store or entity domain → Service → API.
- **Components stay presentational.** They never call `fetch`, never import stores, services, transports or the entity graph, and never call Tauri's `invoke()`.
- **Hooks don't hide I/O.** They do not call `fetch` or import services.
- **Entities live in the graph.** Business records are read only through `@/platform/entities` domain hooks and written with one atomic graph action per fetched batch. Zustand holds only transient UI and workflow state, and never duplicates a graph-owned record.
- **Feature boundaries.** Features import each other only through `index.ts`, `api/` or `model/`.
- **Rendering.** Subscribe at the smallest rendered boundary, and never call a setter in the render body.
- **Composition.** Compound components over boolean props (Vercel Composition Patterns); Vercel React Best Practices for rendering and bundle size.
- **Flat 2.0 is enforced by lint.** No borders, rings, layout shadows, blur or gradients, and no `outline` variants.
- **Libraries.** Shadcn on Base UI for primitives, Assistant UI for the thread. TanStack Query is not used.
- **Accessibility.** WCAG 2.2 AA; status is never shown by colour alone.
- **File size.** At most 600 lines per production `.tsx`. Today that limit only covers `features/settings/ui`; this plan adopts it for all new UI.

## 2. Architecture: four tiers

| Tier | Location | Contents | May import |
|---|---|---|---|
| 1 Tokens | `src/shared/theme/tokens.css` | Existing tokens, reconciled with the prototype (§3) | — |
| 2 UI library | `frontend/packages/uar-ui` | Restyled primitives + composition patterns; props in, callbacks out | React, `@base-ui/react`, `lucide-react`, `cva`, `clsx`, `tailwind-merge`, shiki, TanStack Table/Virtual, `react-resizable-panels` |
| 3 Shell | `src/app/shell/` | Sidebar, inspector, command palette, settings layout, titlebar | tier 2, `@/platform/*` hooks, `nav-destinations.ts` |
| 4 Features | `src/features/<domain>/{api,model,ui,index.ts}` | Containers, hooks, graph actions, services | tiers 2–3, `@/platform/entities`, other features' public entries |

### 2.1 The prototype is a spec, not code to port

`assets/shell.js` and `assets/settings.js` are behaviour references: each block maps onto a hook or
a store slice (§5). CSS classes map onto library components (§4). Copy, states and interaction contracts
come from the HTML pages.

### 2.2 Package layout

```
frontend/packages/uar-ui/
  package.json            name @prometheus-ags/uar-ui, "sideEffects": ["*.css"], peer react@19
  src/
    primitives/           button, status-chip, switch, segmented-control, tabs, dropdown-menu,
                          command-palette, field, input, select, data-table, meter, code-block,
                          key-value-list, kbd, tooltip, sheet
    patterns/             master-detail, summary-strip, wizard, inline-confirm, rule-row,
                          empty-state, phase-strip, execution-step, approval-card,
                          board, score-bar
    index.ts              public entry; no deep imports from the app
  stories/                one story file per primitive and pattern
  tests/                  vitest + testing-library for behaviour (keyboard, aria)
```

Styling uses Tailwind classes against the app's tokens. The package ships no tokens of its own; the
app's `tokens.css` is the only source.

### 2.3 Gates to extend in the first PR

1. Remove `packages/uar-ui/**` from the ESLint ignore list. Apply `eslint-flat2-contract.js` and
   `unicorn/filename-case`; the other workspace packages stay ignored.
2. Teach `check-frontend-boundaries.mjs` a `library` class: files in `packages/uar-ui` may not import
   `@/` app paths, `zustand`, `@prometheus-ags/prometheus-entity-management`, `@tauri-apps/*` or any
   `services/`/`api/` path.
3. Create `scripts/frontend-boundary-allowlist.txt` empty (D3), and fail the gate when it gains an
   entry, keeps a stale one, or has a malformed line.
4. Add the package to `vitest` projects (`unit` and `storybook`) and to Storybook's story glob.

## 3. Tokens (tier 1)

`tokens.css` already carries the palette, phases, radius and motion. Changes:

- **Fonts per D1.**
- **Appearance hooks:** add `[data-density="compact"]` and `[data-motion="reduce"]` rule sets, and a
  `--thread-fs` custom property that defaults to 15px. These are driven by the shell store (§5) and
  mirror `assets/shell.css`.
- **High contrast:** keep the existing `.high-contrast` theme. It is the one theme that may draw strokes,
  which needs a documented exception in `eslint-flat2-contract.js` scoped to that selector.
- No new colours. The prototype's surface ladder maps 1:1 onto `--color-bg`, chrome (`--color-bg-2`),
  `--color-surface`, `--color-card` and `--color-card-hov`.

## 4. The component library (tier 2)

### 4.1 Primitives (restyled shadcn/Base UI with Flat 2.0 fills)

| Prototype | Library component | Built on | Notes |
|---|---|---|---|
| `.btn` primary · secondary · ghost · danger | `Button` | `components/ui/button` | One `primary` per view is a review rule, not a prop constraint |
| `.chip` ok · warn · err · live · accent | `StatusChip` | — | Dot plus text label always; `tone` prop; never colour alone |
| `.switch` | `Switch` | Base UI Switch | `aria-checked`, Space/Enter |
| `.choice` | `SegmentedControl` | ToggleGroup | Arrow-key roving focus; per-option `tone` for Auto · Ask · Deny |
| `.tabs`, `[data-dtab]` | `Tabs` | Base UI Tabs | Controlled by URL where it is route state |
| `.menu` | `DropdownMenu` | Base UI Menu | |
| `.cmdk` | `CommandPalette` | `components/ui/command` | Items come from the route table plus feature contributions |
| `.field`, `.input`, `.select` | `Field`, `Input`, `Select` | `components/ui/field` | Works with react-hook-form via `Controller` |
| `.table` | `DataTable` | TanStack Table + react-virtual | Zebra fills, no rules; sticky header; empty state slot |
| `.meter` | `Meter` | — | `role="meter"`, value + max + label |
| `.code` | `CodeBlock` | shiki | Copy button; read-only |
| `.kv` | `KeyValueList` | `<dl>` | |
| `kbd[data-kbd]` | `Kbd` | — | ⌘→Ctrl, ⌥→Alt, ⇧→Shift off macOS |

### 4.2 Composition patterns

| Pattern | Parts | Used by |
|---|---|---|
| `MasterDetail` | `.Root`, `.List`, `.Filter`, `.Group`, `.Row`, `.Detail`, `.Header`, `.Tabs` | Models & providers, Tools & MCP, Skills, Knowledge & memory, Governance, Desktop, Knowledge, Agents, A2UI registries, Projects |
| `SummaryStrip` | `.Root`, `.Tile` (label, value, detail, status) | every settings and workspace page |
| `Wizard` | `.Root`, `.Steps`, `.Step`, `.Footer`; each step owns a zod schema; Back/Continue/Finish | add provider, add MCP server, import skill, new agent, new knowledge base, upload, new presentation, new project, move chats |
| `InlineConfirm` | `.Trigger`, `.Body`, optional type-to-confirm | delete document, delete KB, remove server, delete agent, project lifecycle |
| `RuleRow` | label, description, control slot, `dimmed` | settings rows everywhere |
| `EmptyState` | icon, title, body, action slot | lists, search results, backend-gap fallbacks |
| `PhaseStrip`, `ExecutionStep` | phase colours from tokens | chat execution record, runs table and trace |
| `ApprovalCard` | request, scope, Allow once · Always · Deny | chat, runs inspector, governance approvals |
| `Board` | `.Column`, `.Card` | Projects overview |
| `ScoreBar` | value, threshold marker, label | skill matching test, retrieval test |

Every pattern gets a Storybook story at 320, 768, 1024 and 1440 px in dark, light and high contrast, plus
an `addon-a11y` pass. Composition rules: patterns accept children and slots, never a pile of booleans;
state that callers need lives in context inside `.Root`.

## 5. Shell (tier 3, `src/app/shell/`)

| Component / hook | Replaces | Behaviour (from `shell.js` / `settings.js`) |
|---|---|---|
| `AppSidebar` | `NavRail` + chat `LeftSidebar` (two 240px columns) | New chat, Search, Workspace (Projects · Knowledge · Agents · A2UI registries · Runs), Recents, Settings, account. Collapses to a 60px rail (⌘B); overlay below 760px |
| `SettingsLayout` + `SettingsNav` | `admin-page.tsx` `PAGE_MAP` | Same route table; Esc returns to chat |
| `Inspector` | none (only `--right-w` exists, unused) | Docked and resizable (360–760px) above 1180px via `react-resizable-panels`, `Sheet` below. Tabs come from a registry so features contribute tabs (Run, Files, Browser, Plan, add-flows) without the shell importing them |
| `CommandPalette` | existing `command-palette` | Fed by the route table plus registered actions |
| `useShellStore` (Zustand + `persist`) | `ui-store` (not persisted) | `sidebarCollapsed`, `inspector {open, tab, width}`, `theme`, `density`, `motion`, `threadFs` |
| `useShortcuts` | scattered handlers | ⌘K, ⌘B, ⌘\\, ⇧⌘D/E/B, ⌘, |
| `useAppearanceEffect` | `use-theme.ts` | Writes `data-theme`/class, `data-density`, `data-motion`, `--thread-fs` on `<html>`; follows `prefers-color-scheme` for system |
| `platform/desktop/` | inline `isTauri` in `Titlebar.tsx` | `useIsDesktop`, `useDesktopRuntime`, `useWindowControls`; `Titlebar.tsx` keeps its place |

`nav-destinations.ts` becomes the single route table for the sidebar, settings nav, command palette and
mobile tab bar.

## 6. Routes (URL holds selection)

```
/chat/:conversationId?            ?new=1 · ?agent=<id> · ?project=<id>
/projects/:projectId?/:tab?
/knowledge/:kbId?/:tab?           /agents/:agentId?/:tab?
/registries/:registryId?/:tab?    /runs?status=&agent=&q=        inspector: ?run=<id>&tab=trace
/settings/models/:providerId?/:tab?    /settings/tools/:serverId?/:tab?
/settings/skills/:skillId?/:tab?       /settings/knowledge-memory/:section?
/settings/governance/:section?         /settings/appearance     /settings/desktop/:section?
```

The string-keyed admin router is retired. That also fixes the orphaned `/admin/models` and
`/admin/credentials` screens; old `/admin/*` paths redirect.

## 7. Features (tier 4)

Each feature keeps the `features/providers` shape:
- **`api/`** holds thin typed `fetch` adapters.
- **`model/`** holds graph actions, narrow hooks and, only when a wizard needs one, a transient store.
- **`ui/`** holds a container per screen and one file per detail tab.

Containers compose tier-2 patterns and pass plain props down.

| Feature | Prototype | Graph entities (`@/platform/entities`) | Transient (Zustand or local state) |
|---|---|---|---|
| `providers`, `models`, `credentials` (exist) | `models-and-providers.html` | Provider, Model, credential state, routing roles | add-provider wizard draft, connection-test result |
| `tools` (exists) | `tools-and-mcp.html` | McpServer, Tool, approval tier | add-server wizard, health-check run |
| `skills` (exists) | `skills.html` | Skill, matching config, evolution config | matching-test prompt and result, import wizard |
| `knowledge` (exists) | `knowledge.html`, the KB part of `knowledge-and-memory.html` | KnowledgeBase, KnowledgeDocument | upload queue, search-test result, embedding-mismatch confirm |
| `memory` (exists) | the memory part of `knowledge-and-memory.html` | MemoryConfig, Memory | — |
| `agents`, `compiler` (exist) | `agents.html` | AgentArtifact, CompileReport, AgentInstance | compiler-session draft, stage progress |
| `presentations` → `registries` | `a2ui-registries.html` | Presentation, Component, CustomSchema | preview client profile |
| `runtime` → `runs` | `runs.html`, Run inspector tab | Run, RunEvent (PGlite), ToolAdmissionEvidence | expanded rows |
| `chat` (exists) | `chat-workspace.html` | Conversation, ConversationPolicy | composer draft |
| `conversation-target` (new) | composer target picker, route line, new-chat chooser | ConversationPolicy (`agent_id`, `routing`) | — |
| `governance` (new UI) | `governance.html` | GovernanceStatus, Policy, PendingApproval, Budget | request-tester inputs |
| `appearance` (new) | `appearance.html` | — (per device) | `useShellStore` |
| `desktop` (new) | `desktop.html` | — (Tauri side, via `platform/desktop`) | restart and diagnostics flows |
| `projects` (new, after backend) | `projects.html` | Project, ProjectMember, ProjectResource, ProjectTask, ProjectEvent | wizard, move-chats selection |

**Worked example — Models & providers:**

```
features/providers/
  api/providers-api.ts          listProviders · getProvider · saveProvider · testConnection(config) · syncModels(id)
  model/providers-graph.ts      ingestProviders(list)  ← one atomic graph action
  model/use-provider.ts         useProvider(id), useProviderList(filter)
  model/use-connection-test.ts  calls a graph/domain action; exposes { status, result, run }
  model/add-provider-store.ts   wizard draft only (preset, base URL, key ref, selected models)
  ui/models-and-providers-page.tsx   MasterDetail container, reads :providerId/:tab from the route
  ui/provider-connection-tab.tsx · provider-models-tab.tsx · provider-routing-tab.tsx
  ui/roles-strip.tsx            SummaryStrip with default chat / fast / embedding roles
  ui/add-provider-wizard.tsx    Wizard in the inspector
  index.ts
```

**Chat specifics:**
- The thread stays on Assistant UI.
- The composer's agent, model and permission controls become composer slots: `ConversationTargetPicker`, `ModelPicker` and `PermissionModePicker`.
- `ExecutionRecord` renders from the events `platform/agui` already normalises.
- `RouteLine` renders the orchestrator's routing decision only when the run stream carries that event (backend delta in `ui-research-findings.md` §5).

## 8. Backend gaps do not ship as notes

The prototypes show **Backend gap** notes; the product doesn't. A `useRuntimeCapabilities()` hook reads
the discovery endpoint, or a small capabilities endpoint added for the purpose. Each surface that depends
on a missing endpoint then hides, or shows `EmptyState` with an honest reason. Examples: per-tool approval
tiers, KB update/delete/re-embed, routing to user agents and the routing switch, design systems, and
Projects. UI work can merge ahead of the backend without dead controls.

## 9. Testing and gates

| Tier | When | Command / scope |
|---|---|---|
| T0 | every edit | `pnpm -C frontend typecheck`, `pnpm -C frontend lint` (includes Flat 2.0 contract and filename case) |
| T0 | every edit | `pnpm frontend:boundaries`, `pnpm frontend:style-gate` |
| T1 | per feature | Vitest for hooks, graph actions, stores and wizard schemas, colocated `*.test.ts(x)` |
| T2 | per pattern | Storybook interaction + a11y tests for every `uar-ui` component |
| T3 | per flow | Playwright: add provider, add MCP server, compile an agent, upload documents, move chats, routing on/off |
| T3 | per flow | Visual checks at 320/768/1024/1440 in dark, light and high contrast |
| T3 | per flow | `budget:bundle`, `budget:performance` |

Coverage stays at or above the 60% thresholds (ADR-0003). Each PR also gets the required Impeccable
critique by two isolated subagents plus a fresh-context adversarial review. Tests run locally; CI stays
deployment-only, per `AGENTS.md`.

## 10. Delivery order (one reviewable PR per step)

1. **Foundation:**
   - D1 font switch.
   - D3 allowlist file and gate.
   - `packages/uar-ui` scaffold with lint and boundary coverage.
   - Primitives with stories.
2. **Patterns:** `MasterDetail`, `SummaryStrip`, `Wizard`, `InlineConfirm`, `RuleRow`, `EmptyState`.
3. **Shell:** `AppSidebar`, `Inspector`, `SettingsLayout`, `useShellStore`, URL routes, admin-router retirement.
4. **Models & providers.** The highest-pain screen; the target state is that liter-llm setup is intuitive.
5. Tools & MCP, Skills.
6. **Chat:** conversation target, routing line, execution record, approval card.
7. Agents, Knowledge, Runs, A2UI registries.
8. Governance, Appearance, Desktop.
9. **Projects**, after the migration and API in `project-management-design.md` land.

## 11. Open items

- **High-contrast exception.** It must be written into `eslint-flat2-contract.js` so the one theme that
  draws strokes doesn't need per-file allowlist entries.
- **Capabilities endpoint.** Its shape for §8 needs a backend owner.
- **Existing Flat 2.0 violations.** The providers and models pages carry violations listed in
  `scripts/frontend-flat2-style-allowlist.txt`. Step 4 should delete those entries rather than add more.

## 12. Progress

### Step 1 — foundation (branch `feat/uar-ui-foundation`, 2026-10-05, uncommitted)

| Item | Change | Evidence |
|---|---|---|
| D1 Inter | `@fontsource-variable/geist` replaced by `inter`, `space-grotesk` and `jetbrains-mono` (frontend and root lockfiles); `main.tsx` and `.storybook/preview.tsx` import them; `tokens.css` font tokens point at the bundled families; the `.eyebrow` rule uses `var(--font-mono)`; Google Fonts links removed from `index.html` | `pnpm build` passes and emits the three font families into `static/assets`; `static/index.html` was regenerated without the CDN links |
| D3 allowlist | New `scripts/frontend-boundary-allowlist.txt` (no entries); gate reads it with growth, stale and malformed checks; `--allowlist` / `--allowlist-base` flags for fixtures | 3 new negative fixtures; reverse checks: an entry already on the base is honoured, and the same violation without an allowlist is reported |
| D2 library | New `frontend/packages/uar-ui` (`@prometheus-ags/uar-ui`, source exports, peer React 19, `clsx` + `tailwind-merge`); first primitive `StatusChip` with 4 tests and 2 stories; added as a `workspace:*` dependency of the frontend | `tsc -p packages/uar-ui` (now part of `pnpm typecheck`); `vitest --project uar-ui` 4/4; Storybook project 2/2 including the fail-closed a11y addon |
| Gate coverage | ESLint and the Flat 2.0 baseline config ignore `packages/*` except `packages/uar-ui`; `check-flat2-style.mjs` scans `packages/uar-ui/src`; boundary gate adds `library-direct-fetch`, `library-app-import` (including relative escapes), `library-state-import`, `library-entity-import`, `library-tauri-import`, `library-service-import`; Storybook and Vitest include the package; `tokens.css` adds `@source` for it | A planted probe file was rejected by lint (border/shadow, filename case), by the boundary gate (app import, fetch) and scanned by the style gate, then removed; negative boundary fixtures now 29 (was 19) |

Pre-existing state on `main`, not changed by this step: `pnpm frontend:boundaries` reports 19 production
violations (14 of them per-row graph writes whose line numbers in `legacyPerRowGraphWriteLocations` have drifted) and
`pnpm frontend:style-gate` reports 143 differences against its allowlist. Typecheck and lint were green
before and after. The full unit suite was not re-run. Tests must run with `NODE_ENV=test`; a shell that
exports `NODE_ENV=production` loads React's production build, which has no `act`.
