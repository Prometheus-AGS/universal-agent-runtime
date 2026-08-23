---
sidebar_position: 1
title: Manage Skills
description: Understand skill provenance, scoped enablement, live activation, restart behavior, and reconciliation safety.
source_records:
  - openspec/specs/skill-builtin-availability/spec.md
  - openspec/specs/skill-governance/spec.md
  - openspec/specs/skill-config-reconciliation/spec.md
  - docs/skill-pack-installation.md
  - docs/skill-authoring.md
current_authority: /docs/skills/overview
---

# Manage skills

## Boundary statement

**A skill is reusable execution context with provenance and policy; its presence
does not mean it activated.** UAR matches eligible skills for a request, binds
the result at run start, injects the prompt overlay, and reports activation in
the response event path.

## Provenance and operator rights

| Provenance | Runtime marker | How it arrives | Edit/delete behavior | Reconciliation behavior |
|---|---|---|---|---|
| Built-in | `origin: builtin`, built-in provider | Seeded from the release or installed pack | May be enabled or disabled; edit and delete are refused | Never tombstoned |
| Configuration-provisioned | `provider_id: fs-skills` | Loaded from configured skill files at startup | Configuration remains its definition authority; scoped state is durable | Removed files create restorable tombstones |
| API-created | `origin: user`, `provider_id: api` | Created through the packaged UI/API or embedded facade | Editable and deletable by the supported control surface | Never tombstoned by filesystem reconciliation |

Reconciliation never hard-deletes. It tombstones only a configuration-managed
record whose source was deliberately removed, retaining the record and scoped
configuration for restore. An empty configuration source is treated as a
possible broken mount and is refused when configuration-managed records exist.
Built-in and API-created skills are never tombstoned by reconciliation.

## Scope and precedence

Every skill can be enabled or disabled at global, agent, and conversation
scope. The most specific explicit decision wins:

1. conversation;
2. agent;
3. global.

A scoped change affects matching on the next request without requiring a
restart. A run already in flight keeps the binding established at run start.
Scoped configuration is durable and survives restart and built-in
re-registration.

## Packaged UI workflow

1. Open **Admin → Skills** at `/admin/skills`.
2. Inspect the built-in badge and origin before choosing an operation.
3. Toggle a skill to change global enablement. Built-ins stay present when
   disabled.
4. Create an API-created skill by giving it a name, description, activation
   triggers, prompt overlay, preferred tools, and enabled state. Import can
   inspect a skill directory before creation.
5. Bind an allowlist to an agent from its agent configuration. A conversation
   selection can further narrow or override eligibility.
6. Start a new chat request containing a trigger and inspect the visible skill
   activation. The event includes the skill identity and selection method.

The UI disables edit and delete for built-in skills. Disabling is the supported
operator action for those records.

## API workflow

The canonical server surface is `/api/uar/skills`:

| Action | Request |
|---|---|
| List with origin | `GET /api/uar/skills` |
| Create API skill | `POST /api/uar/skills` |
| Read or update | `GET` or `PUT /api/uar/skills/{id}` |
| Delete user skill | `DELETE /api/uar/skills/{id}` |
| Set scoped enabled state | `POST /api/uar/skills/{id}/toggle` with global, agent, or conversation scope |
| Preview matching | `GET /api/uar/skills/match?q=...` with optional agent and conversation IDs |
| Replace agent allowlist | `PUT /api/uar/agents/{agent_id}/skills` |
| Refresh skill sources | `POST /api/uar/skills/refresh` |
| Inspect pack provenance | `GET /api/uar/skills/provenance` |

The provenance response reports both the pack's declared count and the runtime's
loaded count. A mismatch is drift, not a second definition of how many skills
should exist.

## Embedded host workflow

`EmbeddedRuntime::skills()` returns the supported `SkillsApi` facade. It exposes
list, enabled-list, get, install, toggle, and query operations without coupling
the host to the internal registry. A fresh embedded database receives built-ins
when default seeding is enabled; those built-ins are durable and restart-safe.

Generated-skill registration is off by default and requires an explicit host
opt-in. An embedded host with no embedding backend still has keyword matching;
it must not claim semantic matching from that fallback.

## Activation and observable state

A successful list or match response is not activation evidence. For a
streaming chat request, observe the skill activation extension/event and the
model response influenced by its overlay. For a non-streaming integration,
retain equivalent runtime evidence at the host boundary.

The retained 2026-08-22 `server-full` record, source SHA
`d41bf7c3a447869896664d44ac0563e1b4a1d9f3`, observed one API-created skill
selected by keyword and influencing a genuine `openai/gpt-5.4-mini` request
through UAR. The packaged UI displayed the same activation for one fresh
request. That result is limited to the named skill, model, profile, checkout,
and date.

## Restart, tombstone, and restore behavior

- Built-ins are seeded once, remain unique, and survive restart in configured
  persistence.
- Scoped disables survive restart and are not overwritten by re-registration.
- A changed configuration skill is updated on restart.
- A removed configuration skill is tombstoned and excluded from normal listing
  and matching.
- Restoring its file restores the skill and its prior scoped configuration.
- Reconciliation logs every tombstone with its skill ID and reason.

## Installation and authoring

- [Install or refresh the pinned skill pack](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/skill-pack-installation.md).
- [Author manifest, WASM, or native skills](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/skill-authoring.md).

Installation or authoring success is not activation evidence; exercise the
skill through the target runtime profile.

## Profile limits

- `server-full` includes the packaged Skills UI and server APIs.
- `minimal` includes the server skill services and API but not the admin UI as a
  profile claim.
- `embedded-mobile` uses `SkillsApi` in process and has no skill HTTP/UI route.

The built-in catalog is intended across profiles when seeding is enabled, but
readiness and activation evidence remain profile-specific. Next, [use
knowledge](/docs/knowledge/overview).
