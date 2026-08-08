# UAR Frontend Migration Plan

**From** React 19 + Vite 5 + Tailwind 3 + TanStack Query + mixed `admin/` · `features/` · `stores/` layout
**To** React 19 + Vite 8 + Tailwind 4 + `@prometheus-ags/prometheus-entity-management@next` + strict feature-based clean architecture, rendering the TravisJames.ai / Run-Trace design under **Flat 2.0**.

Design reference: `docs/UAR Console.dc.html`, `docs/UAR Mood Board.dc.html`, `docs/UAR Shell & Mobile.dc.html`.
Brand assets: `docs/logo/`.

Binding design authority, in precedence order:

1. `docs/knowme-ui-ux-standard.md` (KnowMe repo) — **Flat 2.0**, ember/cyan, ported into `frontend/src/index.css`
2. `docs/uar-uiux-assessment-and-implementation-plan-2026-08-01.md` — the compliance census and the purge rules
3. This document — the target architecture and the render contract
4. **Retired:** `docs/UI_DESIGN.md` (stale Material-3 purple), `docs/admin-aesthetic-spec.md` (Terminal/CRT; its operator-instrument intent survives as a Flat 2.0 high-density console mode)

> **Parity applied.** This revision was written against the actual KnowMe
> implementation, read at `know-me-system/`:
> `docs/knowme-ui-ux-standard.md` (§3 Flat 2.0, §4 tokens, §6 ownership, §7 chat),
> `desktop/src/index.css` (token names and values),
> `desktop/src/components/assistant-ui/markdown-text.tsx` (markdown renderer, MermaidBlock),
> `desktop/src/features/chat/components/ArtifactKindBody.tsx` (artifact kind dispatch),
> `desktop/src/features/chat/components/RichDataRenderers.tsx` (Assistant UI data parts),
> `packages/gen-ui-react/src/content-block.ts` + `ContentBlockView.tsx` (the ContentBlock union),
> and from UAR: `frontend/src/protocols/agui-adapter.ts`,
> `frontend/src/stores/chat-stream-store.ts`, `frontend/src/types/chat-content.ts`,
> `docs/full-implementation/A2UI + AG-UI.md`.
>
> Where the two diverge, the rule is: **KnowMe owns presentation and token names,
> UAR owns the runtime event vocabulary.** The chunk catalog in §8 is the union of
> both, mapped to the shared `ContentBlock` protocol.

---

## 0. Non-negotiables

| Constraint | Rule |
|---|---|
| File naming | **kebab-case for every generated file and directory**. `run-trace-bar.tsx`, `use-run-stream.ts`, `agui-event-schema.ts`. No PascalCase filenames; one barrel per feature. |
| Design system | **Flat 2.0.** No borders, no dividers, no drop shadows, no backdrop-blur, no gradients, no `variant="outline"`. Separation is by surface ladder and spacing only. Mechanically enforced (§4.3). |
| Entity state | `@prometheus-ags/prometheus-entity-management@next`. **TanStack Query removed entirely** — the exemption in `frontend/README.md` is retired. |
| Local state | Zustand + Immer, one slice per feature, colocated. |
| Persistence | PGlite. Conversations, runs, and AG-UI events are client-owned. |
| Transport | AG-UI SSE for all agent traffic. |
| UI | assistant-ui + shadcn/ui restyled for Flat 2.0 + Lucide. |
| Markdown | **Every bubble is markdown** — user, assistant, tool result, citation snippet, A2UI text. One renderer, one plugin chain (§7). |
| Theming | TravisJames.ai tokens, light + dark + high-contrast. |
| Responsive | One component tree; the shell re-stacks. No separate mobile app. |

---

## 1. Target architecture

### 1.1 Layer contract

```
app/          composition root — router, providers, shell. Imports features. Nothing imports app.
features/     vertical slices. Each owns ui / model / api / db. Features import shared, never each other's internals.
shared/       cross-cutting: ui kit, markdown renderer, theme, hooks, types. Imports only platform.
platform/     infrastructure adapters: pglite, entity-management, agui transport, sync. No React, no JSX.
```

Dependency direction is strictly downward: `app → features → shared → platform`. Lint-enforced (§6.3).

### 1.2 Directory tree

```
frontend/src/
├── main.tsx
├── app/
│   ├── app.tsx
│   ├── router.tsx
│   ├── providers/
│   │   ├── entity-provider.tsx
│   │   ├── pglite-provider.tsx
│   │   ├── theme-provider.tsx
│   │   └── assistant-runtime-provider.tsx
│   └── shell/
│       ├── app-shell.tsx
│       ├── nav-rail.tsx
│       ├── nav-destinations.ts
│       ├── mobile-tab-bar.tsx
│       ├── mobile-sheet-host.tsx
│       ├── command-palette.tsx
│       └── breadcrumb-header.tsx
├── features/
│   ├── chat/
│   │   ├── ui/
│   │   │   ├── thread-view.tsx
│   │   │   ├── thread-list.tsx
│   │   │   ├── run-trace-bar.tsx
│   │   │   ├── run-trace-timeline.tsx
│   │   │   ├── run-inspector.tsx
│   │   │   ├── composer.tsx
│   │   │   ├── composer-toolbar.tsx
│   │   │   ├── composer-budget-lane.tsx
│   │   │   └── chunks/              ← one file per chunk type, see §8
│   │   ├── model/
│   │   │   ├── chat-store.ts
│   │   │   ├── run-store.ts
│   │   │   ├── session-config-store.ts
│   │   │   ├── use-run-stream.ts
│   │   │   └── run-phase.ts
│   │   ├── api/agui-run-client.ts
│   │   └── db/{thread-repository.ts,message-repository.ts,run-repository.ts}
│   ├── knowledge/    bases · documents · ingest · retrieval-test · chunking-lab
│   ├── agents/       registry · editor · compiler · registration
│   ├── providers/    providers · model catalog · credentials · connection tests
│   ├── mcp/          servers · health · per-tool approval
│   ├── tools/        built-in tool registry
│   ├── skills/       library · selection strategy · activation replay
│   ├── context/      strategy · simulation · memory
│   ├── web-access/   backends · quotas · domain fence
│   ├── a2ui/         registry · validation · surface renderer
│   └── settings/     config deck · per-agent override matrix
├── shared/
│   ├── ui/           shadcn primitives, kebab-cased, Flat 2.0 variants
│   ├── markdown/     ← §7, used by every bubble
│   │   ├── markdown-bubble.tsx
│   │   ├── markdown-components.tsx
│   │   ├── plugins/
│   │   │   ├── remark-chain.ts
│   │   │   ├── rehype-chain.ts
│   │   │   └── sanitize-schema.ts
│   │   └── blocks/
│   │       ├── code-block.tsx
│   │       ├── mermaid-block.tsx
│   │       ├── svg-block.tsx
│   │       ├── image-block.tsx
│   │       ├── video-block.tsx
│   │       ├── chart-block.tsx
│   │       ├── flow-block.tsx
│   │       ├── math-block.tsx
│   │       └── block-chrome.tsx      copy · share · expand · download
│   ├── theme/{tokens.css,theme-store.ts,use-theme.ts}
│   ├── hooks/  lib/  types/
└── platform/
    ├── pglite/{client.ts,migrate.ts,migrations/*.sql}
    ├── entities/{entity-client.ts,collections.ts,sync-bridge.ts}
    ├── agui/{agui-event-schema.ts,agui-sse-client.ts,agui-normalizer.ts}
    └── telemetry/
```

---

## 2. Phases

| # | Phase | Exit criteria |
|---|---|---|
| 1 | Toolchain — Vite 8, React 19, kebab-case sweep | build green, zero PascalCase filenames |
| 2 | Tailwind 4 + Flat 2.0 tokens + theme switch + purge gate | all three themes render, style lint green, `tailwind.config.ts` deleted |
| 3 | Platform — PGlite schema, entity-management, AG-UI client | conversations survive reload, offline read works |
| 4 | TanStack Query removal | zero `@tanstack/react-query` imports |
| 5 | Shell, navigation, brand | desktop rail and mobile tabs both usable |
| 6 | Markdown pipeline | mermaid, svg, image, video, chart, math render in every bubble type |
| 7 | Chat chunks + run trace + inspector | every chunk type in §8 renders; all AG-UI events land in the trace lane |
| 8 | A2UI surfaces | registry validates and renders `decision-card`; input surfaces round-trip |
| 9 | Configuration surfaces | every mood-board panel exists |
| 10 | Cleanup, budgets, a11y | `admin/` deleted, budgets met |

