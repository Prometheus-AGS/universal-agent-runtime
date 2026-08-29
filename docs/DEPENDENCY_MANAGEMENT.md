# Dependency Management

This document describes how git-sourced dependencies are managed in UAR and provides the standard operating procedure for upgrading them.

## Why Git Dependencies Are Pinned

UAR uses several crates sourced directly from Git repositories rather than crates.io:

| Crate | Repository | Reason |
|-------|-----------|--------|
| `rmcp` | `modelcontextprotocol/rust-sdk` | MCP Rust SDK is pre-release; no stable crates.io version |
| `surreal-memory` | `Prometheus-AGS/surreal-memory-server` | Internal library, stored as a curated source snapshot |
| `liter-llm` | `GQAdonis/liter-llm` | Internal provider runtime, stored as an exact gitlink |
| `kreuzberg` | `kreuzberg-dev/kreuzberg` | Default local document intelligence provider; pinned to a release tag (not a branch) |
| `prometheus_parking_lot` | `Prometheus-AGS/prometheus-parking-lot-rs` | Internal library, not published |

Most git dependencies are **pinned to a specific commit SHA** via `rev = "..."` in `Cargo.toml`. `kreuzberg` is pinned to a release `tag` (currently `v4.9.8`) rather than a branch or commit SHA, so upgrades happen by moving to a newer tag rather than an upstream `main` sync. Pinning ensures:

- **Reproducible builds**: The same SHA is resolved every time regardless of upstream changes
- **CI stability**: The CI pipeline does not break due to unexpected upstream commits
- **Audit trail**: The `Cargo.toml` history shows exactly which upstream version was adopted and when

## Current Pinned Versions

```toml
rmcp          = rev "26b65b6b88c5552447905923f683b6e4720a5600"
surreal-memory = curated snapshot "432eaa1ebbef66fc02b9bb1a1e63cc2fdb2149e8"
liter-llm     = gitlink "c5c6caac617eb931cd5009146a70831422ec236c" (1.18.2)
kreuzberg     = tag "v4.9.8" on kreuzberg-dev/kreuzberg
prometheus_parking_lot = rev "ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0"
```

**Corrected (`uar-post-dependabot-followup-2026-07`)**: this table had
drifted out of sync with `Cargo.toml` on 3 of its 4 entries (`rmcp` and
`prometheus_parking_lot` had both been bumped in later phases without
this table being updated; `surreal-memory` previously showed a `rev`
value here even though `Cargo.toml` actually had `branch = "main"` at the
time — this table was aspirational, not descriptive, on that line). All 4
values above were re-verified directly against live dependency state when that
correction was written. As of 2026-08-28, `surreal-memory` is a curated path
dependency whose immutable upstream provenance is recorded in
`vendor/git/README.md` and `versions.toml`; `liter-llm` is an exact gitlink.
Re-verify this table against those authorities whenever either input moves.

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

**Current execution policy (`deployment-only-actions-local-release-certification`)**:
`scripts/security-audit-local.sh` runs `cargo audit`, root/frontend package
audits, TypeScript SDK audit, OSV-Scanner, Grype against the digest-addressed
candidate image, and the Dependabot alerts gate locally. It writes a
source-bound release receipt. GitHub Actions are reserved for deployment
execution and deployment-specific validation, so no scheduled or release
workflow performs these product/security checks. Any new open Dependabot alert
or undisclosed advisory fails the local gate.

The dated notes below describe the advisory investigations and historical
workflow implementations that first exposed each issue. Their risk dispositions
remain useful; their workflow locations are superseded by the local gate above.

### Resolved (fork): `kreuzberg` → `lopdf` / `quick-xml`

