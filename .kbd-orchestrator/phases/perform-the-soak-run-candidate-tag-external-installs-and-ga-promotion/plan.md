# Plan: perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion

- **Date**: 2026-07-13
- **Backend**: OpenSpec (all 10 new changes created and validated; 4 certification changes carried from prior phase)
- **Inputs**: assessment.md (6 CRITICAL / 7 HIGH / 8 MEDIUM), decision-log.md D1 (multi-tenant: fix C1–C3)
- **Total phase changes**: 14 (10 new + 4 carried)

## Ordering rationale

Security first (isolation is D1's mandate and gates everything customer-facing), then
product-claim gaps (skills, RAG, orchestrator, AG-UI — the operator's functional matrix),
then distribution surfaces (SDKs, docs), then validation with video (needs the fixes
landed to validate them), then the certification tail exactly as goals §1–4 specify.
Changes 1–2 and 3–7 are internally parallelizable batches; 10 hard-depends on 1–7.

## Ordered change list

| # | Change | Closes | Recommended agent |
|---|--------|--------|-------------------|
| 1 | `fix-user-isolation-sessions-memory-kb` | C1 C2 C3 M8/O4 | rust-reviewer-backed implementation + security-reviewer |
| 2 | `harden-jwt-defaults` | H4 | security-reviewer |
| 3 | `fix-skills-scope-semantics` | H2 H3 O1 M4 | general-purpose + rust-reviewer |
| 4 | `ship-skill-pack-install-path` | C5 | general-purpose |
| 5 | `emit-rag-retrieval-events` | H1 H6 M2 O2 | general-purpose + rust-reviewer |
| 6 | `wire-orchestrator-delegation` | C6 | general-purpose + rust-reviewer |
| 7 | `complete-agui-event-parity` | M1 | general-purpose |
| 8 | `resolve-sdk-distribution` | C4 | typescript-reviewer + rust-reviewer (decision task first) |
| 9 | `rewrite-readme-and-docs` | H5 M7 | doc-updater + frontend-design for diagrams |
| 10 | `screen-by-screen-validation` | M6 M3 + operator matrix | e2e-runner (BDD + video bundles) |
| 11 | `certify-operational-resilience` (carried) | goals §1 | local immutable-candidate run + evidence |
| 12 | `produce-supply-chain-artifacts` (carried) | goals §2 | local build and verification + evidence |
| 13 | `certify-release-candidate` (carried) | goals §3 | operator (tag) + external/time-bound |
| 14 | `release-1-0-0` (carried) | goals §4 | operator (GA promotion) |

## Notes

- Changes 11–14 are implementation-complete from the prior phase; they re-enter the
  queue as **evidence/publication** work and MUST rerun certification because changes
  1–9 modify source after the prior validation (per certify-release-candidate's
  "Candidate source changes" scenario).
- Change 8 starts with a decision task (ship minimal SDKs vs withdraw from 1.0) —
  surface to operator before implementation.
- Change 10's per-screen matrix is seeded by the assessment's 20-screen inventory.
- Plan revision 7 supersedes the no-source-change default only for the three
  operator-approved defects observed by change 10: Skills graph visibility,
  approval-event projection, and Knowledge nested-interactive markup. The repairs stay
  bounded to those surfaces and force a fresh immutable certification run.
- Plan revision 8 restores the standing deployment-only GitHub Actions boundary.
  Product, installed-artifact, supply-chain, load, stress, soak, and release
  certification run locally. GitHub Actions remain available only for actual
  deployment execution and deployment-specific validation.
- Plan revision 9 separates source preparation from evidence execution. Land
  and locally verify all scripts, manifests, schemas, and documentation needed
  by changes 11–14 before freezing change 11's immutable candidate. Evidence
  still executes in order 11–14. After the freeze, only evidence/checkpoint
  commits are permitted; any later source change invalidates the three-hour run.
- Plan revision 11 and decision `functional-real-inference-closeout-only`
  supersede changes 11–14. The operator replaced the elapsed-time, supply-chain,
  RC, and publication tail with five bounded real-model functional paths, each
  observed through both the packaged API boundary and the shipped UI. Changes
  11–14 are cancelled rather than represented as passed.
- Mastra-inspired enhancements (inline trace view, in-chat model switching, evals
  panel) are deliberately NOT in this phase — recorded as candidates for the next
  phase to avoid scope creep on the release path.
- Deferred MEDIUMs: M3 thread-sync-to-server (documented local-only for 1.0 unless
  operator objects), M5 context token-limit hardcode (fold into 3 if trivial).

## Next

`/kbd-reflect perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion`
