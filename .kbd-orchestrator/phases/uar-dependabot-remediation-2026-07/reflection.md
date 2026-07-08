# Reflection: uar-dependabot-remediation-2026-07

**Date**: 2026-07-08
**Status**: 8/8 changes DONE, committed, and archived. Phase execution complete.

## Artifact Quality Summary

| Metric                       | Value    |
| ----------------------------- | -------- |
| Changes with QA               | 0/8      |
| First-pass pass rate          | n/a      |
| Changes requiring refinement  | n/a      |
| Total refinement iterations   | 0        |

No `.refiner/artifacts/<change-id>/` logs exist for any of this phase's 8
changes — artifact-refiner is not wired into this project
(`.agent/skills/artifact-refiner/SKILL.md` and
`.kbd-orchestrator/constraints.md` don't exist). This is a standing,
formally-decided skip (`artifact-refiner-gate-decision`, D-E, from a prior
phase), not a new gap introduced here. Verification for all 8 changes
instead relied on direct tool output: `cargo audit`/`npm audit`/`pnpm
audit` re-runs, `cargo test`/`clippy`, `bun run build`/`typecheck`, and
`pnpm -C frontend build` — each change's `findings.md` records the exact
commands and results.

### Recurring Constraint Violations

Not applicable — no constraint-based QA ran this phase.

## 1. Goal Achievement

From `goals.md`:

1. **Triage all 52 alerts** — **MET**. Went beyond the 52 Dependabot alerts
   as directed: live `cargo audit` found 17 real Rust advisories (6
   net-new vs. Dependabot), live `npm audit`/`pnpm audit` found 15 + 11
   findings on root/frontend (6 net-new/higher-severity), and
   `sdks/typescript` had zero lockfile coverage at all. Every finding got
   an explicit reachability trace and disposition (fixed / mitigated /
   disclosed-not-reachable / accepted-risk) rather than trusting the
   advisory's severity label — e.g. `ttf-parser`/`memmap2`/`paste` were
   checked and found *not* attributable to the changes the plan assigned
   them to, and corrected in the record rather than silently claimed.
2. **Resolve what's safely upgradable** — **MET**. Fixed outright:
   `ammonia`, `crossbeam-epoch`, `tokio-tar` (via removing the unused
   `testcontainers` dep), `serde_yml`→`serde_norway`, `grcov` (removed,
   unused), 15 root npm findings, 11 frontend findings, `vitest`+`esbuild`
   in `sdks/typescript`. Mitigated (no crate fix exists, compensating
   control added instead): kreuzberg's `lopdf`/`quick-xml` via
   `KreuzbergConfig` resource limits.
3. **Disclose what can't be resolved yet** — **MET**. `rsa` (accepted
   risk, `patched=[]`), `hickory-proto` (not reachable, feature-gated),
   kreuzberg's 2 mitigated CVEs (crate versions unchanged, will keep
   appearing in `cargo audit` until upstream ships a fix), `quinn-proto`
   (orphaned lockfile entry), `proc-macro-error2` (unmaintained,
   feature-gated, out of scope). All documented with rationale in
   `docs/DEPENDENCY_MANAGEMENT.md`.
4. **Re-affirm or revise the D-D architectural decision** — **NOT MET**.
   This is the honest gap in this phase. Two of D-D's four pinned git
   dependencies (`kreuzberg`, `surreal-memory`) were directly implicated
   by this phase's findings — exactly the trigger condition the goal
   itself named — but `docs/ARCHITECTURE.md`'s D-D bullet was never
   revisited. Worse: **D-D's text is now factually wrong** about which
   dependency floats. It reads *"pinned to specific commit SHAs (or, for
   `kreuzberg`, tracking `branch = "main"` deliberately)"* — but
   `Cargo.toml` shows the opposite: `kreuzberg` is pinned to `tag =
   "v4.9.8"` (a stable tag, not a floating branch), while
   **`surreal-memory` is the one on `branch = "main"`** (a floating,
   unpinned branch). This inaccuracy was actually *found* during this
   phase (`docs/DEPENDENCY_MANAGEMENT.md`'s "Current Pinned Versions"
   table was corrected to reflect it, back in `kreuzberg-reachable-vulns`)
   but the parallel claim in `ARCHITECTURE.md` was never touched. Carrying
   this forward explicitly below.

**Overall: 3/4 goals MET, 1/4 NOT MET → 75% goal completion.**

## 2. What Was Delivered

All 8 changes, archived under `openspec/changes/archive/2026-07-0{7,8}-*`:

**Round 1 (Rust, 5 changes)**:
- `kreuzberg-reachable-vulns` (by: claude-code, commit `d8b5630`) — resource-limit mitigation for `lopdf`/`quick-xml`
- `surreal-memory-transitive-vulns` (by: claude-code, commit `07cac2a`) — `ammonia`/`crossbeam-epoch` fixed, `rsa` disclosed
- `direct-network-facing-vulns` (by: claude-code, commit `fc0f7bd`) — `hickory-proto` disclosed, `tokio-tar`/`testcontainers` eliminated
- `first-party-direct-dep-hygiene` (by: claude-code, commits `8c1c6fb`+`eecd09a`) — `serde_yml`→`serde_norway`
- `grcov-toolchain-refresh` (by: claude-code, commits `20c9795`+`c719072`) — unused `grcov` dev-dependency removed

