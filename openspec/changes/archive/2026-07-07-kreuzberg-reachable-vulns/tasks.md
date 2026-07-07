## 1. Investigate upstream fix availability

- [x] 1.1 Check `kreuzberg`'s upstream repo (git pin currently tag `v4.9.8`)
      for a newer tag/commit whose `Cargo.toml`/`Cargo.lock` pulls a patched
      `lopdf` (fixes RUSTSEC-2026-0187) and patched `quick-xml` (fixes
      RUSTSEC-2026-0194 and RUSTSEC-2026-0195).
- [x] 1.2 Record the finding (clean upstream fix available, or not) before
      touching `Cargo.toml`.

## 2. Apply the fix

**REVISED 2026-07-07** (see `findings.md`): no upstream `kreuzberg` tag
fixes both `lopdf` and `quick-xml`, and a `[patch.crates-io]` override
doesn't work either — the 3 quick-xml resolutions come from 3
semver-incompatible manifests we don't control. Source inspection confirmed
2 of the 3 advisories are actually reachable (`RUSTSEC-2026-0187` lopdf
stack overflow, `RUSTSEC-2026-0194` quick-xml quadratic attribute check;
`RUSTSEC-2026-0195` is NOT reachable — no `NsReader` usage anywhere in the
chain). Neither reachable issue has a consumer-side opt-out. User-directed
approach: add a compensating resource-bounding control instead of
patching/forking dependencies.

- [x] 2.1 Add `max_input_bytes` + `extraction_timeout_secs` to
      `KreuzbergConfig` (`src/config.rs`), add
      `ProcessingError::ResourceLimitExceeded` (`src/uar/file_processing/provider.rs`).
- [x] 2.2 Wire a size-cap check (before calling kreuzberg) and a
      `tokio::time::timeout`-wrapped extraction call into both
      `KreuzbergProvider::process` and `process_bytes`
      (`src/uar/file_processing/kreuzberg.rs`).
- [x] 2.3 Add regression tests: oversized input rejected before reaching
      kreuzberg; in-limit input still processes normally.

## 3. Verify

- [x] 3.1 Run `cargo test --lib` on the affected module — 5/5 pass
      (`test_process_bytes_rejects_oversized_input`,
      `test_process_bytes_accepts_input_within_limit`, plus the 3
      pre-existing kreuzberg provider tests unchanged).
- [x] 3.2 Run `cargo test --lib` for the full workspace suite and confirm no
      regressions vs. baseline (Round 1 shared checkpoint, run once after
      all 5 Round 1 changes land).
- [x] 3.3 Run `cargo clippy --lib` and confirm zero new warnings vs. current
      baseline (Round 1 shared checkpoint).
- [x] 3.4 Run `cargo audit` and confirm the disposition: `RUSTSEC-2026-0195`
      already didn't apply (not reachable, no fix needed); `RUSTSEC-2026-0187`
      and `RUSTSEC-2026-0194` will still be **listed** by `cargo audit` (the
      underlying crate versions are unchanged — this change bounds the
      blast radius, it does not clear the advisory) — confirm this is the
      expected, disclosed outcome, not a regression.

## 4. Update docs and KBD state

- [x] 4.1 Update `docs/DEPENDENCY_MANAGEMENT.md`: note that
      `RUSTSEC-2026-0187`/`RUSTSEC-2026-0194` remain open (no viable
      low-risk crate-level fix currently exists — see `findings.md`),
      mitigated via `KreuzbergConfig.max_input_bytes` /
      `extraction_timeout_secs`; track upstream kreuzberg's eventual fix
      (or a future fork-and-patch change) as the real resolution path.
      Note `RUSTSEC-2026-0195` as confirmed not reachable.
- [x] 4.2 Update `.kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json`
      (`change_status.kreuzberg-reachable-vulns` -> DONE, `changes_completed`
      incremented) once merged.
