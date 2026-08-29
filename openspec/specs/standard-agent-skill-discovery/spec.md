# standard-agent-skill-discovery Specification

## Purpose

Make skills installed in the cross-agent standard user directory automatically available and durable in UAR without requiring a separate import step.

## Requirements

### Requirement: Runtime discovers the standard user skill directory at startup
On every server start, UAR SHALL resolve `~/.agents/skills` against the runtime user's home directory and recursively discover every valid `SKILL.md` beneath that source. The source root MAY itself be a symbolic link, as is permitted by the standard directory convention.

An explicit top-level symbolic-link directory inside the standard source SHALL be treated as a declared skill alias. UAR SHALL load its root manifest, conventional `skills/` subtree, or immediate manifest-bearing collection children as applicable. UAR SHALL resolve the alias's literal target path to a canonical directory, including symbolic links in ancestor path components such as a version-selector `current` link, but MUST reject a target whose final directory entry is itself another symbolic link. UAR MUST NOT follow additional symbolic links inside the resolved target and MUST NOT crawl unrelated repository subtrees such as build or dependency directories.

#### Scenario: Standard directory contains top-level and nested skills
- **WHEN** valid `SKILL.md` manifests exist at both top-level and nested paths beneath `~/.agents/skills`
- **THEN** every distinct source path is loaded into the UAR skills library during startup

#### Scenario: Standard directory is a symbolic link
- **WHEN** `~/.agents/skills` is a symbolic link to a readable skill collection
- **THEN** UAR follows the source root and discovers skills from the linked directory

#### Scenario: A top-level skill entry is a symbolic-link alias
- **WHEN** a top-level directory entry in the standard source links to a readable skill or skill pack
- **THEN** UAR discovers manifests from the alias's declared skill surface under alias-relative identities without following further links or crawling unrelated repository subtrees

#### Scenario: An alias target uses a version-selector path component
- **WHEN** a top-level alias target path passes through an ancestor symbolic link such as `current` and ends at a regular directory
- **THEN** UAR resolves the declared target path and loads its bounded skill surface without following links found inside that target

#### Scenario: Manifest uses optional agentskills fields
- **WHEN** a standard skill omits `version` or declares supported agentskills metadata such as `allowed-tools`, license, authors, language, compatibility, metadata, or model routing
- **THEN** UAR loads the manifest, assigns `0.0.0` when version is absent, and preserves the supported metadata in the skills library

#### Scenario: Manifest exceeds the startup read bound
- **WHEN** a standard `SKILL.md` is larger than 1 MiB
- **THEN** UAR rejects that manifest after a bounded read, logs a sanitized path-level diagnostic, and continues scanning other skills

### Requirement: Standard skill identity and provenance are stable
Each standard-directory skill SHALL have a stable identity derived from its path relative to `~/.agents/skills` and SHALL record distinct `agent-skills` source provenance. Standard skills MUST NOT overwrite project-relative, API-managed, database-loaded, or built-in skills merely because their display names match.

#### Scenario: Two source paths use the same manifest name
- **WHEN** two valid standard-directory manifests have the same `name` but different relative paths
- **THEN** both skills are loaded under distinct stable identities

#### Scenario: Standard and built-in names collide
- **WHEN** a standard-directory skill and a built-in skill have the same display name
- **THEN** both records remain available with their own identities and provenance

### Requirement: Startup reconciliation durably upserts new and changed standard skills
After discovery, UAR SHALL compare standard-directory skills with durable `agent-skills` records and persist only new or changed definitions. Reconciliation SHALL preserve existing global enabled state and scoped configuration for a previously imported skill.

Metadata reconciliation SHALL complete successfully before UAR signals readiness and SHALL NOT invoke optional embedding model initialization or inference. A durable reconciliation failure SHALL prevent readiness. New and changed standard skills SHALL be available to the default keyword matcher during that startup.

#### Scenario: New standard skill is added between starts
- **WHEN** a new valid `SKILL.md` is added beneath `~/.agents/skills` and the server starts again
- **THEN** the skill is added to durable storage and is available for matching during that startup

#### Scenario: Existing standard skill changes between starts
- **WHEN** the manifest or body of an existing standard skill changes without changing its relative path and the server starts again
- **THEN** the durable definition reflects the new source content while its operator-controlled enabled and scoped configuration remain unchanged

#### Scenario: Standard skill is unchanged
- **WHEN** a standard skill is byte-semantically unchanged from its durable record
- **THEN** startup does not rewrite that record

#### Scenario: A large first-start import has an embedding matcher configured
- **WHEN** startup discovers hundreds of new or changed standard skills and an embedding matcher is configured
- **THEN** UAR persists and loads their metadata before readiness without invoking vector inference

#### Scenario: Durable reconciliation fails
- **WHEN** persistence returns an error while UAR is reconciling standard skill metadata
- **THEN** UAR does not signal readiness and reports the startup failure for operator recovery

### Requirement: Standard source failures are non-destructive
An absent, unreadable, cyclic, or invalid standard source SHALL NOT prevent UAR from starting. UAR SHALL skip invalid manifests with attributable diagnostics and MUST NOT delete or tombstone previously imported `agent-skills` records solely because a startup scan cannot reproduce them.

#### Scenario: Standard directory is absent or unreadable
- **WHEN** the runtime cannot read `~/.agents/skills`
- **THEN** startup continues, the condition is logged, and previously imported standard skills remain durable

#### Scenario: One manifest is invalid
- **WHEN** one `SKILL.md` cannot be parsed while other manifests are valid
- **THEN** UAR logs the invalid manifest path, imports the valid manifests, and continues startup

#### Scenario: Previously imported source path disappears
- **WHEN** a previously imported standard skill is absent from a later successful scan
- **THEN** UAR leaves the durable record unchanged because automatic removal is outside startup reconciliation

### Requirement: Reconciliation is observable without exposing skill contents
UAR SHALL log the resolved standard source and the counts of discovered, added, updated, unchanged, and rejected manifests. Logs MUST NOT include skill bodies or other manifest content.

#### Scenario: Startup scan completes
- **WHEN** standard-directory reconciliation finishes
- **THEN** the operational log reports its source path and reconciliation counts without reproducing skill prompt content
