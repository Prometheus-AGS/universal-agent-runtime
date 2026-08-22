## 0. Read first

- [x] 0.1 Read `.kbd-orchestrator/phases/uar-1-0-readiness/EXECUTION-CONTRACT.md`.

## 1. Register builtins on the embedded path

- [x] 1.1 In `src/embedded.rs` (near the existing `SkillService` construction at
      :360-371), call `discover_builtin_skills()` and `register_builtins`,
      gated on `self.seed_defaults` to match `seed_builtin_agents` at :355-358.
- [x] 1.2 Ensure ordering with `initialize()` does not double-register. Prefer
      idempotent registration over ordering assumptions — assert it in 2.3.

## 2. Proof

- [x] 2.1 Test: fresh embedded database → built-ins present in the registry.
- [x] 2.2 Test: restart against the same database → built-ins still present and
      loaded from persistence.
- [x] 2.3 Test: two starts → each built-in appears exactly once.
- [x] 2.4 Test: `seed_defaults` disabled → no built-ins seeded.
- [x] 2.5 **Negative control.** Remove the registration call in a scratch build
      and show 2.1 fails. Record the command and its failing output.

## 3. Spec correction

- [x] 3.1 Amend `docs/SPECIFICATION.md:445`: correct the call sites
      (`server.rs:454` and `:517`, not `:436`), and replace "empty skill
      registry" / "capability at 0%" with the verified claim — built-ins are
      absent on a *fresh embedded database*. This is the one change in the
      phase permitted to edit SPECIFICATION.md, and only this line.

## 4. Stop conditions

- [ ] 4.1 The fix appears to require changing how skills persist → stop. The
      persistence path already works (`registry.rs:69-99`).
- [ ] 4.2 The fix appears to require scoped enable/disable → stop; that is
      `skill-scoped-governance`.
- [ ] 4.3 A pre-existing unrelated failure appears → stop and report.
