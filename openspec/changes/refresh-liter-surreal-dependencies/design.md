## Context

See `proposal.md` for motivation. UAR has two distinct dependency topologies that must move together without being conflated: runtime code consumes the top-level Liter gitlink and a curated regular-file Surreal Memory snapshot, while `crates/prometheus-skill-system` carries nested tool gitlinks used by its own builds. JavaScript security state is likewise split across root pnpm, frontend pnpm, and website npm lockfiles. `versions.toml` remains the architectural authority for exact pins.

The release must be proven locally. GitHub Actions remain restricted to deployment execution and deployment-specific validation.

## Goals / Non-Goals

**Goals:**

- Establish one exact, traceable Liter/SurrealDB baseline across leaf repositories, parent gitlinks, UAR runtime inputs, package locks, deployment manifests, and installed macOS services.
- Adopt and specify the accepted Surreal Memory internal schema, embedding-planning, and atomic-storage behavior rather than treating it as a provenance-only refresh.
- Make patched advisory closure and the sole unpatched advisory exception mechanically auditable.
- Preserve offline source packaging and deterministic generated artifacts.

**Non-Goals:**

- Change UAR public HTTP APIs, UAR entity schemas, provider routing semantics, UAR realtime state, or runtime UI.
- Migrate production SurrealDB state or deploy Kubernetes/OpenTofu infrastructure.
- Replace the archived `image-size` dependency outside the current Docusaurus compatibility range.

## Decisions

### Push leaf commits before recording parent pointers

Surreal Memory is updated and pushed first, followed by the nested `prometheus-skill-system` pointer commit, then UAR. Each recorded gitlink must be reachable from its authoritative remote before the next repository records it. This avoids a parent commit depending on a workstation-only object. The rejected alternative is a single root-only pointer update, which would leave runtime and nested tools inconsistent and could record unreachable commits.

### Preserve the curated Surreal Memory snapshot boundary

UAR will copy only the upstream manifest and implementation changes required for embeddings, migrations, and storage into its curated snapshot. Its standalone manifest adaptations remain intact, including local workspace substitutions. Replacing the snapshot with the upstream workspace manifest was rejected because it would introduce unrelated workspace coupling and break offline packaging.

### Adopt the reviewed Surreal Memory runtime delta explicitly

The accepted source adds migrations 20 and 21 for a durable operation ledger
and supervised executor journal, deterministic model-capacity token windows,
stable-key indexed writes, atomic delete/history mutations, and bounded retries
for authoritative SurrealDB transaction conflicts. These are runtime and
internal-schema changes, not incidental dependency metadata. They are adopted
as the `surreal-memory-runtime` capability and verified by the accepted leaf's
unit, contract, executor-recovery, and integration suites. The rejected
alternative was to retain the code while continuing to claim that entity-schema
and runtime behavior were unchanged.

### Treat generated catalog output as a pinned-input artifact

The existing generator will run against the updated Liter schema twice; both output digests must match. Network-derived models data is not refreshed unless its own pin changes. This separates a Liter schema update from unrelated catalog-source movement.

### Preserve Liter's owned stream through UAR normalization

Liter 1.18.2 returns an owned `'static` response stream. UAR will return its
normalized wrapper immediately and transform chunks as they arrive. The
stream-start timeout therefore governs upstream response establishment only;
it does not cap the duration of a valid model completion after a tool result.
Eagerly collecting the stream was rejected because it converts streaming into
full-response buffering, delays every semantic event, and makes ordinary
completion latency look like a stream-start failure.

### Pin containers by tag and OCI digest

Compose, Kustomize, Helm, and OpenTofu references will use `surrealdb/surrealdb:v3.2.4@sha256:51baed8709f57f67dcf04b30e3177db846803fa9342dae2be58c6fa5f8d59843`. The tag communicates the supported version and the digest makes the artifact immutable. Tag-only and floating-major alternatives were rejected because they cannot prove identical deployment content.

### Bound the unpatched image-size exposure instead of substituting an incompatible package

The advisory is limited to Docusaurus build-time inspection of tracked repository assets. A local gate will fail if ICNS, JXL, HEIF, HEIC, or AVIF inputs enter the documentation tree. The GitHub dismissal will be owned by repository security maintainers, reviewed by 2026-11-24, and reopened if untrusted ingestion begins or a compatible fixed release appears. A speculative dependency replacement was rejected because upstream has no patched compatible release and the current trusted-input path can be enforced directly.

### Verify service deployment as an ordered transaction

Rollback copies and hashes are captured before installation. Services are stopped and restarted SurrealDB → Surreal Memory → UAR, with each layer required to pass health and persistence evidence before the next starts. A failed gate restores captured binaries and prior service state. Rewriting the SurrealDB LaunchAgent is excluded because it already targets the verified Homebrew 3.2.4 binary.

## Risks / Trade-offs

- [SurrealDB v2 data is not directly compatible with v3] → Rehearse the official v2 export-with-v3-format and v3 import sequence against disposable representative data; do not touch production state.
- [A leaf remote advances during execution] → Re-resolve immediately before use and stop if new commits expand beyond dependency-update scope.
- [Curated snapshot drift omits an upstream change] → Diff the pinned upstream subtree against the curated copy and enumerate intentional standalone-manifest adaptations.
- [Dependabot graph refresh is asynchronous] → Poll the exact patched alert IDs after push and report unrelated new alerts rather than silently expanding scope.
- [Local deployment can leave mixed binaries] → Require installed/source hash equality and restore the captured set when any bootstrap or persistence gate fails.

## Migration Plan

1. Push the exact Surreal Memory leaf commit, then the `prometheus-skill-system` pointer commit.
2. Update UAR runtime gitlinks, curated snapshot, catalog, locks, deployment pins, controls, and documentation; complete local verification and commit the exact release candidate.
3. Rehearse, but do not execute against production, the documented SurrealDB v2-to-v3 export/import sequence.
4. Capture installed binaries and service state, install signed/hash-verified artifacts, then restart and verify services in dependency order.
5. Push UAR, wait for the dependency graph, close the approved advisories, archive the OpenSpec change, and push the resulting canonical-spec and evidence commit.
6. On local deployment failure, restore captured binaries and bootstrap only the services that were previously loaded.
