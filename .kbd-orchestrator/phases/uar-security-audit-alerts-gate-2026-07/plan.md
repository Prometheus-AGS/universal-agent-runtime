PLAN: uar-security-audit-alerts-gate-2026-07
Project: universal-agent-runtime (Universal Agent Runtime)
Date: 2026-07-08
OpenSpec available: YES
Changes to implement: 3

CHANGE LIST (ordered)

1. add-dependabot-alerts-ci-gate: add a `gh api dependabot/alerts` job to `security-audit.yml` as a required complement to `cargo audit`.
   - Scope: ci (`.github/workflows/security-audit.yml`), docs (`docs/DEPENDENCY_MANAGEMENT.md`)
   - Depends on: NONE
   - Recommended agent: Claude Code (self-executing, matches every prior change this project)
   - Est. complexity: M
   - Complexity score: Medium — new job logic + a real access-control constraint to design around, not a pure config tweak
   - Model class: frontier
   - Customer value: HIGH
   - Details: Web-verified (per Rule 22/23) that the default `GITHUB_TOKEN` can **never** read the Dependabot alerts REST API from inside Actions, regardless of any `permissions:` block — this is a hard platform limitation (GitHub community discussion #60612), not a config gap. The job must therefore read a dedicated secret (e.g. `secrets.DEPENDABOT_ALERTS_TOKEN`, a classic PAT with `security_events` scope, or a fine-grained token with "Dependabot alerts: Read"). Design it to fail loudly with a clear message if the secret is unset — mirroring this project's established `--require-baseline` fail-loud precedent from the eval harness — rather than silently skip. The job should diff the alert set against `cargo audit`'s ignore-list + `pnpm audit`/`npm audit` output and fail on anything genuinely new/undisclosed, closing the exact gap that let `cmov`/`opentelemetry_sdk` go unnoticed last phase. Update `docs/DEPENDENCY_MANAGEMENT.md` to document the new automated check (this project's established pattern for every prior dependency-security change).
   - Product decision needed before/at execute: which token source to use. **Flag to the user at execute time** — do not assume `SUBMODULES_TOKEN` (already used for submodule checkout) has adequate scope; that's a real unknown, not a safe default.

2. verify-dependabot-alerts-gate-live: push the new job and confirm a real GitHub Actions run passes with all 5 jobs (4 existing + the new one).
   - Scope: ci (verification only, no further code unless the live run surfaces a bug)
   - Depends on: add-dependabot-alerts-ci-gate
   - Recommended agent: Claude Code (self-executing)
   - Est. complexity: S
   - Complexity score: Low — mechanical verification, mirrors last phase's `push-and-verify-security-audit-workflow` change exactly
   - Model class: small
   - Customer value: HIGH
   - Details: Commit + push, then `gh workflow run security-audit.yml` (or wait for the Monday 06:00 UTC cron, next occurrence 2026-07-13) and confirm all 5 jobs pass. **Operator-only remainder, not agent-executable**: if Change 1 requires a net-new secret, a human must actually create it in repo settings before this change can go green — same "P0 operator" pattern as the eval harness's `UAR_LLM__API_KEY` gate. If no such secret exists at execute time, this change should surface that blocker explicitly rather than force a workaround.

3. migrate-vite-rolldown-codesplitting: replace `vite.config.ts`'s deprecated `manualChunks` function form with Rolldown's `codeSplitting` API.
   - Scope: frontend build config (`frontend/vite.config.ts`)
   - Depends on: NONE (independent of changes 1–2; touches unrelated files)
   - Recommended agent: Claude Code (self-executing)
   - Est. complexity: S
   - Complexity score: Low — mechanical config rename with a verified, documented migration path
   - Model class: small
   - Customer value: MEDIUM
   - Details: Web-verified (per Rule 22/23) the real migration path: rename `build.rollupOptions` → `build.rolldownOptions`, replace the `manualChunks(id) {...}` function with `codeSplitting.groups: [{ name(moduleId) {...} }]` using the same match logic already in the file. Verify with `pnpm run build` that the same 4 vendor chunk groupings (`vendor-react`, `vendor-assistant`, `vendor-query`, `vendor-hljs`) still emit, and that `chunkSizeWarningLimit` still applies.

(Goal 4 — Tailwind v4-syntax grep — is not listed as a change: assessment.md already confirmed it MET via a clean grep across `frontend/src`. No code change exists to plan.)

EXECUTION ROUND ORDER
Round 1 (parallel, no shared files): `add-dependabot-alerts-ci-gate`, `migrate-vite-rolldown-codesplitting`
Round 2: `verify-dependabot-alerts-gate-live` (depends on Round 1's CI change; blocked on an operator-supplied secret if Change 1 introduces a new one)

COMMANDS TO RUN
/opsx:new add-dependabot-alerts-ci-gate
/opsx:new verify-dependabot-alerts-gate-live
/opsx:new migrate-vite-rolldown-codesplitting

PLAN COMPLETE
