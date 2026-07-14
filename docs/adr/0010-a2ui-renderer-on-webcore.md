# 10. Build the A2UI renderer on webcore with React and shadcn/ui

Date: 2026-07-13

## Status

Accepted

## Context

The UAR operator UI is React 19. The A2UI renderer must feel native to the existing design system, accessible, and performant. The operator selected shadcn/ui as the baseline and `react-aria-components` for accessibility primitives.

## Decision

- Build the UAR A2UI renderer in `frontend/packages/a2ui-uar/` on top of `@a2ui/web_core`.
- Use shadcn/ui as the baseline component library.
- Use `react-aria-components` for accessible primitives.
- Include the catalog of 14+ components: the 9 from `uar.a2ui/1` plus `EntityCard`, `EntityDiff`, `EntityStream`, `EntityApproval`, `EntityToolProvider`, `EntityChat`, and `EntityCopilot`.
- Cross-test the renderer against the vendored `@a2ui/react` reference implementation.
- Enforce performance budgets: initial render < 16ms, streaming chunk < 8ms.

## Consequences

- A2UI components match the UAR operator UI aesthetic and accessibility standards.
- Performance budgets are validated in CI.
- The renderer is independent of the reference implementation while remaining compatible with the A2UI spec.

## Alternatives considered

- Use `@a2ui/react` directly as the renderer: rejected because the operator wants UAR-owned components that can evolve with the product.
- Build on a different UI library: rejected because shadcn/ui is already the design baseline in the frontend.