---

## 3. Phase 1 — toolchain and naming

### 3.1 Dependencies

```jsonc
{
  "dependencies": {
    "react": "^19.2.0",
    "react-dom": "^19.2.0",
    "react-router": "^7",
    "@assistant-ui/react": "latest",
    "@assistant-ui/react-markdown": "latest",
    "@prometheus-ags/prometheus-entity-management": "next",
    "@prometheus-ags/entity-graph-core": "next",
    "@prometheus-ags/a2ui-react": "next",
    "@prometheus-ags/gen-ui-react": "workspace:*",
    "@electric-sql/pglite": "^0.3",
    "dompurify": "^3",
    "zustand": "^5",
    "immer": "^10",
    "zod": "^4",
    "lucide-react": "^0.575",
    "class-variance-authority": "^0.7",
    "tailwind-merge": "^3",

    "remark-gfm": "^4",
    "remark-math": "^6",
    "remark-breaks": "^4",
    "rehype-katex": "^7",
    "rehype-raw": "^7",
    "rehype-sanitize": "^6",
    "mermaid": "^11",
    "shiki": "^3",
    "katex": "^0.16"
  },
  "devDependencies": {
    "vite": "^8",
    "@vitejs/plugin-react": "^5",
    "tailwindcss": "^4",
    "@tailwindcss/vite": "^4",
    "typescript": "^5.7",
    "vitest": "^3",
    "@playwright/test": "^1.5x"
  }
}
```

Removed: `@tanstack/react-query`, `@tanstack/react-query-devtools`, `postcss.config.js`, `autoprefixer`, `tailwind.config.ts`, `highlight.js` (superseded by Shiki — see §7.3).

### 3.2 Vite 8

```ts
// vite.config.ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwind from "@tailwindcss/vite";
import path from "node:path";

export default defineConfig({
  plugins: [react(), tailwind()],
  resolve: {
    alias: {
      "@app": path.resolve(__dirname, "src/app"),
      "@features": path.resolve(__dirname, "src/features"),
      "@shared": path.resolve(__dirname, "src/shared"),
      "@platform": path.resolve(__dirname, "src/platform"),
    },
  },
  build: {
    outDir: "../static",
    emptyOutDir: true,
    target: "es2022",
    rollupOptions: {
      output: {
        manualChunks: {
          "vendor-react": ["react", "react-dom", "react-router"],
          "vendor-pglite": ["@electric-sql/pglite"],
          "vendor-assistant": ["@assistant-ui/react"],
          "vendor-mermaid": ["mermaid"],
          "vendor-shiki": ["shiki"],
        },
      },
    },
  },
  optimizeDeps: { exclude: ["@electric-sql/pglite"] },
  worker: { format: "es" },
  server: {
    port: 8080,
    proxy: { "/api": { target: process.env.UAR_BACKEND_URL ?? "http://127.0.0.1:1906", changeOrigin: true } },
  },
});
```

Mermaid and Shiki are **lazy** (`import()` on first use) — they must never enter the initial chunk.

### 3.3 Kebab-case sweep

```
AppShell.tsx              → app/shell/app-shell.tsx
ThemeToggle.tsx           → shared/ui/theme-toggle.tsx
KnowMeLogo.tsx            → shared/ui/uar-logo.tsx
EnhancedMarkdownText.tsx  → shared/markdown/markdown-bubble.tsx
EnhancedThread.tsx        → features/chat/ui/thread-view.tsx
McpHealthPage.tsx         → features/mcp/ui/mcp-health-page.tsx
A2uiTestingPage.tsx       → features/a2ui/ui/a2ui-testing-page.tsx
```

```jsonc
// eslint.config.js
{ "rules": { "unicorn/filename-case": ["error", { "case": "kebabCase" }] } }
```

macOS is case-insensitive: rename via `git mv old.tsx tmp && git mv tmp new.tsx`.

---

## 4. Phase 2 — Tailwind 4, Flat 2.0, theming

### 4.1 What Flat 2.0 means here

From the KnowMe standard as ported into `index.css` and audited in the 2026-08-01 assessment:

**Prohibited, everywhere, no exceptions outside allowlisted primitive internals:**

- `border`, `border-*`, `divide-*`, `ring-1`, `outline` class idioms as *visual separation*
- `shadow-*` for layout elevation
- `backdrop-blur`, `bg-gradient-*`, decorative gradients
- `variant="outline"` on buttons and inputs
- all-monospace type systems; visible hairline rules between regions

Per the standard's §3.3, this also bans: card outlines even when transparent-until-hover;
bevels, glassmorphism, glossy controls; default Material `Divider` / `OutlineInputBorder`;
any Shadcn default accepted without stripping its border styling; **and status communicated
by color alone**.

**Required instead:**

- **Separation by surface ladder.** Four steps, each a fill: `bg` → `bg-2` → `surface` → `card`. Adjacent regions differ by one step. Never by a line.
- **Dividers are spacing.** A protocol divider, a section break, a message boundary — vertical space or a surface transition, **never an `<hr>`**. (The current `gen-ui-react` `ContentBlockView` renders `case "divider": <hr>`; that is a Flat 2.0 violation and must become a spacer `<div role="separator">`, matching what `markdown-text.tsx` already does for markdown `hr`.)
- **Modal depth is a scrim**, not a shadow: dim the canvas, raise the surface one step.
- **Separation by spacing.** 4px baseline; a region change is 12–24px of gap, not a rule.
- **State by fill.** Hover raises one step or tints ember-soft; selected is ember-soft with ember text; pressed darkens the fill. Never a border change.
- **Focus is the one exception**: a 3px ember focus ring is an accessibility requirement and is allowed.
- **Type does the work monospace used to do**: JetBrains Mono for machine facts (ids, latencies, token counts, `// eyebrows`), Inter for UI, Space Grotesk for names, Roboto for prose.

The design in `docs/UAR Console.dc.html` and the mood board is already drawn this way — every panel separates by fill, not by line. The one place the earlier draft of this plan introduced a `--color-line` hairline was wrong and has been removed.

### 4.2 Tokens

Tailwind 4 is CSS-first: delete `tailwind.config.ts` and `postcss.config.js`.

