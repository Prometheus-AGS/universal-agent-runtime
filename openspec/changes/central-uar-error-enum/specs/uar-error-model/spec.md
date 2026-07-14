# UAR error model

## Purpose

Define the central `UarError` enum that UAR's public API boundary
(`src/uar/api/`) can return. SDKs in `sdks/{rust,python,typescript}`
consume the stable error codes for typed error matching.

## ADDED Requirements

### Requirement: One central UarError enum
The crate MUST expose a single `pub enum UarError` at
`src/uar/error.rs`, marked `#[non_exhaustive]` so future
variants are not breaking changes. The enum MUST implement
`std::error::Error + std::fmt::Display + std::fmt::Debug`, and
MUST be re-exported at the crate root as `crate::UarError`.

#### Scenario: A handler returns a UarError
- **WHEN** a handler returns `crate::Result<T>` (which is
  `std::result::Result<T, UarError>`) and produces an `Err`
- **THEN** the error is rendered to the HTTP response via the
  unified `IntoResponse for UarError` impl
- **AND** the JSON body is `{ "code": "E_...", "message": "..." }`

### Requirement: Variants grouped by domain
The enum MUST have at least the following variants: `Config`,
`Auth`, `Rag`, `Memory`, `Mcp`, `A2a`, `Llm`, `Internal`. Each
domain variant (all but `Internal`) MUST carry a `code: &'static
str` and a `message: String` field so call sites can supply a
specific, stable leaf code. `Internal` wraps `anyhow::Error` via
`#[from]` for ergonomic `?`-composition at call sites that do not
yet have a more specific variant.

Adoption note: as of this change, most of `src/uar/`'s submodules
(`rag`, `memory`, `mcp`, `governance`, `llm`, etc.) do not have a
dedicated typed `*Error` enum to wrap — `src/uar/compiler/error.rs`
is the only pre-existing one. This change ships the `String`-payload
variant shape above rather than wrapping non-existent typed errors;
migrating individual submodules to typed errors that map onto these
variants is follow-up work, tracked outside this change.

#### Scenario: A new domain-specific failure needs a variant
- **WHEN** a call site needs to signal a domain-specific failure
  (e.g. "no knowledge base configured")
- **THEN** it constructs the matching domain variant via its
  helper constructor (e.g. `UarError::rag("E_RAG_NO_KB", "...")`)
  rather than introducing a new error type or an untyped string

### Requirement: Stable error codes
Every leaf variant MUST have a stable string error code
exposed via the `UarError::code(&self) -> &'static str` method.
The codes are part of the public API and MUST NOT change without
a SemVer major bump. Format: `E_<DOMAIN>_<SPECIFIC>` (e.g.
`E_CONFIG_MISSING_FIELD`, `E_RAG_NO_KB`). `Internal` errors always
report the generic code `E_INTERNAL` (its underlying `anyhow::Error`
is not decomposed into a leaf code).

#### Scenario: A Python SDK consumer matches on an error code
- **WHEN** the SDK calls `await client.chat(...)` and the server
  returns HTTP 400 with body `{"code": "E_AUTH_INVALID_TOKEN", "message": "..."}`
- **THEN** the Python SDK raises `UarError(E_AUTH_INVALID_TOKEN)`
- **AND** the consumer can `except UarError as e: if e.code == "E_AUTH_INVALID_TOKEN": ...`

#### Scenario: A call site has no specific domain classification yet
- **WHEN** a call site propagates an `anyhow::Error` via `?` into a
  `crate::Result<T>`-returning function
- **THEN** it becomes `UarError::Internal`, mapped to HTTP 500 with
  code `E_INTERNAL`
- **AND** this is a valid, permanent variant — not every error needs
  a granular domain code, only errors worth distinguishing for SDK
  consumers do

### Requirement: thiserror for the central enum
`UarError` MUST be defined with `#[derive(thiserror::Error)]`.

Scope note: this change does not require eliminating every
`anyhow!()` call site in `src/uar/`. An audit during this change
found 127 `anyhow!()` call sites across `src/uar/` in total, of
which 8 are in `src/uar/api/` — and all 8 are inside internal trait
implementations (`RetrievalBackend::search_one`, the A2A HTTP
client, `InMemoryAgentRegistry`) several layers removed from an
axum handler's `IntoResponse` boundary, not directly convertible
without changing those traits' signatures. Converting them, and the
~119 sites in other submodules, is deferred to follow-up work (the
`130 anyhow!() in public-API boundary code` figure in this change's
`proposal.md` was an estimate that did not match the audited count).

#### Scenario: A new call site is added at the API boundary
- **WHEN** a new axum handler in `src/uar/api/` is written after
  this change lands
- **THEN** it SHOULD return `crate::Result<T>` and construct a
  `UarError` domain variant instead of introducing a new ad hoc
  error shape

### Requirement: tracing-error context
Every `UarError` rendered via `IntoResponse` MUST capture the
current `tracing_error::SpanTrace` and log it (along with the error
and its code) via `tracing::error!`. The span trace MUST be logged
server-side only and stripped from the public response body (to
avoid leaking internal state to clients).

#### Scenario: A UarError is rendered to an HTTP response
- **WHEN** `UarError::into_response` runs
- **THEN** a `tracing::error!` event is emitted containing the
  error, its `code()`, and the captured `SpanTrace`
- **AND** the HTTP response body contains only `code` and `message`

### Requirement: Optional Sentry integration
A `sentry` Cargo feature flag MUST exist, off by default. When
enabled, `UarError::into_response` MUST report the error to Sentry
via `sentry::capture_error`. DSN/project configuration is operator
work, out of scope for this change.

#### Scenario: The sentry feature is enabled
- **WHEN** the crate is built with `--features sentry`
- **AND** a `UarError` is rendered via `IntoResponse`
- **THEN** `sentry::capture_error` is called with that error

#### Scenario: The sentry feature is not enabled (default)
- **WHEN** the crate is built without the `sentry` feature
- **THEN** no Sentry code is compiled in and no reporting occurs

### Requirement: Result alias at crate root
The crate MUST expose `pub type Result<T> = std::result::Result<T,
UarError>;` in `src/uar/error.rs`, re-exported at the crate root so
`crate::Result<T>` is available to any module.

#### Scenario: A new handler wants a compact signature
- **WHEN** a new function in `src/uar/` needs to return a
  `UarError`-producing result
- **THEN** it can write `crate::Result<T>` instead of spelling out
  `std::result::Result<T, crate::UarError>`
