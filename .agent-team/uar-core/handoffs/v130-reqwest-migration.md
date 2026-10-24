# v1.3.0 scoped reqwest migration handoff

Phase: `uar-post-lifecycle-verification-readiness`, Execute. Product F/V verification remains CLOSED. Commands in this packet were classified as source inspection or authorized implementation/documentation writing. No test, Cargo, build, formatter, lint, compiler probe, or review-driven verification command was run. The lead is the sole Cargo/build writer.

## Changed files

- `tools/mcp-server-fetch/Cargo.toml`: direct reqwest `0.12` becomes `=0.13.5`; feature `rustls-tls` becomes `rustls`; gzip retained.
- `tools/uar-jwt-proxy/Cargo.toml`: direct reqwest `0.12` becomes `=0.13.5`; feature `rustls-tls` becomes `rustls`; stream and http2 retained.
- `sdks/rust/Cargo.toml`: optional direct reqwest `0.12` becomes `=0.13.5`; feature `rustls-tls` becomes `rustls`; explicit `query` added for `src/client.rs` run-stream resume requests; json, multipart, stream retained. Existing TLS comment corrected because reqwest 0.13 defaults to rustls.

No Rust sources or lockfiles changed by this writer. No guards added. No unrelated implementation added.

## Source inspection and compatibility findings

Inspected fetch's complete client path, proxy's client construction/HTTP forwarding/header conversions, and SDK client/request/stream/multipart/error paths. Their used APIs remain represented by the official 0.13.5 interface. No source compatibility fix identified by inspection. This is not a compiler or runtime acceptance claim.

The SDK exposes reqwest types in public `Client::with_client` and `Error::Http`. Downstream code supplying reqwest 0.12 clients must move to the matching reqwest version. Proxy header interoperability uses `http` v1, also used by reqwest 0.13.5. SDK independent `sdks/rust/Cargo.lock` still needs lead-owned resolution if included in this migration; it contained reqwest 0.12.28 and 0.13.4 during inspection.

Official evidence consulted:

- https://docs.rs/reqwest/0.13.5/reqwest/ — supported APIs/features, including `query`, `form`, `rustls`, stream, json, multipart.
- https://github.com/seanmonstar/reqwest/blob/v0.13.5/CHANGELOG.md#v0130 — `rustls-tls` renamed; query/form now optional; rustls defaults to aws-lc and platform verifier; TLS methods renamed with old spellings soft-deprecated.
- https://github.com/seanmonstar/reqwest/blob/v0.13.5/Cargo.toml — `http = 1.1`, Rust minimum 1.85.0, feature wiring and dependent transport ranges.

## Remaining work and uncertainty

Lead owns root pins, lockfile updates, and any explicitly authorized serialized build repair. Compilation, dependency resolution, mobile cross-compilation, TLS runtime behavior, and SDK downstream compatibility are unverified. The rustls backend now selects aws-lc and platform trust verification per upstream; it must not be represented as identical to old root/provider behavior. No broader dependency updates are necessary within these three manifests based on source inspection alone.

## Reviewer correction follow-up

The earlier conclusion that no Rust source migration was needed is superseded by the concrete Android TLS regression identified in `.agent-team/uar-core/reviews/v1.3.0/dependency-publication-code-review.md` and the independent SDK lifecycle finding in `dependency-publication-rust-review.md` (see actual review filenames in that directory). These corrections are scoped to the SDK; root transport correction is owned by the lead.

- `sdks/rust/Cargo.toml`: added the root's six SurrealDB lifecycle `[patch.crates-io]` entries with `../../vendor/patches/surrealdb-lifecycle/` paths. The independent SDK workspace now declares the same production lifecycle API sources as the root. Preserved the lead's existing `=1.3.0` runtime dependency edit.
- `sdks/rust/Cargo.toml`: pinned optional `webpki-root-certs =1.0.11` and enabled it through `http-client`. Official `https://docs.rs/webpki-root-certs/latest/webpki_root_certs/` resolved to 1.0.11 during this turn.
- `sdks/rust/src/client.rs`: `Client::new` builds a fallible HTTP client with explicit rustls and only the bundled Mozilla roots on native targets. It converts `TLS_SERVER_ROOT_CERTS` with `Certificate::from_der`, collects errors, and passes roots to `tls_certs_only`. It uses `build()?`, returning `Error::Http` rather than the previous implicit `Client::new` panic path. Constructor error documentation now includes initialization failure. Custom-client construction remains caller-controlled. Browser WASM retains its browser TLS handling under the same target predicate used by reqwest.

Exact source basis: downloaded `webpki-root-certs-1.0.11/src/lib.rs:37` declares `TLS_SERVER_ROOT_CERTS: &[CertificateDer<'static>]`; downloaded `reqwest-0.13.5/src/tls.rs:166` accepts DER bytes and returns `reqwest::Result<Certificate>`; `reqwest-0.13.5/src/async_impl/client.rs:1901` sets `tls_certs_only = true`; its rustls builder's explicit-roots branch uses `with_root_certificates` rather than `rustls_platform_verifier::Verifier`. Sources are under `/Users/gqadonis/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`. Official API: https://docs.rs/reqwest/0.13.5/reqwest/struct.ClientBuilder.html#method.tls_certs_only . The concrete named failure is Android HTTPS without platform-verifier JVM initialization; certificate and hostname verification remain enabled.

No checks, tests, Cargo commands, builds, formatters, compiler probes, or lockfile edits ran in this correction. Root owns all lock resolution. Native/mobile compilation and HTTPS runtime behavior remain unverified; explicit roots require SDK dependency updates to receive future trust-store revisions, as the prior bundled-root selection did. No unrelated features or guards were added.
