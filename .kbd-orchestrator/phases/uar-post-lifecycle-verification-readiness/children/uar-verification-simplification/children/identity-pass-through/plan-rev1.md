# Plan revision 1 (superseded 2026-10-28 by plan.md revision 2) — identity-pass-through

KBD Plan, 2026-10-28, revision 1. This is a planning artifact only. Nothing is
implemented, and the verification gate stays CLOSED until every change below
is implemented. It reads `assessment.md`: targets T1-T7, gaps G1-G10, risks
R1-R5.

Execution uses the `uar-core` agent team, with one writer per file and the
verification lock in every packet. Each change becomes one OpenSpec change in
the Spec stage, with at least one spec delta (project rule).

## Changes, in order

### P1. Trust declaration (T2; G1) — owner: uar-trust-tools

- **New setting.** Add `security.identity.mode`, one of:
  - `trusted_front { upstream: <name> }`;
  - `validation_hook { kind: builtin_jwks | builtin_shared_secret | external }`;
  - `single_user { owner: <sub> }`.
- **Refuse unsafe starts.** Validate at startup and refuse to start when the
  mode is `trusted_front`, more than one user is possible, and the HTTP or gRPC
  bind address is not loopback, private, or explicitly attested with
  `security.identity.public_bind_attested=true`.
- **Built-in hooks.** `builtin_jwks` and `builtin_shared_secret` reuse today's
  verifier (`security/verifier/mod.rs`) unchanged. This keeps the RS256 and ES
  paths. An `external` hook is a trait the host implements (embedded) or an
  HTTP callback (server). The callback form is open decision O3.
- **Migrate existing configs.** Today's `jwt_required=true` maps to
  `validation_hook`. `jwt_required=false` with loopback maps to `trusted_front`
  with upstream `local`. The old keys stay readable for one release with a
  deprecation warning.
- **Files:** `src/config.rs` (identity section, `validate_identity_policy`
  replaced), `src/config_manager.rs`, `docs/operations/`.

### P2. Identity extraction on every interface (T1, T3; G2, G4) — owner: uar-trust-tools

- **One function.** A single `presented_identity(headers|metadata) -> Option<PresentedIdentity>`.
  It decodes the JWT payload **without verification** in `trusted_front`, or
  calls the hook in `validation_hook` mode. It yields `iss`, `sub`, an optional
  `tenant`, and the raw token for custody (P5).
- **Anonymous handling.**
  - A malformed token is refused (400).
  - A missing token yields `anonymous`, except in `single_user`, where it
    becomes the configured owner.
- **Same function everywhere:**
  - HTTP middleware (`middleware.rs:195-310`);
  - gRPC `authenticate` (`a2a/grpc.rs:51-140`, `grpc_v1.rs:74`), which no
    longer always verifies;
  - ACP, A2A HTTP and the MCP server (`mcp_server.rs:71-87`).
- **The sidecar launch path is unchanged.** It is not a JWT.

### P3. One partition key (T3; G3) — owners: uar-state, with uar-trust-tools

- **The key.** Introduce `OwnerKey::v2(iss, tenant?, sub)`, a length-prefixed
  string `v2:i:…:t:…|-:s:…`. When a token has no `iss`, use
  `urn:uar:issuer:unspecified`.
- **Replace both families.** It replaces Key A (`claims.rs:164-173`) and Key B
  (`messages.rs:133-145`, `user_settings.rs:138-153`) at every site listed in
  `assessment.md` §2.
- **One function.** Every consumer calls one function. A source search for the
  old builders must find none outside P4's migration code.

### P4. Existing data mapping (T5; G7; R2) — owner: uar-state

- **Lazy, ordered lookup.** Read v2 first, fall back to the legacy keys, and
  write v2, until the operator runs a one-shot migration. The operator decision
  is O1.
