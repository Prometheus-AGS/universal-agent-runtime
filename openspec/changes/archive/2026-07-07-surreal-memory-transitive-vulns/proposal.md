## Why

`cargo audit` surfaces 3 advisories reachable through the `surreal-memory`
git dependency (which pulls in `surrealdb-core` as its embedded storage
engine): `ammonia` 4.1.2 (mXSS, `RUSTSEC-2026-0193`), `crossbeam-epoch`
0.9.18 (invalid pointer dereference in `Display`/`Pointer` formatting,
`RUSTSEC-2026-0204`), and `rsa` 0.9.10 (Marvin Attack timing side-channel,
`RUSTSEC-2023-0071`). None of these are in the 52-alert Dependabot
baseline. `surreal-memory` tracks `branch = "main"` rather than a pinned
SHA, so before treating any of these as urgent this change confirms
reachability and checks whether a plain `cargo update` on the affected
transitive crates already resolves them without needing to touch the
`surreal-memory` pin itself.

## What Changes

- `ammonia` 4.1.2 → 4.1.3 (`cargo update -p ammonia`): clears
  `RUSTSEC-2026-0193`. Source-inspected `surrealdb-core`'s only call site
  (`fnc::string::html::sanitize` → `ammonia::clean(&arg)`, the SurrealQL
  `string::html::sanitize()` builtin) — it uses ammonia's default config,
  which does not enable the `math`/`annotation-xml` tags the advisory
  requires, so this specific CVE was not reachable here even before the
  bump. Applied the upgrade anyway since it's free (semver-compatible,
  zero-risk) and fully clears the advisory rather than leaving a
  disclosed-but-technically-open finding.
- `crossbeam-epoch` 0.9.18 → 0.9.20 (`cargo update -p crossbeam-epoch`):
  clears `RUSTSEC-2026-0204`. The flaw only triggers when application code
  formats (`Display`/`{:?}` via the affected impl) an `Atomic`/`Shared`
  pointer created via `::null()`; UAR/surrealdb-core/rayon's own code
  doesn't do this, but the bump is free and fully clears the advisory.
- `rsa` 0.9.10: **no code change** — `RUSTSEC-2023-0071` has `patched = []`
  (no fixed version exists at all; upstream has not shipped a
  constant-time implementation). Traced actual reachability: UAR's own
  JWT usage (`src/uar/security/middleware.rs`, `api_keys.rs`) is
  HMAC-only (`EncodingKey::from_secret`/`DecodingKey::from_secret`, no
  RSA algorithm); UAR's SurrealDB usage is `Root` (username/password)
  auth, not RECORD/scope JWT auth, so `surrealdb-core`'s own `rsa`-via-
  `jsonwebtoken` path isn't exercised either. `rsa` **is** reachable via
  `liter-llm`'s Vertex AI OAuth flow (`crates/liter-llm/src/auth/vertex_oauth.rs`,
  `Header::new(Algorithm::RS256)` + `EncodingKey::from_rsa_pem`, used only
  when a Vertex AI provider is configured with service-account
  credentials) — but the Marvin Attack's threat model (an attacker
  triggering many RSA operations and measuring network-observable timing
  to recover key bits) doesn't fit this flow: it's an outbound,
  self-triggered JWT-signing operation (UAR signs its own assertion to
  authenticate to Google's OAuth endpoint), not a network-facing
  decrypt/verify oracle an external attacker controls the input to or can
  time. Disclosed as accepted risk; no fix is possible at the crate-pin
  level regardless since no patched version exists.

## Capabilities

### New Capabilities

None — see `Modified Capabilities` below.

### Modified Capabilities

- `dependency-security-posture`: adds the "Surreal-Memory Transitive
  Advisory Disposition" requirement (scoped `cargo update` over git-pin
  resync when a compatible fix exists; disclose accepted risk when
  `patched = []`). Added retroactively, same as the other 3 Round-1
  changes — see `openspec/changes/first-party-direct-dep-hygiene/` for
  why. Otherwise no other spec-level requirement changes; this is a
  dependency-lockfile remediation (2 transitive version bumps) plus a
  disclosed non-fixable
finding.

## Impact

- **Affected code**: `Cargo.lock` only (`ammonia`, `crossbeam-epoch`
  transitive resolutions). No `Cargo.toml` changes — the `surreal-memory`
  git pin itself (`branch = "main"`) is untouched; these fixes came from
  scoped `cargo update -p <crate>` calls, lower blast-radius than
  resyncing the whole branch pin.
- **Runtime UX / provider compatibility / realtime state**: none.
- **KBD workflow state**: `progress.json` for
  `uar-dependabot-remediation-2026-07` updated to DONE for this change.
- **Dependencies**: none added; 2 transitive crates bumped within their
  existing semver-compatible ranges.
