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
- [ ] 3.3 cargo test --lib green; commit+push; security-audit.yml green on
      real dispatch; GitHub Dependabot alerts return to 0 open.
- [ ] 3.4 Archive; update phase state.