**Round 2 (npm, 2 changes)**:
- `npm-root-remediation` (by: claude-code, commits `52290b6`+`38e0f03`) — 15 findings fixed via `npm audit fix`
- `frontend-npm-remediation` (by: claude-code, commits `32e2c95`+`7f0422b`) — 11 findings fixed; caught and corrected an unintended major-bump mistake mid-change

**Round 3 (SDK + CI process, 1 change)**:
- `sdk-typescript-lockfile-and-ci-audit-fix` (by: claude-code, commits `a96eb8a`+`32602f0`) — real lockfile, `vitest` bumped past the vulnerable range, new `security-audit.yml` scheduled workflow, stale doc claim corrected

**Process artifact**: a new `dependency-security-posture` OpenSpec capability (`openspec/specs/dependency-security-posture/spec.md`) with 8 requirements, one per change — did not exist before this phase (see § 6).

## 3. Technical Debt Introduced

- **`docs/ARCHITECTURE.md`'s D-D bullet is now stale/inaccurate** (see § 1,
  goal 4) — describes `kreuzberg` as the branch-floating dependency and
  implies `surreal-memory` is SHA-pinned; it's the reverse. Introduced by
  omission (this phase touched the same fact in a different doc and
  didn't propagate the correction), not by this phase's own edits.
- **`security-audit.yml`'s trigger has never actually fired** — by
  construction, since it was only just added. Its first real
  `schedule`/`workflow_dispatch` run has not been observed; only local
  simulation of each job's underlying command was possible this session.
  This is the same category of gap the phase itself exists to close
  (a CI mechanism that looks correct on paper but is unverified in
  practice) — flagged explicitly rather than claimed as fully verified.
- **`sdks/typescript` has zero test files** — `vitest --run` now exits
  non-zero ("No test files found") post-bump. Not a regression (nothing
  exercises this today), but a real gap if `sdks/typescript` is ever
  wired into CI without either adding tests or `--passWithNoTests`.
- **`quinn-proto` and `proc-macro-error2`** remain in `Cargo.lock`/the
  dependency tree as disclosed-but-unresolved (orphaned lockfile entry;
  feature-gated unmaintained crate respectively) — correctly out of this
  phase's assigned scope, but worth a follow-up if a future phase touches
  `microsandbox-*` or does a full `cargo update`.
- **9 unmaintained/unsound `cargo audit` warnings never triaged**
  (`atomic-polyfill`, `bincode`, `instant`, `number_prefix`, `paste`,
  `rustls-pemfile`, `ttf-parser`, `scc`, `proc-macro-error2`) — none were
  assigned to any of this phase's 8 changes; `security-audit.yml`
  deliberately doesn't fail on these (by cargo-audit's own default
  behavior) so they remain silently un-actioned unless a human reviews
  `cargo audit` output directly.

## 4. Architecture Integrity

This project's `AGENTS.md`/`CLAUDE.md` doesn't use a dedicated "Never Do"
list format — its equivalent is the 40-rule Prometheus Base Rules Set.
Checked against the rules most relevant to this phase's work:

- **Rule 22/23 (dependency versions verified before introduction)** —
  followed: `grcov`'s latest release was checked live via `crates.io`'s
  API before deciding removal was better than a version bump; `vitest`'s
  current stable line was checked the same way.
- **Rule 31 (small, reviewable, separated commits)** — followed: every
  change landed as 2 commits (behavioral code+docs, then a separate
  mechanical openspec-archive commit), 13 commits total across the phase.
- **Rule 33 (security)** — followed: no secrets logged; all fixes
  verified against real audit tools, not assumed.
