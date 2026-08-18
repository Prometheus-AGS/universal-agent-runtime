---
sidebar_position: 6
title: Skills
---

# Skills

Skills add reusable instructions and tool declarations to agent execution. UAR
loads the Prometheus skill pack at startup, exposes provenance through
`GET /api/uar/skills/provenance`, and supports live matching and scoped
enablement through the skills API.

## Install the skill pack

A source checkout contains the pinned pack as a submodule. Initialize it before
building:

```bash
git submodule update --init --recursive
```

To install a released pack into the default UAR cache without keeping a skill
system checkout:

```bash
bash scripts/install-uar-skill-pack.sh
```

The installer verifies the pack manifest and writes a versioned directory under
`~/.config/uar/skills/prometheus-skill-pack/`. For an offline or reviewed local
source, pass `--source-dir`; for a system-managed cache, pass `--prefix`. See the
[full installation contract](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/skill-pack-installation.md).

## Sources and precedence

At startup UAR resolves the first valid source in this order:

1. `UAR_BUILTIN_SKILLS_DIR`.
2. A developer sibling checkout selected by `PROMETHEUS_SKILL_SYSTEM_DIR`.
3. The highest version in the installed UAR cache.
4. The embedded submodule shipped with the source or release.

Built-in skills are rediscovered on a fresh database. User-created skills stay
in persistence. Configuration reconciliation tombstones only removed
configuration-provisioned skills; it never hard-deletes them, and built-in or
API-created skills are not tombstoned by that reconciliation.

## Scope

Skill enablement can be global, agent-specific, or conversation-specific.
Conversation scope has highest precedence, followed by explicit durable agent
scope and global defaults. Scoped changes take effect live and survive restart.
