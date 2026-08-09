---
type: Reference
id: ssr-frontend-merge-deploy-and-workflow-validation-status
title: SSR frontend merge deploy and workflow validation status
tags:
- ssr-frontend
- aks-deployment
- github-actions
- bdd-testing
- testid-drift
- county-table
- remediation
sources:
- stdin
timestamp: 2026-07-27T10:40:09.881171+00:00
created_at: 2026-07-27T10:40:09.881171+00:00
updated_at: 2026-07-27T10:40:09.881171+00:00
revision: 0
---

## Context

- Project: **San Saba Royalty — SSR Frontend**
- KBD root: `/Users/gqadonis/Projects/sansaba/ssr-frontend`
- Phase: `july-2026-remediation-full-parity`
- Captured: `2026-07-27T10:32:58Z`
- Source: `manual:San Saba Royalty — SSR Frontend/july-2026-remediation-full-parity`

## Merge status

All post-merge checks observed on the merge commit were green. The frontend change is deployed to AKS and live.

## Workflow results

### `Build & Deploy SSR Frontend`

All 4 jobs passed:

- `Validate Secrets` ✓
- `Build & Push Image` ✓
- `Build & Push Gotenberg Sidecar` ✓
- `Deploy to AKS` ✓

Result: deployment to AKS succeeded.

### `testid-drift-detection`

Passed.

This is important for the remediation change because it included both:

- Added `data-testid` attributes, including:
  - `acq-unit-in-pay`
  - `acq-county-deed-sent`
  - `acq-county-recording-book`
- Removed Net Royalty inputs.

Passing drift detection confirms the test IDs remained consistent after those UI/form changes.

## Workflow caveats

### `bdd-fast`

- Did **not** run on the merge commit.
- Reason: workflow is `pull_request`-only.
- It ran before merge on PR `#13`, not on the push to `main`.
- No failure is implied; it is simply not part of the post-merge path.

### `bdd-thorough`

- Shows consecutive failures, but all on commit `1bb1003d`.
- `1bb1003d` predates this remediation work.
- Workflow schedule: nightly cron at `0 2 * * *`.
- It had not yet picked up the merge at the time of capture.
- The first nightly run on the new code should be the next `02:00 UTC` run.

Assessment: the nightly `bdd-thorough` failures are pre-existing and unrelated to the deployed change, but should be investigated separately because `main` has been failing the thorough BDD suite nightly for at least six runs.

## Deployment risk / follow-up

The deploy went directly to AKS, so the County table read-merge behavior is now live before the manual reload-persistence check from the PR was performed.

Recommended follow-up:

- Run verification against the deployed environment for County table read-merge behavior.
- Specifically confirm reload persistence in the live AKS deployment.

# Citations

1. stdin