```css
/* src/shared/theme/tokens.css */
@import "tailwindcss";

@theme {
  --font-display: "Space Grotesk", system-ui, sans-serif;
  --font-sans:    "Inter", system-ui, sans-serif;
  --font-body:    "Roboto", system-ui, sans-serif;
  --font-mono:    "JetBrains Mono", ui-monospace, monospace;

  --radius-sm: 6px; --radius-md: 8px; --radius-lg: 12px; --radius-xl: 16px;

  --ease-standard: cubic-bezier(0.2, 0, 0, 1);
  --ease-out:      cubic-bezier(0.16, 1, 0.3, 1);
  --duration-fast: 120ms; --duration-base: 200ms; --duration-slow: 320ms;

  /* run-phase palette — the trace bar depends on these being stable */
  --color-phase-context:   #A7B0BC;
  --color-phase-skill:     #F59E0B;
  --color-phase-memory:    #8B5CF6;
  --color-phase-retrieval: #60A5FA;
  --color-phase-reasoning: #2563EB;
  --color-phase-tool:      #22C55E;
  --color-phase-generate:  #FF6A3D;
}

/* Token NAMES and VALUES are KnowMe's, copied from know-me-system/desktop/src/index.css
   so React web, Tauri desktop, and Flutter stay on one versioned source.
   Do not rename these to shadcn's background/foreground vocabulary. */
:root, .dark {
  --color-bg:        #0B0F14;   /* 1 canvas  */
  --color-bg-2:      #111620;   /* 2 chrome — titlebar, rail, bottom nav */
  --color-surface:   #161D29;   /* 3 surface — panels, composer, menus */
  --color-card:      #1C2535;   /* 4 raised — selected, expanded, modal, streaming */
  --color-card-hov:  #202B40;   /* hover */
  --color-muted:     #253044;   /* inputs, secondary controls, skeletons */

  --color-fg:        #E8EDF3;
  --color-fg-sub:    #A7B0BC;
  --color-fg-faint:  #6B7280;

  --color-ember:     #FF6A3D;   /* brand, primary action, active destination */
  --color-ember-fg:  #0B0F14;
  --color-ember-tint:#2A1B17;   /* selection fill */
  --color-cyan:      #00C2DC;   /* reasoning, citations, streaming, user-owned remote AI */
  --color-green:     #22C55E;
  --color-amber:     #F59E0B;
  --color-red:       #EF4444;

  --border: transparent;        /* legacy token resolves to nothing, per §4.1 of the standard */
  --color-focus-ring: rgb(255 106 61 / 0.45);
}

.light {
  --color-bg:        #F7F7F8;
  --color-bg-2:      #FFFFFF;
  --color-surface:   #FAFBFC;
  --color-card:      #FFFFFF;
  --color-card-hov:  #F2F4F7;
  --color-muted:     #F2F4F7;

  --color-fg:        #0B0F14;
  --color-fg-sub:    #4B5563;
  --color-fg-faint:  #6B7280;

  --color-ember:     #E04E28;
  --color-ember-fg:  #FFFFFF;
  --color-ember-tint:#F7DCD3;
  --color-cyan:      #0891B2;
  --color-green:     #16A34A;
  --color-amber:     #D97706;
  --color-red:       #DC2626;

  --color-phase-context:   #6B7280;
  --color-phase-retrieval: #2563EB;
  --color-phase-reasoning: #1D4ED8;
  --color-phase-tool:      #16A34A;
  --color-phase-generate:  #E04E28;
}

.high-contrast { /* same variable names, WCAG 2.2 AA+ ladder — accessibility, not a skin */ }

@layer base {
  html { color-scheme: dark; }
  html.light { color-scheme: light; }
  * { border-color: transparent; }              /* belt and braces */
  body {
    @apply bg-[color:var(--color-bg)] text-[color:var(--color-fg)] font-sans antialiased;
    transition: background-color var(--duration-base) var(--ease-standard),
                color var(--duration-base) var(--ease-standard);
  }
  :focus-visible { outline: none; box-shadow: 0 0 0 3px var(--color-focus-ring); }
  @media (prefers-reduced-motion: reduce) {
    *, *::before, *::after { animation-duration: 1ms !important; transition-duration: 1ms !important; }
  }
}

@utility eyebrow {
  font-family: var(--font-mono);
  font-size: 10px; font-weight: 600;
  letter-spacing: 0.15em; text-transform: uppercase;
  color: var(--color-ember);
}
```

**Cyan is a required role, not decoration.** Per §4.1 of the standard it means
reasoning, citations, streaming, and user-owned remote AI — exactly the thinking
chunk, the citation chunk, and the remote-lane badge. Ember stays brand/action.
Green/amber/red stay status. The trace-lane phase palette sits alongside these and
borrows cyan for reasoning in the KnowMe-aligned build.

The old `hsl(var(--token))` channel indirection is gone; tokens are complete colors.
Find-and-replace `hsl(var(--x))` → `var(--color-x)`.

**Do not** keep the `.aui-root * { box-shadow: none !important }` sledgehammer — with
the lint gate in place there is nothing to suppress, and the `!important` blocks the
focus ring.

### 4.3 The purge, made permanent

The 2026-08-01 census: 622 border idioms, 50 `variant="outline"`, 9 files with shadows,
6 with blur/gradient. Codemod them, then gate:

```js
// eslint.config.js
"no-restricted-syntax": ["error",
  { selector: "Literal[value=/\\b(border|border-[a-z0-9/-]+|divide-[a-z0-9/-]+|ring-1|shadow-(sm|md|lg|xl|2xl)|backdrop-blur[a-z-]*|bg-gradient-[a-z-]+)\\b/]",
    message: "Flat 2.0: separate by surface fill and spacing, not lines/shadows/blur." },
  { selector: "JSXAttribute[name.name='variant'][value.value='outline']",
    message: "Flat 2.0: no outline variants — use the muted filled variant." },
],
```

Allowlist: `shared/ui/**` primitive internals that need a hairline for a native
control affordance, annotated with `// flat2-allow: <reason>`.

Restyle `shared/ui/button.tsx` so `outline` becomes a filled `muted` variant, or
delete the variant and codemod call sites. Add a Storybook visual-regression gate
so the purge cannot regress.

### 4.4 Theme store

```ts
// src/shared/theme/theme-store.ts
import { create } from "zustand";
import { persist } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";

type Mode = "dark" | "light" | "high-contrast" | "system";

export const useThemeStore = create<{ mode: Mode; setMode: (m: Mode) => void }>()(
  persist(immer((set) => ({ mode: "dark", setMode: (mode) => set((s) => { s.mode = mode; }) })),
    { name: "uar-theme" }),
);
```

Mirror the class into a boot script in `index.html` so there is no first-paint flash.

---

## 5. Phase 3 — platform layer

### 5.1 PGlite schema

Conversations are **owned by the client**; the runtime is a sync peer, not the
source of truth for history.

```sql
-- 001-conversations.sql
create table if not exists thread (
  id text primary key,
  title text not null default 'New thread',
  agent_id text,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  archived_at timestamptz,
  session_config jsonb not null default '{}'::jsonb
);

create table if not exists message (
  id text primary key,
  thread_id text not null references thread(id) on delete cascade,
  role text not null check (role in ('user','assistant','system')),
  seq integer not null,
  created_at timestamptz not null default now(),
  chunks jsonb not null default '[]'::jsonb,   -- ordered chunk array, §8
  markdown text,                                -- raw source for copy/export
  usage jsonb, model text, agent_id text,
  status text not null default 'complete'
);
create index if not exists message_thread_seq on message (thread_id, seq);
```

```sql
-- 002-runs-and-events.sql
create table if not exists run (
  id text primary key,
  thread_id text not null references thread(id) on delete cascade,
  message_id text references message(id) on delete cascade,
  status text not null,                          -- running | finished | error | cancelled
  started_at timestamptz not null default now(),
  finished_at timestamptz,
  model text, usage jsonb, cost_usd numeric(12,6),
  phase_timings jsonb not null default '{}'::jsonb
);

create table if not exists run_event (
  run_id text not null references run(id) on delete cascade,
  seq integer not null,
  event_id text not null,
  type text not null,        -- raw AG-UI type
  kind text not null,        -- normalized chunk kind, §8
  at timestamptz not null default now(),
  payload jsonb not null,
  primary key (run_id, seq)
);
create index if not exists run_event_kind on run_event (run_id, kind);
```

```sql
-- 003-outbox.sql
create table if not exists outbox (
  id bigserial primary key,
  entity text not null, op text not null, payload jsonb not null,
  created_at timestamptz not null default now(),
  attempts integer not null default 0, last_error text
);
```

```ts
// platform/pglite/client.ts
import { PGlite } from "@electric-sql/pglite";
import { live } from "@electric-sql/pglite/live";

let instance: PGlite | undefined;
export async function getDb() {
  instance ??= await PGlite.create({ dataDir: "idb://uar", extensions: { live }, relaxedDurability: true });
  return instance;
}
```

Use the `live` extension for thread and message lists so the UI renders from the
database, not a parallel cache.

**Event write policy.** Persist a `run_event` row for every event **except**
`TEXT_MESSAGE_CONTENT` and `REASONING_MESSAGE_CONTENT` deltas, which are coalesced in
memory and written once at `*_END`. Without this a long run writes tens of thousands
of rows and the trace lane stalls.

### 5.2 Entity management

Server-owned entities move to `@prometheus-ags/prometheus-entity-management@next`
(3.0 line) with `@prometheus-ags/entity-graph-core` as the application-owned core
peer. **The real API is a normalized entity graph, not a collection/query cache** —
the earlier draft of this plan sketched a `defineCollection` API that does not exist.
Each `(type, id)` is one canonical node in a Zustand graph; lists store ordered **IDs**
and join rows at render time, so saving an agent in the editor updates every list row,
badge, and matrix cell with no invalidation keys.

```ts
// platform/entities/entity-hooks.ts — the only file that imports the package
import {
  useEntity, useEntityList, useEntityView, useEntityCRUD,
  registerEntityFromSql, registerEntityJsonSchema,
  createGraphAction, createGraphTransaction,
} from "@prometheus-ags/prometheus-entity-management";

export function useAgent(id: string) {
  return useEntity<AgentDto, Agent>({
    type: "Agent",
    id,
    fetch: (agentId) => api.get(`/agents/${agentId}`),
    normalize: (raw) => toAgent(raw),
  });
}

export function useAgents() {
  return useEntityList<AgentDto, Agent>({
    type: "Agent",
    queryKey: "agents:all",
    fetch: () => api.get("/agents"),
    normalize: (raw) => toAgent(raw),
  });
}
```

