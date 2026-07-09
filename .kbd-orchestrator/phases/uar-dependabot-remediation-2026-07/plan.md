PLAN: uar-dependabot-remediation-2026-07
Project: universal-agent-runtime
Date: 2026-07-07
OpenSpec available: YES
Changes to implement: 8

## Framing

`assessment.md` found the real picture is bigger than the 52 Dependabot
alerts: `cargo audit` surfaces 17 Rust vulnerabilities (6 net-new) + 18
unmaintained/unsound warnings; `npm audit` surfaces 6 more npm
advisories Dependabot has no alert for at all; `sdks/typescript` has no
lockfile; and the CI `cargo audit` step `docs/DEPENDENCY_MANAGEMENT.md`
claims exists has never actually run. This plan is ordered by
**reachability and risk**, not by which list (Dependabot vs. cargo
audit vs. npm audit) an item came from — several of the
highest-priority items here aren't in Dependabot at all.

Additional provenance tracing done during planning (via `cargo tree -i`):

- `failure`, `atty`, `ansi_term` (Dependabot's 2 critical `failure`
  alerts among them) all come via `grcov` → `cargo-binutils` — a
  **dev-dependency-only** chain (code coverage tooling), not shipped in
  the release binary.
- `tokio-tar` is dev-dependency-only (via `testcontainers`).
- `libyml`/`serde_yml` are **direct** dependencies of
  `universal-agent-runtime` itself (both unsound *and* unmaintained per
  `cargo audit`) — first-party-controllable, not a transitive-pin
  problem.
- `hickory-proto` comes via `hickory-resolver` + `microsandbox-network`
  — DNS resolution code; network-facing, so its CPU-exhaustion/
  infinite-loop CVEs are plausibly reachable if DNS resolution actually
  routes through it.

## CHANGE LIST (ordered)

