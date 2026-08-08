# UAR Frontend Design Authority

This page is the conflict-resolution index for active UAR frontend work. It records which design sources govern implementation and where an approved UAR decision intentionally diverges from a source standard.

## Authority order

Use the following order when two sources conflict:

1. Approved operator decisions in the active KBD phase decision log.
2. The active KBD phase goals and plan.
3. The UAR frontend migration plan, delivered design comps, and Slash Gate brand assets named by the phase goals.
4. The vendored KnowMe UI/UX standard, except where a higher-precedence source records a scoped divergence.

Historical plans and ADRs remain evidence of earlier intent. They do not supersede a newer, explicit operator decision for active implementation.

### Portability rule for operator decisions

An operator decision does not become a portable frontend authority merely because it exists in KBD working state. When such a decision creates or changes a public design divergence, this page must reproduce its scope and rationale, link to that self-contained record, and retain the KBD path only as plaintext audit provenance. Active KBD state remains the workflow source of truth; this page is the distributable design contract.

## Recorded divergences

| Decision | Divergence | Scope and rationale |
|---|---|---|
| D1 | Base UI component primitives | [D1 scope and rationale](#d1-base-ui-component-primitives) |

### D1: Base UI component primitives

Operator decision D1 retains Base UI-backed local wrappers as UAR's owner for:

- general controls;
- navigation;
- overlays; and
- sidebars.

This is an explicit override of the KnowMe UI/UX standard §6.1 and the corresponding §6.3 ownership row, which require shadcn for those surfaces. It is a recorded divergence, not compliance with the shadcn requirement. D1 makes no claim that Base UI is less work or categorically superior.

D1's rationale is the already-landed UAR migration: the dated D1 review recorded zero direct Radix imports, 34 Base UI wrapper files, and the 24/24 completion of `base-ui-foundation`. Retaining that foundation makes the active goal match the implemented primitive boundary. The remaining Base UI composition, icon migration, dependency pruning, and final verification work stays assigned to later changes in this phase.

Control-plane provenance: operator decision D1, recorded 2026-08-07 in `.kbd-orchestrator/phases/uar-uiux-full-migration-2026-08/decision-log.md`. The rationale is reproduced here so this public authority page does not depend on KBD working-state files being present in a packaged or committed documentation checkout.

The override is narrow. All other requirements remain binding unless another approved decision explicitly changes them, including:

- Assistant UI ownership of thread, composer, and streaming behavior;
- Prometheus Entity Management ownership of durable server entities;
- Zustand ownership of transient UI state;
- PGlite ownership of client-managed conversation persistence;
- the Flat 2.0 rules, token ladder, typography, accessibility, responsive behavior, and acceptance criteria.

## Applying the authority

New or migrated general primitives must be exposed through UAR's local Base UI-backed wrapper boundary. If an older document names shadcn for a D1-covered surface, follow D1 and this page while preserving that document as historical context. Do not broaden the exception to unrelated architecture or design requirements.
