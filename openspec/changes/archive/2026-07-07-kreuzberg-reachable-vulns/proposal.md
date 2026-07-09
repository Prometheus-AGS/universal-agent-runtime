## Why

`cargo audit` (run live during `uar-dependabot-remediation-2026-07`'s
assessment) surfaces 3 High-severity (CVSS 3.1) advisories reachable through
`kreuzberg`, UAR's git-pinned document-intelligence provider: `lopdf`
stack overflow (RUSTSEC-2026-0187) and two `quick-xml` DoS advisories
(RUSTSEC-2026-0194 quadratic runtime, RUSTSEC-2026-0195 unbounded memory
allocation). None of these three are in the 52-alert Dependabot baseline —
they were only found via a direct `cargo audit` + `cargo tree -i`
reachability trace. They are the most concretely exploitable findings in
the whole assessment: kreuzberg's actual job is parsing untrusted,
user-supplied documents (PDF via `lopdf`; XML-based formats like
`.xlsx`/`.docx` via `quick-xml`, reached through `biblib`/`calamine`), so a
crafted input file processed through the document-intelligence pipeline can
trigger a stack overflow or a denial-of-service directly.

## What Changes

- Check whether a newer `kreuzberg` tag/commit (currently pinned to
  `v4.9.8` per `docs/ARCHITECTURE.md`'s D-D decision) already pulls patched
  `lopdf`/`quick-xml` resolutions, per `docs/DEPENDENCY_MANAGEMENT.md`'s
  existing upgrade SOP.
- If a clean upstream bump resolves all 3 advisories, bump the `kreuzberg`
  git pin in `Cargo.toml` (and update `docs/DEPENDENCY_MANAGEMENT.md`'s
  pinned-version record).
- If no upstream tag yet carries the fix, evaluate a `[patch.crates-io]`
  override in UAR's own `Cargo.toml` as a fallback — heavier and riskier
  than a clean bump, so only after the upstream check comes back negative.
- Re-run `cargo audit` afterward to confirm all 3 findings actually clear
  under the `kreuzberg` provenance path (not just that a version number
  changed).

No new capability or requirement-level behavior changes — this is a
dependency-pin remediation with no API surface change.

## Capabilities

### New Capabilities

- `dependency-security-posture`: normative practice for triaging
  `cargo audit` findings against pinned/first-party dependencies — verify
  reachability, then fix, mitigate, or disclose as accepted risk, always
  recorded in `docs/DEPENDENCY_MANAGEMENT.md`. Added retroactively (see
  `openspec/changes/first-party-direct-dep-hygiene/` where this gap was
  caught) since none of this phase's changes originally declared a
  capability delta.

### Modified Capabilities

None — no other spec-level requirement changes. This only changes a pinned
dependency version (or adds a transitive patch override), not any
documented behavior of the document-intelligence capability.

## Impact

- **Affected code**: `Cargo.toml` (`kreuzberg` dependency entry, possibly a
  new `[patch.crates-io]` section), `Cargo.lock`.
- **Affected docs**: `docs/DEPENDENCY_MANAGEMENT.md` (pinned-version record
  for `kreuzberg` under D-D), `docs/ARCHITECTURE.md` if the D-D decision
  narrative needs a note about this recheck.
- **Runtime UX**: none — no change to document-intelligence request/response
  behavior; only the DoS/crash exposure surface for malicious input files.
- **Provider compatibility**: none.
- **Realtime state**: none.
- **KBD workflow state**: yes — `progress.json` / `current-waypoint.json`
  for `uar-dependabot-remediation-2026-07` must be updated to DONE for this
  change once merged, per the phase's Round 1 checkpoint.
- **Dependencies**: `kreuzberg` (git pin) is one of the 4 pins named in
  `docs/ARCHITECTURE.md`'s D-D decision ("git-sourced dependency pins are
  deliberate, not debt") — this change re-verifies, not reverses, that
  decision.
