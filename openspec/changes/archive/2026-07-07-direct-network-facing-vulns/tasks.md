## 1. Reachability investigation

- [x] 1.1 Trace `hickory-proto` reverse deps via
      `cargo tree --target all --all-features -i hickory-proto` — confirms
      it's behind the optional `sandbox-microsandbox` feature, pulled in
      via `microsandbox-network`.
- [x] 1.2 Grep every `microsandbox-*` crate's source for any actual
      `hickory`/DNS-resolution call site — found none; declared but unused
      dependency.
- [x] 1.3 Confirm `dnssec-ring`/`dnssec-aws-lc-rs` features are not
      activated (`cargo tree -f "{p} {f}"`), ruling out `RUSTSEC-2026-0118`
      even if the dependency were used.
- [x] 1.4 Confirm `microsandbox-network`'s own `Cargo.toml` pins
      `hickory-proto`/`hickory-resolver` to `"0.25"` (caret), meaning
      `RUSTSEC-2026-0118`/`-0119`'s required `>= 0.26.x` fix isn't even
      available to us without an upstream bump there.
- [x] 1.5 Trace `tokio-tar` reverse deps — only via `testcontainers` (dev
      dependency). Grep full repo for `testcontainers::`/`GenericImage`/
      `ContainerAsync` — zero call sites, confirmed unused.

## 2. Apply the fix

- [x] 2.1 `hickory-proto`: no code change — disclosed as not reachable
      (see `findings.md`).
- [x] 2.2 Remove unused `testcontainers = "0.23"` line from `Cargo.toml`.
- [x] 2.3 Regenerate `Cargo.lock` scoped to just this manifest change
      (verified the diff is exactly testcontainers + its 10 exclusive
      transitive deps removed, no unrelated version churn).

## 3. Verify

- [x] 3.1 `cargo check --lib --tests` — clean.
- [x] 3.2 `cargo test --lib` — 387/388 pass (1 pre-existing ignore),
      unchanged from baseline.
- [x] 3.3 `cargo clippy --lib` — 499 warnings, same baseline, zero new.
- [x] 3.4 `cargo audit` — confirmed `tokio-tar`/`RUSTSEC-2025-0111` fully
      cleared; `hickory-proto`'s 2 advisories still listed as expected (no
      fix available at our locked ecosystem constraint; disclosed not
      reachable).

## 4. Update docs and KBD state

- [x] 4.1 Add a disposition note to `docs/DEPENDENCY_MANAGEMENT.md` for
      `hickory-proto` (not reachable, dead dependency behind an optional
      feature) — no note needed for `tokio-tar` since it's now fully
      removed, not just mitigated.
- [x] 4.2 Update
      `.kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json`
      (`change_status.direct-network-facing-vulns` → DONE,
      `changes_completed` incremented).
