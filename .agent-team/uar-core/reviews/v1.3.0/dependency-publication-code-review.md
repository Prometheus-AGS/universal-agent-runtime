# Dependency/publication code review

Reviewed 2026-10-24: HEAD `f870a9c337a89142e56bd8945b89c652c3c1385d` plus the working-tree dependency/publication patch. Phase: Execute, `uar-post-lifecycle-verification-readiness`, task 5.1. This is source inspection of the publication patch, not phase acceptance. Product verification remains CLOSED. No test, compiler, build, lint, format, packaging, or diff-check command was run by this reviewer.

## Findings

### [HIGH] [Cargo.toml:398] Migrate the supervisor TLS builder when upgrading reqwest

Exact changed artifact:

```toml
reqwest = { version = "=0.13.5", default-features = false, features = [
```

Its existing production consumer at `src/uar/runtime/supervisor.rs:186` is:

```rust
let client = reqwest::Client::builder().use_rustls_tls().tls_built_in_root_certs(false)
    .add_root_certificate(ca).identity(identity).no_proxy()
```

Issue: reqwest 0.13.5's `ClientBuilder` has no `tls_built_in_root_certs` method. The version upgrade therefore makes the runtime fail to compile. The supervisor module is declared unconditionally at `src/uar/runtime/mod.rs:24`; making supervisor configuration optional at startup (`src/server.rs:1973`) does not exclude this function from compilation. I read the complete supervisor module, its constructor's startup caller, the root manifest (`edition = "2024"`), and the downloaded reqwest 0.13.5 client source. A source-wide search in that reqwest release returned no definition of the old method. This is an API incompatibility, not a stylistic preference.

