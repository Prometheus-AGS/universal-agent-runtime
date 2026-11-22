# Round 3, item 1 (R3-1): window the Models catalog list. Writer: uar-console

Base: c8eb9e23, branch docs/kbd-uiux-architecture-assess-analyze. Phase class: implementation only.
Nothing was executed: no test, build, typecheck, lint, formatter, server or browser.

## Rules applied (summary)

Vercel React Best Practices: the re-render group governs the design. The virtualizer's scroll state
lives in its own component (`VirtualCatalogList`), so a scroll re-renders tens of rows and not the
page, the dialog or the pickers (`rerender-memo`, `rerender-no-inline-components`,
`rerender-derived-state-no-effect`: the focused row index is derived in render from one key).
`rerender-functional-setstate` is used for the focus and scroll-margin updates. No setState in
render; the two effects only subscribe to the DOM (layout effect plus ResizeObserver) and clean up.
Vercel Composition Patterns: the list is a generic, presentational component that takes `rows`,
`scrollElement`, `label` and a `renderRow` slot, so `ModelRow` stays where it is and no boolean mode
props were added. Prometheus entity rules (entity-perf, `.claude/rules/typescript.md`): no new fetch,
store, hook subscription or graph write; `filtered` still comes from the same memo, compare state is
unchanged UI-local state, subscriptions are as narrow as before (the new `scrollElement` state is
the page's single DOM-ref state, set once at mount). Impeccable Operate bar (guidance only, no
engine run): the page must look and behave identically, one native scroll container, native
checkboxes, no new motion or decoration; list semantics are added for assistive technology.
`@tanstack/react-virtual` 3.14.23 / `virtual-core` 3.19.8 were read from
`frontend/node_modules/.pnpm/...` (types and source), not recalled.

## Files changed

- `frontend/src/features/models/ui/virtual-catalog-list.tsx` (new)
- `frontend/src/features/models/ui/models-page.tsx` (import, one `useState`, `ref` on the scroll div,
  the list `div` plus `filtered.map` replaced by `<VirtualCatalogList>`)
- this handoff

No dependency, test, fixture, token or CSS change. No new border classes (the Flat 2.0 rule scans
string literals; the new file has none).

## Design decisions (verified against the installed source)

1. **One scroll container.** `ModelsPage` keeps `flex-1 overflow-y-auto` and exposes it with a
   callback ref into state (`ref={setScrollElement}`). The list passes it to
   `getScrollElement`. The one extra page render happens at mount, not on scroll.
2. **scrollMargin.** `list.top - scrollElement.top + scrollElement.scrollTop`, rounded, re-measured
   by a `ResizeObserver` on the scroll element, each of its children (configured section, catalog
   section) and the catalog section (filter wrap, loading and empty notices). It is stored in list
   state and passed as `scrollMargin`. Items are placed with `translateY(item.start - scrollMargin)`;
   `getTotalSize()` already excludes the margin (core `_getLayoutTotalSize`), so the container height
   is right.
3. **Measured heights.** `estimateSize` 64 px, `gap: 6` (replaces `gap-1.5`; the total excludes a
   trailing gap), `ref={virtualizer.measureElement}` plus `data-index` on each item wrapper, so rows
   that wrap are measured by borderBoxSize via the core's own ResizeObserver.
4. **Bounded mount, including before measurement.** `initialRect` is 960x800, so the first render
   (scroll element still null) yields about 12 rows plus 8 overscan. After measurement the count is
   viewport height / row height + 16, plus at most 3 focus-retention rows. Core `calculateRange`
   returns rows near the first item when the list is below the fold (`findNearestBinarySearch`
   returns 0), so filtering to one key mounts that row at once. There is no fallback that renders
   all rows. If the scroll element has height 0 (hidden), no rows mount; it never widens to all rows.
