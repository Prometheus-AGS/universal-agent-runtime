---
target: provider settings panel
total_score: 29
max_score: 40
na_heuristics:
p0_count: 0
p1_count: 1
timestamp: 2026-08-26T23-55-24Z
slug: features-settings-ui-panels-ai-settings-panels-tsx
---
Method: dual-agent (A: impeccable_design_assessment · B: impeccable_detector_assessment)

## Design Health Score

| # | Heuristic | Score | Key issue |
|---|---|---:|---|
| 1 | Visibility of System Status | 4 | Loading, saving, dirty, success, and error states are surfaced. |
| 2 | Match System / Real World | 3 | Technical provider terminology assumes operator fluency. |
| 3 | User Control and Freedom | 2 | No explicit discard/reset path for dirty drafts. |
| 4 | Consistency and Standards | 3 | The viewport grid ignores the nested panel's actual width. |
| 5 | Error Prevention | 3 | Strong constrained choices; URL validity is not visibly constrained. |
| 6 | Recognition Rather Than Recall | 4 | Labels, hints, selected values, and searchable model choices keep context visible. |
| 7 | Flexibility and Efficiency | 2 | No reset, collapse, filter, or batch path for many providers. |
| 8 | Aesthetic and Minimalist Design | 3 | Compact cards, but the two-column grid can become cramped. |
| 9 | Error Recovery | 3 | Drafts survive failures; errors remain page-level. |
| 10 | Help and Documentation | 2 | Base URL, protocol, and config-file guidance are sparse. |
| **Total** | | **29/40** | **Good** |

## Design Specificity Verdict

The content is authored for an LLM runtime through provider keys, protocols, credentials, model selection, and operator status language. The card-and-grid composition is otherwise category-interchangeable. The deterministic detector returned zero findings for `ai-settings-panels.tsx`. Browser inspection confirmed the live five-provider surface, but it came from `/Users/gqadonis/.uar` rather than this working tree; mutable injection was unavailable, so no overlay or working-tree visual proof is claimed.

## Overall Impression

The provider surface communicates state well and has a solid accessible source structure. Its main scoped defect is architectural rather than decorative: a viewport breakpoint decides a layout whose usable width is owned by a nested settings panel.

## What's Working

- Loading, refreshing, saving, dirty, success, and error states are clear and semantically announced.
- Provider cards group identity, enabled state, and related controls into a compact, scannable unit.
- Masked credentials, constrained protocol choices, searchable model lists, and unavailable-model guidance reduce common operator errors.

## Priority Issues

1. **[P1] Field layout responds to viewport width, not provider-panel width.** In a wide browser with a constrained settings detail region, `lg:grid-cols-2` can force cramped columns. Replace only the existing layout classes with a named container query and prove the actual container boundary in a browser.

## Out-of-scope observations

- **[P2]** Dirty drafts have no explicit discard/reset path. This change must not add one.
- **[P2]** Blanket `opacity-60` on disabled providers can imply that still-editable controls are unavailable. This change must not alter that presentation.

## Persona Red Flags

- **Alex (power user):** broad edits have no reset/discard path, but the global Save flow is efficient.
- **Jordan (first-timer):** Base URL, protocol variants, raw provider keys, and config-file instructions assume technical knowledge.
- **Sam (accessibility-dependent):** source semantics are strong; computed focus visibility, zoom reflow, and disabled-card contrast remain browser-verification concerns.

## Minor Observations

- Provider-list scale may raise cognitive load because every provider remains expanded.
- Long provider headings and keys need browser overflow evidence.
- The focused change should preserve the incumbent visual tokens and state behavior exactly.

## Questions to Consider

- Should responsiveness follow the screen, or the exact space the settings panel has been given?
- Which browser evidence most directly proves that keyboard focus and long controls remain inside the provider card at the boundary?
