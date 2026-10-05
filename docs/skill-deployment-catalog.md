# Skill deployment catalog (schemaVersion 1)

## Purpose and authority
GET /api/uar/skills/deployment-catalog provides the current installed registry metadata required to author reusable collaboration SkillRefs and private DeploymentBinding skillBindings. The existing /api/skills/deployment-catalog alias uses the same handler. Existing GET skill list/read/match response DTOs remain unchanged and do not expose installed locations.

This endpoint is Admin Read: ordinary authentication middleware must establish a nonanonymous UserContext, and the request must supply the exact configured security.settings_admin_key via x-uar-admin-key. It returns 401 for an anonymous principal and 403 for absent, wrong or unconfigured admin key. Disabling settings_mutation_auth_required does not bypass this read boundary. Managed hosts use their existing authenticated transport and protected admin-key source; do not send admin credentials to renderer code or logs.

## Typed response
The response contains schemaVersion: 1 and entries sorted by skillId. Each entry has:

| Field | Meaning |
| --- | --- |
| skillId, title, description | Stored registry identity and display metadata |
| enabled, tombstoned | Current global registry state, including tombstones |
| availability | available only when identity/location are complete, enabled and not tombstoned; otherwise unavailable |
| reasons | missing-skill-id, missing-version, missing-artifact-digest, missing-installed-location, disabled, tombstoned |
| skillRef | Nullable portable domain SkillRef with id, version, digest, required, config, entrypoint and requiredTools |
| privateBinding | Nullable private object containing installedLocation |

All artifact identity, entrypoint and requiredTools values come from the registered Skill. No filesystem scan, digest derivation, mutable-alias guess or fallback version occurs here. Null entrypoint and an empty requiredTools array are valid stored values. Blank required identity/location values count as missing. Missing portable metadata yields skillRef=null; missing location yields privateBinding=null. A complete portable SkillRef remains visible when only location is missing or the entry is disabled/tombstoned, while availability is unavailable.

SkillRef.required=true and SkillRef.config={} are explicit authoring defaults: installed Skill records do not carry the collaboration author's required/config choice. Callers may author those two fields before constructing the immutable definition. These defaults are not assertions about installed runtime configuration.

## Portable and private assembly
An author selects an available entry and retains the complete SkillRef in the portable AgentDefinition.skills array. Main-process deployment authoring creates a private skillBindings entry by combining that authored SkillRef with privateBinding.installedLocation. The same authored required/config must appear in both; the canonical resolver compares complete SkillRefs. installedLocation belongs only to the private binding, never the portable definition/package or renderer display. The DTO deliberately avoids flattening the location into the portable SkillRef.

Discovery is a snapshot and grants no execution, tool, workspace or binding authority. A skill's requiredTools does not override host tool admission policy. Enabled state here is the global registered enabled flag; scoped activation policy still applies through existing runtime semantics. Tombstones describe current registry records, not a historical audit log.

## Existing resolution and stale diagnostics
Ordinary agent and team member binding preflight share compiler::collaboration::bindings::resolve_skills. Team preflight now applies that resolver to every agent member, retaining source definition attribution. A required missing binding reports skill.binding-missing; a required stale/unavailable installed artifact reports skill.installed-artifact-mismatch and prevents activation. An invalid bound SkillRef that differs from the immutable declaration remains the existing collaboration_invalid error. Optional unsupported skills retain existing optional disposition. Package preflight stays portable and requires no private installation metadata.

The resolver verifies current id, version, digest, installedLocation, entrypoint, requiredTools set, enabled and tombstone state; execution-time revalidation remains authoritative. Re-read the catalog and explicitly revise/rebind after a stale diagnostic. Catalog availability does not bypass canonical schema, semantic version/digest validation or admission checks.

## Source ownership and delivery boundary

Builtin manifest metadata.tags accepts both string arrays and comma-separated
scalar strings present in shipped Boss skills. Scalar labels split on commas and
trim surrounding whitespace; existing array values and ordering remain unchanged.
This is tag syntax compatibility only: digest, version, location, tool requirements,
host policy and refresh semantics are unchanged. Source inspection identified
scalar tags in the bundled prometheus-ui-review, better-writing and better-typography
manifests (mini source abc5a9bd6157e8b6fe602c6f889d5db1d0cb777a). Actual native skill
registration and protected catalog availability still require the next packaged
operation; parser source changes alone do not establish loaded skills.
The endpoint/DTO live in src/uar/api/skills/deployment_catalog.rs, reexported through skills.rs. A crate-private registry read includes tombstones. server.rs mounts dedicated protected route state beside the unchanged skill router at both existing aliases. administration_capabilities.rs advertises skills.deployment_catalog as Admin Read; openapi.rs defines the endpoint, header security scheme and typed schemas. bindings.rs adds only the canonical team-member preflight call. No database migration, new service, dependency, cache or scheduler is introduced.

Production work is isolated from the frozen C14 packaging tree. No Cargo, compiler, build, test or review runs occur during this work-ahead. The coordinating lead owns later native authenticated catalog/refusal and stale-binding operation at the complete delivery boundary.
