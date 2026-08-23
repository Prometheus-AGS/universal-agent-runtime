# Execute — `uar-native-service-deployment`

**Entered:** 2026-08-23
**Plan authority:** `plan.md`
**Backend:** OpenSpec
**Status:** in progress at change 1

## Serial change ledger

| Sequence | Change | Worktree | Status | Commit |
|---:|---|---|---|---|
| 1 | `establish-native-service-deployment-contract` | `~/.claude/worktrees/establish-native-service-deployment-contract` | pending | pending |
| 2 | `implement-native-service-runtime-support` | `~/.claude/worktrees/implement-native-service-runtime-support` | pending | pending |
| 3 | `package-native-service-installers` | `~/.claude/worktrees/package-native-service-installers` | pending | pending |
| 4 | `bootstrap-native-provider-model-configuration` | `~/.claude/worktrees/bootstrap-native-provider-model-configuration` | pending | pending |
| 5 | `document-and-deploy-native-services` | `~/.claude/worktrees/document-and-deploy-native-services` | pending | pending |

Each worktree starts from the preceding merged commit. A later change never merges sideways into an earlier worktree.

## Verification timing

Changes 1–4 run only their required cheap static/Tier 0 gates. Change 2 includes a compile-only Windows target check because the Windows-owned source must compile before its commit. Functional binary execution begins only after changes 1–4 are code-complete, inside change 5. No unit-test campaign, synthetic inference, soak, or multi-hour run is permitted.

## Adversarial review receipt

- Critic: MiniMax M3, fresh REST context, verified distinct from producer; round-2 verdict PASS.
- Judge: Kimi K3, fresh REST context, verified distinct from producer; two-round correction cap reached. Its final critical and warnings were corrected in `plan.md` and the affected task contracts; no finding was silently waived.
- Anti-sycophancy screening passed for all retained reports.

## Stop conditions

Stop before reflection on any stop condition in `plan.md`, especially loss of existing configuration/database authority, non-loopback listeners, secret disclosure, Windows SCM requiring a different shutdown architecture, or a required real-provider failure.
