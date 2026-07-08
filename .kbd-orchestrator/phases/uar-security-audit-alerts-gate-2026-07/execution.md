EXECUTION: uar-security-audit-alerts-gate-2026-07
Project: universal-agent-runtime (Universal Agent Runtime)
Date: 2026-07-08
Selected backend: openspec
Dispatched to: SELF (Claude Code CLI, self-executing via /kbd-apply)
Backend rationale: OpenSpec directory exists at project root and is this project's established backend for every prior phase; all 3 changes are well-bounded, single-session slices with no need for a separate decomposition tool. No reason to deviate from the established pattern.
Backend entrypoint: /kbd-apply <change-id>, one change at a time, per this project's standing rule to never invoke bare /opsx:apply.
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/uar-security-audit-alerts-gate-2026-07/plan.md

EXECUTION SCOPE

- add-dependabot-alerts-ci-gate: add a gh api dependabot/alerts job to security-audit.yml as a required complement to cargo audit; update docs/DEPENDENCY_MANAGEMENT.md.
- migrate-vite-rolldown-codesplitting: replace vite.config.ts's deprecated manualChunks function form with Rolldown's codeSplitting API.
- verify-dependabot-alerts-gate-live: push + confirm a real GitHub Actions run passes with all 5 jobs.

DISPATCH CONTRACTS

- add-dependabot-alerts-ci-gate → SELF (claude-code)
  Entry: /opsx:new add-dependabot-alerts-ci-gate (scaffold), then /kbd-apply add-dependabot-alerts-ci-gate
  Model class: frontier
  Concrete model: claude-sonnet-5 (session default; no model_policy in project.json, frontier fallback applies)
  Model rationale: designing the token-source approach and fail-loud behavior for a real access-control constraint (GITHUB_TOKEN cannot read Dependabot alerts at all) is a genuine design decision, not mechanical config — matches plan.md's Medium complexity score.
  Progress file: .kbd-orchestrator/phases/uar-security-audit-alerts-gate-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing.
  BLOCKING PRODUCT DECISION: which token source to use for reading Dependabot alerts (GITHUB_TOKEN cannot do this under any permissions: configuration — confirmed during planning via GitHub community discussion #60612). Must be resolved with the user via AskUserQuestion before this change's core implementation step, not defaulted.

- migrate-vite-rolldown-codesplitting → SELF (claude-code)
  Entry: /opsx:new migrate-vite-rolldown-codesplitting (scaffold), then /kbd-apply migrate-vite-rolldown-codesplitting
  Model class: small
  Concrete model: claude-sonnet-5 (session default)
  Model rationale: mechanical, verified 1:1 config rename (build.rollupOptions → build.rolldownOptions, manualChunks → codeSplitting.groups) with a real pnpm build check as the verification step.
  Progress file: .kbd-orchestrator/phases/uar-security-audit-alerts-gate-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing.

- verify-dependabot-alerts-gate-live → SELF (claude-code)
  Entry: /opsx:new verify-dependabot-alerts-gate-live (scaffold), then /kbd-apply verify-dependabot-alerts-gate-live
  Model class: small
  Concrete model: claude-sonnet-5 (session default)
  Model rationale: mechanical push + gh workflow run + result inspection, mirrors last phase's push-and-verify-security-audit-workflow change exactly.
  Progress file: .kbd-orchestrator/phases/uar-security-audit-alerts-gate-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing. Depends on add-dependabot-alerts-ci-gate's change landing first, including a real secret existing in repo settings if a net-new one is required — confirm with the user before assuming this change can go fully green.

APPROVAL GATES

- git push origin main (the one genuinely irreversible step, same standing gate as every prior dependency-security phase) — confirm with the user before pushing, per this project's established practice.
- Creating a new GitHub Actions secret (if the token-source decision lands on a net-new secret) is an operator-only action outside agent capability — the agent can document the requirement and design the job to fail loudly when unset, but cannot create the secret itself.

FALLBACK CONDITIONS

- If OpenSpec's change-dir workflow becomes too opaque to track (e.g., a change balloons beyond its single-session scope), fall back to a native .kbd-orchestrator/changes/<id>/change.md entry — not expected for any of these 3 changes given their S/M complexity scores.

VERIFICATION REQUIREMENTS

- add-dependabot-alerts-ci-gate: workflow YAML must be valid (actionlint or `gh workflow view` parse check); local dry-run of the new job's script logic where feasible without a live secret.
- migrate-vite-rolldown-codesplitting: `pnpm run build` in frontend/ must produce the same 4 vendor chunk groupings (vendor-react, vendor-assistant, vendor-query, vendor-hljs) as before.
- verify-dependabot-alerts-gate-live: `gh run view <run-id>` must show all 5 jobs (4 existing + new) with conclusion=success.

PROGRESS LEDGER

- [PENDING] add-dependabot-alerts-ci-gate — claude-code
- [PENDING] migrate-vite-rolldown-codesplitting — claude-code
- [PENDING] verify-dependabot-alerts-gate-live — claude-code

OUTPUTS

- NONE yet — execute phase just dispatched

BLOCKERS

- Token-source decision for add-dependabot-alerts-ci-gate (see DISPATCH CONTRACTS above) — must be resolved with the user before that change's core implementation.
- verify-dependabot-alerts-gate-live may be blocked on an operator creating a real GitHub secret, depending on the token-source decision's outcome.

REFLECTION HANDOFF

- Whether the token-source decision required a net-new secret or reuse of an existing one, and whether that made verify-dependabot-alerts-gate-live agent-completable or operator-blocked (same "P0 operator" pattern as the eval harness's UAR_LLM__API_KEY gate) — this shapes next phase's recommendations if it stays open.

EXECUTION READY