Entity types: `Agent`, `Provider`, `Model`, `Credential`, `Skill`, `Tool`,
`McpServer`, `KnowledgeBase`, `Document`, `A2uiComponent`, `Setting`.

Writes are graph actions with rollback, not `useMutation` plus invalidation:

```ts
const saveAgent = createGraphAction({
  type: "Agent",
  apply: (draft, next) => Object.assign(draft, next),      // optimistic
  commit: (next) => api.put(`/agents/${next.id}`, next),   // rolls back on throw
});
```

**Use the package's local-first runtime; do not hand-roll one.** Three v1.3 APIs
replace machinery the earlier draft specified by hand:

| Need | Package API | Replaces |
|---|---|---|
| Persist the graph in PGlite | `createPGlitePersistenceAdapter(pglite)` | a bespoke snapshot table |
| Offline write replay with backoff | `startLocalFirstGraph({ retryPolicy, poisonHandler })` | **the hand-written `outbox` table in §5.1** — delete migration `003` unless a runtime-specific replay ordering is genuinely required |
| Optimistic write + rollback | `createGraphAction` / `createGraphTransaction` | ad-hoc optimistic state in Zustand |

Two more that pay off directly here:

- `registerEntityFromSql({ entityType, createTableSql })` generates and registers a
  JSON Schema straight from a Postgres `CREATE TABLE`, so the §5.1 migrations become
  the schema source with no duplicated TypeScript types.
- `registerEntityJsonSchema` + `useSchemaEntityFields` drive schema-rendered forms and
  include **markdown-aware schema fields** — use them for the agent system prompt,
  skill descriptions, and KB document fields rather than bespoke editors.

Ergonomics fixed upstream in v1.3.1–v1.3.2 that bite exactly the screens in §10:
`useEntityList` returns a `useMemo`-stabilised shape — do not re-wrap it per render or
React 19's `useSyncExternalStore` loops; `isError` exists on list and view results; a
terminal error no longer triggers a refetch storm, so a 404 from a not-yet-created
table renders an empty state instead of a perpetual spinner.

> **Release status.** PEM 3.0 is in progress and not on npm `latest`. The React 19 +
> Vite 8 showcase is the implemented reference; Next.js, agentic A2UI, Flutter, and
> Tauri showcases are planned. Pin an exact `next` version, keep all usage behind
> `platform/entities/`, and check `release/vite-react19-example.md` in that repo
> before relying on a capability.

| Data | Owner |
|---|---|
| Threads, messages, content blocks, citations, attachments, drafts, runs, run events | **PGlite** in IndexedDB, preferably behind a **shared worker** for safe multi-tab ownership |
| Agents, providers, models, skills, tools, MCP, KB metadata, A2UI schemas, settings | **entity-management** |
| Ephemeral UI (open sheet, selected trace node, composer draft, test results) | **Zustand** |

All package usage sits behind `platform/entities/` — no feature imports it directly.

Two boundary rules from §6.2–6.3 of the standard, both of which the current UAR code
violates in places:

1. **Visual components call hooks** — never a store, a database client, or a transport
   directly. `Component → feature hook → Zustand/PEM → transport → database`.
2. **Presence is not satisfaction.** Installing Assistant UI without mounting its
   primitives at the real interaction boundary, or keeping conversations in an
   in-memory Zustand array while PGlite merely sits in `package.json`, does not meet
   the standard. Durable records must be written **incrementally during the stream**
   and restored after a real refresh.

### 5.3 AG-UI transport

Port `protocols/agui-adapter.ts` to `platform/agui/`, preserving its semantics and
widening its output to feed three consumers: message chunks, phase timings, event rows.

```ts
// platform/agui/agui-event-schema.ts
export const AGUI_PROFILE = "uar.agui/1" as const;

export const AGUI_LIFECYCLE = [
  "RUN_STARTED", "RUN_FINISHED", "RUN_ERROR", "STEP_STARTED", "STEP_FINISHED",
] as const;

export const AGUI_STREAM = [
  "TEXT_MESSAGE_START", "TEXT_MESSAGE_CONTENT", "TEXT_MESSAGE_END",
  "REASONING_START", "REASONING_MESSAGE_START", "REASONING_MESSAGE_CONTENT",
  "REASONING_MESSAGE_END", "REASONING_END",
  "TOOL_CALL_START", "TOOL_CALL_ARGS", "TOOL_CALL_END", "TOOL_CALL_RESULT",
  "STATE_SNAPSHOT", "STATE_DELTA", "MESSAGES_SNAPSHOT", "RAW", "CUSTOM",
] as const;

/** CUSTOM `value.name` values emitted by the runtime — verbatim from agui-adapter.ts */
export const UAR_CUSTOM_EVENTS = [
  "uar.citation.added",
  "uar.rag_citations",
  "uar.memory.recall",
  "uar.memory.mutation",
  "uar.artifact.available",
  "uar.artifact.input_required",
  "uar.skill.activated",
  "uar.context.updated",
  "uar.tool.approval_required",
  "uar.tool.denied",
] as const;
```

Phase attribution:

```ts
// features/chat/model/run-phase.ts
export type RunPhase = "context" | "skill" | "memory" | "retrieval" | "reasoning" | "tool" | "generate";

export function phaseOf(kind: ChunkKind): RunPhase | null {
  switch (kind) {
    case "context-update":                       return "context";
    case "skill-activation":                     return "skill";
    case "memory-recall":
    case "memory-mutation":
    case "memory-update":                        return "memory";
    case "citation":
    case "rag-citations":                        return "retrieval";
    case "reasoning":                            return "reasoning";
    case "tool-call":
    case "tool-approval":
    case "tool-denied":                          return "tool";
    case "text":                                 return "generate";
    default:                                     return null;
  }
}
```

A phase's duration is `last event − first event` within the phase, clamped to the run
window; unattributed gaps belong to `generate`. Compute once at `RUN_FINISHED` and
store in `run.phase_timings`.

---

## 6. Phase 5 — shell, navigation, brand

### 6.1 Responsive strategy

```tsx
// app/shell/app-shell.tsx
export function AppShell({ children }: { children: React.ReactNode }) {
  const isCompact = useMediaQuery("(max-width: 900px)");
  return (
    <div className="flex h-dvh flex-col overflow-hidden bg-[color:var(--color-bg)] text-[color:var(--color-fg)]">
      <div className="flex min-h-0 flex-1">
        {!isCompact && <NavRail />}
        <main className="min-w-0 flex-1 overflow-hidden bg-[color:var(--color-bg)]">{children}</main>
      </div>
      {isCompact && <MobileTabBar />}
      <MobileSheetHost />
    </div>
  );
}
```

- Destinations declared once in `nav-destinations.ts`, consumed by rail and tab bar;
  the desktop **Configure** group becomes the mobile Configure tab's hub list.
- Panels declare a presentation, not a placement: `<SidePanel id="inspector">` lands
  in the desktop inspector column or the mobile sheet host. Feature code never
  branches on viewport.
- Rail 240px expanded (wordmark) / 60px collapsed (app icon). Mobile top bar carries
  the 22px mark; launch screen 96px. Never wordmark and icon together.
- Hit targets: 44px compact, 40px desktop rail.
- Motion: fade + 20px slide, 200–320ms, `--ease-out`. No spring, no parallax.

### 6.2 Brand

Copy `docs/logo/` → `frontend/public/brand/`. Replace `KnowMeLogo.tsx` with
`shared/ui/uar-logo.tsx`; prefer the inline mono SVG with `currentColor` so it
inherits theme color, and the two-tone files for rail and launch. Favicon pair via
`<link rel="icon" media="(prefers-color-scheme: dark)">`.

### 6.3 Boundary lint

```js
"import/no-restricted-paths": ["error", { zones: [
  { target: "./src/shared",   from: "./src/features" },
  { target: "./src/shared",   from: "./src/app" },
  { target: "./src/platform", from: "./src/features" },
  { target: "./src/platform", from: "./src/app" },
  { target: "./src/features/*/", from: "./src/features/!(*)/**", except: ["./index.ts"] },
]}]
```

