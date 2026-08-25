---
target: Provider Overrides model picker and API-key masking
total_score: 25
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 1
timestamp: 2026-08-25T14-28-48Z
slug: features-settings-ui-panels-ai-settings-panels-tsx
---
Method: dual-agent (A: /root/impeccable_design_review · B: /root/impeccable_detector_review)

## Design Health Score

| # | Heuristic | Score | Key Issue |
|---|---|---:|---|
| 1 | Visibility of System Status | 2 | Save/error feedback is not live-announced |
| 2 | Match System / Real World | 3 | Provider/model language is clear |
| 3 | User Control and Freedom | 2 | Refresh can discard edits without a dirty-state warning |
| 4 | Consistency and Standards | 4 | Uses repository shadcn/Base UI primitives |
| 5 | Error Prevention | 3 | Bounded models prevent invalid free-text values |
| 6 | Recognition Rather Than Recall | 3 | Display labels with ID fallback reduce recall |
| 7 | Flexibility and Efficiency | 2 | Large model inventories are not searchable |
| 8 | Aesthetic and Minimalist Design | 3 | Compact, coherent provider cards |
| 9 | Error Recovery | 2 | Save failures provide limited recovery guidance |
| 10 | Help and Documentation | 1 | Sparse inline help for provider configuration |
| **Total** | | **25/40** | **Acceptable; targeted hardening needed** |

## Design Specificity Verdict

The panel is moderately product-specific: provider IDs, protocols, model ownership, and credential handling are domain-grounded. The deterministic detector returned zero findings for the target file. Manual review found accessibility and responsive issues outside the detector's model. Browser overlay injection was unavailable, so source, screenshot, focused tests, and two isolated reviews were used.

## Overall Impression

The bounded model picker is the right interaction and now handles valid, duplicate, empty, and stale inventories explicitly. The largest remaining opportunity is making the surrounding save/refresh lifecycle more legible and recoverable.

## What's Working

- The model is selected from provider-owned enabled models with display-name fallback.
- Empty inventories are explicit and disabled; stale defaults show recovery guidance.
- The existing shadcn/Base UI select supplies keyboard and focus behavior.
- API keys retain exact character-count masking without exposing plaintext.

## Priority Issues

- **[P1] Save and error feedback lacks live semantics.** Add `role="status"` for success/loading and `role="alert"` for errors. Suggested command: `$impeccable harden`.
- **[P2] Dirty-state and Refresh behavior are unclear.** Add a dirty indicator and confirm before destructive reload. Suggested command: `$impeccable clarify`.
- **[P2] Long model inventories do not scale.** Introduce a searchable combobox only beyond an evidence-based threshold. Suggested command: `$impeccable optimize`.
- **[P2] Existing field labels and two-column layout need a broader accessibility/responsive pass.** Suggested command: `$impeccable audit`.

## Persona Red Flags

- **Alex, power user:** long inventories lack search and dirty state is not obvious.
- **Sam, screen-reader user:** the model trigger is named and exposes invalid state; surrounding fields and save/error banners still have incomplete semantics.
- **Riley, intermittent-network operator:** failure is visible but recovery guidance and confirmation of preserved local edits are weak.

## Minor Observations

Disabled provider-card opacity may reduce text contrast. The fixed save-flash timeout can overlap on rapid saves. Duplicate model IDs are deduplicated first-occurrence-wins.

## Questions to Consider

- Should the next pass broaden into save/refresh accessibility and dirty-state protection?
- At what model count should the select become searchable?
- Should the provider form receive responsive layout and label-association work as a separate change?
