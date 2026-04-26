## Context

The React admin console already has a compact runtime shell with grouped navigation, command palette support, mobile slide-over navigation, and first-class runtime surfaces for cockpit, runs, approvals, protocols, providers, memory, and A2UI testing. Existing Playwright coverage checks several admin pages load, but it does not verify the librefang-inspired runtime console structure as an integrated operator experience across desktop and mobile viewports.

This change is a hardening step. It should add evidence that the console is reachable, navigable, and visually coherent without changing the production data flow. Detailed replay of runtime events into the entity graph is intentionally left to `runtime-event-replay-entity-sync-tests`.

## Goals / Non-Goals

**Goals:**

- Add deterministic Playwright coverage for `/admin` runtime console navigation across desktop and mobile viewports.
- Verify the command palette can open and navigate to key runtime surfaces.
- Verify cockpit, provider, protocol, memory, approval, and A2UI testing surfaces render stable operator-facing landmarks.
- Check for obvious layout regressions such as error boundaries, inaccessible navigation, or overlapping primary shell controls.
- Use stable accessibility queries first, with narrow `data-testid` additions only where the current UI lacks a durable selector.
- Keep KBD progress synchronized as the OpenSpec artifact and implementation tasks advance.

**Non-Goals:**

- Do not redesign the runtime console UI in this change.
- Do not add new runtime entities, provider APIs, or realtime event normalization.
- Do not perform pixel-perfect visual snapshot testing as the primary gate; use semantic assertions and bounded geometry checks.
- Do not introduce a new browser testing dependency beyond the existing Playwright setup.
- Do not test external provider credentials or live model calls.

## Decisions

### Decision: Use targeted Playwright specs instead of broad snapshot tests

Runtime console acceptance should be based on browser-observable behavior: pages load, nav works, command palette routing works, landmarks remain visible, and critical controls do not overlap. Full-page snapshots are too brittle for a dense operational console whose data can vary by environment.

Alternative considered: add screenshot snapshots for every admin surface. This would catch more visual drift but would create noisy failures from font rendering, dynamic data, and theme differences.

### Decision: Cover a small viewport matrix

The first test matrix should cover one desktop viewport and one mobile viewport. Desktop proves the persistent sidebar and sticky context layout; mobile proves the menu button, slide-over navigation, and narrow content flow.

Alternative considered: test many breakpoints. That would increase confidence, but it would slow the phase without materially improving coverage for the current closure gap.

### Decision: Prefer accessible selectors, then narrow test IDs

Tests should use roles, labels, headings, and visible text where possible. If the runtime shell lacks a stable selector for command palette, mobile navigation, or a dense panel, add narrow `data-testid` attributes to shell-level elements rather than coupling tests to CSS classes.

Alternative considered: select by Tailwind classes or DOM structure. That would be fragile because the visual system is still evolving.

### Decision: Keep data expectations fixture-light

The visual tests should not require live backend data. Empty states are acceptable when they are the intended operator-facing state, and tests should assert the empty-state text or shell landmarks instead of requiring seeded runtime entities.

Alternative considered: seed entity graph state in this change. That belongs in the follow-up event replay change because it touches runtime normalization and store behavior.

### Decision: Keep production layering unchanged

If implementation needs additional selectors or labels, components may receive static accessibility/test attributes. Components must not import services, hooks must not import services, and tests must not drive changes that collapse the existing component -> hook -> store -> service layering.

Alternative considered: add test-only service shortcuts or direct store mutation helpers. That would undermine the frontend architecture rules and blur this change with realtime replay testing.

## Risks / Trade-offs

- [Risk] Tests pass while deeper realtime state is broken. -> Mitigation: explicitly scope this change to shell and visual navigation; keep `runtime-event-replay-entity-sync-tests` as the next required change.
- [Risk] Command palette assertions are flaky because keyboard shortcuts differ by platform. -> Mitigation: open the palette through both the expected shortcut and any visible trigger if one is introduced; assert final navigation rather than internal implementation details.
- [Risk] Geometry checks become brittle. -> Mitigation: limit geometry assertions to primary shell controls and visible panel overlap checks, not exact positions or pixel-perfect dimensions.
- [Risk] Mobile menu behavior is sensitive to animation timing. -> Mitigation: wait on visible landmarks and URL changes instead of fixed sleeps.
- [Risk] Existing admin pages have duplicate headings or nav labels. -> Mitigation: use scoped locators and roles where possible, and add narrow test IDs only for shell landmarks.

## Migration Plan

1. Add the OpenSpec requirements for visual verification and modified test gates.
2. Add or adjust stable shell selectors/accessibility labels only where Playwright cannot reliably target existing UI.
3. Add targeted Playwright tests under `frontend/e2e/` for desktop navigation, mobile navigation, command palette routing, and surface visibility.
4. Run `bun run lint`, `bun run typecheck`, and the targeted Playwright suite from `frontend/`.
5. Update KBD progress and artifact-refiner QA evidence before verification/archive.

Rollback is straightforward: remove the new E2E spec and any selector-only attributes added for it. No data migration or API rollback is required.

## Open Questions

- Should the final implementation include theme toggling coverage in this change, or leave light/dark visual parity to a dedicated UI audit?
- Should command palette opening be tested only by keyboard shortcut, or should the shell add a visible command trigger for discoverability and test stability?
