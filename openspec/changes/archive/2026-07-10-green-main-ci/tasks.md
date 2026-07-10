## 1. Adopt prepared fixes

- [x] 1.1 Commit the working-tree `ci.yml` diff (review preserved rationale).
- [x] 1.2 (folded into 2.1 — quick-tests.yml is deleted, not aligned)

## 2. Legacy test-workflow consolidation (revised from "fix comprehensive-tests knowns" — see design D2)

- [x] 2.1 Delete tests-quick.yml, tests-full.yml, quick-tests.yml,
      comprehensive-tests.yml (never-green legacy harness; coverage
      superseded by CI + live-integration + bdd-chat + security-audit +
      eval-nightly). Confirm README badges reference none of them.

## 3. Others

- [x] 3.1 Delete template-cleanup.yml.
- [x] 3.2 Diagnose live-integration.yml's failing conclusion; fix (root
      cause: invalid YAML since inception — unquoted trailing-colon command).

## 4. Verify + bookkeeping

- [x] 4.1 Push; real-dispatch/watch every touched workflow to green (or
      explicitly advisory); iterate on newly surfaced failures, disclosing
      each. (3 iterations: config_integration env-prefix fix b2a6e42;
      microsandbox source-removal completion 25a4bb1 for cargo fmt;
      workflow_mirror doctest double-unwrap e563d68. All 4 push workflows
      green on e563d68: CI, Live Integration, BDD Chat, Deploy.)
- [x] 4.2 Update phase progress.json/waypoint; openspec validate --strict;
      archive.