---

## 7. Phase 6 — markdown everywhere

### 7.1 The rule

**Every text surface in the app renders through one component**,
`shared/markdown/markdown-bubble.tsx` — the port of KnowMe's
`desktop/src/components/assistant-ui/markdown-text.tsx`. No `whitespace-pre-wrap`
divs, no ad-hoc `marked` calls, no second renderer, and — per §7.6 of the standard —
**no string replacement inside a monolithic message component**. Each rich format is a
typed React 19 renderer registered with the Assistant UI thread.

Four rules the KnowMe implementation already enforces, carried over verbatim:

1. **One MermaidBlock, exported and shared.** A ```mermaid fence and a `mermaid`
   artifact must render identically; two loaders would mean two themes and two error
   states. `shared/markdown/blocks/mermaid-block.tsx` is imported by both.
2. **Renderers are lazy where the dependency is expensive** (`await import("mermaid")`),
   preserve stable keys while streaming, and never block ordinary text rendering.
3. **Every renderer has an error boundary and a source fallback.** A failed diagram
   shows its source and the parse error, never a blank block.
4. **`defer` on `MarkdownTextPrimitive`** so token arrival does not thrash the tree.

Surfaces that use it:

| Surface | Notes |
|---|---|
| Assistant text chunk | full plugin chain |
| **User message** | full chain — users paste code, tables, and diagrams |
| Reasoning chunk | full chain, muted type scale |
| Tool call result | full chain when the result is text/markdown; JSON viewer when it parses as JSON |
| RAG citation snippet | inline chain (no block elements) with the matched span highlighted |
| Memory chunk content | inline chain |
| A2UI text fields | inline chain, after schema validation |
| Agent system prompt preview, skill descriptions, KB document preview, compiler rationale | full chain, read-only |
| Error chunk detail | inline chain |

Two variants, one implementation:

```tsx
<MarkdownBubble source={md} />                    {/* full: blocks allowed */}
<MarkdownBubble source={md} variant="inline" />   {/* inline: no headings, no block chrome */}
```

### 7.2 Plugin chain

```ts
// shared/markdown/plugins/remark-chain.ts
export const remarkChain = [
  remarkGfm,        // tables, task lists, strikethrough, autolinks, footnotes
  remarkBreaks,     // single newline = <br>, matches how models emit text
  remarkMath,       // $inline$ and $$block$$ — §7.6 "optional math when a workflow requires it"
];

// shared/markdown/plugins/rehype-chain.ts
export const rehypeChain = [
  rehypeRaw,                          // allow inline SVG / limited HTML from agents
  [rehypeSanitize, sanitizeSchema()], // ALWAYS immediately after rehypeRaw
  rehypeKatex,
];
```

`rehype-sanitize` runs on every path — model output and A2UI payloads are untrusted.
Never `dangerouslySetInnerHTML` outside a sanitized pipeline. **Two sanitizers, two
jobs**, matching KnowMe:

- **`rehype-sanitize` + schema** for markdown-derived HTML (below).
- **`DOMPurify.sanitize(content, { USE_PROFILES: { svg: true, svgFilters: true } })`**
  for a raw SVG artifact, which never passes through remark at all.
- Mermaid runs with `securityLevel: "strict"` and receives diagram syntax, not HTML;
  its SVG output is renderer-produced, which is why inserting it is safe.

```ts
// shared/markdown/plugins/sanitize-schema.ts  (from docs/full-implementation/A2UI + AG-UI.md)
export function sanitizeSchema() {
  const schema = structuredClone(defaultSchema);

  schema.tagNames = Array.from(new Set([...(schema.tagNames ?? []),
    "svg","path","g","defs","marker","polygon","polyline","circle","ellipse",
    "rect","line","text","tspan","use","symbol","clipPath","linearGradient","stop",
    "video","source","picture","figure","figcaption","details","summary","kbd","mark","sup","sub",
    // custom elements = the "MDX components"
    "pm-chart","pm-flow","pm-video","pm-image","pm-svg","pm-mermaid","pm-map","pm-timeline",
  ]));

  schema.attributes ??= {};
  schema.attributes["*"] = Array.from(new Set([...(schema.attributes["*"] ?? []),
    "className","class","id","style","title","role","data*","aria*",
  ]));
  schema.attributes["a"]        = ["href","target","rel"];
  schema.attributes["img"]      = ["src","alt","title","width","height","loading"];
  schema.attributes["video"]    = ["src","poster","controls","autoplay","loop","muted","playsinline","width","height"];
  schema.attributes["source"]   = ["src","type"];
  schema.attributes["svg"]      = ["viewBox","xmlns","width","height","fill","stroke","preserveAspectRatio"];
  schema.attributes["pm-chart"] = ["type","spec","data","options","title"];
  schema.attributes["pm-flow"]  = ["spec","title"];
  schema.attributes["pm-video"] = ["src","poster","title"];
  schema.attributes["pm-image"] = ["src","alt","title"];
  schema.attributes["pm-map"]   = ["center","zoom","markers"];

  schema.protocols = { href: ["http","https","mailto"], src: ["http","https","data","blob"] };
  return schema;
}
```

### 7.3 Block extensions

Each renders inside `block-chrome.tsx` — a Flat 2.0 chrome (surface-step fill,
`--radius-lg`, **no border**) with a hover-revealed action row: **Copy · Share ·
Expand · Download**, plus a mono language/type label in the header strip.

| Fence / tag | Component | Behavior |
|---|---|---|
| ` ```lang ` | `code-block.tsx` | **Shiki** with a themed pair (`--color-*` mapped light/dark), line numbers past 8 lines, soft wrap toggle, copy. Replaces highlight.js. Lazy-loaded; unhighlighted `<pre>` shows first, highlights on idle. |
| ` ```mermaid ` | `mermaid-block.tsx` | Lazy `mermaid.render()` to SVG, theme-synced via `mermaid.initialize({ theme: 'base', themeVariables })` fed from tokens. Render failure shows the source with the parse error, never a blank block. Pan/zoom + expand-to-modal. Re-renders on theme change. |
| ` ```svg ` or raw `<svg>` | `svg-block.tsx` | Sanitized passthrough, `max-width: 100%`, expand-to-modal, download as `.svg`. |
| `![alt](src)` / `<pm-image>` | `image-block.tsx` | Lazy, intrinsic-ratio placeholder, click to lightbox, download. Broken src → quiet caption, no layout jump. |
| `<video>` / `<pm-video>` | `video-block.tsx` | Native controls, `preload="metadata"`, poster, no autoplay with sound. |
| ` ```chart ` / `<pm-chart>` | `chart-block.tsx` | Vega-Lite-ish spec → chart, tokens for series colors, table fallback for a11y. |
| ` ```flow ` / `<pm-flow>` | `flow-block.tsx` | Node/edge spec → flow diagram, same chrome. |
| `$$…$$`, `$…$` | `math-block.tsx` | KaTeX; CSS imported once, not per-block. |
| GFM table | `markdown-components.tsx` | Rounded surface container, mono uppercase headers, zebra by fill, horizontal scroll on overflow, **no rules** (Flat 2.0). |
| Task list | components | Ember checkbox fill, no border. |
| Blockquote | components | Ember-tint fill + inset padding — **not** a left border. KnowMe already does this (`bg-muted/50 rounded-xl px-4 py-3`); UAR's `enhanced-markdown-text.tsx` uses `border-l-2 border-primary/30` and must be converted. |
| `hr` | components | A spacer, not a rule: `<div className="my-5 h-3" role="separator" />`. Same for the `divider` ContentBlock. |
| Inline video link | `markdown-components.tsx` | An `<a href>` ending in `.mp4/.webm/.mov` renders as a `<video controls preload="metadata">` rather than a link — KnowMe's `a` component does this today. |
| Code header | `code-block.tsx` | A `bg-muted` strip above the block carrying the lowercase language and a copy button with copied-state feedback (`CodeHeader` in KnowMe). |

### 7.3.1 Artifact kinds

An `artifact` chunk dispatches on a **normalized** kind. The alias table below is
KnowMe's `artifactKind()` verbatim — React and Flutter must resolve the same artifact
to the same body, so this list is a shared contract, not a local convenience.

