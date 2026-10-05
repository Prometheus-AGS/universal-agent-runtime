## Why
C15 reusable team authoring cannot construct exact SkillRefs and private skill bindings from the supported HTTP skill list: SkillResponse omits stored artifact digest, installed location, entrypoint and required tools. Team binding preflight also skips member skill resolution, although execution already requires exact installed identity.

## What Changes
- Add authenticated admin-only GET /api/uar/skills/deployment-catalog with schemaVersion 1, typed portable SkillRef and separate private installedLocation binding metadata.
- Report enabled, tombstoned and availability with explicit missing-metadata reasons using the current registry, without filesystem scans or fabricated identities.
- Preserve existing SkillResponse, list, owner matching, aliases and authentication middleware.
- Advertise Admin Read endpoint and OpenAPI schema.
- Reuse canonical resolve_skills for team-member binding preflight so required unavailable or stale skills prevent activation before execution.

## Capabilities
### New Capabilities
- skill-deployment-catalog: trusted installed metadata for reusable collaboration authoring.
### Modified Capabilities
- collaboration-package-catalog: team binding preflight validates exact member SkillRefs through the existing resolver.

## Impact
API/registry/binding preflight only; no renderer, dependency, persistence migration, service or scheduler change. Private installed paths require authenticated owner and configured admin key; discovery grants no execution/tool authority. Native operations and completed-delivery verification remain with the coordinating lead. No provider, realtime, usage or pricing behavior changes.
