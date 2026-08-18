# Refinement log — `resolve-sdk-distribution`

## Iteration 1 — 2026-08-18T20:09:16Z

- Specify: derived blocking constraints for release metadata, local verification
  plus publication order, and workflow scope/evidence.
- Plan: reconcile manifest authorship and install commands, preserve the Rust
  embedded dependency with an exact registry version, verify all SDKs locally,
  and remove hosted routine verification.
- Execute: corrected metadata/docs, generated the standalone Rust SDK lock,
  observed focused SDK checks, and wrote per-requirement evidence.
- Reflect: the independent critic and judge BLOCKED. The candidate removed only
  the SDK job while retaining other routine jobs in the same workflow; it did
  not explain or verify the stale lockfile reconciliation; and its refiner state
  plus validation receipt were incomplete.
- Persist: retained all three findings and selected one correction slice.
- Content type: `direct:content`; evaluation: output inspection.

## Iteration 2 — 2026-08-18T20:31:08Z

- Plan: retire the legacy all-routine CI workflow, document the standalone
  lockfile's stale root-runtime graph, verify that exact final lock once, and
  complete refiner state before requesting re-review.
- Execute: applied that correction. The final locked Rust SDK test observed 3
  unit tests and 1 doctest passing after Cargo reconciled the root path graph;
  no SDK manifest dependency was added beyond the existing runtime dependency's
  exact registry version.
- Reflect: the judge passed the correction, but the critic BLOCKED the remaining
  claim that runtime-first made the Rust SDK publishable. Root metadata proved
  the runtime itself has four path-only normal dependencies, so a two-step order
  was incomplete.
- Persist: retained the critic's finding and required an explicit complete
  prerequisite/remediation chain with no publishable-now claim.
- Content type: `direct:content`; evaluation: output inspection.

## Iteration 3 — 2026-08-18T20:38:46Z

- Plan: name the four path-only runtime dependencies, record the complete
  internal-crates → runtime → SDK release order, and change acceptance language
  from publishable-now to release-ordered.
- Execute: updated OpenSpec, per-requirement evidence, and the artifact summary
  without changing SDK behavior or publishing any package.
- Reflect: the independent critic and judge both PASS the exact iteration-3
  candidate. The artifact may terminate with all three constraints satisfied.
- Content type: `direct:content`; evaluation: output inspection.