| Normalized | Aliases | Body |
|---|---|---|
| `agui` | **any `agui_*` prefix, checked FIRST** | Collapsible activity row: human title + one-line summary + pretty JSON. Never a bare dump. |
| `markdown` | `markdown`, `md`, `document` | `MarkdownBubble` |
| `mermaid` | `mermaid` | the shared `MermaidBlock` |
| `svg` | `svg`, `image/svg+xml` | DOMPurify SVG profile, `role="img"` + accessible label |
| `video` | `video`, `mp4`, `webm` | player with CORS/mixed-content fetch fallback |
| `link` | `opengraph`, `open_graph`, `link` | OpenGraph card: thumbnail, title, description, host |
| `code` | `code` | mono block |
| `document` | `pdf`, `doc`, `docx`, `attachment` | file card: name, size, 6-line excerpt |
| `a2ui` | `a2ui`, `application/a2ui+json` | the A2UI renderer (§9) |
| `unknown` | — | collapsed “View event payload” |

Three behaviors that are easy to lose and were hard-won in KnowMe:

- **`agui_*` is tested before the switch.** An `agui_confirmation` must reach the
  actionable confirmation UI, not a JSON dump — the failure mode was one surface
  asking the user a question while the other showed them raw state.
- **Media envelopes unwrap.** `{"url": …}` or `{"path": …}` — a user-attached video
  carries a durable app-storage `path`, never a file-picker path (purgeable on
  Android, expiring on iOS). Resolve through a `local-media-src` helper.
- **Remote media degrades quietly.** A failed OpenGraph thumbnail renders *nothing*
  and is `aria-hidden`; a failed video renders a plain-language line, not a broken
  player. A `use-remote-content` hook does the native-fetch fallback when the webview
  cannot reach the origin.

### 7.4 Streaming vs finalize

Two-stage render, matching the pattern in `docs/full-implementation/A2UI + AG-UI.md`:

1. **Streaming.** Render markdown progressively with `remark-gfm` only. Fence blocks
   whose closing ``` has not arrived render as plain `<pre>` — **no mermaid, no Shiki,
   no KaTeX mid-stream**; a half-parsed diagram flickers and throws.
2. **Finalize.** On `TEXT_MESSAGE_END`, run the full chain once, persist both the raw
   markdown (`message.markdown`) and the resulting chunk array, and swap the bubble in
   a single commit.

Memoize with `unstable_memoizeMarkdownComponents` (already used) so unchanged blocks
do not re-render as tokens arrive. Carry `STREAMING_DEBOUNCE_FIX.md` and
`STREAMING_FLICKER_FIX.md` forward as **regression tests**, not comments.

### 7.5 Copy, share, export

- **Bubble level**: copy raw markdown, copy rendered rich text, share (permalink to
  `thread#message`), export message to `.md`.
- **Block level**: copy code / copy diagram source / download SVG / download image.
- **Thread level**: export as markdown with citations as footnotes.

Because `message.markdown` is persisted in PGlite, all of this works offline.

---

## 8. Phase 7 — the complete chunk catalog

One file per chunk under `features/chat/ui/chunks/`, one entry in the union, one
row in the trace lane. Adding a chunk type means touching exactly: the union, the
normalizer switch, the renderer map, the phase map, and one story.

Per §7.5 of the KnowMe standard: **new protocol variants must cause compile-time
exhaustiveness failures until React and Flutter renderers are implemented.** Every
switch over the union ends in `default: return assertNever(block)`.

### 8.1 The shared ContentBlock protocol

KnowMe already ships the cross-platform contract in
`packages/gen-ui-react/src/content-block.ts`, mirrored from the Rust
`crates/gen_ui_types/src/content_block.rs` (serde `tag="type"`, camelCase). It is the
**wire and storage shape**; keep it in lockstep and do not fork it.

```ts
export type ContentBlock =
  | { type: "text"; text: string }
  | { type: "thinking"; text: string }
  | { type: "code"; language: string; code: string }
  | { type: "citation"; source: string; quote: string }
  | { type: "memory"; operation: string; key: string; value: string | null }
  | { type: "toolUse"; id: string; name: string; inputJson: string }
  | { type: "toolResult"; toolUseId: string; outputJson: string; isError: boolean }
  | { type: "skill"; name: string; status: string }
  | { type: "artifact"; id: string; kind: string; content: string }
  | { type: "image"; url: string | null; dataBase64: string | null; mime: string;
      path?: string; alt?: string; width?: number; height?: number }
  | { type: "divider" };
```

Rules carried from the source file's own comments:

- `image.path` points into **app-owned storage**, never at a file-picker path — a
  picked path is purgeable on Android and expires on iOS, so a transcript holding one
  renders now and breaks later.
- `image.alt` is required for anything a user can see; `width`/`height` let the
  renderer reserve space so an arriving image does not shift layout.
- `divider` must render as spacing (§4.1), not the `<hr>` the current
  `ContentBlockView` emits.

UAR's richer runtime events do **not** get new wire variants. They ride as
`artifact` (kind-dispatched, §7.3.1) or as typed **Assistant UI data parts** (§8.4),
and are projected into the view union below.

### 8.2 The view union

`ContentBlock` is what persists; `Chunk` is what the thread renders. The projection is
one function, `toChunks(block[]): Chunk[]`, so the extra UAR detail (scores,
thresholds, timings, risk class, approval state) has somewhere to live without
breaking Flutter parity.

```ts
// features/chat/model/chunk.ts
export type ChunkKind =
  | "text" | "markdown"
  | "reasoning" | "thinking"
  | "tool-call" | "tool-approval" | "tool-denied"
  | "skill-activation"
  | "memory-recall" | "memory-mutation" | "memory-update"
  | "citation" | "rag-citations"
  | "context-update"
  | "a2ui-display" | "a2ui-input"
  | "artifact"
  | "image" | "video" | "file"
  | "state-snapshot" | "state-delta"
  | "step" | "usage" | "error" | "raw";

interface ChunkBase { id: string; kind: ChunkKind; at: string; runId?: string; seq: number }

export interface TextChunk       extends ChunkBase { kind: "text";      text: string }
export interface MarkdownChunk   extends ChunkBase { kind: "markdown";  source: string }
export interface ReasoningChunk  extends ChunkBase { kind: "reasoning"; text: string; signature?: string; redacted?: boolean }
export interface ThinkingChunk   extends ChunkBase { kind: "thinking";  text: string; budgetTokens?: number; usedTokens?: number }

export interface ToolCallChunk extends ChunkBase {
  kind: "tool-call"; toolCallId: string; toolName: string;
  server?: string; transport?: "stdio" | "sse" | "http";
  args: Record<string, unknown>; argsPartial?: string;
  result?: string; resultMime?: string;
  status: "pending" | "running" | "complete" | "failed" | "cancelled";
  durationMs?: number; riskClass?: "read" | "write" | "destructive" | "sandbox";
}
export interface ToolApprovalChunk extends ChunkBase {
  kind: "tool-approval"; toolCallId: string; toolName: string;
  args: Record<string, unknown>; reason?: string;
  decision?: "approved" | "denied"; decidedAt?: string;
}
export interface ToolDeniedChunk extends ChunkBase {
  kind: "tool-denied"; toolCallId: string; toolName: string; reason: string; policy?: string;
}

export interface SkillActivationChunk extends ChunkBase {
  kind: "skill-activation"; skillId: string; skillName: string;
  selectionMethod?: "keyword" | "embedding" | "hybrid" | "llm" | "explicit";
  score?: number; threshold?: number; status: "active" | "complete";
}

export interface MemoryRecallChunk extends ChunkBase {
  kind: "memory-recall";
  items: { id: string; content: string; type: "episodic" | "semantic" | "procedural" | "preference"; importance?: number; score?: number; pinned?: boolean }[];
}
export interface MemoryMutationChunk extends ChunkBase {
  kind: "memory-mutation"; operation: "create" | "update" | "delete" | "pin" | "decay";
  memoryId: string; content?: string; memoryType?: string; importance?: number;
}
export interface MemoryUpdateChunk extends ChunkBase {
  kind: "memory-update"; scope: "session" | "agent" | "global"; summary: string; itemCount: number;
}

export interface CitationChunk extends ChunkBase {
  kind: "citation"; source: string; content: string; url?: string;
}
export interface RagCitationsChunk extends ChunkBase {
  kind: "rag-citations";
  citations: { marker: number; chunkId: string; documentId?: string; documentName: string;
               knowledgeBase?: string; relevanceScore: number; snippet: string;
               page?: number; span?: [number, number] }[];
}

export interface ContextUpdateChunk extends ChunkBase {
  kind: "context-update"; strategy: string;
  messagesRemoved: number; tokensSaved: number;
  wasApplied: boolean; summaryGenerated: boolean;
  windowBefore?: number; windowAfter?: number; summarizerModel?: string;
}

export interface A2uiDisplayChunk extends ChunkBase {
  kind: "a2ui-display"; profile: string; component: string; version?: string;
  payload: unknown; validation: "valid" | "invalid" | "unknown-component"; validationError?: string;
}
export interface A2uiInputChunk extends ChunkBase {
  kind: "a2ui-input"; profile: string; component: string; requestId: string;
  payload: unknown; status: "awaiting" | "submitted" | "expired" | "cancelled"; response?: unknown;
}
export interface ArtifactChunk extends ChunkBase {
  kind: "artifact"; artifactId: string; title?: string;
  mime: string;                   // text/markdown | text/html | image/svg+xml | application/json | text/x-mermaid …
  content?: string; url?: string; bytes?: number;
}

export interface ImageChunk extends ChunkBase { kind: "image"; url: string; alt?: string; width?: number; height?: number }
export interface VideoChunk extends ChunkBase { kind: "video"; url: string; poster?: string; durationMs?: number }
export interface FileChunk  extends ChunkBase { kind: "file";  name: string; mime: string; bytes: number; url?: string }

export interface StateSnapshotChunk extends ChunkBase { kind: "state-snapshot"; state: unknown }
export interface StateDeltaChunk    extends ChunkBase { kind: "state-delta";    delta: unknown }
export interface StepChunk  extends ChunkBase { kind: "step"; name: string; status: "started" | "finished"; durationMs?: number }
export interface UsageChunk extends ChunkBase { kind: "usage"; inputTokens: number; outputTokens: number; totalTokens: number; costUsd?: number; model?: string }
export interface ErrorChunk extends ChunkBase { kind: "error"; message: string; code?: string; retryable?: boolean; attempt?: number; maxAttempts?: number }
export interface RawChunk   extends ChunkBase { kind: "raw"; type: string; payload: unknown }

export type Chunk = /* union of all of the above */;
```

