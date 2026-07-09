EXECUTION: uar-post-dependabot-followup-2026-07
Project: universal-agent-runtime
Date: 2026-07-08
Selected backend: openspec
Dispatched to: SELF (Claude Code CLI, self-executing)
Backend rationale: OpenSpec is available at project root and this project's
established practice (all 8 changes of the prior phase,
uar-dependabot-remediation-2026-07, plus many earlier phases) drives every
change through an OpenSpec change dir. Prior phase discovered
`openspec validate`/`archive` hard-require at least one delta spec even for
changes with no product-facing behavior change (see
`dependency-security-posture` capability, `openspec/specs/dependency-security-posture/spec.md`)
-- apply that same pattern here for change #3 (dependency triage fits
naturally); changes #1/#2/#4 (docs fix, pin fix, git push+CI verification)
don't fit `dependency-security-posture` as naturally and may need their own
minimal capability or an explicitly-scoped "Capabilities: None" with a
placeholder delta -- decide per-change at execute time, per plan.md's note.
Backend entrypoint: /opsx:new <change-id> (scaffold) -> /kbd-apply <change-id>
(task-by-task loop) -> /kbd-apply verify -> /kbd-apply archive
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/uar-post-dependabot-followup-2026-07/plan.md

EXECUTION SCOPE

- pin-surreal-memory-to-sha: move surreal-memory from branch="main" to a fixed rev (user-approved via AskUserQuestion during planning)
- fix-d-d-pin-characterization: correct docs/ARCHITECTURE.md's D-D bullet (kreuzberg/surreal-memory pin-type swap), sequenced after the pin change lands
- triage-unassigned-unmaintained-warnings: fix or disclose 9 crates (bincode, instant, number_prefix, paste, ttf-parser, atomic-polyfill, rustls-pemfile, scc, proc-macro-error2)
- push-and-verify-security-audit-workflow: push to origin/main + gh workflow run security-audit.yml dispatch + confirm a real run appears (LAST, requires explicit user confirmation before the push)

DISPATCH CONTRACTS

- pin-surreal-memory-to-sha -> SELF
  Entry: /opsx:new pin-surreal-memory-to-sha; then /kbd-apply pin-surreal-memory-to-sha
  Model class: small
  Concrete model: session default
  Model rationale: plan.md scored Low complexity -- single manifest-line edit + scoped Cargo.lock regen, mechanical once the SHA is confirmed current
  Progress file: .kbd-orchestrator/phases/uar-post-dependabot-followup-2026-07/progress.json
  Handoff: report completion by updating progress.json and committing

- fix-d-d-pin-characterization -> SELF
  Entry: /opsx:new fix-d-d-pin-characterization; then /kbd-apply fix-d-d-pin-characterization
  Model class: small
  Concrete model: session default
  Model rationale: plan.md scored Trivial complexity -- a documentation text fix, no code/build impact
  Progress file: .kbd-orchestrator/phases/uar-post-dependabot-followup-2026-07/progress.json
  Handoff: report completion by updating progress.json and committing; sequence after pin-surreal-memory-to-sha so the wording is accurate

- triage-unassigned-unmaintained-warnings -> SELF
  Entry: /opsx:new triage-unassigned-unmaintained-warnings; then /kbd-apply triage-unassigned-unmaintained-warnings
  Model class: mid
  Concrete model: session default
  Model rationale: plan.md scored Medium complexity -- 5 reachable crates each need their own maintained-alternative investigation, not a single mechanical action; disclosed accepted-risk is an acceptable outcome per plan.md's fallback conditions
  Progress file: .kbd-orchestrator/phases/uar-post-dependabot-followup-2026-07/progress.json
  Handoff: report completion by updating progress.json and committing; "no safe fix, disclosed" is an acceptable disposition for any/all of the 5 reachable crates

- push-and-verify-security-audit-workflow -> SELF
  Entry: /opsx:new push-and-verify-security-audit-workflow; then /kbd-apply push-and-verify-security-audit-workflow
  Model class: small
  Concrete model: session default
  Model rationale: plan.md scored Trivial (mechanical) complexity -- the actual work is a git push + gh CLI dispatch + run inspection, not code
  Progress file: .kbd-orchestrator/phases/uar-post-dependabot-followup-2026-07/progress.json
  Handoff: report completion by updating progress.json and committing; MUST ask for explicit user confirmation before `git push origin main` -- do not push without asking, per plan.md's Approval Gates section and this project's own git-safety norms

APPROVAL GATES

- push-and-verify-security-audit-workflow's `git push origin main` step requires explicit user confirmation before executing -- the one genuinely irreversible/shared-state action in this phase. All other 3 changes are local-only edits with their own verify checkpoints, no approval gate needed.

FALLBACK CONDITIONS

- If the re-verified `git ls-remote` SHA for surreal-memory differs from the one resolved at planning time (f9ab1c29944b86d44c23ea0e6192fa3d39acbde8), use the new SHA and note the drift in that change's findings -- not a blocker.
- If security-audit.yml's manual dispatch fails one or more jobs for reasons unrelated to the workflow's own correctness (environment difference, transient outage), disclose the failure and likely cause rather than silently re-running until green.
- If any of the 5 "reachable" crates in the triage change has no safe fix path, disclosed accepted-risk is an acceptable, expected outcome (matches the prior phase's rsa/hickory-proto precedent).

VERIFICATION REQUIREMENTS

- Shared checkpoint once all 4 land: `cargo check --lib --tests` clean; `cargo test --lib` no regression vs. 387/388 baseline; `cargo clippy --lib` zero new warnings vs. 499 baseline; `cargo audit` confirms whatever triage-unassigned-unmaintained-warnings actually fixed is cleared, disclosed items remain listed as expected; `gh run list --workflow=security-audit.yml` shows at least one non-404 real run; ARCHITECTURE.md/DEPENDENCY_MANAGEMENT.md internally consistent with actual Cargo.toml pin state.

PROGRESS LEDGER

- [PENDING] pin-surreal-memory-to-sha — SELF (first)
- [PENDING] fix-d-d-pin-characterization — SELF
- [PENDING] triage-unassigned-unmaintained-warnings — SELF
- [PENDING] push-and-verify-security-audit-workflow — SELF (last, needs user confirmation before push)

OUTPUTS

- NONE yet — populated per-change as OpenSpec change dirs are created and archived

BLOCKERS

- NONE

REFLECTION HANDOFF

- kbd-reflect should consume: whether the SHA pin drifted between planning and execute time, the final cargo audit disposition for the 9 triaged crates, and — most importantly — whether security-audit.yml's first real run actually succeeded or surfaced environment differences from local simulation. That last item is this phase's (and the prior phase's) whole reason for existing.

EXECUTION READY
