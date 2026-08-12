# Assessment — uar-1-0-readiness

Written 2026-08-11 in the Claude Code harness. Every claim below was grounded by
reading the file, not by reading `docs/SPECIFICATION.md`. That distinction is the
whole point: the prior phase's spec errors came from trusting the capability list
over the call graph.

**Baseline:** `origin/main` at `88c38015`.

## Headline: grounding changed the phase, it did not confirm it

Four of the spec's own citations have drifted, and one gap is materially worse
than recorded. **A phase planned from `SPECIFICATION.md` alone would have been
wrong in four places before the first line of code.**

| Spec says | Grounded reality | Effect on scope |
|---|---|---|
| GAP-05 at `src/server.rs:436` | **`src/server.rs:448`** — and there is a *second*, separate registration at **`:511`** (`skill_service.register_builtins`) | GAP-05 is **two** call sites, not one. Fixing 448 alone leaves the pack loader unregistered |
| GAP-03 at `a2a/task_store.rs:16` | **`:17-21`** — `tasks` *and* `context_index`, two flat maps | Partitioning must cover both, or `get_by_context` leaks across tenants |
| GAP-02 "no JWKS code" | **Confirmed, and stronger** — zero occurrences of `jwks`, `jwk`, or `rs256` anywhere in `src/` | Unchanged, now verified exhaustively |
| C-21 "tenant isolation" | **`tenant_id` appears nowhere in `src/`.** `UserClaims` is `{sub, name, roles, exp}` (`src/uar/security/claims.rs:4-9`) | See below — this is the finding that reshapes the phase |

## The finding that reshapes the phase: there is no tenant concept to partition by

`grep -rn "tenant_id" src/` returns **nothing**. `UserClaims` carries no tenant.
The A2A module contains no tenant reference at all.

So GAP-03 as written — *"A2A task store not tenant-partitioned"* — presumes a
tenant identity that the runtime does not have. **Partitioning a store by a key
that does not exist is not implementable as specified.** This is exactly the
error shape the prior phase kept hitting, caught this time *before* the spec was
written rather than by the executor mid-implementation.

GAP-03 therefore splits into an ordered pair:

1. **GAP-03a — introduce the tenant claim.** Add `tenant_id` to `UserClaims`,
   populate it from the verified token, and thread it into `A2AState`. Fail
   closed when absent and `jwt_required` is true.
2. **GAP-03b — partition both maps** in `TaskStore` by that claim, covering
   `tasks` *and* `context_index`.

**03a is a hard prerequisite for 03b, and 03a depends on GAP-02's verifier**,
because the claim has to come from somewhere trustworthy. That makes the
execution order load-bearing — precisely the kind of thing `HARNESS-HANDOFF.md`
says the contract must state explicitly rather than leave to the executor.

## A live defect the spec does not record

`src/uar/security/middleware.rs:84-88` calls:

```rust
let mut context = resolve_user_context(
    false,                                            // <- jwt_required, hardcoded
    state.config.security.jwt_secret.expose_secret(),
    auth_header,
)?;
```

The first parameter is `jwt_required`, and it is **hardcoded `false`** at the
only call site. `resolve_user_context` uses it in two places
(`middleware.rs:36`, `:57`) to decide between `401` and falling through to
`anonymous_context()`. With `false` pinned, **both branches fall through**: an
absent token and an *invalid* token both yield anonymous access rather than 401.

`security.jwt_required` defaults to `true` (`src/config.rs:1011`), is settable by
CLI (`:1112`), and is covered by a passing test (`:1778`) — the config plumbing
works perfectly and its value is then discarded at the point of use.

**Status: needs operator ruling, not a silent fix.** It is a real
authentication-bypass shape and it sits in the exact function GAP-02 rewrites, so
fixing it here is nearly free. But it is not in `SPECIFICATION.md`, and the prior
phase's lesson is that unreviewed scope expansion by the implementer is a failure
mode even when defensible. Recorded as **OQ-1** below.

*Caveat I can state now rather than after implementing:* I have not traced whether
some upstream layer independently enforces auth, which would soften the severity.
The middleware is mounted once (`src/server.rs:1131-1134`) and is the only JWT
path, so I have no evidence of a second gate — but "no evidence of" is not "proof
of absence", and I am labelling it that way deliberately.

## Gap-by-gap, grounded

### GAP-02 — no JWKS/RS256 verifier

- **Now:** `DecodingKey::from_secret(jwt_secret.as_bytes())` with
  `Validation::default()` (`middleware.rs:45-46`). HS256, one shared symmetric
  secret, no issuer or audience check, no key rotation.
- **Missing:** any asymmetric path. Zero `jwks`/`jwk`/`rs256` occurrences in `src/`.
- **Also missing:** any `TokenVerifier` trait. `grep -rn "TokenVerifier"` returns
  nothing, so the PID FR-5.1 widening is **new construction**, not a widening of
  something present. The spec's phrasing implies otherwise.
- **Blocks:** San Saba adoption (`SPECIFICATION.md:442`).

### GAP-03 — A2A task store not tenant-partitioned

- **Now:** `RwLock<HashMap<String, Task>>` keyed by task id, plus
  `RwLock<HashMap<String, String>>` for `context_id → task_id`
  (`task_store.rs:17-21`). Constructed once, process-wide, at `server.rs:652`.
- **Missing:** the tenant concept itself. See above.
- **Security property, undelegatable** — `SPECIFICATION.md:392` states tenant
  isolation of UAR's own stores cannot be delegated to Forge's RLS.

### GAP-05 — builtins not registered on embedded

- **Now, two sites:** `native_skills::register_builtins` at `server.rs:448`, and
  `skill_service.register_builtins(builtins)` at `server.rs:511` behind the
  builtin-pack loader.
- **Embedded path:** `sdks/rust/src/runtime.rs:120-123` exposes `skill_service()`
  and calls **neither**.
- **Precision the spec overstates** (judge ruling 2026-08-09, still uncorrected in
  `SPECIFICATION.md:445`): the registry is empty of *built-ins* always; empty
  *overall* only on a fresh device, because `SkillService::initialize` loads
  persisted skills via `DatabaseStorageProvider`. "Empty skill registry" and
  "capability at 0%" both overstate the code. **Amend the spec in this phase** —
  the prior phase deferred it deliberately to avoid editing the artifact it was
  measuring against. That constraint no longer applies.

## Open questions for the operator

| # | Question | Why it needs you |
|---|---|---|
| **OQ-1** | Fix the hardcoded `jwt_required: false` (`middleware.rs:85`) in this phase? | Real auth-bypass shape, sits inside GAP-02's rewrite, but is unrecorded scope. Recommend **yes** — it is cheaper here than anywhere else, and shipping a new verifier on top of a discarded flag would be worse |
| **OQ-2** | Is `tenant_id` a new JWT claim UAR mints, or does it arrive from flint-gate? | Decides whether GAP-03a is a claims-shape change or an integration contract. PID FR-5.1 likely constrains this |

Both are answerable now and neither blocks writing the analysis.

## What this phase deliberately does not claim

Closing these three gaps does not produce a 1.0-ready runtime. It closes the
three gaps that `SPECIFICATION.md` records as **adoption-blocking and UAR-local**.
The structural measurement limits from the conformance phase — no semantics, one
profile, no real-provider behaviour — are untouched and out of scope here.
