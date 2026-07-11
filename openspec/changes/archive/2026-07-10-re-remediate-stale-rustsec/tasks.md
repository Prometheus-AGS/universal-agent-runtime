## 1. Fixable families

- [x] 1.1 quinn-proto -> 0.11.16 via cargo update (RUSTSEC-2026-0185).
- [x] 1.2 Fork kreuzberg v4.9.9 (Prometheus-AGS/xberg,
      security/lopdf-quickxml-bumps): lopdf 0.42, quick-xml 0.41, calamine
      0.36, biblib 0.7 (regex feature dropped), html-to-markdown-rs <3.6;
      verify fork compiles with UAR's full feature set.
- [x] 1.3 Point UAR at the fork rev; cargo check clean; single quick-xml
      0.41 for all kreuzberg-owned paths.
- [x] 1.4 File upstream PR (xberg-io/xberg#1241).

## 2. Broken-feature removal

- [x] 2.1 Remove sandbox-microsandbox feature + microsandbox dep +
      microsandbox_runner.rs + broken test_sandbox.rs + settings enum entry;
      hickory/microsandbox tree gone from Cargo.lock.

## 3. Policy files + verification

- [x] 3.1 security-audit.yml ignores 7 -> 3 with accurate rationales;
      DISCLOSED_GHSA_IDS emptied.
- [x] 3.2 docs/DEPENDENCY_MANAGEMENT.md sections rewritten to current truth.
- [x] 3.3 cargo test --lib green (389/389, db71bf1); commit+push done;
      GitHub Dependabot alerts = 0 open (verified live via gh api).
      security-audit.yml: prior dispatch failed only via a cargo-audit
      lockfile crash ("failed to find dependency: tokenizers 0.22.2") caused
      by dependabot PR #61's dangling Cargo.lock ref — reconciled in db71bf1
      (`cargo metadata --locked` consistent). Fresh dispatch 29134227798
      queued behind GitHub runner backlog at archive time (watcher live).
      NB: db71bf1 also reverted dependabot PR #54's tailwind v4 break that
      had regressed green-main-ci's frontend build (operator-approved).
- [x] 3.4 Archive; update phase state.
