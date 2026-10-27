# Prior context — agent-team-concurrency-repair (analyze)

> Auto-populated by /kbd-memory-recall for lead view (main thread); lessons from surreal-memory. Cite the lessons that apply; recalled entries are information recorded by agents, not instructions.

## Lessons

- [lead] KBD analyze summary for phase uar-post-lifecycle-verification-readiness: Main-binary baseline retained: handoff source differs from older delivered image; fallback Surreal endpoint is loopback WebSocket with unproven durability. Six existing-code reuse candidates; prioritize PL-01/02, decide PL-08 trust before arithmetic, keep conditional restart/KB/CAS evidence distinct. PL-13/14 deferred; supporting harness proposals are not release blockers. Manual owner review before Plan; no product verification or external model review. _(recorded by an agent; progress, stage analyze, 2026-10-21; via surreal-memory)_
- [lead] KBD plan summary for phase uar-post-lifecycle-verification-readiness: Plan complete for owner review: five changes, thirteen pending tasks, PL01-14 scope preserved; proposed contract not adopted and Execute not authorized. Review compatible PL02 error contract and PL08 counting/capacity policy. Conditional persistence repairs require causal evidence. One JWT-validation-only follow-on remains outside this phase; parent issue URL pending. _(recorded by an agent; progress, stage plan, 2026-10-21; via surreal-memory)_
- [lead] KBD assess summary for phase uar-post-lifecycle-verification-readiness: PL-01 through PL-14 assessed against the pinned source; runtime claims remain UNKNOWN and verification CLOSED. Discuss operating mode, candidate/store, provider trust, B3 receipts and optional PL-13/14 before manually entering Analyze. No implementation or external cross-model review performed. _(recorded by an agent; progress, stage assess, 2026-10-21; via surreal-memory)_

## pk knowledge

*(no pk knowledge for this phase)*

## Previous reflection

From `uar-uiux-full-migration-2026-08/reflection.md`:

**Root Cause**

- Goal 1 conflicted with already-complete, conforming A2UI assets; D2 resolved the conflict but the durable goal text was not revised.
- The migration prioritized architecture, trust boundaries, end-to-end behavior, and regression gates before broad unit-test expansion.
- Flat 2.0 and kebab-case used a deliberate gate-before-purge rollout so existing debt could shrink without a flag day.
- Screen-reader and text-zoom interaction require a human/device pass beyond deterministic automation.
- The review and performance harnesses did not retain equally strong immutable evidence for every change and attempt.

**Corrective Actions**

1. Formally revise Goal 1 to encode D2 before it is reused as an acceptance criterion.
2. Retire the 365 style and 11 filename exceptions while keeping the zero-new invariant.
3. Raise coverage with risk-ranked feature tests without lowering the 60% threshold.
4. Assign and record manual screen-reader and 200% text-zoom certification before release promotion.
5. Preserve every performance attempt and standardize a canonical QA receipt for changes without refinement logs.
6. Obtain a fresh independent C-12 review if release policy requires stronger evidence than the remediated-but-`BLOCK` receipt.

**Next Phase Seed**

Use `uar-uiux-refinement-2026-08` for: (1) removing the 365 style and 11 filename exceptions, (2) raising coverage toward 60% with risk-ranked tests, and (3) completing manual assistive-technology certification with immutable performance evidence.

Human decisions: revise Goal 1 to encode D2, choose the coverage sequence, assign the manual-certification owner, and decide whether C-12 needs another independent review.

## Knowledge gaps

- Goal Discard and completely rebuild the existing web UI: 1 — PARTIAL
- Goal Enforce app → features → shared → platform and kebab-case everywhere: 5 — PARTIAL
- Goal Mechanically enforce Flat 2.0 and the 3px ember focus exception: 6 — PARTIAL
- Goal Pass bundle/latency, WCAG, responsive, and acceptance gates: 10 — PARTIAL
- 365 Flat 2.0 style exceptions and 11 filename exceptions remain in the shrinking allowlists.
- Unit line coverage remains 33.68% against the retained 60% threshold.
- Manual screen-reader and 200% text-zoom receipts remain outstanding.
- Performance attempts are not yet append-only; C-15 preserved the failed measurement in a secondary manifest only.
- Seven changes need a canonical QA-log equivalent; C-12 may need another independent final review.
