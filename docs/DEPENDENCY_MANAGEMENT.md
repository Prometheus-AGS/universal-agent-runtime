# Dependency Management

This document describes how git-sourced dependencies are managed in UAR and provides the standard operating procedure for upgrading them.

## Why Git Dependencies Are Pinned

UAR uses several crates sourced directly from Git repositories rather than crates.io:

| Crate | Repository | Reason |
|-------|-----------|--------|
| `rmcp` | `modelcontextprotocol/rust-sdk` | MCP Rust SDK is pre-release; no stable crates.io version |
| `surreal-memory` | `Prometheus-AGS/surreal-memory-server` | Internal library, not published |
| `kreuzberg` | `kreuzberg-dev/kreuzberg` | Default local document intelligence provider; pinned to a release tag (not a branch) |
| `prometheus_parking_lot` | `Prometheus-AGS/prometheus-parking-lot-rs` | Internal library, not published |

Most git dependencies are **pinned to a specific commit SHA** via `rev = "..."` in `Cargo.toml`. `kreuzberg` is pinned to a release `tag` (currently `v4.9.8`) rather than a branch or commit SHA, so upgrades happen by moving to a newer tag rather than an upstream `main` sync. Pinning ensures:

- **Reproducible builds**: The same SHA is resolved every time regardless of upstream changes
- **CI stability**: The CI pipeline does not break due to unexpected upstream commits
- **Audit trail**: The `Cargo.toml` history shows exactly which upstream version was adopted and when

## Current Pinned Versions

```toml
rmcp          = rev "26b65b6b88c5552447905923f683b6e4720a5600"
surreal-memory = rev "f9ab1c29944b86d44c23ea0e6192fa3d39acbde8"
kreuzberg     = tag "v4.9.8" on kreuzberg-dev/kreuzberg
prometheus_parking_lot = rev "ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0"
```

**Corrected (`uar-post-dependabot-followup-2026-07`)**: this table had
drifted out of sync with `Cargo.toml` on 3 of its 4 entries (`rmcp` and
`prometheus_parking_lot` had both been bumped in later phases without
this table being updated; `surreal-memory` previously showed a `rev`
value here even though `Cargo.toml` actually had `branch = "main"` at the
time — this table was aspirational, not descriptive, on that line). All 4
values above are re-verified directly against live `Cargo.toml` state as
of this correction. `surreal-memory` is genuinely `rev`-pinned now (see
below) — the table's claim finally matches reality. Re-verify this table
against `Cargo.toml` whenever a pinned dependency is bumped, rather than
assuming it's already correct.

## Upgrade SOP

Follow these steps when upgrading a pinned git dependency:

### 1. Identify the target commit

```bash
# Get the latest commit SHA on the default branch
git ls-remote https://github.com/<org>/<repo>.git HEAD

# Or list tags if upgrading to a specific release
git ls-remote --tags https://github.com/<org>/<repo>.git
```

### 2. Update Cargo.toml

Change the `rev = "..."` field for the target crate to the new SHA:

```toml
rmcp = { git = "https://github.com/modelcontextprotocol/rust-sdk", rev = "<new-sha>", features = [...] }
```

### 3. Refresh Cargo.lock

```bash
cargo update -p rmcp
```

This updates only the named package in the lock file without touching other dependencies.

### 4. Verify the build

```bash
cargo build
```

Fix any compilation errors caused by upstream API changes before proceeding.

### 5. Run the test suite

```bash
cargo test --lib          # Unit tests (fast, no Docker)
cargo test                # Full suite (requires Postgres + Redis)
```

### 6. Commit both files together

