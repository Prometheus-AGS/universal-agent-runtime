# Assessment A — agent-selection notice

Method: isolated source/design assessment A. Phase: uar-post-lifecycle-verification-readiness, user-outcome-implementation. Verification CLOSED. No browser, detector, CLI engine probe, tests, checks, builds, services, or model calls were run. This assessment is a pre-edit design critique and gives no runtime or visual acceptance score.

## Design specificity and overall impression

The direction fits UAR's governed operator chat. A selected specialist is a routing fact worth exposing in the answer stream, and “Selected agent: {title}” describes that fact without promising completion. The existing chunk surface is the appropriate visual scope. The uncomfortable scenario is that the persisted notice looks correct after completion while the live message path still hides it, duplicates it, or announces it again on reconnect. A renderer alone cannot establish the requested outcome.

Evidence read: frontend/PRODUCT.md; the selection UI design handoff; frontend/src/features/chat/ui/chunks/{chunk-surface,chunk-renderer,skill-chunk,context-chunk}.tsx; frontend/src/platform/agui/{runtime-chunk-projection,agui-normalizer,agui-adapter}.ts; frontend/src/shared/content/to-chunks.ts; AGENTS.md and current-waypoint.json. No other assessment was read.

## Prioritized findings

1. **P1 — Preserve the selection through both live rendering and message persistence, not just the runtime projection.** The design says legacy/native paths normalize identically, but the existing adapter gives legacy agui.* events no normalizer/runtimeChunk projection. The existing shared content-block projection has no agent-selection case and manufactures message/index chunk IDs. Meanwhile ChunkRenderer explicitly hides raw/step chunks. Correction: name the exact event-to-visible-message reduction and persisted content-block path in the implementation packet; preserve event correlation through that path, and keep the live notice on the same semantic chunk kind after finalization/reload. A trace-only projection or generic CUSTOM fallback does not satisfy the user outcome. This is an implementation-contract gap, not evidence that a proposed component is already defective.

2. **P2 — “One polite live announcement” needs an explicit replay rule.** ChunkSurface's live prop produces aria-live="polite" on the mounted surface; it does not itself distinguish new selection from historical/replayed selection. UarAguiAdapter's seen set is scoped to one adapter instance, so source inspection does not establish announcement suppression across a fresh connection or rehydration. Correction: specify that only a newly accepted live selection announces, while persisted/replayed history remains readable without an unsolicited announcement; keep focus unchanged. Use the existing stream/message identity rather than a component-level duplicate entity store. Actual assistive-technology behavior is unverified.

3. **P2 — Ordinary wrapping does not resolve an unbroken fallback ID or specialist name.** The design explicitly falls back to actual agent ID and asks for long-name support. Existing skill/context siblings use flex rows; copying those rows with an unrestricted text span does not establish narrow-screen wrapping. Correction: use a shrinkable text boundary (min-width: 0), overflow wrapping for unbroken names/IDs, and a nonshrinking decorative icon. Keep the full escaped value available in normal text; do not rely on hover, truncation, or horizontal scrolling. Mobile/200% zoom rendering remains unverified.

## Fact-based strengths

- The design distinguishes agent selection from skill activation and completion; it avoids carrying SkillChunk's active/complete status or a success checkmark into the new fact.
- It reuses ChunkSurface's existing surface/foreground tokens and spacing instead of introducing another card system, colors, or animation.
- Immutable typed props and transport-owned normalization preserve the specified entity boundary; the proposed component does not need fetching or a parallel agent record.

## Cognitive load and emotional fit

There is one informational notice and no additional decision, so no >4-option decision point is introduced. Keeping revision/thread IDs in inspectable metadata avoids flooding the chat. “Intent-based routing” is secondary technical detail; render it quietly and only when supported by the actual method rather than claiming a reason beyond the event. The primary emotional benefit is early reassurance that a specialist was selected. Do not phrase selection as “working” or “completed” without separate evidence.

## Heuristic disposition

No numeric design-health score is issued because this is an unrendered proposal under a closed verification gate. Source supports system-status wording, consistency with existing surfaces, and recognition of selected identity. User control and efficiency are not new interactions on this read-only notice. Contrast, actual hierarchy, accessible announcement behavior, error recovery, help discoverability, zoom/mobile layout, and theme parity remain unverified. A source-only disposition must not be promoted to a rendered score.

## Persona red flags

- **Sam, assistive-technology user:** a replayed selection could be announced as new unless live/history semantics are explicit; the icon must remain decorative and selected identity must remain text.
- **Riley, edge-case operator:** a long unbroken identifier needs overflow wrapping, and reload/finalization must not make the selected agent disappear or duplicate it.
- **Alex, operator:** internal routing metadata in the main sentence would compete with the answer; the proposed concise identity plus secondary method avoids that if implemented as specified.

## Handoff and remaining limits

Only this assessment file was authored. Production source was not changed. No unsolicited production guards were added. Findings trace to the explicit streamed-visibility, replay, accessibility, and long-name requirements. Machine/visual audit and runtime streaming/reconnect acceptance remain deferred by the root operator lock. No rendering PASS is claimed.

Questions skipped: approved intent is fixed; the corrections resolve the specified streamed-selection outcome without requiring a new product preference.
