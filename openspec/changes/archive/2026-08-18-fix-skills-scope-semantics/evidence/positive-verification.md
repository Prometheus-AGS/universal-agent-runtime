# Positive verification — `fix-skills-scope-semantics`

Profile scope: `server-full` only.

## Persisted session selection

Command:

```text
cargo test --quiet --locked -p universal-agent-runtime --no-default-features --features server-full --test skill_scoped_governance persisted_session_skill_selection_gates_overlay_and_activation -- --exact --test-threads=1
```

Observed tail:

```text
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.38s
```

## Built-in service edit guard and disable path

Command:

```text
cargo test --quiet --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::runtime::skills::service::tests::builtin_update_is_refused_while_disable_remains_available -- --exact --test-threads=1
```

Observed tail:

```text
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 601 filtered out; finished in 0.00s
```

## Built-in HTTP edit guard and toggle continuity

Command:

```text
cargo test --quiet --locked -p universal-agent-runtime --no-default-features --features server-full --test skills_api_integration_test update_builtin_is_refused_while_toggle_remains_available -- --exact --test-threads=1
```

Observed tail:

```text
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 26 filtered out; finished in 0.00s
```

## Tier 0 and structural checks

Commands:

```text
cargo fmt --all -- --check
cargo check --locked -p universal-agent-runtime --no-default-features --features server-full
cargo clippy --locked -p universal-agent-runtime --no-default-features --features server-full --lib --no-deps
openspec validate fix-skills-scope-semantics --strict
git diff --check -- src/uar/api/skills.rs src/uar/runtime/skills/service.rs tests/skill_scoped_governance.rs tests/skills_api_integration_test.rs openspec/changes/fix-skills-scope-semantics
```

Observed results:

```text
cargo fmt: exit 0, no output
cargo check: Finished `dev` profile; exit 0 with 3 known warnings
cargo clippy: Finished `dev` profile; exit 0; 572 warnings
Change 'fix-skills-scope-semantics' is valid
git diff --check: exit 0, no output
```

Candidate SHA-256 values:

```text
40ed210705f29bbc960d3a8be7ff9287966d9b50b14653ad9ed43d657c5e6bde  src/uar/api/skills.rs
42b78af4642f6ff83f68aba4cc7c926fdd2354d47a454455707c411ac3c9c399  src/uar/runtime/manager.rs
700521d983edaa2affbb23a3f4012bcf711ac72a6a1fe9d1cb0b21e403baf44b  src/uar/runtime/skills/service.rs
6fa8abd61c0497f9abca309fe4aaa35b445a7962bb955ea11483175cbb6000eb  tests/skill_scoped_governance.rs
2dd3d31cc3057ad49f3bfb23078dc4f869fdab73f0d7174bf3284af18c2017a1  tests/skills_api_integration_test.rs
```

## Delivered foundation consumed by this change

The process-reopen persistence, scope precedence, live-effect stability, origin
serialization, built-in delete refusal, and user deletion results are retained
verbatim in
`openspec/changes/archive/2026-08-15-skill-scoped-governance/verification.md`
and its evidence directory. This change does not duplicate or weaken those tests.
