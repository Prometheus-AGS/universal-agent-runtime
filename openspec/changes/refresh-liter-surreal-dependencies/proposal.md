## Why

UAR's vendored Liter and Surreal Memory inputs, deployable SurrealDB images, and JavaScript lockfiles no longer represent one verified dependency baseline. Refreshing them now closes the current Dependabot findings, makes SurrealDB 3.2.4 exact across build and deployment surfaces, and preserves reproducible provenance for local and offline releases.

## What Changes

- Advance the Liter and Surreal Memory leaf repositories and the nested `prometheus-skill-system` pointers only to reviewed, remotely reachable commits.
- Pin SurrealDB SDK, server, Compose, Kustomize, Helm, and OpenTofu surfaces to 3.2.4, with container deployments using the published immutable OCI digest.
- Refresh UAR's Liter vendor pointer, deterministic provider catalog, curated Surreal Memory snapshot, and vendor provenance.
- Adopt the reviewed Surreal Memory durable-operation migrations, model-safe token-window planning, idempotent indexed writes, and atomic storage mutations carried by the accepted upstream commit.
- Patch the affected `nanoid`, `dompurify`, and `js-yaml` dependency graphs and add `/website` to Dependabot coverage.
- Bound the unpatched `image-size` documentation-build exposure with a local repository-input gate, an owned review date, and explicit reopen conditions.
- Rehearse the documented SurrealDB v2-to-v3 export/import path with disposable data and rebuild and redeploy the affected macOS services from verified artifacts.
- Preserve UAR public HTTP APIs, UAR entity schemas, provider compatibility, and realtime-state behavior; the internal Surreal Memory schema advances through migrations 20 and 21, and no runtime UX change is expected.

## Capabilities

### New Capabilities

- `surreal-memory-runtime`: Define the adopted durable-operation schema, embedding token-window, idempotent indexed-write, and atomic mutation guarantees of the refreshed Surreal Memory runtime.

### Modified Capabilities

- `dependency-security-posture`: Require lockfile-specific advisory closure evidence and an enforceable, time-bounded exception process for an unpatched documentation build dependency.
- `offline-reproducible-build`: Require immutable, remotely reachable vendor provenance and deterministic provider-catalog generation for refreshed runtime inputs.
- `helm-chart`: Require the default SurrealDB image to use the exact supported version and OCI digest.
- `gke-deployment`: Require rendered SurrealDB Kubernetes workloads to use the exact supported version and OCI digest.
- `native-service-deployment`: Require the local SurrealDB service and dependent native services to be version-verified and restarted in dependency order.

## Impact

- Affects Rust and JavaScript dependency manifests and lockfiles, vendor gitlinks/snapshots, the internal Surreal Memory schema and storage/embedding behavior, generated provider catalogs, deployment manifests, local security auditing, and dependency documentation.
- Requires coordinated commits in `surreal-memory-server`, `prometheus-skill-system`, and UAR, pushed in leaf-to-parent reachability order.
- Requires local-only Rust, frontend, Docusaurus, packaging, infrastructure-rendering, migration-rehearsal, and macOS service verification; GitHub Actions remain deployment-only.
- KBD workflow state must record this Spec, Plan, Execute, and Reflect sequence. Runtime UX, UAR public APIs, provider compatibility, and UAR realtime entity state remain unchanged; the adopted Surreal Memory runtime delta is specified separately.
