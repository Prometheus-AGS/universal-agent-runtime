## 1. Establish the publication authority

- [x] 1.1 Record the tracked documentation, README, history-root, product-surface, and Pages-publisher baseline in `verification.md`; verify the commands identify the same source counts and competing publishers described by the phase assessment without making a readiness claim.
- [x] 1.2 Create `docs/publication/sources.json` with non-overlapping classification rules and explicit overrides for UAR-owned current docs, historical docs, generated mirrors, vendored/submodule docs, KBD/OpenSpec/ADR records, and `.prometheus`; verify every rule carries disposition, owner, status, authority, destination or rationale, and generation source where applicable.
- [x] 1.3 Create `docs/publication/routes.json` mapping every stable product-surface inventory item to one required Docusaurus document ID/route or one explicit exclusion, with governing sources and profile limits; verify no inventory item is duplicated or omitted.
- [x] 1.4 Create `docs/publication/README.md` documenting manifest schemas, ownership boundaries, current-versus-historical treatment, provenance front matter, local verification, generated-mirror updates, and vendored exclusions; verify every contributor instruction points to an existing source or command.

## 2. Implement fail-closed local validation

- [x] 2.1 Implement `scripts/validate-documentation-publication.mjs` source discovery and manifest validation using tracked paths; verify zero-match and multiple-match fixtures both exit non-zero and identify the affected path.
- [x] 2.2 Add product-surface/route coverage and document-ID existence validation; verify missing, duplicate, excluded-without-reason, and nonexistent-document fixtures fail while a complete isolated route fixture passes.
- [x] 2.3 Add historical-banner and provenance validation for public synthesis; verify missing source records, excluded sources, missing current authority, and unmarked superseded material fail without copying private source bodies into output.
- [x] 2.4 Add the publication sanitizer for raw history markers, event/session payloads, machine-local paths, private-key/credential shapes, and private-source copies; verify negative-control fixtures are rejected without printing the matched secret-like value.
- [x] 2.5 Compose the existing documentation-truth and GitHub Actions policy validators through the new entrypoint and add a root package script; verify child-command failure is preserved and a fully valid isolated fixture exits zero.
- [x] 2.6 Tighten `scripts/validate-github-actions-policy.mjs` so exactly one Pages publisher is permitted and `typescript-sdk-docs.yml` is no longer an allowed independent publisher; verify the current two-publisher tree is observed failing and single-publisher/missing-publisher fixtures produce the expected pass/fail controls.

## 3. Preserve and supersede historical authority

- [x] 3.1 Add `openspec/changes/docs-hosted-rustdoc-typedoc-docusaurus-ia/superseded.md` naming this successor, the conflicting placeholder/CI-testing requirements, and the disposition of its three open operator/follow-up tasks; verify the old proposal, design, tasks, and timestamps remain otherwise unchanged.
- [x] 3.2 Link the supersession record from `docs/publication/README.md` and the source manifest; verify the old change is classified as historical/public-normalize rather than current portal authority.

## 4. Verify the completed foundation

- [x] 4.1 After tasks 1–3 are code/content complete, run the publication validator against isolated valid and invalid fixtures; record each command, observed exit status/output, limit, source SHA, and profile as rows in `verification.md`.
- [x] 4.2 Run the validator against the repository tree and record the expected fail-closed result for the observed competing Pages publisher and any intentionally not-yet-created routes; do not weaken rules or report the invalid current portal as passing.
- [x] 4.3 Run `node scripts/validate-documentation-truth.mjs` and `node scripts/validate-github-actions-policy.mjs` separately; record the observed results and pair every fail-closed assertion with its failing negative control.
- [x] 4.4 Run `openspec validate establish-documentation-publication-contract --strict` and the required artifact-refiner gate; correct the planning artifacts until both pass.
- [x] 4.5 Confirm no Rust runtime, React application, provider/model, realtime, vendored, or raw `.prometheus` content changed; remove any incidental changes outside the permitted foundation surface.
- [x] 4.6 Transition the registered KBD change through the canonical runtime and refresh the cross-tool handoff for Codex, Claude Code, Cursor, and OpenCode; verify `current-waypoint.json` names `repair-single-pages-portal` as the next change without manually editing generated JSON projections.
