## Context

C-10 is the plan-assigned continuation of `migrate-cross-cutting-pages`, but the existing May proposal and tasks describe entity-store retirement that the superseding phase plan assigns to C-14. The current application already has a minimal `AppShell` with a 240px rail and three compact destinations. It does not implement the binding `docs/ui/uar-frontend-migration-plan.md` Phase 5 contract or the delivered Slash Gate brand.

The shell is an app-composition concern. It may import feature routes, shared UI, and hooks, but it must not fetch or import stores/services directly. Route identity comes from React Router. Presentation state is transient and remains in the existing Zustand UI store, exposed only through `useUiState`. Provider, entity, persistence, and AG-UI data flows are unchanged.

The UI routing audit consulted the vendored KnowMe standard, the UAR shell/mobile design artifact, UI/UX Pro Max, Anthropic frontend-design, Vercel React and composition guidance, and the current Vercel interface guidelines. Impeccable and `ux-designer` were unavailable in this harness, so audit/critique/polish are performed manually against the same binding artifacts. Generic palette/type recommendations are non-authoritative; the UAR token ladder and Space Grotesk/Inter/JetBrains Mono roles remain binding.

## Goals / Non-Goals

**Goals:**

- Establish the target kebab-case `app/shell/` composition with a 240px expanded and 60px collapsed desktop rail, compact bottom tabs, route breadcrumbs, a mobile sheet host, and an app-wide command palette.
- Declare route destinations once and project them into desktop work/Configure groups, compact tabs, the Configure sheet, breadcrumbs, and palette commands.
- Use the delivered UAR Slash Gate mark consistently: expanded rail wordmark, collapsed app icon, compact top-bar mark, and light/dark favicon pair.
- Preserve semantic navigation, predictable browser history, 40px desktop/44px compact targets, visible focus, reduced-motion-safe transitions, and status text that does not depend on color.
- Preserve the current routes and current feature implementations while projecting `/admin/knowledge`, `/admin/agents`, and `/admin/runs` as first-class work destinations.

**Non-Goals:**

- Moving admin pages, API clients, stores, or routes into final feature directories; C-14a–C-14c own that migration and retirement.
- Building the run trace/inspector, content-block catalog, bundle budget, or final responsive/WCAG certification; C-11–C-15 own those outcomes.
- Removing `cmdk` or migrating its existing selectors/editors. C-10 only prevents the new shell from adding another `cmdk` surface.
- Adding remote navigation, user-defined URLs, server-driven menu configuration, or new runtime APIs.

## Decisions

### 1. Replace the stale change surface in place

The OpenSpec identifier remains `migrate-cross-cutting-pages` because the KBD plan explicitly absorbs that existing 0/31 change as C-10. Its old entity migration tasks are replaced, not executed alongside the shell. Those concerns remain visible in the proposal as deferred C-14 ownership so work is not silently lost.

Alternative considered: create `app-shell-and-navigation` as a second change. Rejected because the phase plan explicitly forbids duplicating the absorbed change and names `frontend-app-shell` as C-10's delta capability.

### 2. One typed destination inventory drives every navigation projection

`nav-destinations.ts` owns stable ids, paths, labels, descriptions, group membership, icons, route matching, and compact-tab eligibility. Rail groups, compact tabs, Configure sheet rows, breadcrumbs, and palette items derive from it. Work destinations are Chat, Knowledge, Agents, and Runs. Configure destinations are Providers, MCP & tools, Skills, A2UI, and Runtime settings; About is a system utility.

The current `/admin/*` paths remain intact. Destination matching gives `/admin/knowledge`, `/admin/agents`, and `/admin/runs` their work identities before the broader `/admin/*` Configure predicate, preventing false active states.

Alternative considered: duplicate arrays per surface. Rejected because labels, route ownership, and active-state rules would drift.

### 3. CSS owns the 900px presentation switch

The shell renders one semantic composition and uses the binding 900px breakpoint to expose the desktop rail or compact top/bottom chrome. This avoids OS sniffing and duplicated route state; the feature content is mounted once. A narrow `matchMedia("(min-width: 901px)")` lifecycle listener closes an already-open compact modal when the viewport becomes desktop-sized, but it does not choose or duplicate the shell composition.

