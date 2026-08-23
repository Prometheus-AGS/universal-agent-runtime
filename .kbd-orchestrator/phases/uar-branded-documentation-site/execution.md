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

## Next change

Run `/opsx:new repair-single-pages-portal`. It owns npm-only site build wiring,
generated API-reference staging, removal of the competing publisher, and the
single deployment-only Pages workflow.
