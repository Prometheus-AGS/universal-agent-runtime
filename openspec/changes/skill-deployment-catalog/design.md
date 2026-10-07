## Context
The current registry retains complete Skill metadata. The HTTP SkillResponse intentionally exposes a smaller general-purpose response. Filesystem and builtin loaders already populate exact artifact digest and installed location. Disabled and tombstoned records remain relevant to authoring diagnostics. Existing team execution uses resolve_skills; team binding preflight currently omits that call.

## Goals / Non-Goals
Expose stored metadata through a distinct trusted administration route, keep portable identities separate from private locations, and produce actionable preflight diagnostics through the canonical resolver. Do not alter existing responses, scan files, infer identity, add a resolver or grant authority by catalog selection.

## Decisions
1. Add feature submodule skills/deployment_catalog.rs with typed camelCase DTOs. Response schemaVersion is 1. Each entry reports skillId/title/description/enabled/tombstoned/availability/reasons, nullable domain SkillRef and nullable privateBinding installedLocation. SkillRef required=true/config={} are documented authoring defaults that callers may explicitly author; all artifact fields come from stored Skill metadata. A missing installedLocation does not erase an otherwise complete portable identity. Optional entrypoint=null and requiredTools=[] are valid stored values.
2. Require a nonanonymous authenticated principal and exact configured x-uar-admin-key; no bypass when settings mutation authentication is disabled. Existing auth middleware still handles bearer/host authentication. Missing principal returns 401; absent/wrong/unconfigured admin key returns 403. Do not expose location through SkillResponse or owner matching.
3. Add crate-private sorted registry read including tombstones. No persistence writes or independent cache. Wire new route into both existing /api/uar/skills and /api/skills router mounts using dedicated state; preserve old router and callers.
4. Call existing resolve_skills for each agent member in team binding preflight, retaining its required/optional diagnostics and source identity. Leave package preflight portable and execution-time revalidation unchanged.

## Risks / Tradeoffs
Catalog metadata is a snapshot, not activation authorization; artifacts may change afterward and canonical admission must continue revalidating. Admin discovery exposes private installation paths intentionally only within trusted main-process administration. Tombstones are current registry entries, not historical audit records. Catalog availability does not prove the target agent tool policy admits requiredTools.

## Migration Plan
Additive route and schemas; no migration. Plan committed before code. Product code remains uncommitted until lead coordinates. Rollback removes the additive route/read method and team preflight call without changing stored data.

## Validation Boundary
No Cargo/compiler/build/test/review runs during current work-ahead. Scoped Rust formatting only. Lead later runs a real authenticated admin read, unauthenticated/nonadmin refusal, complete/incomplete/disabled/tombstoned metadata, old list response preservation, exact member skill preflight and stale refusal through the normal packaged authoring flow at the completed delivery boundary.