Alternative considered: branch the entire shell with `useMediaQuery`. Rejected because it duplicates responsive composition and can cause first-render churn; viewport-specific placement belongs to CSS and the single sheet host.

### 4. Shell presentation state flows through hook → Zustand store

The existing UI store gains `navRailCollapsed`, `commandPaletteOpen`, and `shellSheet`, with actions exposed through `useUiState`. Components call only the hook. Route selection closes transient overlays. The store holds ids and booleans, never React nodes or business data.

Alternative considered: local state in each rail/header/sheet component. Rejected because the keyboard shortcut, header trigger, compact tab, and host are siblings that require one inspectable source of truth.

### 5. Base UI Autocomplete owns the command palette

The installed `@base-ui/react` 1.6.0 exposes an official command-palette pattern using `Autocomplete` inside `Dialog`. C-10 follows that pattern with static typed route commands, `autoHighlight="always"`, an accessible input name, and close-on-selection. `cmdk` remains only for existing consumers until their owning migration.

Alternative considered: reuse the current `cmdk` wrapper. Rejected because D1 selects Base UI for general controls and navigation, Base UI now has a first-party command-palette primitive, and a new `cmdk` shell would create an avoidable mixed-primitive boundary.

### 6. The mobile Configure tab opens the shared sheet host

The fourth compact tab is a semantic button, not a fake route. It selects the `configure` sheet id; `MobileSheetHost` renders the Configure destination list through Base UI Dialog semantics. Navigation from the sheet uses normal router navigation and closes the sheet. The host's id-based switch is intentionally extensible for C-11's inspector without storing component instances.

Alternative considered: squeeze every Configure destination into bottom navigation. Rejected because it violates the four-tab design and produces unlabeled/overflowing targets.

### 7. Brand assets are copied; the reusable mark is inline

All delivered assets copy mechanically from `docs/ui/logo/` to `frontend/public/brand/` except `.DS_Store`. `shared/ui/uar-logo.tsx` owns an inline monochrome Slash Gate and inline wordmark/app-icon projections that preserve the delivered vector geometry while allowing host-loaded fonts and explicit light/dark/high-contrast palettes. The public assets remain authoritative for browser and install metadata. The HTML head selects light/dark favicons with media queries.

Alternative considered: retain `KnowMeLogo`. Rejected because it renders a different brand and contradicts the delivered C-10 assets.

## Risks / Trade-offs

- [Current admin pages still contain an inner navigation shell and `cmdk` palette] → C-10 leaves them behaviorally intact; C-14 replaces/removes that tree. The app-wide shell is the authoritative top-level chrome now.
- [Both compact and desktop navigation elements exist in the DOM] → CSS `display: none` makes the inactive landmark non-rendered/non-focusable; the content subtree is not duplicated.
- [Static `/admin/*` matching can drift as routes migrate] → focused route-matching tests make the compatibility map explicit; C-14 updates the one destination inventory when final paths move.
- [Global palette shortcut can duplicate listeners under remount/Strict Mode] → one effect in the shell installs and cleans up one `keydown` listener.
- [Delivered SVG wordmarks contain live text and fixed colors] → preserve their vector geometry in the shared inline React projection so host fonts and theme classes apply; retain the shipped files under `/brand/` for browser/install use.
- [Final mobile/WCAG certification is not part of this wave] → C-10 adds deterministic component-level responsive/accessibility evidence; C-15 remains the cross-device certification gate.

## Migration Plan

1. Replace the stale proposal/tasks and add the `frontend-app-shell` delta and this design.
2. Add the destination inventory, UI-store actions, shell components, and app composition wiring while preserving route targets.
3. Copy brand assets, replace all current `KnowMeLogo` consumers, and update favicon/manifest references.
4. Add focused shell interaction and contract tests; run typecheck, lint, architecture boundaries, and focused tests only during implementation.
5. Complete manual UI audit/critique/polish, artifact-refiner QA, isolated adversarial review, OpenSpec verification, canonical C-10 transition, and archive.

Rollback is structural: restore the previous `AppShell` import and old logo component while leaving current route pages untouched. No data or API migration is involved.

## Open Questions

None. cand-011 is resolved in favor of Base UI Autocomplete; the existing route compatibility strategy and C-14 ownership are explicit.
