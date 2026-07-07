## Why

`cargo audit` and Dependabot both flag `hickory-proto` (CPU exhaustion +
DNSSEC validation bypass/DoS CVEs) and `tokio-tar` (file smuggling via
malformed PAX headers). `hickory-proto` is pulled in only when the
optional `sandbox-microsandbox` feature is enabled (via
`microsandbox-network`); `tokio-tar` is a dev-dependency-only crate (via
`testcontainers`). This change traces actual reachability for both before
deciding whether a fix is needed at all.

## What Changes

- **`hickory-proto` 0.25.2** (`RUSTSEC-2025-0006`, `RUSTSEC-2026-0118`,
  `RUSTSEC-2026-0119`): **no code change.** Traced with
  `cargo tree --all-features -i hickory-proto`: it's pulled in transitively
  via `microsandbox-network` (behind UAR's optional, off-by-default
  `sandbox-microsandbox` feature), but grepping `microsandbox-network` and
  every other `microsandbox-*` crate's source for `hickory`/`Resolver`/
  `lookup_ip`/DNS-usage patterns turns up **zero call sites** — the
  dependency is declared in `microsandbox-network`'s `Cargo.toml` but never
  actually invoked. `RUSTSEC-2025-0006` is already patched in our locked
  0.25.2. `RUSTSEC-2026-0118` additionally requires the `dnssec-ring`/
  `dnssec-aws-lc-rs` feature to even compile in the vulnerable
  `DnssecDnsHandle` code — `cargo tree -f "{p} {f}"` confirms neither
  feature is activated in our build. Even if we wanted to preemptively fix
  `RUSTSEC-2026-0118`/`-0119` (both need `hickory-proto >= 0.26.x`),
  `microsandbox-network`'s own `Cargo.toml` pins `hickory-proto = "0.25"` /
  `hickory-resolver = "0.25"` (caret ranges) — a fix isn't even available
  to us without microsandbox-network's own upstream bumping first. Disclosed
  as not reachable (dead dependency in our current feature configuration).
- **`tokio-tar` 0.3.1** (`RUSTSEC-2025-0111`, file smuggling,
  `patched = []` — the crate is archived/unmaintained, no fix will ever
  exist): traced its only reverse dependency, `testcontainers` (a
  dev-dependency) — grepped the entire repo for `testcontainers::`,
  `GenericImage`, `ContainerAsync`, and any actual API usage: **zero call
  sites**. `testcontainers` is a completely unused dev-dependency. Removed
  it from `Cargo.toml` entirely, which fully eliminates `tokio-tar` (and 10
  other transitively-exclusive crates: `bollard`, `bollard-stubs`,
  `hyperlocal`, `hyper-named-pipe`, `parse-display`,
  `parse-display-derive`, `redox_syscall`, `structmeta`,
  `structmeta-derive`) from the dependency tree — a complete fix, not just
  a disclosed non-issue.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None — no spec-level requirement changes. This removes an unused
dev-dependency and disclosures a non-reachable transitive advisory; no
UAR source code changed.

## Impact

- **Affected code**: `Cargo.toml` (removed `testcontainers` line),
  `Cargo.lock` (183 lines removed: `testcontainers`, `tokio-tar`, and 9
  exclusively-used transitive crates — no other package versions changed).
- **Runtime UX / provider compatibility / realtime state**: none.
- **Test infrastructure**: `testcontainers` was never actually used by any
  test in this repo (verified via full-repo grep) — removing it has zero
  effect on test coverage or behavior.
- **KBD workflow state**: `progress.json` for
  `uar-dependabot-remediation-2026-07` updated to DONE for this change.
