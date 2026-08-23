# Execution — uar-branded-documentation-site

The canonical runtime state is `.kbd-orchestrator/current-waypoint.json`; this
file records the human-readable implementation handoff.

## Accepted baseline

- Source SHA: `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70`
- Worktree: `~/.claude/worktrees/uar-branded-documentation-site`
- Verification remains local until documentation code and content are complete.
- GitHub Actions owns deployment execution and deployed-artifact validation only.

## Completed changes

### establish-documentation-publication-contract

- Added authoritative source and route manifests.
- Added the composed fail-closed publication validator and isolated controls.
- Tightened the Pages-publisher policy to exactly one workflow.
- Preserved the earlier portal change and added an explicit supersession record.
- Recorded row-form evidence and a converged artifact-refiner review.
- Canonical KBD revision `340` records the change complete.

The current repository is intentionally not publication-valid: 20 planned
documents are not written yet, 11 existing source documents require later
normalization or exclusion, and two workflows still compete to publish Pages.
Those failures are dependencies, not suppressed findings.

### repair-single-pages-portal

- Replaced the mixed npm/pnpm site build with the frozen npm command chain.
- Added fail-closed Rust and TypeScript reference staging.
- Removed the competing TypeScript SDK Pages workflow.
- Made `docs.yml` the sole publisher with deployed-route validation only.
- Recorded focused local controls and explicitly deferred the full site build
  and live deployment to the final phase gate.
- Canonical KBD revision `343` records the change complete.

## Next change

Run `/opsx:new brand-uar-docusaurus-site`. It owns theme tokens, brand assets,
homepage, navigation, responsive behavior, and local search.