Always commit `Cargo.toml` and `Cargo.lock` as a single atomic commit:

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(deps): upgrade rmcp to <new-sha>"
```

Including the SHA in the commit message provides a clear audit trail.

## Upgrading to crates.io Once Available

When a git-sourced crate publishes a stable crates.io release:

1. Remove the `git = "..."` and `rev = "..."` fields
2. Add `version = "x.y.z"` instead
3. Run `cargo update -p <crate>` to pull from the registry
4. Follow steps 4–6 above

## Security Advisories

Run `cargo audit` periodically to check for known vulnerabilities in all dependencies (including transitive ones):

```bash
cargo install cargo-audit
cargo audit
```

**Corrected (`uar-dependabot-remediation-2026-07`)**: this document previously
claimed "the CI pipeline runs `cargo audit` as part of the release workflow
(`release.yml`)." That step exists in `release.yml`, but that workflow only
triggers on a pushed version tag or a published GitHub release — and as of
this writing, this repository has never cut one (`gh run list
--workflow=release.yml` returns zero runs, ever), so the step has never
actually executed. A dedicated `.github/workflows/security-audit.yml` now
runs `cargo audit` + `npm audit` (root) + `pnpm audit` (`frontend/`) +
`npm audit` (`sdks/typescript/`) on its own weekly schedule (`workflow_dispatch`
also available for on-demand runs), deliberately decoupled from
`release.yml`'s tag/release trigger — "when we cut a release" and "how often
we scan for CVEs" are separate concerns. It ignores only the advisories
already disclosed with a rationale in this document (see sections below);
any new, undisclosed advisory fails the job. For non-release branches, you
can still run `cargo audit` / `npm audit` / `pnpm audit` manually before
merging significant dependency changes.

### Known open advisories: `kreuzberg` → `lopdf` / `quick-xml`

As of `uar-dependabot-remediation-2026-07` (`openspec/changes/kreuzberg-reachable-vulns/`),
`cargo audit` reports 3 advisories reachable through the `kreuzberg` pin
(`lopdf` 0.40.0, `quick-xml` 0.37.5/0.39.4/0.40.1):

- **`RUSTSEC-2026-0187`** (lopdf stack overflow) and **`RUSTSEC-2026-0194`**
  (quick-xml quadratic attribute-check DoS) are **confirmed reachable**
  (source-inspected call sites in kreuzberg/biblib/calamine). No upstream
  `kreuzberg` tag through `v5.0.0-rc.35` fixes both, and a
  `[patch.crates-io]` override doesn't work either (3 semver-incompatible
  quick-xml resolutions from manifests we don't control). **Mitigated, not
  fixed**: `KreuzbergConfig.max_input_bytes` (default 100 MiB) and
  `extraction_timeout_secs` (default 60s) bound the blast radius of a
  crafted document without patching the vulnerable crates. These 2
  advisories will keep appearing in `cargo audit` output until kreuzberg
  ships a stable release with both fixed, or a future change forks and
  backports the fixes.
- **`RUSTSEC-2026-0195`** (quick-xml unbounded namespace-allocation DoS) is
  **confirmed not reachable** — nothing in kreuzberg/biblib/calamine uses
  `NsReader`, the only affected API. No action needed; still listed by
  `cargo audit` since the crate version is unchanged.

See `openspec/changes/kreuzberg-reachable-vulns/findings.md` for the full
reachability trace.

### Known open advisory: `rsa` (Marvin Attack, no fix exists)

`cargo audit` lists **`RUSTSEC-2023-0071`** (`rsa` 0.9.10, timing
side-channel key recovery) via `jsonwebtoken` ← `liter-llm`'s Vertex AI
OAuth service-account JWT signing (`Algorithm::RS256`). This advisory has
**`patched = []`** — no version of the `rsa` crate fixes it; there is
nothing to upgrade to. Traced reachability: the signing is an outbound,
self-triggered operation (UAR signs its own assertion to authenticate to
Google's OAuth endpoint), not a network-facing decrypt/verify oracle an
external attacker controls the input to or can time — the Marvin Attack's
threat model doesn't fit this flow. UAR's own JWT usage elsewhere
(`src/uar/security/`) is HMAC-only and does not use `rsa` at all.
**Accepted risk, disclosed** — see
`openspec/changes/surreal-memory-transitive-vulns/findings.md`.

### Known open advisory: `hickory-proto` (dead dependency, not reachable)

`cargo audit` lists 2 advisories for `hickory-proto` 0.25.2
(`RUSTSEC-2026-0118`, `RUSTSEC-2026-0119`), pulled in only when the
optional `sandbox-microsandbox` feature is enabled (via
`microsandbox-network`). Neither is reachable: no `microsandbox-*` crate
actually calls into `hickory-proto`/`hickory-resolver` anywhere (it's a
declared-but-unused dependency of `microsandbox-network`), and
`RUSTSEC-2026-0118` additionally requires the `dnssec-ring`/
`dnssec-aws-lc-rs` feature, which isn't activated. Both advisories require
`hickory-proto >= 0.26.x` to fix, but `microsandbox-network`'s own
`Cargo.toml` pins `hickory-proto`/`hickory-resolver` to `"0.25"` — a fix
isn't available to us without their upstream bumping first. **Not
reachable, disclosed** — see
`openspec/changes/direct-network-facing-vulns/findings.md`. Re-check this
disposition if UAR ever adds real DNS-resolution logic on top of
`microsandbox-network`.

### Resolved: `serde_yml` → `serde_norway` (unmaintained + unsound)

As of `uar-dependabot-remediation-2026-07`
(`openspec/changes/first-party-direct-dep-hygiene/`), `cargo audit` flagged
`serde_yml` 0.0.12 as both unmaintained and unsound. Unlike this project's
git-pinned dependencies (see "Why Git Dependencies Are Pinned" above),
`serde_yml` was a direct, first-party-controllable dependency with no
upstream fix in progress — replaced with `serde_norway` 0.9.42 (an
actively maintained fork with a compatible `Serialize`/`Deserialize`-based
API) across its 3 call sites. This also fully eliminated `libyml` (an
unmaintained native library pulled in only transitively via `serde_yml`)
from `Cargo.lock`. `anyhow`/`memmap2` unsoundness reports surfaced in the
same assessment were re-checked against the currently pinned versions
(`anyhow` 1.0.103, `memmap2` 0.9.11) and do not apply — no action needed.

### Known orphaned advisory: `quinn-proto` (not in resolved graph)

`cargo audit` lists `RUSTSEC-2026-0185` (`quinn-proto` 0.11.14, remote
memory exhaustion), but `cargo tree -i quinn-proto --target all --all-features`
resolves to zero reverse dependencies — the entry is present in
`Cargo.lock` but not reachable in the currently resolved dependency graph
under any feature/target combination. `reqwest`'s enabled features
(`json`, `stream`, `rustls-tls-native-roots`, `multipart`) never activate
HTTP/3 either, ruling out that suspected path too. Not assigned to any of
this phase's 8 changes; likely to self-prune on a future full
`cargo update`. See
`openspec/changes/first-party-direct-dep-hygiene/findings.md`.

### Resolved: unused `grcov` dev-dependency removed

As of `uar-dependabot-remediation-2026-07`
(`openspec/changes/grcov-toolchain-refresh/`), `cargo audit` attributed 18
unmaintained/unsound warnings (`atty`, `failure`, `ansi_term`,
`proc-macro-error`, plus their exclusive chain: `cargo-binutils`,
`rustc-cfg`, `tabled`, `tabled_derive`, `clap` 2.34.0) to the
`grcov = "0.8"` entry in `Cargo.toml`'s `[dev-dependencies]`.
Investigation found this entry unused: `grep -rn "use grcov\|grcov::"`
finds zero call sites in the repo, and the coverage tool actually invoked
(`tools/coverage.sh`, `tools/test-all.sh`, and 3 CI workflows) is a
separately `cargo install`'d CLI binary, entirely decoupled from this
workspace's `Cargo.lock`. **Fully eliminated** by removing the unused
`[dev-dependencies]` entry — a complete fix, not a disclosure. **Plan
correction**: the phase's `plan.md` also listed `paste` as expected to
clear; `cargo tree -i paste` shows it's pulled in via `kreuzberg`/`burn`,
unrelated to `grcov` — left unchanged. `instant`, `number_prefix`, `scc`,
and `ttf-parser` from the same 18-warning tally are likewise unrelated to
`grcov` (see `openspec/changes/grcov-toolchain-refresh/findings.md` for
each crate's actual path) and out of this change's scope.

### Known orphaned/gated advisory: `proc-macro-error2` (unmaintained)

`cargo audit` lists `proc-macro-error2` 2.0.1 (`RUSTSEC-2026-0173`,
unmaintained) via `microsandbox` (behind the optional, off-by-default
`sandbox-microsandbox` feature) → `oci-spec`/`sea-orm-macros`. Pre-existing
and unrelated to the `grcov` removal above (confirmed via
`git diff Cargo.lock` — this entry was untouched by that change). Same
disposition class as `hickory-proto` (unreachable unless
`sandbox-microsandbox` is enabled); not assigned to any of this phase's 8
changes. See `openspec/changes/grcov-toolchain-refresh/findings.md`.

### Resolved: root `package-lock.json` npm audit findings

As of `uar-dependabot-remediation-2026-07`
(`openspec/changes/npm-root-remediation/`), a live `npm audit` against the
root `package.json`/`package-lock.json` (npm-managed dev tooling only —
`playwright`, `eslint`, `prettier`, `tailwindcss`, `typescript`,
`monocart-coverage-reports`, `@tauri-apps/cli`; the shipped frontend app
builds via `pnpm -C frontend`, a separate lockfile) found 15 findings (11
moderate, 4 high — `ajv`, `brace-expansion`, `js-yaml`, `uuid`, `dompurify`,
`flatted`, `lodash-es`, `minimatch`, `picomatch`, and a
`chevrotain`/`langium`/`@mermaid-js/parser`/`mermaid` chain that all traced
to a single vulnerable `lodash-es` resolution). All 15 had
`fixAvailable: true` (semver-compatible, no `--force` needed). **Fully
resolved** via `npm audit fix` — `package.json` had zero diff (lockfile-only
change), `npm audit` confirms 0 vulnerabilities afterward. See
`openspec/changes/npm-root-remediation/findings.md`.

### Resolved: frontend `pnpm-lock.yaml` npm audit findings

As of `uar-dependabot-remediation-2026-07`
(`openspec/changes/frontend-npm-remediation/`), a live `pnpm audit` against
`frontend/` found 11 findings (4 high, 4 moderate, 3 low): `vite` (direct
devDependency), `undici` (×7, transitive via `packages/prometheus-entity-management`'s
`jsdom`), `js-yaml` (transitive via `eslint`'s `@eslint/eslintrc`), and
`esbuild` (transitive, dual-resolved via both `vite` and `tsup`). All 4
resolved to patched versions within their parents' already-declared ranges
(`pnpm update`/`pnpm -r update`, no override needed) **except** `esbuild`
via `tsup` (pinned to exactly `^0.27.0`, no compatible patched release
available) — a single `pnpm-workspace.yaml` override was added, pinned to
the exact patched version (`"0.28.1"`, not an open-ended range). **Caught
and corrected mid-change**: an initial `pnpm audit --fix` run generated an
open-ended `vite` override that resolved to an unintended major-version
bump (`vite@8.1.3`); reverted and redone deliberately. `pnpm audit` now
reports 0 vulnerabilities; frontend build/typecheck verified green. See
`openspec/changes/frontend-npm-remediation/findings.md`.

### Resolved: `sdks/typescript` had no lockfile at all

As of `uar-dependabot-remediation-2026-07`
(`openspec/changes/sdk-typescript-lockfile-and-ci-audit-fix/`),
`sdks/typescript/package.json` declared `"vitest": "^2.0.0"` with no
`package-lock.json`/`pnpm-lock.yaml`/`yarn.lock` at all. Dependabot's
critical alert (`GHSA-5xrq-8626-4rwp`) affects `vitest < 3.2.6` or
`>= 4.0.0, < 4.1.0` — the entire declared `^2.0.0` range falls inside the
vulnerable window, so a lockfile regenerate alone would not have fixed it;
the declared range itself had to move. Bumped to `^4.1.10` (current stable,
matching the version line already used by `frontend/`) and generated a real
`package-lock.json` via `npm install`. This also surfaced the same
`esbuild`-via-`tsup` blocker seen in `frontend-npm-remediation` (`tsup`
pins `esbuild` to exactly `^0.27.0`, no compatible patched release exists)
— resolved via an `overrides` entry in `package.json`, pinned to the exact
patched version (`"0.28.1"`). `npm audit` now reports 0 vulnerabilities
(was 1). `tsc --noEmit`, `tsup` build, and `vitest --run` all confirmed
working; note `sdks/typescript` currently has zero test files, so
`vitest --run` exits non-zero ("No test files found") — pre-existing gap,
not introduced by this change, and not currently exercised by any CI
workflow. See
`openspec/changes/sdk-typescript-lockfile-and-ci-audit-fix/findings.md`.

### Resolved/disclosed: 9 never-assigned unmaintained/unsound warnings

As of `uar-post-dependabot-followup-2026-07`
(`openspec/changes/triage-unassigned-unmaintained-warnings/`), 9
unmaintained/unsound `cargo audit` warnings that no change in the prior
phase was assigned to were triaged:

- **`instant` (`RUSTSEC-2024-0384`) — fixed.** Pulled in via `notify`
  7.x → `notify-types` 1.0.1. Bumped `notify = "7"` → `notify = "8"`;
  `notify-types` 2.0.0 dropped `instant` for `web-time`. The only call
  site (`src/uar/runtime/skills/watcher.rs`) uses long-stable core API,
  unaffected by the major bump.
- **`bincode` (`RUSTSEC-2025-0141`) — no fix exists, accepted risk.**
  Reachable via `burn` (always-compiled). The bincode maintainers
  permanently ceased development after a doxxing/harassment incident — no
  version will ever be patched. A fix requires `burn` to migrate
  serialization backends, outside UAR's control.
- **`paste` (`RUSTSEC-2024-0436`) — no single fix point, accepted risk.**
  Reachable via both `kreuzberg`/`biblatex` and the `burn` family
  (independent paths). Stable, simple proc-macro crate with no unsound
  behavior reported; two unrelated upstream owners would each need to
  move off it.
- **`ttf-parser` (`RUSTSEC-2026-0192`) — no fix through current
  kreuzberg, accepted risk.** Same `lopdf` dependency already covered by
  the "Known open advisories: `kreuzberg` → `lopdf` / `quick-xml`"
  section above.
- **`number_prefix` (`RUSTSEC-2025-0119`) — too deep to control,
  accepted risk.** Reachable via `indicatif` → `hf-hub` → `fastembed` →
  `mempalace-core` → `surreal-memory` — 4 hops beyond `surreal-memory`
  itself, none controlled by UAR.
- **`rustls-pemfile` (`RUSTSEC-2025-0134`) and `proc-macro-error2`
  (`RUSTSEC-2026-0173`) — not reachable by default.** Both via
  `microsandbox-*`, behind the optional, off-by-default
  `sandbox-microsandbox` feature — same disposition class as
  `hickory-proto` above.
- **`scc` (`RUSTSEC-2026-0205`) — dev-only.** Via `serial_test`
  (`[dev-dependencies]`); never ships in the release binary.
- **`atomic-polyfill` (`RUSTSEC-2023-0089`) — orphaned lockfile entry.**
  `cargo tree -i --target all --all-features` finds zero reverse
  dependencies — same class as `quinn-proto` above; likely self-prunes on
  a future full `cargo update`.

See `openspec/changes/triage-unassigned-unmaintained-warnings/findings.md`
for the full investigation trace.