**Update (`uar-final-production-hardening-2026-07`,
`re-remediate-stale-rustsec`)**: the "fork and backport" future-work noted
below is now done. UAR pins `kreuzberg` to the **Prometheus-AGS/xberg** fork
(branch `security/lopdf-quickxml-bumps`, based on upstream `v4.9.9`) which
bumps `lopdf` → 0.42 (clears `RUSTSEC-2026-0187`), `quick-xml` → 0.41,
`calamine` → 0.36 and `biblib` → 0.7 (clearing `RUSTSEC-2026-0194/0195` for
all kreuzberg-owned copies; biblib 0.7 dropped its `regex` feature — no other
API changes), and constrains `html-to-markdown-rs` to 3.5.x (3.6+ broke
`ImageInfo.dimensions` semver). An upstream PR with the same bumps has been
filed against `kreuzberg-dev/kreuzberg`; repoint at an upstream tag once a
release containing them ships. One `quick-xml` 0.39 copy remains via
`opendal` 0.57 (the **latest** upstream release) ← `liter-llm`'s cache layer:
DoS-class, reachable only when S3-backed cache storage is configured —
**upstream-blocked, disclosed** (tracked in `security-audit.yml`'s ignore
list).

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

### Resolved (feature removed): `hickory-proto`

**Update (`uar-post-dependabot-followup-2026-07`)**: GitHub's own
Dependabot/GHSA database tracks 2 additional `hickory-proto` advisories
not yet present in the RustSec advisory database `cargo audit` uses —
`GHSA-q2qq-hmj6-3wpp` (CPU exhaustion via O(n²) name compression) and
`GHSA-3v94-mw7p-v465` (NSEC3 unbounded loop on cross-zone responses).
Confirmed via `cargo tree -i hickory-proto --target all --all-features`
that the reachability path is identical to the 2 advisories below (same
`hickory-proto` 0.25.2, same `microsandbox-network` route, same optional
`sandbox-microsandbox` feature) — same disposition applies: **not
reachable, disclosed**. This confirms `cargo audit`'s RustSec-only
coverage can lag GitHub's own GHSA database for a given crate.

**Update (`uar-security-audit-alerts-gate-2026-07`)**: this check is now
automated. `security-audit.yml`'s `dependabot-alerts-gate` job calls
`gh api repos/<org>/<repo>/dependabot/alerts` on every scheduled/dispatched
run and fails if any **open** alert's GHSA ID isn't in the job's inline
`DISCLOSED_GHSA_IDS` allowlist (currently `GHSA-q2qq-hmj6-3wpp` and
`GHSA-3v94-mw7p-v465`, both `hickory-proto`, both disclosed above) —
closing the gap that let 2 real CVEs (`cmov`, `opentelemetry_sdk`) go
unnoticed by CI the phase before this one. The job authenticates with
`secrets.SUBMODULES_TOKEN` (reused; already provisioned for private-submodule
checkout across every workflow) rather than the default `GITHUB_TOKEN`,
which cannot read this endpoint under any `permissions:` grant — a hard
Actions platform limitation. **Whenever a new advisory is triaged and
disclosed here, its GHSA ID must also be added to `DISCLOSED_GHSA_IDS` in
`security-audit.yml`**, mirroring the existing `cargo audit --ignore`
convention for RustSec IDs. A manual `gh api dependabot/alerts` check
between scheduled runs remains useful but is no longer the only backstop.

**Update (`uar-final-production-hardening-2026-07`,
`re-remediate-stale-rustsec`)**: the optional `sandbox-microsandbox` feature
— the sole path pulling `hickory-proto` into `Cargo.lock` — has been
**removed entirely**. The feature had been documented-broken for multiple
phases (its integration test called an API that doesn't exist, so it could
not compile), was excluded from every CI job, and pinned `hickory-proto`
0.25.2 into the lockfile where both `cargo audit` (RUSTSEC-2026-0118/0119)
and GitHub Dependabot (the 2 GHSA IDs above) flagged it on every scan.
`hickory-proto`, `hickory-resolver`, and the whole `microsandbox-*` tree are
gone from `Cargo.lock`; the sandbox subsystem retains its Wasmtime and
remote-HTTP runners. Both RUSTSEC ignores and both `DISCLOSED_GHSA_IDS`
entries have been dropped from `security-audit.yml`. Re-adding a microVM
runner in the future should use a maintained upstream with current
`hickory` (or no DNS dependency at all).

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

