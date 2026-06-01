# Assessment — `knowledge-page-aesthetic-pass`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `readme-architecture-diagram` (100%)
**Carry-over from:** `direct-entity-migration-models` (knowledge page redesign deferred while data layer was migrated)

---

## 1. Phase goal

Apply the terminal admin aesthetic per `docs/admin-aesthetic-spec.md` to `frontend/src/admin/pages/knowledge-page.tsx` (778 LOC). Data layer is already direct (via `useKnowledgePage` compat hook from the migration arc); this phase is **visual-only**.

The page currently uses ShadCN-Tailwind defaults: `bg-card`, `text-foreground`, `text-muted-foreground`, `bg-primary/10`, `border-primary/40`, emerald/blue/amber/destructive status badges, `font-display` headings, `<Progress>` ring spinners, and a generic empty state with a `BookOpen` icon in a `bg-primary/15` rounded tile. Audit found 71 non-terminal token references — every one of them needs to migrate to the terminal palette or a token under the `data-admin-theme="terminal"` selector.

### Out of scope

- Data-layer changes (already direct).
- Search behavior (semantic search via `runSearch` stays functional, just restyled).
- Document upload mechanics (FormData / file input intact).
- Touching the Dialog / AlertDialog Radix primitives' internals — only their contents get styled.

---

## 2. Current state inventory

### 2.1 Page composition (778 LOC, single component)

| Block | Responsibility | Lines (approx) |
|---|---|---|
| Imports + STATUS_VARIANT map | emerald/blue/amber/destructive status pills | 1–55 |
| Component declaration + state hooks | useKnowledgePage, dialog flags, file input ref, search query, drag-over flag | 57–107 |
| Effect blocks | mount fetch, document polling refs | 108–183 |
| **KB list / sidebar** | header, grid of KBs, empty state, "Add KB" CTA | 195–305 |
| **Detail panel** | selected KB header, search box, upload zone, document table, search results panel | 310–648 |
| Add KB dialog | name + description inputs | 656–708 |
| Delete KB AlertDialog | confirm dialog | 710–742 |
| Delete document AlertDialog | confirm dialog | 744–778 |

### 2.2 Aesthetic deviations from the spec

| Token / pattern | Current usage | Spec requires |
|---|---|---|
| `bg-card`, `bg-card/80` | Page surfaces | `bg-[hsl(var(--terminal-surface))]` |
| `text-foreground` | Body text | `text-[hsl(var(--terminal-fg))]` |
| `text-muted-foreground` | Secondary text | `text-[hsl(var(--terminal-fg-dim))]` |
| `bg-primary/10`, `bg-primary/15` | Icon tiles, accent backgrounds | phosphor: `bg-[hsl(var(--phosphor)/0.12)]` + matching border |
| `border-primary/40`, `border-primary/50` | Focus + active borders | `border-[hsl(var(--phosphor)/0.4)]` |
| `bg-emerald-500/15 text-emerald-400` (status `indexed`) | Document status | phosphor variant |
| `bg-amber-500/15 text-amber-400` (status `pending`) | Document status | `bg-[hsl(var(--amber)/0.12)] text-[hsl(var(--amber))]` |
| `bg-blue-500/15 text-blue-400` (status `processing`) | Document status | phosphor or amber (no blue in the spec) |
| `text-destructive`, `bg-destructive/15` (status `failed`) | Error states | `text-[hsl(var(--signal-red))]` |
| `font-display` | Headings | drop — admin uses mono-display per spec |
| `rounded-xl`, `rounded-lg` | Decorative radii | flat — terminal aesthetic uses 0 radius on cards, light radius on chips |
| `<Loader2 className="animate-spin" />` | Loading indicators | `<LoadingCursor />` flicker block |
| Generic empty state with icon tile | "No knowledge bases" | `<EmptyFrame title hint action>` |
| Inline error spans (`<p className="text-destructive">`) | Errors | `<ErrorBar code="KB" message ... />` |
| `<Progress value={undefined} className="h-1" />` | Upload indeterminate progress | scanline overlay OR `<LoadingCursor />` inline |
| Focus ring `ring-primary/50` | Cards / buttons | `outline-2 outline-[hsl(var(--phosphor-glow))]` |

### 2.3 Shared components ready to use

All authored during `direct-entity-migration-models`:
- `frontend/src/components/admin/loading-cursor.tsx` — `<LoadingCursor>` blinking `▍`
- `frontend/src/components/admin/empty-frame.tsx` — `<EmptyFrame>` ASCII frame
- `frontend/src/components/admin/error-bar.tsx` — `<ErrorBar>` mono-prefix bar

The page imports `LoadingCursor` and `ErrorBar` already (from the data migration) but only at the chrome level. The deep page body still uses pre-migration patterns.

### 2.4 Tests