`markdown` and `text` are distinct on purpose: `text` is the raw streaming buffer,
`markdown` is the finalized source persisted for copy/export (§7.4).

### 8.3 Source → chunk mapping

The runtime delivers some chunks as AG-UI CUSTOM events and some as **pseudo-tool
calls** (a convention already in `chat-stream-store.ts` / `enhanced-thread.tsx`).
Both paths normalize to the same chunk union.

| Chunk | AG-UI source | Pseudo-tool name |
|---|---|---|
| `text` | `TEXT_MESSAGE_START/CONTENT/END` | — |
| `reasoning` / `thinking` | `REASONING_*` | — |
| `tool-call` | `TOOL_CALL_START/ARGS/END/RESULT` | — |
| `tool-approval` | CUSTOM `uar.tool.approval_required` | — |
| `tool-denied` | CUSTOM `uar.tool.denied` | — |
| `skill-activation` | CUSTOM `uar.skill.activated` | `__skill__` |
| `memory-recall` | CUSTOM `uar.memory.recall` | `__memory_recall__` |
| `memory-mutation` | CUSTOM `uar.memory.mutation` | `__memory_mutation__` |
| `memory-update` | derived | `__memory_update__` |
| `citation` | CUSTOM `uar.citation.added` | — |
| `rag-citations` | CUSTOM `uar.rag_citations` | — |
| `context-update` | CUSTOM `uar.context.updated` | `__context__` |
| `a2ui-display` | CUSTOM `uar.artifact.available` | `__a2ui_display__` |
| `a2ui-input` | CUSTOM `uar.artifact.input_required` | `__a2ui_input__` |
| `artifact` | CUSTOM `uar.artifact.available` (non-A2UI mime) | — |
| `state-snapshot` / `state-delta` | `STATE_SNAPSHOT` / `STATE_DELTA` | — |
| `step` | `STEP_STARTED` / `STEP_FINISHED` | — |
| `usage` | `RUN_FINISHED` / `agui.done` | — |
| `error` | `RUN_ERROR`, transport retry | — |
| `raw` | `RAW`, unknown CUSTOM | — |

**Unknown CUSTOM names must not be dropped** — they become `raw` chunks, hidden in
the bubble by default and visible in the trace lane and inspector. That is how a new
runtime event shows up before the UI knows about it.

### 8.4 Render treatment and registration

Renderers are **registered as Assistant UI data parts**, not switched inside a message
component — KnowMe's `RichDataRenderers.tsx` pattern:

```tsx
export function RichDataRenderers() {
  useAssistantDataUI({ name: "memory",        render: MemoryData });
  useAssistantDataUI({ name: "citation",      render: CitationData });
  useAssistantDataUI({ name: "ragCitations",  render: RagCitationsData });
  useAssistantDataUI({ name: "skill",         render: SkillData });
  useAssistantDataUI({ name: "contextUpdate", render: ContextUpdateData });
  useAssistantDataUI({ name: "artifact",      render: ArtifactData });
  useAssistantDataUI({ name: "toolApproval",  render: ToolApprovalData });
  useAssistantDataUI({ name: "runLifecycle",  render: RunLifecycle });
  useAssistantDataUI({ name: "runError",      render: RunError });
  useAssistantDataUI({ name: "runCancelled",  render: RunCancelled });
  return null;
}
```

`runCancelled` is not optional: a stopped turn must state what survived
(“kept N blocks, still in context”) or the actions beneath it are buttons with no
stated consequence. Its status line is the live region — never the action row, or
every re-render re-announces four button labels.

Flat 2.0 throughout: each chunk is a filled surface at one step above its parent,
`--radius-lg`, mono metadata, no border, no shadow.

| Chunk | Bubble treatment | Trace lane |
|---|---|---|
| `text` / `markdown` | Full `MarkdownBubble` (§7) | generate segment |
| `reasoning` / `thinking` | **Collapsed by default**, `surface` fill, **cyan** mono label `// thinking`, duration + token metadata, user-controlled expansion, streams live when expanded. Model reasoning is visibly distinct from tool activity and from sources. | reasoning segment |
| `tool-call` | Header row: wrench, mono `server__tool`, status chip, duration; expandable args/result; JSON viewer when the result parses; risk class chip for write/destructive | tool segment, one row per call |
| `tool-approval` | Warning fill, args summary, **Approve / Deny** buttons; persists as pending across reload | tool row, amber |
| `tool-denied` | Destructive fill, policy reason, no actions | tool row, red |
| `skill-activation` | Amber `Sparkles` + name + status line on a `surface` fill; selection method and score vs threshold when present | skill segment |
| `memory-recall` | **Cyan** `Brain` eyebrow `Memory · <operation>`, key in semibold, value beneath; type, importance, and pinned marker per item; read/proposed/written/updated/rejected is stated in words, never implied | memory segment |
| `memory-mutation` | One quiet line: operation + memory id + content preview | memory segment |
| `memory-update` | One quiet line: scope + item count | memory segment |
| `citation` | Inline superscript marker; expandable source card on a `card` fill with ember `BookOpenText` eyebrow, source, quote, and a jump/open action that does not lose the reader's place | retrieval segment |
| `rag-citations` | Numbered `[n]` markers inline; hover-to-source panel; footer list with document, KB, score, snippet with the matched span highlighted | retrieval segment |
| `context-update` | Single quiet line: `compacted 34 → 12 messages, saved 21.4k tokens`, expandable to before/after and summarizer model | context segment |
| `a2ui-display` | Validated surface via the A2UI renderer (§9) | its own row |
| `a2ui-input` | Interactive surface; awaiting state persists in PGlite | its own row |
| `artifact` | Titled card by mime: markdown → MarkdownBubble; html → sandboxed iframe; svg → svg-block; mermaid → mermaid-block; json → JSON viewer; else download chip | its own row |
| `image` / `video` / `file` | The §7.3 media blocks | — |
| `state-snapshot` / `state-delta` | Hidden in the bubble; inspector only | inspector |
| `step` | Hidden in the bubble; a tick on the trace bar | tick |
| `usage` | Run footer: tokens, model, cost | — |
| `error` | Destructive fill, message + code, **Retry** when retryable, attempt counter during backoff | red segment |
| `raw` | Hidden by default; `Show raw events` reveals a mono row | grey row |

