# Verification: select-and-observe-presentations

Date: 2026-09-05. Local phase-end evidence; no client-display or release claim.

| Dimension | Result |
| --- | --- |
| Completeness | 4/4 tasks; 2/2 requirements implemented |
| Correctness | All seven delta scenarios mapped below |
| Coherence | Host-owned snapshots/receipts; typed leaf-level UI subscriptions |

## Requirement and scenario evidence

`src/uar/a2ui/presentation_selection.rs` distinguishes omitted negotiation,
support-only Auto, mode-only missing support, requested Text/A2UI/Hybrid and
parent ceilings. Its full negotiation matrix passed.

`src/uar/runtime/presentations.rs` captures validated template content and
identities, retains single-use host preparations, and separates publication
receipts from unconfirmed display. `a2ui_output.rs` enforces the ceiling at the
host boundary and validates a whole preparation before replay/event writes.

- Legacy client and unsupported mode: resolver tests plus actual HTTP/SSE runs verify legacy, text and incompatible-profile outcomes.
- Changed template and disabled/deleted template: `admitted_contents_survive_edit_disable_delete_and_child_narrowing` verifies immutable admitted content, revisions and narrower descendants.
- Text ceiling on legacy tools and non-tool paths: host tests cover renderer, artifact, direct state and reserved projection ingress; forged producer output cannot manufacture a receipt.
- Selection without publication: tests preserve empty template receipts and terminal fallback without claiming client display. Policy-summary diagnostics are explicitly excluded from generated-surface publication.

`runtime/presentation_history_tests.rs` and `api/sse.rs` tests establish receipt
ordering, ring-eviction retention, historical cursor bounds, independent
provenance replay and exact tenant stream access. The frontend provenance domain
tests cover decoding, subscription leases, atomic ingestion and stale callbacks.

## Commands and observed results

- Full `cargo test --locked --no-default-features --features server-full`: exit0; library744passed/1ignored, BDD9scenarios49steps, broad integration94passed/1ignored, doctests26passed/17ignored.
- Four later catalog API regressions:4passed,0failed; formatting passed.
- Full frontend unit suite:462passed/82files before the final wording/layout correction.
- Final `pnpm typecheck && pnpm lint`: exit0.
- Final targeted `presentation-run-details.test.tsx` and `run-trace-ui.test.tsx`:21passed,8.52s.
- Final `pnpm build`: exit0,14.20s; four pinned PGlite eval warnings retained.
- `openspec validate select-and-observe-presentations`: valid.

Five real host/local-stub HTTP/SSE cases passed strict requested/effective mode,
fallback, terminal and publication assertions. A2UI/hybrid emitted artifacts,
frozen revision1 receipts and textual summaries. Browser legacy chat finished;
its run details survived full page reload. Final browser measurements at390px
show trace client/scroll width390 and local JSON pane width342; desktop document
width1440. Tabs stack vertically and keyboard traversal plus Enter activation
works. Captures are under `/tmp/uar-presentation-evidence.gBZ6BG/`.

## Findings

No critical issue remains for the specified local contract. Two observed UI
defects were corrected after independent critiques/adversarial review: overly
broad publication wording and intrinsic-width/tab-direction overflow. Independent
finish review cleared the correction. No backend semantics changed for wording.

Warnings: browser transport remains legacy; negotiated clients were exercised
through the real HTTP contract, not fabricated browser state. Full200% zoom,
numerical contrast, every assistive technology and every browser are unverified.
Live-provider429, live-peer, billing and release certification remain outside
this receipt. The prior fixture credential incident still requires operator
rotation. Do not convert these limitations into passed claims at archive.

Ready for spec sync/archive with the recorded warnings and operator approval.
