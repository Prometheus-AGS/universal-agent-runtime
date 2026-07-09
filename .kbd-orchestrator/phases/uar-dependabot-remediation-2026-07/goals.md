# Goals — uar-dependabot-remediation-2026-07

Seeded from `prometheus-package-integration`'s reflection.md (candidate 3:
"a fresh Dependabot/security posture re-check") plus the user's explicit
choice to handle this first, once the 52-vulnerability count surfaced on
push (`git push origin main` → GitHub, 2026-07-06).

## Verified starting numbers (pulled live via `gh api ... /dependabot/alerts`, not just the push summary)

**52 open alerts total**: 3 critical, 10 high, 32 medium, 7 low.

By ecosystem/manifest:

| Manifest | Ecosystem | Count |
|---|---|---|
| `Cargo.lock` | rust | 11 (incl. 2 critical: `failure` crate type-confusion, GHSA-jq66-xh47-j9f3 / GHSA-r98r-j25q-rmpr) |
| `package-lock.json` (root) | npm | 30 |
| `frontend/pnpm-lock.yaml` | npm | 10 (incl. `undici`, `vite` — high) |
| `sdks/typescript/package.json` | npm | 1 (critical: `vitest`, GHSA-5xrq-8626-4rwp — arbitrary file read/execute via Vitest UI server) |

## Goals

1. **Triage all 52 alerts** — for each, determine: fixed-version available? direct or transitive dependency? actually reachable in this codebase's usage (e.g. is the vulnerable code path exercised), or dormant? This project's established pattern (from `uar-security-deps-and-hygiene`) is to verify each claim directly rather than trust the advisory's severity label alone — some "critical" alerts may be low real-world risk here (e.g. `vitest`'s UI-server RCE only matters if the UI server is ever exposed, which it may not be in this project's CI/dev usage).
2. **Resolve what's safely upgradable** — bump pinned versions where a fix is available and compatible; re-verify build/test/clippy green after each ecosystem's batch (Rust via `cargo test --lib` + `cargo clippy --lib`, npm via the existing `bun run build`/`bun run check`/frontend `pnpm` build).
3. **Disclose what can't be resolved yet** — e.g. a fix requires a major version bump with breaking changes, or the vulnerable dependency is itself transitively pinned by something else not yet updated. Document explicitly, don't silently leave it looking resolved.
4. **Re-affirm or revise the D-D architectural decision** (git-sourced dependency pins in `docs/DEPENDENCY_MANAGEMENT.md`/`docs/ARCHITECTURE.md`) if any of the pinned git dependencies (`rmcp`, `surreal-memory`, `kreuzberg`, `prometheus_parking_lot`) are implicated.

## Non-goals

- Full dependency modernization / major-version migrations unrelated to a security advisory (out of scope; would be its own phase).
- The other two candidates from `prometheus-package-integration`'s reflection (original #14's carried-over live-bus/409 test gap; live macOS/Windows verification of `provisioning.rs`) — explicitly deferred, not abandoned, per the user's choice to handle this first.