### 8.5 Trace lane and inspector

- `run-trace-bar.tsx` — flex segments sized by `phase_timings` share, 3% minimum so
  short phases stay visible, phase color from the `--color-phase-*` tokens, `title`
  per segment, keyboard-navigable listbox.
- `run-trace-timeline.tsx` — one row per `run_event`, virtualized past ~200 rows,
  filter chips per kind, jump-to-chunk on click.
- `run-inspector.tsx` — tabs: **Payload** (pretty JSON) · **Timing** (start, duration,
  gap) · **Raw AG-UI** (verbatim event, copyable). Reads `run_event`, so it works
  offline after reload.

---

## 9. Phase 8 — A2UI

**Use the official bridge, not a hand-rolled renderer.** PEM ships
`@prometheus-ags/a2ui-react`, which renders **official A2UI v0.9.1** surfaces through
the maintained engine and crosses a **default-deny application policy** before any
graph action runs. It is independently certified with packed-consumer and real-browser
accessibility/keyboard evidence. The alpha AG-UI chat/state APIs live on the explicit
`./ag-ui` subpath — treat that boundary as real and keep alpha imports in one file.

```
features/a2ui/
├── ui/{a2ui-surface-renderer.tsx,a2ui-registry-page.tsx}
│   └── surfaces/{decision-card.tsx,metric-series.tsx,diff-viewer.tsx,approval-form.tsx}
├── model/{a2ui-store.ts,surface-registry.ts,action-policy.ts}
└── api/a2ui-client.ts
```

- The runtime's own profile constant is `uar.a2ui/1`
  (`features/a2ui/a2ui-protocol.ts`). Where a surface is official A2UI v0.9.1, render it
  through the bridge; where it is a UAR-specific profile, render it through the local
  registry. Version both so a `/2` can land without a break.
- **`action-policy.ts` is default-deny.** A surface may only trigger graph actions the
  policy names. Protocol validity is not application authority — that separation is the
  whole point of the bridge's policy layer.
- Validate before render. Failure → a destructive-tinted fallback showing the error and
  the raw payload. Never blank, never `eval`, never unsanitized HTML.
- Unknown component or major-version mismatch → fallback card with the payload and a
  link into the registry page.
- Interactive artifacts round-trip through the artifact-response flow: the renderer
  submits `{ a2uiResponse: { artifactId, response } }`, the id resolved from
  `a2uiResponse.artifactId` → `artifact_id` → `surfaceId` → the block id, exactly as
  KnowMe's `ArtifactData` does. Pending state persists in PGlite so a reload does not
  lose an unanswered prompt.
- Text inside A2UI payloads renders through `MarkdownBubble variant="inline"` after
  sanitization.
- A2UI/A2A conformance work also ships an **A2A v1 JSON-RPC** conformance agent with
  caller-scoped task access. If UAR federates agents (`billing-bot` in the designs),
  that is the binding to target rather than a bespoke delegation protocol.

---

## 10. Phase 9 — configuration surfaces

| Feature | Screens | Notes |
|---|---|---|
| `providers` | catalog, detail, credentials, test transcript | Test = 4 probes (auth, list models, completion, streaming TTFT). Results ephemeral. |
| `knowledge` | bases, detail, documents, ingest queue, retrieval test, chunking lab | Ingest progress over SSE; lab compares strategies and promotes a winner. |
| `agents` | registry, editor tabs, compiler, registration | Compiler streams over AG-UI like any other run and renders with the same chunks. |
| `skills` | library, strategy, activation replay | Replay = batch AG-UI dry run. |
| `context` | strategy, thresholds, simulation, memory | Simulation runs client-side against the PGlite thread. |
| `mcp` | servers, health, per-tool approval | 60s health poll; red dot surfaces on the rail. |
| `tools` | built-in registry, schema, grants | |
| `web-access` | backends, quotas, domain fence, per-agent grants | |
| `settings` | config deck, per-agent override matrix | The reconciliation view. |

**Override model** — one resolver so the matrix and the runtime cannot disagree:

```ts
export function resolve<T>(key: SettingKey, agentId?: string, threadId?: string): Resolved<T>;
// thread override → agent override → runtime default
// Resolved<T> = { value: T; source: "thread" | "agent" | "default" }
```

`source` drives the ember "override" fill in the matrix — no second lookup.

---

## 11. Phase 10 — cleanup, budgets, a11y

- Delete `src/admin/`, `src/entities/fetchers/`, migrated `src/stores/*`, the
  `[data-admin-theme="terminal"]` block, and `docs/UI_DESIGN.md`'s authority claim
  (mark it superseded in the file header).
- Budgets, CI-enforced:
  - initial JS ≤ 250 KB gzip, excluding PGlite WASM, Mermaid, and Shiki chunks
  - thread list first paint ≤ 1s on cold IndexedDB
  - a 500-event run renders its trace lane in ≤ 100ms
  - a 2,000-line markdown message finalizes in ≤ 250ms
- A11y: status never by color alone; 3px ember focus ring everywhere; trace bar,
  toolbar chips, and block chrome keyboard-operable; contrast ≥ 4.5:1 in all three
  themes; `prefers-reduced-motion` honored; mermaid/chart blocks carry a text
  alternative.
- Tests: unit for normalizer + phase attribution + sanitize schema; snapshot per
  chunk type; integration for PGlite migration and outbox replay; Playwright for
  offline reload, theme switch, compact navigation, and markdown block interactions.

### Acceptance checklist (§12 of the KnowMe standard, applied to UAR)

**Visual** — no visible borders, dividers, or layout shadows remain; adjacent regions
differ by an approved background token; light and dark both pass contrast; ember stays
restrained and cyan keeps its meaning; empty, loading, streaming, degraded, offline,
and error states are all intentionally designed.

**Architecture** — Shadcn for primitives, Assistant UI for thread/composer/streaming,
PEM 3.x (not TanStack Query) for entity reactivity, Zustand for transient state only,
PGlite for conversation persistence, components calling hooks rather than stores or
transports.

**Chat** — a new user can send a prompt without configuring a provider; conversations
can be created, searched, resumed, renamed, and archived; streaming survives text,
thinking, citation, tool, memory, artifact, and media blocks; markdown, Mermaid,
sanitized SVG, images, and video all have safe renderers **and fallbacks**; refresh
restores history and per-thread drafts; **raw transport or runtime errors never appear
as assistant content**.

**Responsive** — reviewed at 320, 768, 1024, and 1440 CSS px in both themes; sidebar on
desktop, bottom navigation on phone; keyboard, screen-reader, reduced-motion, and
text-scaling checks pass.

---

## 12. Risks

| Risk | Mitigation |
|---|---|
| **PEM 3.0 is pre-`latest`** | Pin an exact `next` version; all usage behind `platform/entities/`; verify each capability against that repo's `release/*.md` evidence gates before depending on it. |
| Divergence from KnowMe | Token names, markdown renderer, artifact-kind table, and data-part registration are ports, not reinterpretations. `ContentBlock` stays in lockstep with `crates/gen_ui_types`. |
| Mermaid/Shiki bundle weight | Lazy-loaded, separate chunks, never in the initial graph; plain `<pre>` renders first. |
| Mermaid parse failures on partial fences | Never render mid-stream; finalize-only (§7.4). |
| Sanitizer too strict / too loose | Schema is a single reviewed file with unit tests, including XSS fixtures. |
| Event volume on long runs | Coalesce deltas, persist lifecycle + CUSTOM only, virtualize the timeline. |
| Flat 2.0 regression | ESLint style gate + Storybook visual regression, not code review. |
| Tailwind 4 breaking shadcn | Re-generate shadcn against the Tailwind 4 preset and restyle variants; don't hand-port. |
| Two sources of truth for chat | Only PGlite writes thread state; the network layer writes the outbox. |

---

## 13. Sequencing

Phases 1–2 are plumbing with no user-visible change. Phase 3 is the riskiest — land it
behind a flag with the old data path still callable. Phases 4–7 can run in parallel
across two engineers once phase 3 is green; the markdown pipeline (6) should land
before the chunk work (7) because every chunk renderer depends on it. Phases 8–9 are
additive per feature. Phase 10 gates release.

Ship behind `?ui=next` until phase 7 is complete, then flip the default and keep the
old shell reachable for one release.
