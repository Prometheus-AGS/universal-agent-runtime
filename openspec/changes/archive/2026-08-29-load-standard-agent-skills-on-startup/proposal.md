## Why

UAR currently reconciles only the project-relative `skills/` directory, so skills installed in the cross-agent standard `~/.agents/skills` directory are invisible to the runtime skills library. Operators need every server start to discover that standard source and durably reflect new or changed skill definitions without manual import.

## What Changes

- Resolve the current user's standard `~/.agents/skills` directory at startup and recursively discover its `SKILL.md` manifests, including nested skills.
- Give discovered standard skills stable source-relative identities and distinct provenance so they can coexist with project, API, database, and built-in sources.
- Upsert new and changed standard skills into the durable skills library on every server start while preserving operator-controlled enabled and scoped configuration.
- Complete metadata reconciliation before readiness without invoking optional vector inference.
- Fail safely when the standard directory is absent, unreadable, or resolves to no valid manifests; startup continues and previously imported standard skills are not removed.
- Log source path and reconciliation counts without logging skill bodies or other file contents.

## Capabilities

### New Capabilities

- `standard-agent-skill-discovery`: Startup discovery and durable reconciliation of the standard `~/.agents/skills` directory.

### Modified Capabilities

None.

## Impact

- Backend skill discovery, filesystem-provider parsing, startup provider registration, and durable reconciliation under `src/uar/runtime/skills/**` and `src/server.rs`.
- No frontend, provider-routing, public API, or realtime-event contract changes.
- No new dependency is required; existing filesystem walking, persistence, and agentskills-compatible parsing are reused.
- KBD's current waypoint remains stale and is not rewritten by this runtime capability; the OpenSpec change is the authoritative plan for this addition.
