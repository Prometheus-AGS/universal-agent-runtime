# Analysis — uar-1-0-readiness

Written 2026-08-11 in the Claude Code harness, after `assessment.md`. Research
was tiered per the analyze pipeline; every claim below was verified by reading
source on disk, not from training-era memory.

**Budget:** well under cap. Tier 4 (web comparison) was never reached — Tier 1/3
answered the only contested question.

## Headline: the JWKS verifier is an adopt, not a build

`jsonwebtoken` is **already a direct dependency** at `Cargo.toml:393` (v11.0.0),
and v11 ships a complete `jwk` module. Verified by reading the vendored crate
source at
`~/.cargo/registry/src/index.crates.io-*/jsonwebtoken-11.0.0/src/`:

| Need | Provided by | Evidence |
|---|---|---|
| Parse a JWKS document | `jwk::JwkSet` | `jwk.rs:672` |
| Select key by `kid` | `JwkSet::find(&str) -> Option<&Jwk>` | `jwk.rs:678` |
| JWK → verification key | `DecodingKey::from_jwk(&Jwk)` | `decoding.rs:213` |
| RSA and Ed25519 key material | `from_rsa_components` / `from_ed_components` | `decoding.rs:121`, `:204` |
| RS256 / ES256 / EdDSA algorithms | `Algorithm` enum | `algorithms.rs:53-72` |

**So GAP-02 needs no new cryptographic dependency.** What UAR must write is the
*non-cryptographic* remainder: fetch the JWKS document over HTTPS, cache it,
refresh on unknown `kid`, and enforce issuer/audience. That is orchestration
around a vetted primitive — exactly the split we want for a security change.

I evaluated the dedicated wrapper crates (`jwt-authorizer`, `alcoholic_jwt`,
`jwks`) and **reject them**: none is present in the tree, each would add a
transitive HTTP-client and caching stack alongside the `reqwest` we already
carry, and all of them wrap the same `jsonwebtoken` primitives we can call
directly. Adding a dependency to avoid ~150 lines of fetch-and-cache is a poor
trade when the dependency sits on the authentication path.

> **Correction to my own assessment.** I wrote that the FR-5.1 widening is "new
> construction, not a widening". That is right about `TokenVerifier` — no such
> trait exists — but I implied the crypto was also greenfield. It is not. The
> verifier is assembly, not invention.

## Dependency findings

**`reqwest` 0.12 is present and TLS-capable** (`Cargo.toml:268-274`). I flagged
"confirm the TLS feature before assuming it" and then confirmed it:
`default-features = false` with `rustls-tls-native-roots` enabled, alongside
`json`, `stream`, `multipart`. Outbound HTTPS to an IdP works today. **The JWKS
fetch needs no new HTTP dependency and no feature change.**

**No cache crate is present.** No `moka`, no `dashmap`. Recommend **build, not
adopt**: a `RwLock<HashMap<String, DecodingKey>>` with a refresh timestamp is
sufficient for a key set that turns over on the order of days, and it matches the
concurrency primitive already used throughout (`TaskStore` uses exactly this
shape). Pulling `moka` for one small map is disproportionate.

**A duplicate `jsonwebtoken` exists and is not our problem.** v10.4.0 is in the
lockfile via `surrealdb-core v3.2.4`; v11.0.0 is ours and `liter-llm`'s. Verified
with `cargo tree -i`. Transitive, isolated, no action — recorded so a future
reader does not re-investigate it.

## Build-vs-adopt, per gap

| Gap | Verdict | Rationale |
|---|---|---|
| **GAP-02** JWKS verifier | **Adopt `jsonwebtoken::jwk` + build the fetch/cache layer** | Crypto is vetted and already present; only orchestration is missing |
| **GAP-03a** tenant claim | **Build** | A field on `UserClaims` (`claims.rs:4-9`). No library involved |
| **GAP-03b** partition the store | **Build** | Change the two map key types in `task_store.rs:17-21`. No library involved |
| **GAP-05** register builtins on embedded | **Build** | Call two existing functions from the SDK path. Pure wiring |
| **FR-5.1 `TokenVerifier`** | **Build** | The trait and `Presented` enum are the contract; PID owns the future adapters |

Nothing here warrants a new dependency. That is a finding, not an omission — I
looked, and the ecosystem answer was "you already have it."

## Execution order — load-bearing, not advisory

The assessment established that GAP-03 presumes a tenant identity the runtime
does not have. That forces a strict chain:

```
GAP-02 (verifier + TokenVerifier trait)
   └─> GAP-03a (tenant_id claim, populated from the verified token)
          └─> GAP-03b (partition tasks + context_index)

GAP-05  — independent, may run in parallel
```

**Why the order cannot be relaxed:** populating `tenant_id` from an unverified
token is worse than having no tenant field at all, because downstream code would
then treat an attacker-controlled string as an isolation boundary. 03a must
consume a claim the verifier has already authenticated.

GAP-05 touches `server.rs` and the Rust SDK; GAP-02/03 touch
`security/` and `api/a2a/`. **No file overlap**, so GAP-05 may run concurrently —
which matters because `HARNESS-HANDOFF.md` records that changes editing the same
file with no stated order is a top executor-failure mode.

## Risks

**R-1 — The tenant claim's provenance is unresolved (OQ-2).** If flint-gate mints
`tenant_id`, 03a is an integration contract and the claim name must match theirs;
if UAR mints it, 03a is a local shape change. Guessing wrong means rework in the
one place where rework is expensive. **This is the phase's single genuine
blocker for 03a**, and it does not block GAP-02 or GAP-05.

**R-2 — The auth-bypass defect (OQ-1) sits inside GAP-02's blast radius.**
`middleware.rs:85` hardcodes `jwt_required: false`. If GAP-02 lands without
addressing it, the new verifier inherits a caller that discards its verdict for
invalid tokens. Recommend folding it in; recorded as an operator question rather
than assumed.

**R-3 — Fail-closed behaviour needs a test that can fail.** The prior phase's
strongest artifact was a negative control demonstrating the L4 test *could* fail.
Apply the same discipline: every "session refused" assertion needs a paired case
proving the assertion is capable of failing, or it proves nothing.

**R-4 — `Validation::default()` currently checks no issuer or audience.**
(`middleware.rs:46`.) A JWKS verifier that fetches the right key but skips `iss`
and `aud` accepts any token signed by that IdP for any application. The spec's
own note at `SPECIFICATION.md:507,520` requires audience acceptance and
fail-closed behaviour. Do not treat "signature verifies" as "token is valid".

## Open questions carried to spec

Unchanged from assessment — **OQ-1** (fold in the bypass fix) and **OQ-2** (tenant
claim provenance). OQ-2 now has a named consequence: it gates GAP-03a only, so
the spec can order GAP-02 and GAP-05 ahead of it and keep the phase moving while
the answer is pending.