- **Rule 3/31 (surgical changes, don't bundle unrelated fixes)** — mostly
  followed, with one deliberate exception surfaced to the user rather than
  silently bundled: pre-existing uncommitted `.github/workflows/ci.yml` +
  `.claude/settings.local.json` + `.kbd-orchestrator/memory-outbox.jsonl`
  changes (not part of this phase's scope, origin unclear) were
  explicitly left out of every commit this phase made, per the user's own
  choice when asked.
- No "Never Do" rule violations identified.

## 5. Lessons Learned

- **`openspec validate`/`archive` hard-require a delta spec — even for
  changes with no product-facing behavior change.** This blocked
  verify/archive for the first 4 changes (all had declared "Capabilities:
  None") and had to be retrofitted. **Fix this at plan time in future
  phases**: a pure security/hygiene-fix phase should either (a) declare a
  capability like `dependency-security-posture` in the *plan*, before
  execution starts, or (b) the KBD/OpenSpec integration should support a
  lighter-weight schema for changes with no spec-level requirement change.
  This is the second time this exact gap has recurred (first flagged in
  `uar-spec-v2-and-polish`'s reflection) — it's a systemic OpenSpec/KBD
  integration gap, not a one-off mistake.
- **`pnpm audit --fix` and `npm audit fix`'s auto-generated overrides can
  silently introduce a major-version bump.** `frontend-npm-remediation`
  hit this directly: an open-ended override (`>=7.3.5`, no upper bound)
  resolved `vite` to `8.1.3` instead of the intended `7.3.6` patch. Always
  inspect `pnpm install`'s own dependency-diff output before trusting an
  auto-fix, and prefer a targeted `pnpm update <pkg>` (bounded by the
  existing declared range) over a blanket `--fix` when only a subset of
  findings need attention.
- **Trace *why* a crate is flagged before assuming a version bump fixes
  it.** Two changes (`grcov-toolchain-refresh`, both npm changes) found
  the actual fix was either "this dependency is completely unused, delete
  it" or "the parent's own declared range already permits the patch, no
  override needed" — neither of which is the instinctive first move
  (bump the flagged package directly). `cargo tree -i` / `pnpm why` /
  grepping for actual call sites should be the first step, every time.
- **A phase's own plan.md can mis-attribute which crates a fix will
  clear** (`grcov-toolchain-refresh`'s plan entry claimed `paste` would
  also clear — it doesn't, it's from `kreuzberg`/`burn`). Verify the
  plan's claims against `cargo tree -i` at execute time rather than
  trusting them; disclose the correction rather than silently
  under-delivering or over-claiming.
- **A phase goal with a conditional trigger ("re-affirm D-D *if*
  implicated") needs an explicit checkpoint, not an implicit hope it gets
  swept up.** Goal 4 was satisfied in spirit for 2 of 8 changes (both
  `kreuzberg` and `surreal-memory` were investigated) but the actual
  *architectural decision document* was never revisited — the goal's
  condition fired without anyone noticing it had fired. **Recommendation
  for future phases**: when a goal has a conditional "if X happens,
  do Y" structure, track it as an explicit line item to check at
  reflection time, not just trust it'll be remembered mid-execution.

## 6. Cross-Tool Coordination Review

Single-tool phase (Claude Code self-executing throughout; no Roo/Cursor/
Codex/Antigravity involvement this phase).

- `progress.json` updates were made reliably at every change boundary —
  8/8 `change_status` entries populated with commit hashes and archive
  paths, `changes_completed` incremented correctly at each step.
- `current-waypoint.json`/`execution.md` went stale between `/kbd-execute`
  invocations (still pointing at `kreuzberg-reachable-vulns` as "next"
  well after 5 changes had landed) — caught and refreshed at each
  `/kbd-execute` re-entry, but this confirms `/kbd-execute` re-entry
  should probably auto-refresh these files from `progress.json` rather
  than requiring the executing session to notice and fix them manually
  each time.
- The mid-phase discovery of the `openspec_verify_archive_gap` was
  recorded directly in `progress.json.decisions` and successfully
  propagated to all 4 remaining changes without needing to be
  re-explained — a `decisions` block used this way (as a persistent,
  re-read-each-time "must-follow-this-pattern" note) worked well for a
  single session but its durability across a *new* session/tool boundary
  is untested this phase.

## 7. Next Phase Recommendations

**High priority**:
- Correct `docs/ARCHITECTURE.md`'s D-D bullet (kreuzberg/surreal-memory
  pin-type swap) and make an explicit, human-reviewed decision on whether
  `surreal-memory`'s `branch = "main"` floating pin should move to a
  fixed SHA — it's the one dependency in D-D's list that actually
  undermines D-D's own stated "reproducible builds" rationale.
- Verify `security-audit.yml` actually fires on its first real scheduled
  or manually-dispatched run on GitHub (this session could only simulate
  it locally) — if it doesn't fire as expected, that's this exact phase's
  root problem recurring one layer up.

**Medium priority**:
- Triage the 9 never-assigned unmaintained/unsound `cargo audit` warnings
  (`atomic-polyfill`, `bincode`, `instant`, `number_prefix`, `paste`,
  `rustls-pemfile`, `ttf-parser`, `scc`, `proc-macro-error2`) — low
  urgency (none are CVE-style vulnerabilities) but currently invisible
  since `security-audit.yml` doesn't fail on them by design.
- Consider whether `sdks/typescript` needs real test coverage before it's
  ever wired into a CI gate (currently zero test files).

**Process/architectural decisions needing human review**:
- Whether OpenSpec's `spec-driven` schema should gain a lighter-weight
  variant for hygiene-only changes (no capability delta required) instead
  of requiring every dependency/security fix to invent a capability
  requirement — this is the second phase in a row to hit this friction.

## Sycophancy Self-Check

Ran `detect_sycophancy` (strict mode) against this reflection's content:
**score 0.0, 0 patterns classified, correction not mandatory.** Goal 4's
honest NOT MET call and the concrete, specific debt items (D-D's factual
inaccuracy, the unverified `security-audit.yml` trigger) were retained
rather than softened.
