## Why

Assessment H2/H3/O1: per-agent skill bindings are an in-memory inverted
allowlist lost on restart, builtin global-disable is overwritten on boot,
SkillResponse omits origin so the admin UI cannot gate pack skills, and
pack skills are deletable contrary to the disable-only requirement.

## What Changes

- Persist disable state at global, per-agent and per-conversation scope.
- Preserve persisted builtin enabled-state across restart re-registration.
- Serialize origin; UI and API make pack/builtin skills disable-only.
- Verify conversation toggles gate backend activation (O1); add
  classify-to-overlay Rust integration test.

## Capabilities
### New Capabilities
- `skill-governance`

## Impact
Skill service/registry, skills API, admin UI skills page, run manager, tests.