No vitest covers visual aspects. Knowledge page has no contract test specific to it. CI grep gates DO cover this page:
- Banned fonts grep is scoped to `frontend/src/admin/` — currently passes 0 matches (the page doesn't reference Inter/Roboto etc.).
- `outline: none` grep — currently passes.

Risk: if the aesthetic rewrite introduces a banned font or strips outlines, CI will block on grep. **This is the intended safety net.**

---

## 3. Definition of done

| # | Criterion | Verification |
|---|---|---|
| K1 | All ShadCN-token references (`bg-card`, `text-foreground`, `text-muted-foreground`, `bg-primary*`, `border-primary*`, `font-display`) replaced with terminal-aesthetic equivalents under `[hsl(var(--terminal-*))]`/`--phosphor`/`--amber`/`--signal-red` | grep tightened |
| K2 | `<LoadingCursor>` replaces every `<Loader2 className="animate-spin" />` on this page | grep |
| K3 | `<EmptyFrame>` replaces the "No knowledge bases" / "No documents" empty states | grep |
| K4 | `<ErrorBar>` replaces inline `<p className="text-destructive">` error spans | grep |
| K5 | Document STATUS_VARIANT map uses terminal palette (phosphor for indexed, amber for pending/processing, signal-red for failed) | file diff |
| K6 | Focus rings use `outline-[hsl(var(--phosphor-glow))]` not `ring-primary/50` | grep |
| K7 | `<Progress>` upload bar replaced with `<LoadingCursor />` or scanline animation | grep |
| K8 | `pnpm --filter ./frontend test` ≥ 40/40 | output |
| K9 | `pnpm --filter ./frontend build` clean | output |
| K10 | `pnpm run ci-gates` exits 0 (no banned fonts, no `outline: none`, no bridge revival) | output |
| K11 | Page LOC delta within ±15% of current 778 (target: don't bloat) | `wc -l` |
| K12 | Manual UX preserved: KB create, delete (with confirm), document upload, semantic search, document delete (with confirm) | code review |
| K13 | Audit doc row for KnowledgeBase / Document updated: "compat hook" annotation replaced with full terminal aesthetic citation | file diff |

---

## 4. Gap analysis

### 4.1 Rewrite shape

The page has 3 logical regions:

1. **Sidebar (KB list)** — 110 LOC. Replace `<button>` cards with terminal bordered rows; ASCII empty state; new-KB button uses phosphor accent.
2. **Detail panel** — ~330 LOC. The biggest region. Header chrome, search box, drag-and-drop upload zone (most opinionated visual), document table, search results panel.
3. **Dialogs (3)** — ~120 LOC. ShadCN Dialog/AlertDialog wrappers stay; only their contents (Labels, Inputs, Buttons) get terminal-styled.

The drag-and-drop upload zone is the most aesthetic-heavy piece. Current:
- `border-2 border-dashed` + `bg-primary/5` when dragOver
- "Drop files here or click to browse" copy

Terminal version:
- ASCII-art-bordered drop zone (similar to `<EmptyFrame>` but interactive)
- Mono copy: `drop files · click to browse`
- Phosphor flash on dragOver

### 4.2 Document status pills

Today's STATUS_VARIANT map:

```ts
const STATUS_VARIANT: Record<string, string> = {
  indexed:    "bg-emerald-500/15 text-emerald-400 border-emerald-500/25",
  pending:    "bg-amber-500/15 text-amber-400 border-amber-500/25",
  processing: "bg-blue-500/15 text-blue-400 border-blue-500/25",
  failed:     "bg-destructive/15 text-destructive border-destructive/25",
};
```

Terminal mapping (no blue in the spec; processing folds into amber):

```ts
const STATUS_VARIANT: Record<string, string> = {
  indexed:    "border-[hsl(var(--phosphor)/0.5)] text-[hsl(var(--phosphor))] bg-transparent",
  pending:    "border-[hsl(var(--amber)/0.5)] text-[hsl(var(--amber))] bg-transparent",
  processing: "border-[hsl(var(--amber)/0.5)] text-[hsl(var(--amber))] bg-transparent",
  failed:     "border-[hsl(var(--signal-red)/0.5)] text-[hsl(var(--signal-red))] bg-transparent",
};
```

### 4.3 Compat hook stays

`useKnowledgePage` is the page's data layer — unchanged this phase. The data path (graph → optimistic helpers → REST → SSE reconcile) is already correct from the migration arc.

### 4.4 Risk areas

- **Page is 778 LOC monolithic.** A full rewrite in one Edit is risky; better to do region-at-a-time edits.
- **Drag-and-drop visual feedback.** ShadCN's pattern uses dashed borders; the terminal aesthetic prefers solid + ASCII frames. The user-experience expectation (drop-zone affordance) must remain obvious.
- **Search results panel** mixes scrolling text + metadata; needs careful token swap to keep readable density.
- **Markdown body in search results** (if any `<ReactMarkdown>` wrapper exists) — leave its prose styling alone; terminal aesthetic for chrome only.

---

## 5. Sequencing recommendation

3 changes, ordered by visual impact + risk:

1. **`knowledge-redesign-sidebar-and-empty-states`** — KB sidebar header, KB row cards, "No knowledge bases" empty state, "Add KB" CTA, status pill map. Smaller LOC region; validates the visual choices before going wide.
2. **`knowledge-redesign-detail-and-upload`** — detail-panel header, search box, drag-and-drop upload zone (with ASCII frame variant), document table, document status pills, search results panel.
3. **`knowledge-redesign-dialogs-and-final-pass`** — Add-KB dialog body, Delete-KB AlertDialog body, Delete-doc AlertDialog body. Audit-doc row update. Final `pnpm run ci-gates` confirmation.

Each change runs `pnpm test` + `pnpm build` + `pnpm run ci-gates`.

---

## 6. Decisions (defaults — no questions needed)

| Decision | Choice | Rationale |
|---|---|---|
| Drag-zone style | Solid border + inset ASCII frame on dragOver | Matches terminal aesthetic; preserves affordance |
| Processing status color | Amber (no blue in palette) | Spec contains only phosphor/amber/signal-red accents |
| Indeterminate progress | `<LoadingCursor />` inline | Replaces `<Progress value={undefined} />` |
| Dialog primitives | Keep Radix Dialog/AlertDialog wrappers | Out of scope per §1; only their contents are restyled |
| Page rewrite cadence | Region-at-a-time (3 changes) | Risk mitigation for 778 LOC single-file page |
| Playwright screenshot | Defer | Still no live dev server in autonomous flow; capture in next browser-smoke walkthrough |

---

## 7. Progress signal

Assessment complete. Defaults sufficient. Next: `/kbd-plan knowledge-page-aesthetic-pass`.