- **The mapping rule:**
  - legacy `v1:k:local-jwt:i:urn:uar:local:jwt:t:X:s:S` maps to
    `v2:i:<declared-local-issuer>:t:X:s:S`;
  - legacy `v1:k:issuer:i:<configured>:…` maps to `v2:i:<configured>:…`;
  - Key B rows (tenant + sub) map using the node's single declared issuer. A
    node with more than one issuer and Key B rows needs an operator mapping
    file.
- **Stores.** SurrealDB and PostgreSQL providers. The memory provider has no
  durable rows.
- **A migration receipt.** Rows read, rows mapped, rows left ambiguous. No row
  is deleted.

### P5. Token custody and opt-in forwarding (T4; G6; R4) — owners: uar-trust-tools and uar-runtime

- **Custody.** A `PresentedToken` lives in the request's run context only. It
  is never serialized: no `Serialize`, and Debug is redacted. The scrubber
  keeps capturing it.
- **Opt-in forwarding.** Per destination:
  - `mcp.servers[].forward_presented_identity`;
  - `a2a.peers[].forward_presented_identity`.

  Each takes `none` (the default), `exchange` (call the hook) or `raw` (owner
  accepts passthrough risk, with a warning at startup).
- **The exchange hook.** Modelled on RFC 8693 token exchange: subject token,
  audience, scope in; an audience-bound token out. It runs per call.
- **Continuations and background runs.** Teams, schedules and action
  continuations use the token of the request that resumes them. A run without a
  live token forwards nothing and fails a forwarding destination with an
  explicit `identity_unavailable`, never a silent call.

### P6. Authority replacement (T6; G5, G9; R3) — owners: uar-trust-tools and uar-runtime

| Feature | Replacement |
|---|---|
| Effective settings, observers, reconciliation | `X-UAR-Admin-Key` (exists) or the sidecar launch principal |
| `HostAuthority` | launch-bound sidecar host only; never from a presented token |
| A2A `uar_instance_id` delegation | peer configuration only |
| Shared provider credentials | available only in `validation_hook` mode |
| `skill_required_roles` | removed; skill scope policy remains the gate |
| API-key role delegation | depends on O2 |

All role checks are removed only after their replacement is in place, within
the same change.

### P7. Local identity proxy (T7; G8; R5) — owner: uar-lead (tools)

