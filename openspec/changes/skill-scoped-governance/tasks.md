## 0. Read first

- [x] 0.1 Read `.kbd-orchestrator/phases/uar-1-0-readiness/EXECUTION-CONTRACT.md`.
- [x] 0.2 Read `fix-skills-scope-semantics`. This change supersedes it. **Do not
      execute both.** Its task 1.3 is deliberately dropped — see the proposal.

## 1. Scoped configuration model

- [x] 1.1 Add a durable scoped-config record keyed `(skill_id, scope)` where
      scope is Global, Agent(id), or Conversation(id). Keep the existing
      `Skill::enabled` as the global value or migrate it — state which, and why,
      in the change's verification record.
- [x] 1.2 Implement most-specific-wins resolution: conversation > agent > global.
- [x] 1.3 Persist through the storage providers so state survives restart.

## 2. Do not clobber stored config at startup

- [x] 2.1 Built-in re-registration must not overwrite stored scoped
      configuration. Built-ins already persist (`registry.rs:69-99`), so this is
      a merge-on-register rule, not a restore-after-register workaround.

## 3. Live effect

- [x] 3.1 A scoped change affects the next matching pass with no restart.
- [x] 3.2 An in-flight run keeps its start-time binding, per `skill-hot-reload`.

## 4. Origin exposure

- [x] 4.1 Serialize `origin` in the skills API response.
- [x] 4.2 Confirm `delete_skill_permanent` already refuses built-ins
      (`service.rs:390-401`) and add a test if none exists. **Do not
      reimplement** the guard.

## 5. Proof

- [x] 5.1 Scope-matrix tests: conversation over agent over global, both directions.
- [x] 5.2 Restart tests: global and per-agent disables survive.
- [x] 5.3 Live-effect test: disable, then next request does not activate.
- [x] 5.4 In-flight test: disable mid-run, run completes with the original binding.
- [x] 5.5 Delete tests: built-in refused, user skill removed.
- [x] 5.6 **Negative control** for 5.2: show the restart test fails when the
      merge rule from task 2.1 is removed. Record command and failing output.

## 6. Stop conditions

- [ ] 6.1 The work appears to require deleting or hard-disabling a built-in at
      the storage layer → stop.
- [ ] 6.2 The work appears to require config-file reconciliation → stop; that is
      `skill-config-reconciliation`.
- [ ] 6.3 Marking `fix-skills-scope-semantics` superseded appears necessary →
      stop and report. That is an operator action on another author's change.
- [ ] 6.4 A pre-existing unrelated failure appears → stop and report.
