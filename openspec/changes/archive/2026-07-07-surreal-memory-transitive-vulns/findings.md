# Findings: surreal-memory-transitive-vulns

## Reachability summary

| Advisory | Crate | Path | Reachable? | Disposition |
|---|---|---|---|---|
| `RUSTSEC-2026-0193` (ammonia mXSS via MathML `annotation-xml`) | `ammonia` 4.1.2 | `surrealdb-core` → `ammonia` (via `surreal-memory`) | Not reachable — `surrealdb-core`'s only call site (`fnc::string::html::sanitize`) uses `ammonia::clean()` with default config; the CVE requires `math`+`annotation-xml` tags to be explicitly enabled, which nothing here does | **Fixed anyway** — `cargo update -p ammonia --precise 4.1.3`, free/zero-risk, fully clears the advisory |
| `RUSTSEC-2026-0204` (crossbeam-epoch invalid pointer deref in `Display`) | `crossbeam-epoch` 0.9.18 | `surrealdb-core`/`rayon-core` → `crossbeam-deque` → `crossbeam-epoch` | Not reachable — requires application code to `Display`/format an `Atomic`/`Shared` pointer created via `::null()`; no such call site in UAR, surrealdb-core, or rayon's normal operation | **Fixed anyway** — `cargo update -p crossbeam-epoch` (0.9.18→0.9.20), free/zero-risk, fully clears the advisory |
| `RUSTSEC-2023-0071` (rsa Marvin Attack timing side-channel) | `rsa` 0.9.10 | `jsonwebtoken` → (a) `surrealdb-core`, (b) `liter-llm`, (c) direct UAR dep | (a)/UAR direct: not reachable — UAR's own JWT usage (`src/uar/security/middleware.rs`, `api_keys.rs`) is HMAC-only (`from_secret`), never constructs an RSA-signed token; SurrealDB auth is `Root` (username/password), not RECORD/scope JWT, so surrealdb-core's own jsonwebtoken+rsa path isn't exercised. (b) **is** reachable: `liter-llm`'s Vertex AI OAuth (`crates/liter-llm/src/auth/vertex_oauth.rs:191-200`) signs a service-account JWT assertion with `Algorithm::RS256` + `EncodingKey::from_rsa_pem`, used whenever a Vertex AI provider is configured with service-account credentials | **No fix possible** — advisory has `patched = []` (no version of `rsa` fixes this; upstream has not shipped a constant-time implementation as of this writing). Even where reachable, the Marvin Attack's threat model doesn't fit: it requires an attacker to trigger many RSA decrypt/sign operations and measure network-observable timing to recover key bits. Here the signing is outbound and self-triggered — UAR signs its own assertion locally to authenticate to Google's OAuth endpoint; there's no external, attacker-controlled input to the signing operation, and the only network leg (UAR → Google) carries the finished token, not a timing-observable oracle. **Accepted risk, disclosed.** |

## Why no `surreal-memory` pin resync was needed

The original plan considered resyncing `surreal-memory`'s `branch = "main"`
pin to pull in a newer `surrealdb-core`. That turned out to be unnecessary
and higher blast-radius than needed: both fixable advisories (`ammonia`,
`crossbeam-epoch`) are transitive dependencies resolved independently via
`Cargo.lock`, and a scoped `cargo update -p <crate>` bumps them within
their existing semver-compatible ranges without touching `surreal-memory`'s
own commit or pulling in any unrelated upstream changes.
