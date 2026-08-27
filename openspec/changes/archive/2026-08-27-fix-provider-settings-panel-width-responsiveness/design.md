## Context

See `proposal.md` for motivation and `specs/frontend-configuration-surfaces/spec.md` for the behavioral contract. The provider list body currently owns the available panel width, while each provider field grid switches at the viewport-level `lg` breakpoint. The target panel file is already near its 600-line structure limit, and provider values and drafts remain owned by the existing settings hook and cache.

KBD Analyze selected the installed Tailwind CSS v4 container-query feature (`cand-001`) and Playwright Test (`cand-002`). No data model, provider protocol, persistence, entity graph, store, service, or transport change is involved.

## Goals / Non-Goals

**Goals:**

- Make the one-column/two-column decision from the provider panel's measured inline size while retaining exactly two columns at the desktop state.
- Choose a concrete, reusable width boundary before implementation.
- Prove behavior in a real browser when the panel is constrained independently of a wide viewport.
- Keep the source edit class-level and place new test behavior in a focused browser spec.

**Non-Goals:**

- Changing provider records, model inventories, settings drafts, save/reload behavior, or realtime reconciliation.
- Adding a layout dependency, JavaScript resize observer, free-form width state, or component-local business state.
- Testing or replacing browser-owned unload-confirmation copy.
- Repairing unrelated full-suite baseline failures or expanding the provider panel beyond the responsive gap.

## Decisions

### 1. Query the named provider-panel container at Tailwind `@xl`

Add `@container/provider-panel` to the scrolling provider-list body and replace the field grid's viewport variant with `@xl/provider-panel:grid-cols-2`, retaining `grid-cols-1` as the default. Tailwind's standard `@xl` container token is 36rem (576px at a 16px root). This is a more conservative boundary than the 32rem `@lg` alternative for long URLs, model labels, and provider content; browser evidence must measure the actual grid tracks and control bounds rather than infer usable column width from incomplete padding arithmetic.

A named container makes the intended ancestor explicit and prevents a nearer unrelated `@container` from changing the query target. Keeping one column as the base state preserves usable layout when the container-query variant is unsupported or does not evaluate while the base utilities remain available.

Alternatives considered:

- **Viewport `lg` breakpoint:** rejected because it caused the assessed conformance gap.
- **Tailwind `@lg` container token (32rem):** rejected because its approximate 233px post-padding columns leave less room for long provider content.
- **Intrinsic `auto-fit/minmax`:** retained as `cand-005` reference material but not selected because an unconstrained intrinsic grid can exceed the required two-column desktop composition.
- **`@tailwindcss/container-queries`:** rejected because Tailwind v4 already provides the capability.
- **`use-resize-observer` or direct ResizeObserver state:** rejected because presentation-only width behavior does not justify React state, resize callbacks, or rerenders.

### 2. Add a focused Playwright spec with deterministic provider data

Create `frontend/e2e/provider-settings-responsive.spec.ts` under the default Playwright project. Intercept the provider-settings read request with a deterministic provider containing representative long content; do not depend on installed runtime configuration. Observe settings-namespace mutation requests, do not activate Save, and assert that the test issues zero durable mutation requests.

Use a 1440×900 browser viewport for every layout state. Resolve the provider-list body from the deterministic provider group and its immediate parent so the same stable semantic target exists before and after the named-container class is added. Derive the 36rem boundary from the measured root font size, then constrain that body to content-box widths at `boundaryPx - 1` and `boundaryPx + 1`; first assert the measured content-box width equals the intended value so a fixture mistake or different breakpoint cannot produce a false result. Finally remove the artificial constraint and verify the normal desktop state.

For each state:

- compare the Base URL and Protocol control bounding boxes to prove vertical stacking or same-row two-column geometry;
- assert the document and provider card do not overflow horizontally;
- assert every visible provider control's rendered bounding box remains within the provider card, while permitting a text-entry control to scroll its own long value internally;
- focus the Enable toggle, then Tab through Base URL, Protocol, API Key, API-key reveal, and Default Model in DOM order; after each Tab, assert the expected active element, intersection with the provider-list scroll viewport, and a computed visible focus indicator;
- modify a provider field before crossing the boundary, then verify the value and modified indicator survive the layout transition.

The state matrix is: constrained narrow at `boundaryPx - 1` with exactly one computed grid track, constrained wide at `boundaryPx + 1` with exactly two computed grid tracks, and normal unconstrained desktop with exactly two computed grid tracks. The test must restore any temporary inline constraint during cleanup.

Use functional geometry and focus assertions as the acceptance gate. Screenshots remain failure diagnostics, not the sole proof.

Alternatives considered:

- **Extend `a11y-responsive-certification.spec.ts`:** rejected because that suite is excluded from the default Playwright configuration and covers broad surface certification rather than this focused provider contract.
- **Extend the installed-service route test:** rejected because it relies on machine configuration and cannot deterministically constrain the panel or provider content.
- **Component-class assertion only:** rejected because the existing test already demonstrates that class presence does not prove real container behavior.

### 3. Preserve the existing settings-state boundary

The implementation changes CSS classes only. Provider values continue through `useSettings("provider")`; the draft cache remains authoritative, and no component state mirrors panel width or provider data. The browser transition test observes state preservation without introducing new state machinery.

### 4. Carry adopted-candidate evidence into implementation planning

- `cand-001` — Tailwind CSS v4 built-in container queries. Official v4 documentation demonstrates `@container`, named `@size/name:*` variants, and arbitrary container values: <https://github.com/tailwindlabs/tailwindcss.com/blob/main/src/blog/tailwindcss-v4/index.mdx> and <https://github.com/tailwindlabs/tailwindcss.com/blob/main/src/blog/tailwindcss-v3-2/index.mdx>. The installed project version is 4.3.3.
- `cand-002` — Playwright Test. Official documentation provides explicit viewport configuration and retrying locator/screenshot assertions: <https://github.com/microsoft/playwright/blob/main/docs/src/test-api/class-testoptions.md> and <https://github.com/microsoft/playwright/blob/main/docs/src/api/class-locatorassertions.md>. The installed project version is 1.62.1.

## Risks / Trade-offs

- **[The 36rem token may still expose unusually long provider content]** → Use deterministic long fixture values, retain `min-w-0`, and assert both page-level and card-level overflow in the browser test.
- **[Inline-size containment can alter intrinsic sizing of the provider-list body]** → Place the container on the existing full-width scrolling body, not on a flex-sizing shell, and verify both constrained and unconstrained browser states.
- **[A test can appear narrow while the queried content box remains above the boundary]** → Measure the actual content-box width before asserting layout.
- **[The panel file is near its structure limit]** → Restrict production edits to existing class strings; put all new proof in a separate Playwright spec. If implementation needs new behavior or markup, stop and plan a coherent extraction instead of compressing code.
- **[Artificial test sizing may miss shell interactions]** → Keep the browser viewport wide, load the real settings shell, and constrain only the actual provider-panel container.

## Migration Plan

1. Add the focused browser test, target the provider-list body through its existing provider-group relationship, and confirm the geometry assertion fails against the viewport-based grid rather than because a future container class is absent.
2. Replace the provider-list and field-grid classes with the named container-query form.
3. Run focused component and browser tests, TypeScript, targeted lint, settings structure, frontend build, and strict OpenSpec validation at their prescribed verification tiers.
4. Roll back by restoring the two class strings and removing the focused browser spec; no data or configuration migration is required.
