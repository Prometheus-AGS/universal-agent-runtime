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
rmcp          = rev "085470025f690050e8776ffa939e7ba71d3abc01"
surreal-memory = rev "c6f95c905c16907ad58ef9049f32dcc9531d40eb"
kreuzberg     = tag "v4.9.8" on kreuzberg-dev/kreuzberg
prometheus_parking_lot = rev "32b481d6c5694545d35789894f6feecf5ac4ca3e"
```

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

The CI pipeline runs `cargo audit` as part of the release workflow (`release.yml`). For non-release branches, run it manually before merging significant dependency changes.

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