5. **Markup parity.** `ModelRow` markup, classes and text are untouched. Each row sits in one
   wrapper `div` (`absolute left-0 top-0 w-full`, `role="listitem"`) inside a container
   (`relative w-full`, `role="list"`). `frontend/e2e/support/models-page.ts` `catalogRow()` takes
   the last `div` containing the checkbox; the wrappers are ancestors of the `ModelRow` root, and
   Base UI's checkbox root is not a `div`, so the last match is still the `ModelRow` root with all
   the row text. `models-count` (filtered total), the header text and
   `compare-checkbox-<key>` ids are unchanged.
6. **Keyboard.** `rangeExtractor` adds the focused row and its two neighbours to the default range.
   Focus is tracked by row key (`onFocus` / `onBlur` on the wrapper, which bubble in React), so a
   focused row is not unmounted when scrolled away and Tab / Shift+Tab always find the adjacent row
   in the DOM at once, before any scroll. The browser scrolls a focused row into view, which then
   moves the window.
7. **Assistive technology.** `role="list"` with `aria-label="Catalog models"`; each mounted item has
   `role="listitem"`, `aria-setsize = filtered.length` and `aria-posinset = index + 1`.
8. **Filter change and a shorter list.** The container height follows the new total, the browser
   clamps `scrollTop` on layout and fires `scroll`, and the virtualizer listens for it. If its offset
   is briefly stale and past the end, `findNearestBinarySearch` returns the last row, so the last
   row plus 8 overscan rows are mounted, which is what the clamped viewport shows. I added no
   explicit scroll write. See risks.
9. **React Compiler lint.** The repo's existing annotation, copied from `combobox-picker.tsx` and
   `run-trace-timeline.tsx`: `// eslint-disable-next-line react-hooks/incompatible-library` on the
   `useVirtualizer` call only.

## What changes for a user

- Browser find-in-page (Cmd/Ctrl+F) only matches mounted rows (tens), not the whole catalog. Use the
  page's search box. Page-level text selection ("select all") covers mounted rows only.
- The scrollbar thumb length may shift as unmeasured rows (estimated 64 px) are measured while
  scrolling. Wrapped rows are measured, so positions stay correct.
- Screen readers announce each row as "n of N" using `aria-posinset` and `aria-setsize`, and the
  list as "Catalog models, list". Before, the rows had no list semantics.
- The scroll position of a long list is preserved across compare toggles (no remount; row keys are
  `model key`).
- The page renders one extra time at mount (callback ref state), not per scroll.
- Nothing else is visible: same rows, same 6 px gap, same hover, checked and dimmed states.

## Risks

- The exact `catalogRow()` / identity-spec behavior, performance numbers and long-task counts are
  reasoned, not measured. R3-1's 200 ms limit is for the picker; this change removes the 110k-node DOM
  behind it, but the 150-172 ms entity-persistence timer task noted in the plan is untouched.
- Item 8 relies on the browser clamping `scrollTop` and the virtualizer's scroll listener, not on a
  written reset. If a frame of partial blank space shows in a refresh-while-scrolled-deep case, the
  fix is a layout effect that scrolls to `scrollMargin` when `rows` shrinks past the viewport.
- `scrollMargin` is read from `getBoundingClientRect` and assumes the scroll element has no padding or
  border (true today: `flex-1 overflow-y-auto`). Adding either would offset rows.
- A focused row stays mounted until it blurs. If it is removed by a filter while focused, its key
  can stay in state; it is then ignored (not found in `rows`) and the next focus or blur resets it.
- Absolute positioning with `transform` gives each row its own stacking context; nothing in
  `ModelRow` uses `position: fixed` descendants.
- `models-page.tsx` is still 889 lines (the instruction was not to refactor it).
- E2E specs that count every catalog row in the DOM would now see tens. I found none:
  `model-catalog-identity.spec.ts` only targets one key at a time (`toHaveCount(1)` after filtering).

## Unverified (everything executed)

Typecheck, lint (including Flat 2.0 and the incompatible-library annotation placement), unit and
e2e tests, rendered layout at 320 / 768 / 1024 / 1440, 8,171-row mount count, scroll and Tab
behavior, long-task timings for open / type / close of the Add model picker, and the
identity-scenario baseline. All remain for the boundary run.
