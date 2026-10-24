# Dependency/publication Rust source review

Reviewed 2026-10-24 against `git diff f870a9c3` in the shared working tree. Phase: `uar-post-lifecycle-verification-readiness`, Execute. Scope: root and SDK manifests/locks, versions pins, eight Parking Lot manifests and provenance README, MCP client/header substitutions, ingestion import substitutions, and two tool manifests. Applied `prometheus-rust-workspace`, `rust-best-practices`, and `review-verification-protocol`.

## HIGH — SDK server/full resolves unpatched lifecycle dependency

[sdks/rust/Cargo.lock:7307] The refreshed independent SDK lock selects the registry SDK rather than UAR's lifecycle patch:

```toml
name = "surrealdb"
version = "3.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
```

The consuming manifest ends with `sdks/rust/Cargo.toml:51`:

```toml
[workspace]
```

It contains no `[patch.crates-io]`. The root's six lifecycle patches therefore do not apply when the SDK is the workspace root. This matters for the existing `server` feature, which enables `universal-agent-runtime/minimal`, and `full`, which enables `server`. Root `minimal` enables `surreal-backend`; `src/uar/persistence/providers/mod.rs` includes the provider under that feature.

The provider's full `subscribe_transport_state` and `close_scope` functions were inspected. At `src/uar/persistence/providers/surreal.rs:479-480` they require:

```rust
    ) -> Result<tokio::sync::watch::Receiver<surrealdb::TransportState>> {
        self.db.subscribe_transport_state().map_err(anyhow::Error::from)
```

At line 508, `close_scope` calls `self.db.shutdown().await`. Those symbols exist in `vendor/patches/surrealdb-lifecycle/surrealdb-3.3.0/src/lib.rs`; source searches of the downloaded registry `surrealdb-3.3.0/src` found none. This is a source-level build incompatibility for the SDK's server/full configurations, not a style concern. Both manifests use edition 2024. The prior SDK lock selected registry 3.2.4, so the separate-workspace patch omission predates this patch; refreshing to 3.3.0 does not repair it.

Minimal fix: add the same six `[patch.crates-io]` entries to `sdks/rust/Cargo.toml`, with paths relative to that manifest (`../../vendor/patches/surrealdb-lifecycle/...`), and refresh its lock through the operator-authorized dependency workflow. This finding does not block the root image on its own: the root workspace already selects patched 3.3.0 packages.

## Minor documentation observation

[vendor/git/README.md:33] `UAR enables only tokio-runtime` describes the removed facade dependency. Root `Cargo.toml` now consumes only `prometheus-parking-lot-core`. Replace that sentence with the runtime-free-core selection and retain the explanation that optional adapters remain exported source. No production correctness impact.

## Remaining source assessment

The actual root manifest hunk changes reqwest from `0.12` to exact `=0.13.5`, replaces `rustls-tls-native-roots` with `rustls`, and adds `query`, `form`, and `http2`. Root, SDK, and tool manifests select exact 0.13.5. The lockfile binds root and rmcp 3.7.0 to that same reqwest package. Searches found no remaining `reqwest_mcp` or `prometheus_parking_lot::` references in owned source/tests/tools/SDK. Remaining reqwest 0.12 lock entries are transitive dependencies, not owned client type boundaries.

The standalone Parking Lot manifest diffs preserve inherited package fields, edition 2024, Rust 1.95, dependency requirements/default-feature settings/added features, target predicates, and lint values. Sibling paths are adjusted relative to each manifest. Root lock retains only the runtime-free Parking Lot core, removing the facade's optional backend resolution. The changed ingestion code only substitutes re-export paths; host admission and execution ownership are unchanged. MCP changed functions retain transport-owned-header rejection, no-proxy behavior, disabled redirects, and header redaction.

## Evidence and limits

Only source-inspection commands (`git diff`, `git show`, `rg`, `cat`, `sed`, `nl`) and this report write were performed. No Cargo, test, lint, format, build, packaging, or other verification gate ran. The operator verification lock remains CLOSED. Build compatibility, lockfile reproducibility, Rust 1.95 fallback behavior, pinned-nightly behavior, and publication success are not certified by this review. Installed dependency source was inspected for concrete APIs; no claim that every selected package is the latest available release is made.

Verdict: fix the HIGH SDK server/full compatibility gap before treating the SDK dependency update as complete. No separate blocking source finding was identified for the root-image dependency patch. This report does not complete phase acceptance task 5.2 or replace the lead's authorized production build receipt.

## Correction review — manifest fixed; SDK lock pending

Fresh source inspection on 2026-10-24 finds `[patch.crates-io]` in `sdks/rust/Cargo.toml` with all six lifecycle packages. For example:

```toml
surrealdb = { path = "../../vendor/patches/surrealdb-lifecycle/surrealdb-3.3.0" }
surrealdb-engine-api = { path = "../../vendor/patches/surrealdb-lifecycle/surrealdb-engine-api-3.3.0" }
```

The other entries cover engine-local, core, kvs-surrealkv, and kvs-rocksdb at the same 3.3.0 patched paths, matching the root patch set with the correct SDK-relative prefix. The manifest omission identified above is resolved at source level. The SDK lock still reads `source = "registry+https://github.com/rust-lang/crates.io-index"` for SurrealDB 3.3.0, so lock refresh remains pending; the complete HIGH finding is not yet closed. No compile or dependency-resolution result is claimed.

The provenance README now states `UAR depends directly on the runtime-free core; its existing host owns worker execution.` The minor documentation observation is resolved. Only source inspection and this append were performed; product acceptance remains CLOSED. This correction review covers the two requested corrections only.

## Correction review — SDK lifecycle lock resolved

Fresh SDK lock source inspection on 2026-10-24 finds:

```toml
name = "surrealdb"
version = "3.3.0"
dependencies = [
```

The previous registry `source` and `checksum` fields are absent. The same local-package representation is present for `surrealdb-core`, `surrealdb-engine-api`, `surrealdb-engine-local`, and `surrealdb-kvs-surrealkv`, all version 3.3.0. The SDK manifest's six lifecycle patch entries remain the source selection; `surrealdb-kvs-rocksdb` appears under `[[patch.unused]]`, appropriate for an unselected backend. The SDK package's lock dependencies also now include `webpki-root-certs` alongside reqwest 0.13.5.

The HIGH finding above is resolved at manifest/lock source level. No remaining blocker from this finding is identified. This resolution does not certify SDK compilation or runtime behavior.

For the lead's separate production-build repairs, root lock inspection finds zip 8.6.0 (alongside existing 3.0.0/7.4.0 entries), and pgvector 0.4.2 points to the single sqlx 0.8.6 package. This records source state only; the build failures and successful repair behavior require the lead's actual build receipts. No Cargo, tests, builds, or other verification commands ran in this correction review. Whole-phase product acceptance remains CLOSED and task 5.2 remains outside this report.
