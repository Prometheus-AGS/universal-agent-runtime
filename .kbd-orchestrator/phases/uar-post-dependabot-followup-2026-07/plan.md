PLAN: uar-post-dependabot-followup-2026-07
Project: universal-agent-runtime
Date: 2026-07-08
Source assessment: `.kbd-orchestrator/phases/uar-post-dependabot-followup-2026-07/assessment.md`

## Product decisions resolved this planning session

- **R1 (Goal 2 — `surreal-memory` pin)**: user chose, via `AskUserQuestion`,
  to **pin to a fixed SHA** (not re-affirm the float, not defer the
  decision to execute time). Current `main` HEAD resolved via
  `git ls-remote https://github.com/Prometheus-AGS/surreal-memory-server.git HEAD`:
  `f9ab1c29944b86d44c23ea0e6192fa3d39acbde8`. This SHA is what change #2
  below pins to — re-verify it's still current at execute time in case
  time has passed since planning.

## Change list

All 4 changes are independent (no ordering dependency between them), low
complexity, and low blast-radius (docs + a single dependency-pin edit + a
git push + a dependency-reachability triage). Given the small scope
relative to the prior 8-change phase, this runs as a single round with
one shared verification checkpoint at the end, rather than multiple
rounds.

1. **fix-d-d-pin-characterization**: correct
   `docs/ARCHITECTURE.md`'s D-D bullet (kreuzberg/surreal-memory
   pin-type swap)
   - Scope: `docs/ARCHITECTURE.md` only (the parallel table in
     `docs/DEPENDENCY_MANAGEMENT.md` is already correct, from the prior
     phase)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: XS · Complexity score: Trivial · Model class: small
   - Customer value: MEDIUM — a wrong architectural-decision record is a
     trap for the next person (human or agent) who reads D-D and trusts
     it at face value
   - Details: replace *"pinned to specific commit SHAs (or, for
     `kreuzberg`, tracking `branch = "main"` deliberately)"* with
     accurate text reflecting: `rmcp`/`prometheus_parking_lot` = SHA-pinned,
     `kreuzberg` = tag-pinned (`v4.9.8`), `surreal-memory` = (after change
     #2 lands) also SHA-pinned. Cross-reference change #2's outcome before
     finalizing wording, since #2 changes what's actually true about
     `surreal-memory`.
   - Verify: proofread against live `Cargo.toml` state after change #2
     lands; no code/build impact to verify.

2. **pin-surreal-memory-to-sha**: move `surreal-memory` from
   `branch = "main"` to a fixed `rev`
   - Scope: `Cargo.toml` (`surreal-memory` dependency line), `Cargo.lock`
   - Depends on: NONE (but change #1's wording depends on this landing
     first, so sequence #2 before finalizing #1's text)
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: HIGH — this is the actual risk-reducing action Goal
     2 exists for; makes `surreal-memory` builds reproducible the same
     way the other 3 D-D pins already are
   - Details: change `branch = "main"` to
     `rev = "f9ab1c29944b86d44c23ea0e6192fa3d39acbde8"` (re-verify this is
     still `main`'s HEAD at execute time — re-run the `git ls-remote`
     check; if it has moved, use the newly-resolved SHA and note the
     drift in the change's findings). Regenerate `Cargo.lock` scoped to
     just this manifest edit (learn from the prior phase's
     `direct-network-facing-vulns` incident — use a manifest-edit +
     scoped `cargo check`, not a bare `cargo update`, to avoid an
     unrelated ~190-package churn).
   - Verify: `cargo check --lib --tests` clean; `cargo test --lib`
     unchanged vs. current baseline; `cargo audit` shows no new findings
     (pinning to a specific commit should not change resolved crate
     versions if that commit is `main`'s current HEAD); confirm
     `Cargo.lock`'s `surreal-memory` entry now shows a `rev` not a
     `branch`.

3. **push-and-verify-security-audit-workflow**: push the 16 unpushed
   local commits (15 from the prior phase + this phase's own commits) to
   `origin/main`, then verify `security-audit.yml` actually runs
   - Scope: git push (no file changes); `gh workflow run security-audit.yml`
     dispatch
   - Depends on: NONE structurally, but should land **last** in practice
     since pushing mid-phase would put changes #1/#2/#4's commits on the
     remote before they're verified locally — sequence this change's push
     step after #1, #2, and #4 are all committed locally.
   - Recommended agent: Claude Code (with explicit user confirmation
     before the push itself — see Approval Gates below)
   - Est. complexity: XS (mechanical) · Complexity score: Trivial ·
     Model class: small
   - Customer value: HIGH for closing the loop this phase and the prior
     one both exist to close — an audit workflow that has never actually
     run provides zero real assurance regardless of how correct it looks
     on paper
   - Details: `git push origin main`, then
     `gh workflow run security-audit.yml` to trigger a manual
     `workflow_dispatch` run (don't wait for the Monday 06:00 UTC cron),
     then `gh run watch` or `gh run list --workflow=security-audit.yml`
     to confirm it completes and inspect each of the 4 jobs' outcomes.
   - Verify: `gh run list --workflow=security-audit.yml` shows a non-404,
     real run; all 4 jobs (`rust-audit`, `npm-root-audit`,
     `frontend-audit`, `sdk-typescript-audit`) either pass, or fail for a
     reason to disclose (e.g. an environment difference between local and
     CI runners) — either outcome is informative, but "doesn't run at
     all" is not acceptable to leave unresolved.

4. **triage-unassigned-unmaintained-warnings**: fix or disclose the 5
   reachable-in-a-normal-build crates from the assessment's Goal 4 table
   (`bincode`, `instant`, `number_prefix`, `paste`, `ttf-parser`); disclose
   the other 4 (`atomic-polyfill`, `rustls-pemfile`, `scc`,
   `proc-macro-error2`) with a one-line rationale each, mirroring the
   prior phase's `quinn-proto`/`hickory-proto` disposition pattern
   - Scope: possibly `Cargo.toml` (if any of the 5 reachable crates has a
     maintained-alternative fix, mirroring `serde_yml`→`serde_norway`),
     `docs/DEPENDENCY_MANAGEMENT.md` (disclosure entries for all 9)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: M · Complexity score: Medium · Model class: mid —
     the 5 reachable ones each need their own "does a maintained
     alternative exist, and is a swap actually safe" investigation, not a
     single mechanical action
   - Details: for each of `bincode`/`instant`/`number_prefix`/`paste`/
     `ttf-parser`: check if a maintained alternative crate exists AND
     whether the owning first-party dependency (`burn`, `notify`,
     `kreuzberg`, or the `fastembed`/`surreal-memory` chain) can
     realistically swap it — these are all *transitive*, not direct like
     `serde_yml` was, so a swap may not even be possible without
     upstream cooperation. If no safe fix exists, disclose as
     accepted-risk (matching `rsa`'s precedent) rather than forcing a
     workaround. For the 4 feature-gated/dev-only/orphaned ones, a short
     disclosure entry (same style as `hickory-proto`/`quinn-proto`) is
     likely sufficient — no code change expected.
   - Verify: `cargo audit` re-run — for any crate actually fixed, confirm
     it drops off the warning list; for disclosed ones, confirm the
     `docs/DEPENDENCY_MANAGEMENT.md` entry accurately states current
     reachability; `cargo test --lib` unchanged.

## Approval Gates

- **Change #3's `git push origin main`** requires explicit user
  confirmation before executing, per this project's own git-safety
  norms (pushing is a shared-state, harder-to-reverse action). Do not
  push without asking first, even though this phase's own goal is to get
  the workflow running — the push itself is the one genuinely irreversible
  step in this whole phase.
- No other approval gates — the other 3 changes are local-only edits with
  their own verify checkpoints.

## Fallback Conditions

- If change #2's re-verified `git ls-remote` SHA differs from the one
  resolved at planning time (`f9ab1c29944b86d44c23ea0e6192fa3d39acbde8`),
  use the new SHA and note the drift — don't treat this as a blocker.
- If `security-audit.yml`'s manual dispatch (change #3) fails one or more
  jobs for reasons unrelated to the workflow's own correctness (e.g. a
  transient registry outage, a runner environment difference), disclose
  the failure and its likely cause rather than silently re-running until
  green — this phase's job is to prove the trigger *fires*, not
  necessarily that every job passes on the first real run.
- If any of change #4's 5 "reachable" crates turns out to have no safe
  fix path at all (likely, since none are direct dependencies), disclosed
  accepted-risk is an acceptable, expected outcome — matches this
  project's established practice from the prior phase.

## Verification Requirements (shared checkpoint, run once all 4 land)

- `cargo check --lib --tests` clean.
- `cargo test --lib` — no regression vs. current baseline (387/388, 1
  pre-existing ignore, as of the end of the prior phase).
- `cargo clippy --lib` — zero new warnings vs. current baseline (499).
- `cargo audit` — confirms whatever change #4 actually fixed is cleared;
  disclosed items remain listed as expected.
- `gh run list --workflow=security-audit.yml` shows at least one
  non-404, real run (change #3's core deliverable).
- `docs/ARCHITECTURE.md`'s D-D bullet and `docs/DEPENDENCY_MANAGEMENT.md`
  are both internally consistent with the actual `Cargo.toml` pin state
  after change #2 lands.

## Commands to run

```
/opsx:new fix-d-d-pin-characterization
/opsx:new pin-surreal-memory-to-sha
/opsx:new push-and-verify-security-audit-workflow
/opsx:new triage-unassigned-unmaintained-warnings
```

Per this project's established practice, `proposal.md` + `tasks.md` are
written per-change at execute time. Each change needs a
`dependency-security-posture` (or, for #1/#3 which aren't really
dependency-security matters, potentially a new capability, or explicitly
scoped outside OpenSpec) delta spec before `openspec verify`/`archive`
will succeed — decide per-change at execute time which fits, per the
lesson from the prior phase's `openspec_verify_archive_gap` discovery.

## Sycophancy self-check

- S-02: did not simply default to "pin to a SHA" without surfacing the
  trade-off first — used `AskUserQuestion` and got an explicit answer
  before committing this plan to a specific resolution.
- S-03: this plan does not assume all 4 changes will land cleanly —
  change #3 explicitly allows for "the workflow ran but some jobs failed"
  as an acceptable, disclosed outcome, and change #4 explicitly allows for
  "no safe fix exists, disclose accepted-risk" for any/all of its 5
  crates, matching this project's established evidence-over-optimism
  practice.

Ran `detect_sycophancy` (standard strictness) against this plan draft:
**score 0.0, 0 patterns classified, correction not mandatory.** Saved to
`.kbd-orchestrator/phases/uar-post-dependabot-followup-2026-07/sycophancy/plan-2026-07-08T09-08-40Z.json`.