Fix: migrate the builder to the new `tls_certs_only([ca])` API and retain client identity, no-proxy, no-redirect and timeout settings. Removing only the old call would broaden this trusted controller connection from the configured private CA to platform roots, contrary to the constructor's explicit contract. The new method's implementation sets `tls_certs_only = true`; its documentation states that it excludes native and built-in roots. Upstream context: [TLS option consolidation issue](https://github.com/seanmonstar/reqwest/issues/2885), [0.13 maintainer migration discussion](https://github.com/seanmonstar/reqwest/discussions/2906).

### [HIGH] [sdks/rust/Cargo.toml:34] Preserve Android HTTPS initialization behavior

Exact changed artifact:

```toml
reqwest = { version = "=0.13.5", default-features = false, features = ["json", "multipart", "query", "stream", "rustls"], optional = true }
```

Issue: this replaces the SDK's previous reqwest 0.12 web-PKI rustls backend with reqwest 0.13's platform verifier. An Android consumer using the default SDK `http-client` feature and `Client::new("https://...")` now reaches rustls-platform-verifier without the JVM/context initialization or Kotlin verifier component that it requires. The first certificate verification calls `with_context`, which calls `global()`, whose missing-global branch is `expect("Expect rustls-platform-verifier to be initialized")`. That HTTPS task panics instead of returning an SDK request result.

Evidence: `sdks/rust/src/client.rs:34-39` constructs `reqwest::Client::new()` directly. The lockfile selects `rustls-platform-verifier 0.7.1`. In the downloaded reqwest 0.13.5 manifest, feature `rustls` enables that verifier; `async_impl/client.rs:750-760` chooses it for ordinary root validation. The verifier's `src/verification/android.rs` calls `with_context` during certificate verification, and `src/android.rs:87-90,208-213` contains the initialization requirement and panic path. Searches of `sdks`, `src`, and `src-tauri` found no verifier initialization. The SDK's custom-client constructor is optional and does not guard its default constructor. Both affected manifests use edition 2024. I read both SDK constructors in full and traced the request send path. The SDK manifest explicitly identifies Android/iOS embedding as a supported use case, so the new host integration requirement is a concrete compatibility regression.

Fix: either preserve a platform-independent, explicitly rooted TLS configuration for the SDK default constructor, or provide and document the Android verifier component plus initialization entrypoint before HTTPS requests. Keep certificate validation enabled. Upstream context: [initialization documentation repair PR](https://github.com/rustls/rustls-platform-verifier/pull/236), [Android initialization documentation](https://docs.rs/rustls-platform-verifier/0.7.1/rustls_platform_verifier/android/index.html). This finding affects Android SDK use; it is not a Linux GHCR runtime failure.

## Other inspected scope

The Parking Lot facade's core and util modules re-export the exact core contracts now imported directly by ingestion worker/tests. The eight standalone manifests materialize the former workspace package fields, dependency requirements/features and lint values, retaining `rust-version = "1.95"`; UAR's existing exclusions cover those packages. I found no supported behavioral or owner-isolation defect in that narrowing.

The MCP alias substitutions preserve header validation and secret handling, no-proxy and no-redirect client configuration. rmcp 3.7.0's downloaded manifest accepts reqwest `0.13.2` (a compatible range) with defaults disabled; its transport configuration continues to use the same HTTP header types and concrete client API. Root/tool exact reqwest pins and root/SDK lock entries identify 0.13.5. The remaining reqwest 0.12 lock entry is transitive and is not proof that an owned alias was missed.

Read-only commands included `git diff --staged --stat` (empty), `git diff --stat`, scoped `git diff`, `git show HEAD:<manifest>`, `git rev-parse HEAD`, `cat`, `sed`, `nl`, and `rg` for source/manifests/callers. `firecrawl developer ... --json --limit 10` could not perform dependency retrieval: `error: unknown command 'developer'`. Official upstream pages and the locally downloaded exact crate sources were used instead.

No production files changed by this reviewer. Only this review artifact was added. No new guards were proposed beyond preserving the existing private-CA boundary and the supported Android TLS initialization contract. Compilation, runtime TLS behavior, dependency resolver success at Rust 1.95, and publication are unverified by execution here; those remain with the lead's authorized gate.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 2 | warn |
| MEDIUM | 0 | info |
| LOW | 0 | note |

Verdict: WARNING — 2 HIGH compatibility issues should be resolved before accepting the complete dependency patch. The supervisor API issue directly blocks the runtime build; the SDK issue affects Android consumers.

## Source-only follow-up: both findings resolved in implementation

Re-read the actual supervisor, SDK constructor, SDK feature/dependency declarations, SDK error conversion and vendor README after the writer's completion handoff (`.agent-team/uar-core/handoffs/v130-reqwest-migration.md`). The earlier findings and verdict above are retained as review history and are superseded by this follow-up.

Finding 1 resolution, `src/uar/runtime/supervisor.rs:186-189`:

```rust
let client = reqwest::Client::builder().tls_backend_rustls().tls_certs_only([ca])
    .identity(identity).no_proxy()
    .redirect(reqwest::redirect::Policy::none()).timeout(Duration::from_secs(60))
    .build().map_err(|_| SupervisorError::Unavailable)?;
```

Both selected APIs exist in the downloaded reqwest 0.13.5 source. Its `tls_certs_only` branch builds a root store from the supplied certificates and bypasses platform roots. Client identity, redirect/proxy restrictions, and timeout remain present. This addresses the exact removed-API failure while preserving the trusted controller boundary.

Finding 2 resolution, `sdks/rust/src/client.rs:35-51`:

```rust
let base_url = Url::parse(base_url.as_ref())?;
let http = reqwest::Client::builder();
#[cfg(not(all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none"))))]
let http = {
    // Preserve bundled roots without requiring Android JVM verifier setup.
    let roots = webpki_root_certs::TLS_SERVER_ROOT_CERTS
        .iter()
        .map(|certificate| reqwest::Certificate::from_der(certificate.as_ref()))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    http.tls_backend_rustls().tls_certs_only(roots)
};
Ok(Self {
    base_url,
    http: http.build()?,
    api_key: None,
})
```

The `http-client` feature enables the optional exact `webpki-root-certs =1.0.11` dependency. Its source declares `TLS_SERVER_ROOT_CERTS` as a slice of full certificate DER values. The native client therefore follows reqwest's explicit-root branch rather than Android platform-verifier initialization; browser wasm keeps the browser client builder. DER/build errors convert through the existing `Error::Http(#[from] reqwest::Error)` variant. This preserves the SDK's earlier bundled-root behavior and resolves the named Android panic path by source inspection.

The SDK also now mirrors the six root Surreal lifecycle patches with SDK-relative paths. The writer reports no outstanding source edits. The lead still owns the SDK lock refresh: the inspected pre-refresh lock lacks the newly direct root-certificate dependency on the SDK package and still has registry source identities for patched Surreal packages. This is explicitly pending integration work, not a claim of a passing resolver result. Vendor README now correctly says UAR consumes the runtime-free core and owns execution.

Further inspection of changed root pins and their production call sites found no additional supported actionable compatibility blocker. The exact downloaded manifest minimums inspected are rmcp 3.7.0 Rust 1.88, kreuzberg 4.10.4 Rust 1.91, jsonwebtoken 11.1.0 Rust 1.88, prost-types 0.14.4 Rust 1.85, http-body-util 0.1.5 Rust 1.61 and jiff 0.2.38 Rust 1.70. These metadata observations do not establish that the entire transitive resolution builds at the Rust 1.95 floor.

Only source reads and this review-artifact append were performed during follow-up. No verification command was run. No new production guard or adjacent improvement is requested. Compilation, SDK lock integration, Android execution and image publication remain unverified by this reviewer; product acceptance stays CLOSED.

## Review Summary

| Severity | Open count | Status |
|----------|------------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass; 2 resolved in source |
| MEDIUM | 0 | info |
| LOW | 0 | note |

Verdict: APPROVE for the inspected implementation corrections. SDK lock refresh remains assigned to the lead, and this source-only approval provides no build, runtime, publication or phase-acceptance evidence.
