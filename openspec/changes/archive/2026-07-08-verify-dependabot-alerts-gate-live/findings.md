# Findings: verify-dependabot-alerts-gate-live

**Date**: 2026-07-08

## Live run

- **Run**: https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/28950786923
- **Trigger**: `workflow_dispatch` (manual, since the Monday 06:00 UTC cron hadn't fired yet at verification time)
- **Duration**: 2026-07-08T14:33:26Z → 2026-07-08T14:37:14Z (~3m48s)
- **Overall conclusion**: `success`

## Per-job results

| Job | Conclusion |
|---|---|
| Dependabot alerts gate | `success` |
| cargo audit | `success` |
| npm audit (root) | `success` |
| npm audit (sdks/typescript) | `success` |
| pnpm audit (frontend) | `success` |

All 5 jobs passed, including the new `dependabot-alerts-gate` job added
this phase.

## Token-scope confirmation

**`secrets.SUBMODULES_TOKEN` has sufficient scope.** The job's log shows:

```
DISCLOSED_GHSA_IDS: GHSA-q2qq-hmj6-3wpp GHSA-3v94-mw7p-v465
All 2 open Dependabot alert(s) are already disclosed.
```

The `gh api repos/{owner}/{repo}/dependabot/alerts` call succeeded from
inside the real Actions job using the reused `SUBMODULES_TOKEN` — the
fail-loud preflight check (401/403 diagnostic) never fired. The user's
`AskUserQuestion` choice to reuse `SUBMODULES_TOKEN` rather than
provision a new dedicated secret is confirmed correct on the first real
run, no further token work needed.

## Outcome

Both goals this phase set out to verify are now confirmed for real, not
just locally simulated:

1. The Dependabot-alerts CI gate exists and works.
2. It ran green on real GitHub Actions with the actual production
   credential.

No blockers surfaced. No code changes were needed beyond what
`add-dependabot-alerts-ci-gate` already shipped.
