# Current Waypoint — universal-agent-runtime

- **Phase:** uar-dependabot-remediation-2026-07
- **Status:** executing (Round 0 prerequisite ✅; Round 1: 0/5; Round 2: 0/2; Round 3: 0/1)
- **Progress:** 0 of 8 changes
- **Next pending change:** `kreuzberg-reachable-vulns` (Round 1, first of 5 parallel Rust changes)
- **Exact next command:** `/opsx:new kreuzberg-reachable-vulns` then `/kbd-apply kreuzberg-reachable-vulns`
- **Recommendation source:** seeded from `prometheus-package-integration`'s reflection.md candidate 3 ("a fresh Dependabot/security posture re-check"), chosen explicitly by the user after a 52-alert count surfaced on `git push` on 2026-07-06

## Round 0 (prerequisite, complete)

CI had been broken on every run since 2026-07-04 (missing system deps in the
workflow) plus `cargo fmt` drift blocking every open PR. Both fixed and
committed (`c2ff066`, `83b9505`) before the 8-change execution began, so
Round 1-3 land against a clean CI gate.

## Why this phase

Seeded from `prometheus-package-integration`'s reflection, candidate 3, over
two other candidates (original #14 test gap; cross-platform live
verification of `provisioning.rs` — both explicitly deferred, not
abandoned). The trigger was a live count of 52 open Dependabot alerts (3
critical/10 high/32 medium/7 low) discovered on push. `assessment.md` then
found the real picture is bigger: `cargo audit` surfaces 17 Rust
vulnerabilities (6 net-new vs. Dependabot) plus 18 unmaintained/unsound
warnings; `npm audit`/`pnpm audit` surface 6 more npm advisories Dependabot
has no alert for; `sdks/typescript` has no lockfile at all; and the CI
`cargo audit` step `docs/DEPENDENCY_MANAGEMENT.md` claims exists has never
actually run (only triggers on a tag/release, none ever cut).

## Goals (see `phases/uar-dependabot-remediation-2026-07/goals.md` for full detail)

1. Triage all 52 alerts (+ the net-new cargo-audit/npm-audit findings) —
   fixed version available? direct or transitive? actually reachable?
2. Resolve what's safely upgradable, re-verifying build/test/clippy green
   after each ecosystem's batch.
3. Disclose explicitly what can't be resolved yet — not every finding needs
   a code change to count as "handled."
4. Re-affirm or revise the D-D architectural decision (git-sourced pins:
   `rmcp`, `surreal-memory`, `kreuzberg`, `prometheus_parking_lot`) if any
   are implicated.

## Planned change order (see `plan.md` for full detail — ordered by reachability/risk, not by source list)

- **Round 1 (Rust, 5 changes, shared cargo audit/test/clippy checkpoint):**
  `kreuzberg-reachable-vulns` (lopdf + quick-xml x2, highest priority — most
  concretely reachable finding in the whole assessment),
  `surreal-memory-transitive-vulns` (ammonia/crossbeam-epoch/rsa,
  reachability TBC), `direct-network-facing-vulns` (hickory-proto +
  tokio-tar), `first-party-direct-dep-hygiene` (serde_yml/libyml are direct
  deps), `grcov-toolchain-refresh` (dev-only, lower urgency)
- **Round 2 (npm, 2 changes, shared audit/build checkpoint):**
  `npm-root-remediation`, `frontend-npm-remediation`
- **Round 3 (closes out, own checkpoint):**
  `sdk-typescript-lockfile-and-ci-audit-fix` (vitest bump + new scheduled
  security-audit workflow — deliberately not repurposing `release.yml`)

## Execute-phase dispatch (see `execution.md` for full contract)

Backend: `openspec`, self-executing via Claude Code CLI, driven per-change
through `/kbd-apply` (never bare `/opsx:apply`). None of the 8 change dirs
exist under `openspec/changes/` yet — `plan.md` deferred `proposal.md`/
`tasks.md` authorship to execute time per this project's established
practice, so each change is scaffolded via `/opsx:new` immediately before
`/kbd-apply` drives its tasks.

## Decisions carried forward (still load-bearing)
- D-A: RAG hardened in-process; Knowledge Service extraction deferred
- D-B: MemPalace stays off
- D-C: LibreFang integration scoped to UAR side
- D-D: dependency pins deliberate — reaffirmed for `rmcp`/`surrealdb`/
  `wasmtime` in `uar-security-deps-and-hygiene` (2026-07-04); this phase
  re-examines the git-tracked pins (`surreal-memory`, `kreuzberg`) plus the
  first-party direct deps (`serde_yml`/`libyml`) specifically

## Carried-over debt (unrelated to this phase's scope, tracked for later)
- 17 pre-existing `bun run typecheck` errors
- CH-06 per-agent/per-task budget configuration surface (global-only today)
- CH-08 activation-outcome correlation (recall wired; outcome half unsolved)
- Durable cost/spend history for CH-07 dashboard
- `main()` always loads full `AppConfig` before dispatching any subcommand
  (found during `uar-spec-v2-and-polish`'s CH-15 `compile` subcommand)

## Prior phase archive

- **`uar-security-deps-and-hygiene`** (2026-07-04): 10/10 changes across 4
  risk-ordered rounds. Upgraded `surrealdb` 3.0.5→3.2.0 (session-hijack +
  privilege-escalation CVEs), `rmcp` to `rmcp-v1.8.0` tag (DNS rebinding,
  `GHSA-89vp-x53w-74fx`), `wasmtime` 41→46, added `.github/dependabot.yml`,
  fixed `write-position-reminder.sh`'s `.stage`/`.status` schema mismatch at
  the source (also fixed in the sibling `prometheus-skill-system` repo).
  See its own `reflection.md` for full detail — its recommendation seeded
  `prometheus-package-integration`, whose own reflection candidate 3 in
  turn seeded this phase.
- **`uar-spec-v2-and-polish`** (2026-07-04): 7/7 changes, G4+G5 MET.
- **`uar-next-harness`**: 16/24 changes, G1-G3 MET, G4-G5 deferred.
