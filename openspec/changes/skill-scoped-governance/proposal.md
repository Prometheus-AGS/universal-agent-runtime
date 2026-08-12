## Why

Operator requirement, 2026-08-12: all skills SHALL be configurable — enabled or
disabled at global, agent, and conversation scope — with changes taking effect in
real time and surviving restart. Built-in skills are configurable but never
deletable; user-added skills are both.

Today `Skill` carries a single global `enabled: bool`
(`src/uar/domain/skills.rs:60-62`). There is no per-agent or per-conversation
scope in the domain model. Per-agent bindings exist only as an in-memory
inverted allowlist that is lost on restart.

### This change supersedes `fix-skills-scope-semantics`

That change (0/5 tasks, untouched since `3a54b965`) specifies substantially the
same requirement set and declares the capability `skill-governance`. **This
change adopts that capability name and absorbs its scope rather than running in
parallel** — two changes writing skill persistence with no precedence rule is the
top cross-change failure mode in `.kbd-orchestrator/HARNESS-HANDOFF.md`.

One of its tasks is deliberately dropped. Task 1.3 — *"merge persisted
enabled-state over builtin re-registration at startup"* — was a workaround for
built-ins being registered in-memory only. Built-ins now persist
(`registry.rs:69-99`, commit `fdd69a2f`), so the correct behaviour is simpler:
**startup re-registration must not overwrite stored configuration**, rather than
restoring it afterwards.

> `fix-skills-scope-semantics` should be marked superseded when this change
> archives. That is an operator action on someone else's authored change, not
> something this change performs.

### What is already done and must not be redone

`add-skill-kind-and-origin` (8/11) defines `SkillOrigin { Builtin, User }`
(`skills.rs:19-26`) and `SkillService::delete_skill_permanent` already rejects
built-ins with `system_skill_immutable` (`service.rs:390-401`). **Built-in
non-deletability is implemented.** This change consumes it; it does not rebuild
it.

## What Changes

- Add a durable scoped-configuration record: `(skill_id, scope, enabled)` where
  scope is global, per-agent, or per-conversation. Resolution is
  most-specific-wins: conversation over agent over global.
- Persist scoped state so it survives restart, for built-in and user skills alike.
- Startup re-registration of built-ins SHALL NOT overwrite stored configuration.
- Changes take effect for subsequent matching without a restart, consistent with
  the existing `skill-hot-reload` capability. Runs already in flight keep the
  binding they started with, per that capability's stated requirement.
- Serialize `origin` in the skills API response so clients can render
  disable-only affordances for built-ins.

## Capabilities

### New Capabilities
- `skill-governance`

## Impact

`src/uar/domain/skills.rs`, `src/uar/runtime/skills/service.rs`,
`src/uar/runtime/skills/registry.rs`, skill storage providers, `src/uar/api/skills.rs`.

## Non-goals

- Deleting built-in skills. Already prohibited and out of scope.
- Config-file reconciliation — `skill-config-reconciliation` owns it.
- Admin UI work. The API must expose `origin`; rendering is a frontend concern
  tracked by `builtin-skills-ui-affordance`.
