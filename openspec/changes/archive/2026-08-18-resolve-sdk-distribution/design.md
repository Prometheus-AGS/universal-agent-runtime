## Context

ADR-0007 already records the operator decision to ship the Rust, Python, and
TypeScript SDKs at 1.0.0 with a shared supported surface. ADR-0017 subsequently
licensed the entire repository under MIT. The SDK implementations, tests,
examples, and documentation now exist, so withdrawing them would contradict
accepted architecture and discard completed work.

This change reconciles the stale July assessment with the repository as it
exists today. The remaining defects are bounded: TypeScript still names
`Developer` as author, Rust omits authors and declares its optional runtime
dependency by path without a registry version, customer docs use package names
that differ from the manifests, and `.github/workflows/ci.yml` is a legacy
routine-development workflow despite the current deployment-only Actions
policy.

## Goals / Non-Goals

**Goals:**

- Ship all three SDKs as MIT-licensed 1.0.0 packages.
- Make package metadata and customer install commands match the actual packages.
- Preserve the implemented streaming, agent/run, knowledge, typed-model, test,
  example, and generated-documentation surfaces.
- Observe all routine verification locally and retire the legacy CI workflow.

**Non-Goals:**

- Publishing any package or changing public SDK behavior.
- Adding manifest dependencies or expanding the SDK surface.
- Refactoring existing SDK implementations or changing deployment workflows.

## Decisions

### Ship all three SDKs under MIT

Preserve ADR-0007 and ADR-0017. The canonical KBD decision
`sdk-distribution-1-0-mit` records this scope for the active phase. Withdrawal
was rejected because the committed implementations already satisfy the planned
1.0 surface and are referenced by the product documentation.

### Reconcile metadata without changing package identities

Use `Prometheus AGS` as authorship metadata in all manifests. Keep the existing
package names and correct customer docs to match them. Add the root crate's
exact `1.0.0` version alongside the Rust SDK's development path so Cargo can
resolve the dependency when packaging from a registry. Until that sibling
release exists, verify package metadata and contents locally and retain the
observed package-preparation failure as a publication-order control.

The standalone SDK lockfile was stale relative to the current root path
dependency: its runtime package entry predated the runtime's current dependency
set. Cargo cannot honor `--locked` after the honest registry-version correction
without reconciling that entry and its transitive graph. Regenerate it through
Cargo and verify the resulting exact graph with the SDK's locked test command;
do not describe the resulting lock entries as newly chosen SDK dependencies.

The runtime is not itself registry-publishable today. Its normal dependency
graph contains four path-only requirements without registry versions:
`sycophancy-core`, `surreal-memory`, `prometheus_parking_lot`, and the locally
patched `liter-llm`. Therefore the complete release order is:

1. publish or replace the three UAR-owned internal crates and reconcile the
   local `liter-llm` patch with a registry release;
2. add verified registry versions alongside those four runtime paths and prove
   `cargo package` for `universal-agent-runtime`;
3. publish `universal-agent-runtime` 1.0.0;
4. prove and publish `universal-agent-runtime-sdk` 1.0.0.

This change selects all three SDKs for release and records that gate. It does
not claim the Rust SDK is registry-publishable before the later supply-chain
and release changes satisfy the chain. Removing embedded support or splitting
it into another crate remains a future architecture alternative, not an
unreviewed shortcut in this reconciliation.

### Keep routine SDK verification local

Retire `.github/workflows/ci.yml`; keeping its non-SDK checks would still violate
the current GitHub Actions policy. Run the language-specific test, build,
example, and docs commands locally and retain their output as change evidence.
Deployment workflows remain untouched.

## Risks / Trade-offs

- **[Risk] The Rust SDK package references a runtime crate not yet published at
  1.0.0, and that runtime has four path-only prerequisites.** → The exact SDK
  requirement remains honest; the full four-step remediation/publication chain
  above is a blocking input to the later supply-chain and release changes.
- **[Risk] Retiring legacy CI removes hosted routine-development feedback.** →
  The repository explicitly requires these checks locally; the later
  release-candidate change binds that evidence to the immutable candidate.
- **[Risk] Existing SDK tests may reveal surface drift.** → Stop and correct
  only observed SDK defects; do not widen the public contract speculatively.
