# Negative controls — `fix-skills-scope-semantics`

Profile scope: `server-full` only. Both controls temporarily inverted one
candidate branch, ran Tier 0, observed the focused assertion fail, and restored
the exact candidate hash.

## Session-policy filter inversion

Temporary inversion: remove the two run-manager lines that retain only skill IDs
selected by the effective persisted conversation policy.

Command:

```text
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --test skill_scoped_governance persisted_session_skill_selection_gates_overlay_and_activation -- --exact --test-threads=1 --nocapture
```

Observed failure:

```text
thread 'persisted_session_skill_selection_gates_overlay_and_activation' panicked at tests/skill_scoped_governance.rs:581:5:
assertion failed: excluded_history.iter().all(|event| !matches!(&event.event,
    NormalizedEvent::SkillActivated { skill_id, .. }
        if skill_id == "session-selected-skill"))
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out
negative-control-exit=101
```

Restored source:

```text
42b78af4642f6ff83f68aba4cc7c926fdd2354d47a454455707c411ac3c9c399  src/uar/runtime/manager.rs
```

The restored positive test passed 1/0.

## Built-in edit guard inversion

Temporary inversion: remove the `SkillOrigin::Builtin` rejection from
`SkillService::update_skill`.

Command:

```text
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --test skills_api_integration_test update_builtin_is_refused_while_toggle_remains_available -- --exact --test-threads=1 --nocapture
```

Observed failure:

```text
assertion failed: expected status 409 but received 200
response title: "Mutated pack skill"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out
negative-control exit: 101
```

Restored source:

```text
700521d983edaa2affbb23a3f4012bcf711ac72a6a1fe9d1cb0b21e403baf44b  src/uar/runtime/skills/service.rs
```

The restored service and HTTP tests each passed 1/0.
