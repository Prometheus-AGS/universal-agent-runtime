## Why

The phase assessment found 4 of `security-audit.yml`'s 7 `cargo audit`
ignore rationales had gone stale — upstream patches now exist for lopdf
(≥0.42), quick-xml (≥0.41), hickory-proto 0119 (≥0.26.1), and quinn-proto
(≥0.11.15) — and GitHub re-opened 2 hickory Dependabot alerts on the next
push. Under the 100%-customer-ready mandate, suppressed advisories must be
genuinely unfixable, not merely stale.

## What Changes

- quinn-proto: `cargo update` → 0.11.16 (RUSTSEC-2026-0185 cleared).
- kreuzberg: pin to the Prometheus-AGS/xberg fork (v4.9.9 + lopdf 0.42,
  quick-xml 0.41, calamine 0.36, biblib 0.7, html-to-markdown-rs
  constrained to 3.5.x) clearing RUSTSEC-2026-0187 and the kreuzberg-owned
  RUSTSEC-2026-0194/0195 copies; upstream PR xberg-io/xberg#1241 filed.
- **Remove the broken `sandbox-microsandbox` feature entirely** (never
  compiled — its test called a nonexistent API; excluded from all CI): this
  purges hickory-proto/hickory-resolver/microsandbox-* from Cargo.lock,
  clearing RUSTSEC-2026-0118/0119 AND both open GitHub Dependabot alerts
  (GHSA-q2qq-hmj6-3wpp, GHSA-3v94-mw7p-v465). Wasmtime + remote runners
  remain.
- Shrink security-audit.yml ignores 7 → 3 (quick-xml 0.39 via opendal 0.57
  [latest upstream, S3-cache-only reachability] ×2 advisories + rsa Marvin
  [no patch exists]); empty DISCLOSED_GHSA_IDS; rewrite
  docs/DEPENDENCY_MANAGEMENT.md sections to current truth.

## Capabilities

### Modified Capabilities
- `dependency-security-posture`: suppressed advisories must be genuinely
  unfixable with accurate, current rationales (not stale "no fix exists"
  claims).

### New Capabilities
(none)

## Impact

Cargo.toml/lock, src/sandbox/* (feature removal), settings enum,
security-audit.yml, DEPENDENCY_MANAGEMENT.md. External: fork repo +
upstream PR. KBD: change 3/9.
