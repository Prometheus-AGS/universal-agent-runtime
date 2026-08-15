## Why

GAP-02 (`docs/SPECIFICATION.md:442`): UAR verifies JWTs with a single shared
symmetric secret and no asymmetric path. `src/uar/security/middleware.rs:45-46`
builds `DecodingKey::from_secret(...)` with `Validation::default()`. There are
**zero** occurrences of `jwks`, `jwk`, or `rs256` in `src/`. This blocks San Saba
adoption, which authenticates against a JWKS-publishing IdP.

Two further defects live in the same function and are cheaper to fix here than
anywhere else:

1. **`jwt_required` is discarded at the point of use.** `middleware.rs:85` passes
   a hardcoded `false` into `resolve_user_context`, whose only two uses of that
   parameter (`:36`, `:57`) decide between `401` and anonymous fallthrough. With
   `false` pinned, **an absent token and an invalid token both yield anonymous
   access**. `security.jwt_required` defaults to `true` (`src/config.rs:1011`),
   is CLI-settable (`:1112`), and has a passing test (`:1778`) — the config
   plumbing works and its value is then thrown away.
2. **`Validation::default()` checks neither issuer nor audience** (`:46`). A
   verifier that fetches the correct key but skips `iss`/`aud` accepts any token
   that IdP signed, for any application. `SPECIFICATION.md:507,520` requires
   audience acceptance and fail-closed behaviour.

This change extends the existing `jwt-hardening` capability rather than creating
a parallel one. `harden-jwt-defaults` (0/3, untouched since `3a54b965`,
2026-07-14) already specifies issuer/audience enforcement and fallback-secret
refusal. **Two changes editing `middleware.rs` with no stated precedence is the
top cross-change failure mode in `.kbd-orchestrator/HARNESS-HANDOFF.md`.**

## What Changes

- Add a `TokenVerifier` trait shaped to PAGS-SPEC-PID-001 FR-5.1: one trait, a
  `Presented` enum (`Jwks` now; `SdJwtVp` and `DidAuth` reserved for PID P4),
  returning a single `Principal`. No `TokenVerifier` exists in `src/` today, so
  this is new construction.
- Implement the `Jwks` lane using `jsonwebtoken` 11's `jwk` module, **already a
  direct dependency** (`Cargo.toml:393`): `JwkSet` (`jwk.rs:672`),
  `JwkSet::find` by `kid` (`jwk.rs:678`), `DecodingKey::from_jwk`
  (`decoding.rs:213`). No new dependency.
- Fetch the JWKS document with the existing `reqwest` 0.12
  (`Cargo.toml:268-274`, `rustls-tls-native-roots` already enabled — no feature
  change). Cache keys in a `RwLock<HashMap<String, DecodingKey>>` with a refresh
  timestamp; refresh on unknown `kid`. No cache crate is added.
- Honour `security.jwt_required` at the call site (`middleware.rs:85`).
- Enforce `iss` and `aud` when configured, per the existing `jwt-hardening` delta.
- Retain the HS256 shared-secret lane. PID §6.1 preserves the RS256/JWKS lane
  explicitly, and existing deployments depend on the symmetric path.

## Capabilities

### New Capabilities
- `jwt-hardening`

> `jwt-hardening` does **not** yet exist under `openspec/specs/`. It is declared
> only by the unarchived change `harden-jwt-defaults`, so requirements here are
> `ADDED` rather than `MODIFIED`. Whichever change archives first creates the
> capability; the second adds to it.
>
> **Requirement names are disjoint, but one overlaps in substance and needs a
> precedence rule.** `harden-jwt-defaults` states *"JWT verification uses a
> deliberate secret and full claim validation"*, whose second scenario enforces
> `iss`/`aud`. This change states *"Signature validity alone does not establish
> token validity"*, covering the same ground for the JWKS lane. They agree on
> behaviour — reject on claim mismatch — so neither contradicts the other.
> **Precedence: this change's requirement governs the JWKS lane;
> `harden-jwt-defaults` governs the shared-secret lane and the config surface.**
> Stated here because an executor that finds two requirements over one behaviour
> and no ordering will guess, which `HARNESS-HANDOFF.md` records as a top
> cross-change failure. Everything else is genuinely disjoint: fallback-secret
> refusal is theirs; the verifier abstraction, the JWKS lane, and enforcement at
> the point of use are this change's.

## Impact

`src/uar/security/middleware.rs`, `src/uar/security/claims.rs`, new
`src/uar/security/verifier/`, `src/config.rs` (JWKS URL, issuer, audience).

**Interaction with `fix-sidecar-loopback-auth` (5/6 done).** That change
deliberately defaults JWT enforcement off for the loopback `uar-sidecar` binary
only. It is **compatible**: the sidecar sets `jwt_required` explicitly, and the
defect here is that the flag is *ignored*. Honouring the flag makes the sidecar's
intent effective rather than accidental. The executor must not "fix" a sidecar
test that starts failing by reverting this — see the stop conditions in tasks.

## Non-goals

- SD-JWT VC and DID auth lanes (PID P4 owns these; the enum reserves the shape).
- Any `frf-did` / `frf-wallet` dependency — PID §2.2 supersedes them.
- Replacing the HS256 lane.
