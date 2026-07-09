## 1. Investigation

- [x] 1.1 Re-confirm reachability for all 9 crates (already traced in
      the phase's `assessment.md`) via `cargo tree -i --target all
      --all-features`.
- [x] 1.2 For each of the 5 reachable-in-a-normal-build crates
      (`bincode`, `instant`, `number_prefix`, `paste`, `ttf-parser`),
      check whether the immediate parent has a newer version that drops
      the flagged dependency:
      - `notify` 7→8: confirmed via crates.io — `notify-types` 2.0.0
        dropped `instant` for `web-time`.
      - `burn` (→ `bincode`): checked `RUSTSEC-2025-0141` directly — the
        bincode project is permanently abandoned (doxxing/harassment
        incident cited in the advisory), no version will ever be
        patched; not a `burn`-version question.
      - `kreuzberg`/`burn` (→ `paste`): stable, simple proc-macro crate,
        no unsound behavior reported, multiple unrelated owners — no
        single actionable fix point.
      - `kreuzberg` → `lopdf` (→ `ttf-parser`): same `lopdf` dependency
        already disclosed as unfixable-through-`v5.0.0-rc.35` in the
        prior phase's `kreuzberg-reachable-vulns`.
      - `indicatif`→`hf-hub`→`fastembed`→`mempalace-core`→`surreal-memory`
        (→ `number_prefix`): traced the full chain — 4 hops beyond
        `surreal-memory` itself, none of which UAR or (as far as this
        investigation can tell) `surreal-memory`'s own maintainers
        control directly.
- [x] 1.3 For the 4 remaining crates (`atomic-polyfill`,
      `rustls-pemfile`, `scc`, `proc-macro-error2`), confirm their
      already-known disposition class from the assessment (orphaned,
      feature-gated, dev-only respectively) still holds.

## 2. Apply the fix

- [x] 2.1 `Cargo.toml`: `notify = "7"` → `notify = "8"`.
- [x] 2.2 `cargo check` to regenerate `Cargo.lock` — confirm `instant`
      fully absent (`cargo tree -i instant` → no match), `notify`
      resolves to `8.2.0`.
- [x] 2.3 The other 8 crates: no code change — disclosed with rationale
      in `docs/DEPENDENCY_MANAGEMENT.md` (see step 4).

## 3. Verify

- [x] 3.1 `cargo check --lib --tests` — clean.
- [x] 3.2 `cargo test --lib` — 387/388 pass (1 pre-existing ignore), no
      regression.
- [x] 3.3 `cargo clippy --lib` — 499 warnings, same as baseline, zero new.
- [x] 3.4 `cargo audit` — confirmed `instant` cleared (warnings count
      9 → 8); all other 8 disclosed items still listed as expected, no
      new findings.

## 4. Update docs and KBD state

- [x] 4.1 Add disposition entries to `docs/DEPENDENCY_MANAGEMENT.md` for
      all 9 crates (1 fixed, 8 disclosed).
- [x] 4.2 Update
      `.kbd-orchestrator/phases/uar-post-dependabot-followup-2026-07/progress.json`
      (`change_status.triage-unassigned-unmaintained-warnings` → DONE,
      `changes_completed` incremented, `next_change` →
      `push-and-verify-security-audit-workflow`).
