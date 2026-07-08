## Why

The prior phase (`uar-dependabot-remediation-2026-07`) surfaced 9
unmaintained/unsound `cargo audit` warnings that weren't assigned to any
of its 8 changes. This phase's assessment traced reachability for all 9
live; this change fixes what's fixable and discloses the rest with a
documented rationale, rather than leaving them silently un-actioned.

## What Changes

- **Fixed**: `instant` (`RUSTSEC-2024-0384`, unmaintained) — pulled in via
  `notify` 7.x → `notify-types` 1.0.1. `notify` 8.2.0 (current stable)
  dropped `instant` entirely in favor of `web-time`. Bumped
  `notify = "7"` → `notify = "8"` in `Cargo.toml`. The only call site
  (`src/uar/runtime/skills/watcher.rs`) uses stable core API
  (`Config`/`Event`/`EventKind`/`RecommendedWatcher`/`RecursiveMode`/
  `Watcher`/`Error`) unaffected by the major bump — compiles clean, no
  behavior change expected.
- **Disclosed, no fix available** (8 crates — details in `findings.md`):
  - `bincode` (via `burn`): upstream bincode project permanently ceased
    development after a doxxing/harassment incident (RUSTSEC-2025-0141) —
    no version will ever be patched; fixing requires `burn` itself to
    migrate serialization backends, outside UAR's control.
  - `paste` (via `kreuzberg`/`biblatex` and `burn`-family): simple,
    stable proc-macro crate that stopped receiving updates; no known
    unsound behavior, multiple unrelated upstream owners, no single fix
    point.
  - `ttf-parser` (via `kreuzberg` → `lopdf`): same underlying `lopdf`
    dependency already disclosed in the prior phase's
    `kreuzberg-reachable-vulns` — no kreuzberg release through
    `v5.0.0-rc.35` fixes it.
  - `number_prefix` (via `indicatif` → `hf-hub` → `fastembed` →
    `mempalace-core` → `surreal-memory`): 4 layers removed from any repo
    UAR controls (even `surreal-memory`, now first-party-adjacent, only
    pulls this in via `fastembed`'s own dependency choices).
  - `rustls-pemfile`, `proc-macro-error2` (both via `microsandbox-*`,
    behind the optional off-by-default `sandbox-microsandbox` feature):
    same disposition class as `hickory-proto` from the prior phase — not
    reachable in a default build.
  - `scc` (via `serial_test`, dev-dependency): never ships in the release
    binary.
  - `atomic-polyfill`: orphaned `Cargo.lock` entry, zero reverse
    dependencies found under any feature/target combination — same class
    as `quinn-proto`/`proc-macro-error2`'s sibling case, likely self-prunes
    on a future full `cargo update`.

## Capabilities

### New Capabilities

None — see `Modified Capabilities` below.

### Modified Capabilities

- `dependency-security-posture`: adds the "Abandoned Crate Disclosure"
  requirement (when an unmaintained-crate advisory has no patched version
  and no available fix within the project's control, disclose the
  specific reason — permanently abandoned upstream, too-deep transitive
  chain, or feature-gated non-default — rather than a generic "no fix
  available" note).

## Impact

- **Affected code**: `Cargo.toml` (`notify` version bump), `Cargo.lock`,
  `docs/DEPENDENCY_MANAGEMENT.md` (9 disposition entries).
- **Runtime UX / provider compatibility / realtime state**: none — the
  `notify` bump only changes a build-time-resolved dev-adjacent watcher
  dependency's internal timer implementation (`instant` → `web-time`),
  not UAR's own skill hot-reload behavior.
- **KBD workflow state**: `progress.json` for
  `uar-post-dependabot-followup-2026-07` updated to DONE for this change
  once verified.
