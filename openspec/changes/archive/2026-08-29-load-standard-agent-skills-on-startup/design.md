## Context

See `proposal.md` for motivation and `specs/standard-agent-skill-discovery/spec.md` for the behavior contract. UAR currently registers one project-relative filesystem provider (`fs-skills`) and one database provider, then runs a reconciliation routine hard-coded to `fs-skills`. Its filesystem parser requires a version and stops descending once it finds a parent `SKILL.md`, while the existing built-in loader already supports optional agentskills metadata and nested skill relationships. The operator's standard root is a symbolic link and currently exposes hundreds of manifests, so identity collisions and per-skill embedding startup work are concrete constraints.

## Goals / Non-Goals

**Goals:**

- Reuse one agentskills-compatible parser for built-in and standard-directory manifests.
- Discover nested manifests safely through a symlinked standard root.
- Persist only semantic changes while preserving operator enablement state.
- Keep startup available when the optional user source is broken.
- Keep embedding model cold-start and inference off the server readiness path.

**Non-Goals:**

- Watch `~/.agents/skills` continuously after startup; the requested synchronization boundary is server start.
- Automatically remove or tombstone standard skills whose source path disappears.
- Execute skill-pack hooks or automatically register pack-level MCP servers.
- Backfill vector embeddings for imported standard skills; startup makes them immediately available to the default keyword matcher and clears stale vectors when definitions change.
- Change project-relative `skills/`, API mutation, built-in immutability, frontend, provider routing, or realtime behavior.

## Decisions

### Use a dedicated standard-directory provider

Register a second read-only filesystem provider with id `agent-skills` and a root resolved through `dirs::home_dir().join(".agents/skills")`. Keeping provenance distinct makes reconciliation selective and prevents this source from entering the writable `skills/dynamic` API path. Treating the standard root as another `fs-skills` path was rejected because its records could then be tombstoned by project configuration reconciliation and name collisions would be indistinguishable.

### Derive identity from the relative directory path

Use `agents::<normalized-relative-directory>` rather than the manifest name. The relative path is stable across a symlink target move and distinguishes same-named nested skills. Name-only IDs were rejected because the observed source aggregates multiple packs and can contain duplicate manifest names. Absolute paths were rejected because they expose workstation layout and change when a linked target moves.

### Share the agentskills-compatible parser

Extract the existing built-in manifest parser into a reusable helper that accepts identity, provider, and origin, then keep source-specific policy in the callers. This preserves optional version fallback and rich metadata consistently, including both whitespace-delimited and YAML-list `allowed-tools` representations observed in the standard source. Extending the older `SkillManifest` parser independently was rejected because it would create two subtly different interpretations of the same standard.

### Canonicalize the root link and bound top-level aliases

Canonicalize the configured root once so a standard-directory symlink works, then use the existing `walkdir` dependency without following links while scanning that physical tree. The observed standard library is also a top-level symlink farm, so treat each explicit top-level directory alias as a declared skill source. Resolve its literal target path to a canonical directory, including a version-selector symlink such as `current` in an ancestor component, but reject a target whose final directory entry is itself another symlink. Reject aliases back to the standard root or an ancestor, and scan only the canonical target's conventional skill surface without following links found inside it. A single-skill alias contributes its root `SKILL.md`; a pack contributes its `skills/` subtree; a collection with neither contributes immediate child directories that contain a root manifest. Arbitrary repository subtrees such as `.build`, `.git`, tools, and dependency checkouts are outside the declared skill surface. Limit manifest content to 1 MiB and read at most one additional sentinel byte to detect overflow, then resolve the closest discovered ancestor as `parent_skill_id`. This discovers nested manifests and intentional aliases without enabling recursive directory-entry chains, cycles, build-artifact ingestion, or unbounded pre-readiness file reads. The current filesystem recursion stops beneath a directory containing its own manifest and therefore cannot satisfy nested discovery.

### Reconcile metadata before readiness without vector inference

Compare serialized `Skill` values after copying durable enabled/scoped state into the discovered candidate. Persist and register changed candidates without embeddings before readiness, clearing any stale vector attached to a changed definition. A durable persistence failure aborts startup before readiness; optional source absence and per-manifest rejection remain non-fatal. Startup does not schedule background embedding writes. This keeps the standard import deterministic and avoids cross-runtime stale-vector races while making every imported skill immediately available to the default keyword matcher. Waiting for embedding before readiness was rejected after the real standard library caused the sidecar readiness deadline to expire. Timestamp-only comparison was rejected because the durable model has no source timestamp and timestamps do not prove semantic change.

### Never infer deletion from standard-source absence

Standard reconciliation is upsert-only. Missing paths, invalid manifests, empty scans, and removed source entries retain durable records. This makes an optional user-level source non-destructive and matches the user's explicit new/changed requirement. Automatic removal can be designed later with an explicit source-health and operator-intent signal.

## Risks / Trade-offs

- [Hundreds of skills increase startup work] → Parse sequentially with bounded file reads and complete only metadata upserts before readiness; unchanged starts perform no durable writes.
- [Duplicate display names can confuse users] → Preserve every source path under a distinct stable id and expose `agent-skills` provenance; do not silently discard one definition.
- [A skill body is user-controlled prompt content] → Treat the standard directory as an explicit local trust source, parse only known metadata, never execute content during discovery, and never log bodies.
- [An oversized local manifest delays readiness or exhausts memory] → Bound each manifest read to 1 MiB and reject larger inputs with a sanitized diagnostic.
- [A top-level alias can resolve to an entire repository] → Scan only root manifests, conventional `skills/` subtrees, or immediate manifest-bearing collection children; ignore build, VCS, tools, and dependency trees.
- [The configured root and its direct library entries can be linked or unreadable] → Canonicalize the root, resolve each explicit top-level alias once, never follow links within those declared surfaces, log sanitized path-level diagnostics, preserve prior durable records, and continue startup.
- [Upsert-only behavior retains removed skills] → This is intentional and safer than destructive inference; operators can disable or delete records through existing controls.
- [Durable metadata persistence fails partway through reconciliation] → Return the error and do not signal readiness; the next start repeats semantic reconciliation while already-written metadata remains intact.
- [A new or changed standard skill has no persisted vector] → Clear stale vectors and rely on the default keyword matcher; automatic vector backfill is explicitly outside this startup change.

## Migration Plan

1. Ship the parser/provider/reconciliation change with no database schema migration.
2. On the first upgraded start, discover the standard directory and add durable `agent-skills` records.
3. On later starts, upsert only semantic changes and report reconciliation counts.
4. Rollback leaves imported rows intact; older UAR versions load them through the database provider as ordinary durable skills.