### Resolved: `quinn-proto` orphan updated

**Update (`uar-final-production-hardening-2026-07`)**: `cargo update -p
quinn-proto` moved the orphaned lock entry to 0.11.16 (≥ the 0.11.15 patch
for `RUSTSEC-2026-0185`); the ignore has been removed from
`security-audit.yml`. The entry remains unreferenced in the resolved graph.

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

### Superseded: root `package-lock.json` npm audit graph

The historical npm remediation in `uar-dependabot-remediation-2026-07`
(`openspec/changes/npm-root-remediation/`) applied to a secondary
`package-lock.json`. The root project now declares `pnpm@11.15.0`, uses a
pnpm workspace for builds and releases, and its current manifest cannot be
re-resolved by npm because of conflicting ESLint peer graphs. Keeping the
stale npm lock created a second, non-reproducible dependency graph and new
Dependabot findings unrelated to the graph that ships.

As of `clear-dependabot-alert-baseline`, the root `package-lock.json` is
removed. Root install and audit CI use `pnpm install --frozen-lockfile` and
`pnpm audit` against the authoritative `pnpm-lock.yaml`. The frontend's stale
npm fallback lock is removed for the same reason: its workspace dependency
uses the `workspace:*` protocol and cannot be regenerated by npm. Website and
TypeScript SDK remain npm-managed and retain their independently reproducible
`package-lock.json` files.

### Disposition: Tauri `glib` 0.18 `VariantStrIter` unsoundness

Dependabot alert #140 (`GHSA-wrw7-89jp-8q8g`, `RUSTSEC-2024-0429`)
flags `glib` 0.18.5 in `src-tauri/Cargo.lock`. The patched glib line begins
at 0.20.0, but the current upstream desktop stack—Tauri 2.11.5, Wry 0.55.1,
Tao 0.35.3, GTK 0.18.2, and WebKit2GTK 2.0.2—still requires glib 0.18.
Forcing glib 0.20 would create incompatible GTK Rust types and is not a
valid dependency resolution.

The vulnerable surface is specifically iteration through
`glib::VariantStrIter`/`Variant::array_iter_str`. Repository source search
finds no first-party reference, and source search of the resolved Tauri,
Wry, Tao, GTK, WebKit2GTK, and GIO crates finds no reference either. The
only matches are the glib implementation and its own tests. Alert #140 is
therefore dismissed as `not_used`, with this reachability evidence, rather
than hiding the risk behind an incompatible override or deleting the
reproducibility lock. Reassess this disposition when Tauri's Linux stack
moves to GTK bindings that accept glib 0.20 or newer.

### Resolved: completion-audit Rust advisories

The `clear-dependabot-alert-baseline` completion audit on 2026-08-02 found
three newly published RustSec advisories before GitHub had opened corresponding
Dependabot alerts: `RUSTSEC-2026-0213` in `ammonia` 4.1.3 and
`RUSTSEC-2026-0222` / `RUSTSEC-2026-0223` in Wasmtime 46.0.1. The root
and Tauri `Cargo.lock` files now resolve `ammonia` 4.1.4 and the
Wasmtime/WASI/Wiggle 46.0.2 patch family. These are compatible patch-line
updates and require no new audit allowlist entries. Both audited graphs report
zero actionable vulnerabilities after applying their existing, documented
upstream-blocked exceptions.

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

### Resolved: 2 real CVEs found only via GitHub Dependabot, not `cargo audit`