- **Header selector.** `tools/uar-jwt-proxy` reads a non-secret
  `x-uar-test-subject` header to choose among configured subjects (default:
  today's `PROXY_JWT_SUB`).
- **Unsigned tokens.** In `trusted_front` mode it sends an **unsigned** token
  (`alg:none` payload), so it no longer reads UAR's secret.
- **Stable identity.** It keeps emitting today's `iss`, `sub` and tenant, so
  existing scenarios keep their identity (R5).
- **gRPC.** It adds an h2c listener that forwards gRPC with the same identity
  injection, so I15-I18 need no credentials.
- **Local only.** The selector header is honored by the local proxy only.

### P8. Specifications and documentation — owner: uar-lead

- **OpenSpec changes.** One per P1-P7, each with spec deltas. Capability
  candidates: a new `identity-trust-declaration`, a new
  `presented-identity-custody`, a modified `owner-partitioning`, and a modified
  `local-identity-fixture`. Exact names are settled in the Spec stage against
  `openspec/specs/`.
- **Documents.** Update `docs/operations/principal-provider-credentials.md` and
  the security and identity docs. Add a deployment page with the three modes
  and the trusted-front requirement.
- **Superseding rules.** The CLAUDE.md rule that "team APIs require a verified
  owner" changes meaning. Record the supersession in `.prometheus/decisions.md`;
  do not rewrite the earlier entries.

## Dependencies

- P1 → P2 → P3 → P4.
- P5 and P6 depend on P2.
- P7 depends on P1 and P2, for the unsigned-token acceptance.
- P8 runs alongside, closing with each change.
- No change depends on the A2UI architecture work.

## Verification at the gate (run only after P1-P8 are implemented and the operator approves)

1. One build for the owned profile:
   `cargo build --locked --no-default-features --features server-full`.
   Then rebuild the proxy (`tools/uar-jwt-proxy`).
2. `openspec validate` for each identity change.
3. Restart the owned fixture in `trusted_front` mode, through the new proxy.
4. BDD behaviors, each run once:
   - **New identity scenarios.**
     - Two subjects isolated on one node, using the selector header. This is
       catalog E09.
     - The same `sub` under two issuers stays separate.
     - An unsafe configuration (multi-user, no validator, public bind) refuses
       to start.
     - Under `validation_hook`, a forged token is refused.
     - Forwarding: none by default; `exchange` calls the hook; a destination
       that requires forwarding fails with `identity_unavailable` on a resumed
       run that has no live token.
     - Legacy rows remain readable after the key change (P4 receipt).
   - **Regression.** One earlier owner-scoped scenario per family (C03, C07,
     C39, C05, C36) to show existing identity is stable (R5).
5. gRPC I15-I18, through the proxy's gRPC listener.

## Open decisions (operator)

| Id | Question | Recommended |
|---|---|---|
| O1 | Data mapping: lazy fallback plus an optional one-shot migration, or a mandatory migration at upgrade? | Lazy fallback plus an optional migration with a receipt |
| O2 | API keys: retire them (conflict with T1), keep them as a built-in validation hook, or keep them unchanged? | Keep them as a built-in validation hook (`validation_hook{builtin_api_key}`), so UAR issues no other credentials |
| O3 | External validation hook in server mode: an HTTP callback, an in-process plugin, or both? | HTTP callback first; in-process plugin for embedded hosts |
| O4 | Where `tenant` comes from in `trusted_front`: the token claim, a header, or not used? | The token claim, optional |
| O5 | `raw` forwarding: allow it at all? | Allow it only with an explicit owner setting and a startup warning |
| O6 | The deprecation window for `jwt_required` and the remote profile | One release |

## Out of scope

- Any identity provider, login or role system inside UAR.
- A2UI surface work, which waits on PEM #70-#74.
- San Saba's A2A task tenant partition. Re-check it after P3, because it may
  follow from the new key.

---

## Revision 1 independent review — 2026-10-28

A fresh-context `artifact-critic` reviewed `assessment.md` and this plan.
Verdict: **defects found — 4 critical, 8 warnings, 3 suggestions.** About 60
citations were checked and found correct. The MCP quote was not re-fetched by
the critic.

The lead re-verified two of the findings:

- `CredentialPrincipal::from_verified_context` fails without a verified
  identity (`claims.rs:199-232`).
- A schedule grant restores only when `principal.storage_key() == owner_key`
  (`claims.rs:321`).

### Critical

- **C1. P2 never says what an unverified decode becomes.** Key A requires
  `verified_identity()`. If the identity is filled from an unsigned payload,
  forged `roles`, `uar_credential_kind`, `uar_api_key_id` and
  `uar_instance_id` reach every gate that trusts "verified". If it is not
  filled, every Key A consumer returns 401. The plan needs a distinct
  `PresentedIdentity` type that cannot become a `VerifiedIdentity`, with
  those claims stripped.
- **C2. `single_user` bypasses the refusal.** P1 refuses only for
  `trusted_front`. A `single_user` node on 0.0.0.0 receiving tokens with
  different `sub` values is the forbidden case. Presented tokens must be
  ignored in `single_user`, and a public `single_user` node needs a warning
  or a refusal.
- **C3. The lazy fallback hides data and breaks persisted grants.**
  - Once any v2 row exists, list queries stop falling back, so legacy
    sessions, KBs and memory vanish from lists.
  - Schedule and follow-up grants (`claims.rs:321`) and persisted observers
    (`messages.rs:83-85`) compare the old key and fail.
  - `principal_owner_mappings` (`credentials/mod.rs:163`) stores v1 keys.
  - Memory dedupe IDs hash `owner_key()` (`memory/owned.rs:135`).
- **C4. The mapping omits key families.**
  - `local-host` (sidecar and The Boss data, `claims.rs:222-228`), with no v2
    issuer.
  - Tenantless Key A `…:local:s:…` (`claims.rs:238`) and Key B `v1:s:`
    (`messages.rs:142`, `user_settings.rs:150`).

### Warnings (summary)

- **W1. Identity stability for existing scenarios is unverified (R5).**
  - The proxy sends `iss` only when `jwt_issuer` is configured, and
    `<declared-local-issuer>` is undefined.
  - Admin-gated scenarios would need the secret `X-UAR-Admin-Key` after P6.
- **W2. The P1 bind check is bypassable or boilerplate.**
  - Bypassable: hostnames, injected listeners, same-host proxies and tunnels,
    and DNS rebinding outside sidecar mode.
  - Boilerplate: "private" networks, and routine attestation in containers.
  - It should use the governance bound-address registry
    (`governance/runtime_control.rs:466-472`). The proxy listen address can be
    overridden by an environment variable.
- **W3. P1's migration is incomplete.** `jwt_required` consumers are missing
  (`server.rs:493`, `a2a/handler.rs:87`, `grpc.rs:72`,
  `settings/manager.rs:1403-1425`, `bin/uar-sidecar.rs`). Mapping
  `false`+loopback to `trusted_front` makes nodes that verify bearers today
  accept `alg:none`.
- **W4. P3 is under-scoped.** The key builders appear in 27 files, with 48
  `session_owner_key()` calls and 135 `.storage_key()` calls.
- **W5. P5 can still leak tokens.**
  - Exchanged tokens are not scrubbed.
  - The exchange callback is itself a forwarding destination with no TLS
    requirement.
  - Run context is undefined for durable A2A sends and team children.
  - Scheduled runs lose `exchange` destinations.
- **W6. P6 has holes.**
  - The admin key is skipped when `settings_mutation_auth_required=false`.
  - Inbound peer authentication contradicts T1.
  - Delegation contracts match `sub` only (`routes.rs:1731`,
    `thread_service.rs:1381`).
  - Unaddressed: `workspace_authorities`, `api_key_admin_principals`,
    `trusted_host_principals`, sidecar delegation grants, the
    `issuer_kind`-dependent credential scope, and `api_keys/policy.rs:140`.
- **W7. O2 is ill-posed.** `validation_hook{kind}` is single-valued. UAR still
  issues API-key JWTs and delegation grants, so "UAR issues no other
  credentials" is false.
- **W8. The gate is not well-posed.**
  - Commands are not exact.
  - "Forged token refused" needs a test-sent JWT, which the 2026-10-26 rule
    forbids.
  - "Same `sub`, two issuers" cannot run, because the selector picks `sub`
    only.
  - There are no exchange or forwarding fixtures, and no sidecar, Boss,
    schedule or list-after-write scenario.
  - Whether `pnpm build:mac:arm64` applies is undetermined.
  - The dependency list omits P7 → P3/P4.

### Suggestions

- A distinct `PresentedIdentity` type.
- Run the P1 check on actual bound addresses.
- Restrict the proxy to loopback.

### Lead assessment

The findings fall into two groups.

**Design gaps the plan must decide:**

- the identity type boundary (C1);
- `single_user` semantics (C2);
- a migration that rewrites keys everywhere they are compared, not a read
  fallback (C3, C4);
- how the trust declaration is enforced against real bound sockets (W2).

**Scope corrections:** W3-W8.

None of this contradicts the operator's decision. It shows that the key change
touches persisted equality checks beyond the data rows, and that "no
validation" needs a hard type boundary to stay safe.

**Readiness: BLOCKED.** A revision 2 is needed before Spec.