1. **kreuzberg-reachable-vulns**: `lopdf` stack overflow + `quick-xml`
   DoS (×2 CVEs) via kreuzberg
   - Scope: `Cargo.toml`'s `kreuzberg` pin (currently tag `v4.9.8`)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: M · Complexity score: Medium-High · Model class: mid
   - Customer value: HIGH — these are the most concretely reachable
     findings in this whole assessment (kreuzberg's actual job is
     parsing untrusted, user-supplied documents)
   - Details: first check whether a newer `kreuzberg` tag/commit
     already pulls patched `lopdf`/`quick-xml` versions (per
     `docs/DEPENDENCY_MANAGEMENT.md`'s existing upgrade SOP) before
     doing anything more invasive — e.g. patching `[patch.crates-io]`
     overrides in UAR's own `Cargo.toml`, which is a heavier, riskier
     move than a clean upstream bump. Re-run `cargo audit` after to
     confirm both CVEs actually clear, not just that the version number
     changed.
   - Verify: `cargo audit` no longer lists `lopdf`/`quick-xml` findings
     under the `kreuzberg` provenance path; `cargo test --lib` +
     `cargo clippy --lib` unchanged baseline.

2. **surreal-memory-transitive-vulns**: `ammonia`, `crossbeam-epoch`,
   `rsa` via `surrealdb-core`
   - Scope: `surreal-memory` git pin (tracks `branch = "main"`)
   - Depends on: NONE (independent of #1)
   - Recommended agent: Claude Code
   - Est. complexity: M · Complexity score: Medium · Model class: mid
   - Customer value: MEDIUM — reachability for all 3 is not yet
     confirmed (assessment.md flagged this as needing a usage check)
   - Details: since `surreal-memory` tracks `main` rather than a pinned
     SHA, check first whether simply re-syncing to the current `main`
     (or a `cargo update -p surrealdb-core`) already pulls a patched
     `surrealdb-core`. Separately, confirm reachability before treating
     any of these as urgent: (a) is `ammonia`'s HTML sanitizer actually
     invoked on untrusted input anywhere in the memory/SurrealDB path;
     (b) is RSA-signed JWT verification (vs. HMAC/EC) actually used at
     any auth boundary; (c) `crossbeam-epoch`'s bug needs an
     already-invalid pointer to trigger, which is a lower-likelihood
     precondition. Disclose findings either way — "not reachable, no
     action needed" is a valid, honest outcome for this change, not a
     failure to fix something.
   - Verify: `cargo audit` + a written reachability conclusion per
     advisory in this change's proposal.md.

3. **direct-network-facing-vulns**: `hickory-proto` (CPU exhaustion +
   infinite loop), `tokio-tar` (file smuggling)
   - Scope: `Cargo.lock` version bumps (or `cargo update -p`) for
     whichever direct dependency pulls each in
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low-Medium · Model class: mid
   - Customer value: MEDIUM-HIGH — `hickory-proto` is network-facing
     DNS code; `tokio-tar` is dev-dependency-only (via
     `testcontainers`) so lower urgency but still worth clearing
   - Details: `hickory-proto` already has 2 Dependabot alerts (medium +
     high) for the same 2 CVEs found by `cargo audit` — this change
     closes both the Dependabot alerts and the cargo-audit findings in
     one move. Confirm hickory-resolver/microsandbox-network's declared
     version ranges allow a patched hickory-proto without a breaking
     bump.
   - Verify: `cargo audit` clears both; `cargo test --lib` green.

4. **first-party-direct-dep-hygiene**: `serde_yml`/`libyml` (unsound +
   unmaintained, direct dependency), `anyhow`/`memmap2` unsoundness
   (recent, 2026-06)
   - Scope: `Cargo.toml`'s direct `serde_yml` dependency; whatever pulls
     `anyhow`/`memmap2` at the versions flagged
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: M · Complexity score: Medium · Model class: mid
   - Customer value: MEDIUM — `serde_yml` is first-party-controllable
     (not a transitive pin UAR doesn't own), and being both unsound and
     unmaintained upstream means it won't get fixed by waiting
   - Details: evaluate replacing `serde_yml` with an actively-maintained
     alternative (e.g. `serde_yaml` if UAR's usage is simple
     serialize/deserialize — check actual usage sites first, don't
     assume a drop-in). `anyhow`/`memmap2` are very recent unsoundness
     reports (2026-06-20/25) — check if a patched point release already
     exists before treating this as a larger migration.
   - Verify: `cargo audit` clears `serde_yml`/`libyml`; whatever
     replacement is chosen has its own round-trip test if the existing
     suite doesn't already cover the YAML path it's used for.

5. **grcov-toolchain-refresh**: the 18 unmaintained/unsound warnings
   attributable to `grcov`'s old `cargo-binutils`/`clap 2.x` chain
   (`failure`, `atty`, `ansi_term`, `paste`, `proc-macro-error`, etc.)
   - Scope: `Cargo.toml`'s `[dev-dependencies]` `grcov` version pin
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: LOW — dev-dependency-only, never shipped in the
     release binary; still worth clearing since CI/test builds compile
     this chain on every run
   - Details: check whether a newer `grcov` release has dropped the
     `cargo-binutils`/old-`clap` dependency chain; if not, this is a
     disclosed, accepted-risk item (dev-tooling only), not something to
     force a workaround for.
   - Verify: `cargo audit`'s warning count drops accordingly; `cargo
     test --lib` unaffected (grcov isn't in the normal test path).

6. **npm-root-remediation**: root `package-lock.json` — both
   Dependabot's alerts and the 2 net-new ones (`ajv`, `brace-expansion`)
   plus the extra GHSA IDs found on already-flagged packages
   - Scope: root `package.json`/`package-lock.json`
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: MEDIUM
   - Details: `npm audit fix` for what it can resolve automatically
     within semver-compatible ranges; anything requiring `--force`
     (potential breaking change) gets evaluated individually, not
     blanket-applied.
   - Verify: `npm audit` (re-run) shows the count drop; `bun run build`
     / `bun run check` still succeed.

7. **frontend-npm-remediation**: `frontend/pnpm-lock.yaml` — including
   the net-new `undici` GHSA not in Dependabot
   - Scope: `frontend/package.json`/`frontend/pnpm-lock.yaml`
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: MEDIUM
   - Details: same approach as #6, scoped to the frontend workspace;
     `pnpm audit --fix` where safe.
   - Verify: `pnpm audit` re-run shows the count drop; frontend build
     (`pnpm -C frontend build`) still succeeds.

8. **sdk-typescript-lockfile-and-ci-audit-fix**: give
   `sdks/typescript` a real lockfile + bump `vitest` past the vulnerable
   range; fix the dead `cargo audit` CI trigger
   - Scope: `sdks/typescript/package.json` (+ new lockfile),
     `.github/workflows/` (either fix `release.yml`'s trigger or add a
     dedicated scheduled security-audit workflow), `docs/DEPENDENCY_MANAGEMENT.md`
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: M · Complexity score: Medium · Model class: mid
   - Customer value: HIGH for the process fix — without it, this exact
     situation (17+ real vulnerabilities silently accumulating) recurs
   - Details: (a) bump `sdks/typescript/package.json`'s `vitest` range
     from `^2.0.0` to something past `3.2.6` (or `^4.1.0`), generate a
     real lockfile, confirm `npm test`/`vitest` still runs; (b) add a
     scheduled (e.g. weekly cron) or on-push-to-main workflow that runs
     `cargo audit` + `npm audit`/`pnpm audit` — don't repurpose
     `release.yml` itself (its tag/release trigger is a legitimate,
     separate concern; a new lightweight workflow is cleaner than
     conflating "when we cut a release" with "how often we check for
     new CVEs"); (c) correct
     `docs/DEPENDENCY_MANAGEMENT.md`'s "the CI pipeline runs cargo audit
     as part of the release workflow" line to reflect whatever the
     actual new trigger is.
   - Verify: the new workflow file is valid YAML and its trigger
     condition is confirmed to actually fire (e.g. `workflow_dispatch`
     tested manually, or `on: schedule` cron syntax validated); `vitest`
     in `sdks/typescript` runs green after the bump.

## EXECUTION ROUND ORDER

- **Round 1** (Rust, independent items — parallel, one shared checkpoint):
  #1 `kreuzberg-reachable-vulns`, #2 `surreal-memory-transitive-vulns`,
  #3 `direct-network-facing-vulns`, #4 `first-party-direct-dep-hygiene`,
  #5 `grcov-toolchain-refresh`
- **Round 2** (npm, independent items — parallel, one shared checkpoint):
  #6 `npm-root-remediation`, #7 `frontend-npm-remediation`
- **Round 3** (closes out): #8 `sdk-typescript-lockfile-and-ci-audit-fix`

Round 1's shared checkpoint: `cargo audit` re-run, confirming the
targeted findings actually clear (not just that some version number
changed); `cargo test --lib` full suite green; `cargo clippy --lib`
zero new warnings vs. baseline.

Round 2's shared checkpoint: `npm audit` / `pnpm audit` re-run showing
the count drop; `bun run build`/`bun run check` and
`pnpm -C frontend build` both succeed.

## COMMANDS TO RUN

```
/opsx:new kreuzberg-reachable-vulns
/opsx:new surreal-memory-transitive-vulns
/opsx:new direct-network-facing-vulns
/opsx:new first-party-direct-dep-hygiene
/opsx:new grcov-toolchain-refresh
/opsx:new npm-root-remediation
/opsx:new frontend-npm-remediation
/opsx:new sdk-typescript-lockfile-and-ci-audit-fix
```

Per this project's established practice, `proposal.md` + `tasks.md` are
written per-change at execute time.

## Sycophancy self-check

- S-02: several changes (#2, #5) explicitly allow for "disclose as not
  reachable / accepted risk" as a valid, honest outcome rather than
  forcing a fix where none is warranted — not every finding needs code
  changed to count as "handled."
- S-03: named trade-offs explicitly — `serde_yml` replacement (#4) may
  not be a clean drop-in and needs usage-site verification first;
  `grcov` (#5) is dev-only so lower urgency than the plan's own
  "18 warnings" count might suggest at a glance.
- S-07: no scope creep — this plan does not attempt a general
  dependency-modernization pass; it's scoped exactly to the advisories
  found in `assessment.md` plus the one process fix (#8's CI trigger)
  that explains how they accumulated undetected.

PLAN COMPLETE