As of `uar-post-dependabot-followup-2026-07`
(`openspec/changes/push-and-verify-security-audit-workflow/`), after
pushing this phase's work and dispatching `security-audit.yml` for the
first time, `gh api repos/.../dependabot/alerts` was checked directly
(prompted by GitHub reporting a much larger alert count at push time than
this project's own `cargo audit`/`npm audit`/`pnpm audit` tooling showed).
Found 2 real, reachable, always-compiled Rust CVEs that `cargo audit`'s
RustSec advisory database doesn't (yet) track:

- **`cmov`** (`CVE-2026-50185`, `GHSA-3rjw-m598-pq24`) — aarch64
  `Cmov`/`CmovEq` can produce wrong results if high bits of registers are
  set. Reachable via `liter-llm`'s `aws-sigv4` → `hmac`/`md-5` →
  `digest` → `ctutils` → `cmov` chain (used for AWS request signing,
  e.g. Bedrock). Fixed via a scoped `cargo update -p cmov --precise
  0.5.4` (patched version) — no `Cargo.toml` edit needed, already within
  the resolved range.
- **`opentelemetry_sdk`** (`CVE-2026-48504`, `GHSA-w9wp-h8wv-79jx`) —
  unbounded memory allocation in W3C Baggage propagation. A direct,
  always-compiled UAR dependency (`opentelemetry-otlp`/
  `opentelemetry_sdk`, both pinned `0.31.x`). Fixed by bumping the whole
  `opentelemetry` family in `Cargo.toml`: `opentelemetry` 0.31.0→0.32.0,
  `opentelemetry-otlp` 0.31.1→0.32.0, `opentelemetry_sdk` 0.31.0→0.32.1.
  This also required bumping `tracing-opentelemetry` 0.32.0→0.33.0 — its
  version number does not track `opentelemetry`'s 1:1; `0.32.0` was
  actually built against `opentelemetry` `0.31.x`'s API and failed to
  compile against `0.32.x` (`Layered<...>: Subscriber` trait-bound
  errors) until bumped to `0.33.0`.

Both confirmed fixed: `cargo tree -i cmov` → `0.5.4`; `cargo tree -i
opentelemetry_sdk` → `0.32.1`. `cargo check`/`test`/`clippy`/`audit` all
clean afterward, no regressions. **Lesson**: `cargo audit` alone is not
sufficient coverage — its RustSec advisory database can lag behind
GitHub's own GHSA database for the same crate. Check
`gh api repos/<org>/<repo>/dependabot/alerts?state=open` directly,
especially right after a push, rather than relying on `cargo audit`
exclusively.

### Bounded `image-size` documentation-build exception

The website's Docusaurus dependency graph currently carries `image-size`
advisories 1138808 and 1138809 without a compatible patched release. The
exposure is limited to trusted, tracked repository assets during a local
documentation build. `scripts/security-audit-local.sh` rejects ICNS, JXL,
HEIF, HEIC, and AVIF inputs before that build is accepted by checking both
filename extensions and content MIME types, so renaming affected content does
not bypass the gate.

- Owner: repository security maintainers.
- Review due: 2026-11-24.
- Reopen immediately if untrusted image ingestion is introduced or a compatible
  fixed `image-size` release becomes available.
- Do not broaden the exception to any other advisory or package root.

### Lockfile-only `rkyv` 0.7 advisory disposition

The 2026-08-28 release audit reported `RUSTSEC-2026-0235` for `rkyv`
0.7.46. The entry is an optional dependency declared by `rust_decimal`
1.42.0, which is required by SurrealDB 3.2.4. It is present in Cargo's
resolved lock metadata but is not activated by any supported target or
feature: `cargo tree --locked --all-features --target all --edges all -i rkyv@0.7.46`
returns no reverse dependency. The vulnerable archive-validation code is
therefore absent from the UAR build and release binary.

`scripts/security-audit-local.sh` ignores only this advisory ID and first
re-resolves the locked metadata plus all-target/all-edge inverse graph. The
audit fails before applying the ignore if `rkyv` 0.7.46 has any active reverse
dependency. Remove the exception when SurrealDB's dependency graph no longer
records `rkyv` 0.7, or immediately if a supported UAR feature begins activating
it. The same audit found `RUSTSEC-2026-0258` in `h2` 0.4.14; that finding was
fixed rather than accepted by updating the lockfile to `h2` 0.4.16.
