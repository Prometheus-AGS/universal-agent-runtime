# Plan revision 2 (superseded 2026-10-28 by plan.md revision 3) — identity-pass-through

KBD Plan, 2026-10-28. Planning only. Nothing is implemented, and the
verification gate stays CLOSED until P1-P9 are implemented and the operator
approves the gate commands.

Revision 1 and its review are in `plan-rev1.md`. This plan uses the targets
T1-T7, gaps G1-G10 and risks R1-R5 from `assessment.md`, including its
addendum.

The work is done by the `uar-core` team: one writer per file, with the
verification lock in every packet. Each change becomes one OpenSpec change
with at least one spec delta.

## The central idea of revision 2: keep the key format, change only the input

Revision 1 tried to rewrite every owner key. The review showed that this would
break stored equality checks in at least twelve places. Revision 2 keeps every
existing key format and the `storage_key()` function exactly as they are. It
changes only where the principal's fields come from, and it separates the
**key** from **authority**:

- **The key** is derived from a `PresentedIdentity` or a `VerifiedIdentity`,
  through rules that reproduce today's key for every existing user (§P3).
- **Authority** (roles, host authority, API-key markers, instance delegation,
  shared provider credentials, workspace and administrator allowlists) is
  derived **only** from a `VerifiedIdentity`, the sidecar launch token, or the
  admin key. A `PresentedIdentity` never grants authority (§P2).

So no stored row is rewritten when the primary issuer is unchanged. That
removes revision 1's C3 and C4, the lazy fallback and its blind spots.

## Changes, in order

### P1. Trust declaration, enforced on bound sockets (T2; G1; W2, W3) — owner: uar-trust-tools

**Setting:** `security.identity`.

- **`mode`**, one of:
  - `validation_hook`;
  - `trusted_front { upstream: <name> }`;
  - `single_user { owner_sub: <sub>, owner_iss?: <iss> }`.
- **`validators`**, an ordered list, used by `validation_hook`:
  - `{ builtin_shared_secret }`;
  - `{ builtin_jwks }`;
  - `{ builtin_api_key }`;
  - `{ external_http { url } }`, which must be https or loopback.
- **`optional: bool`.** When true, a request without a credential is
  anonymous. This is today's `jwt_required=false` behaviour.
- **`primary_issuer`.** Defaults to today's `security.jwt_issuer`, otherwise
  `urn:uar:local:jwt`.
- **`public_bind_attested { upstream, reason }`**, optional.

**Existing configs map unchanged (W3).**

| Today | Maps to | Behaviour |
|---|---|---|
| `jwt_required=true` | `validation_hook` with the current verifier, `optional=false` | unchanged |
| `jwt_required=false` | `validation_hook`, `optional=true` | unchanged: a presented token is still verified, and an invalid one becomes anonymous |

`trusted_front` and `single_user` are explicit choices only. No existing node
starts accepting `alg:none` tokens.

These `jwt_required` consumers move to the declaration:

- `config.rs:731, 802-841`;
- `server.rs:493`;
- `a2a/handler.rs:87`;
- `a2a/grpc.rs:72`;
- `settings/manager.rs:1403-1425`;
- `bin/uar-sidecar.rs:20, 145-148`;
- the governance `JwtRequired` reason (`governance/runtime_control.rs:461-463`).

The old keys stay readable, with a deprecation warning, for one release (O6).

**Enforcement against actual sockets (W2).**

1. After every listener binds (HTTP, gRPC and injected listeners), read the
   governance bound-ingress registrations
   (`governance/runtime_control.rs:455-480`). Hostnames are never judged.
2. `trusted_front` and `single_user` require every registered address to be
   loopback (`127.0.0.0/8`, `::1`), unless `public_bind_attested` is present.
   A private network is not treated as safe.
3. Without the attestation, the node refuses to serve.
4. With it, the node logs a startup warning naming the upstream.
5. `validation_hook` has no bind restriction.
6. In `trusted_front` and `single_user` on loopback, the Host header must be
   `localhost`, `127.0.0.1`, `[::1]` or a configured name. This reuses
   `sidecar_guard.rs:285-290` and blocks DNS rebinding.

The declaration can confirm locality. It cannot confirm that a validating
upstream exists, so for production `validation_hook` is the recommended mode.
The documentation says this.

**Files:**

- `src/config.rs` and `src/config_manager.rs`;
- `src/uar/governance/runtime_control.rs`;
- `src/server.rs` (bind and serve);
- `src/uar/security/sidecar_guard.rs` (the Host check, reused);
- `bin/uar-sidecar.rs`;
- `docs/operations/identity-modes.md` (new).

### P2. Two identity types and one intake (T1; G2, G4; C1, C2) — owner: uar-trust-tools

**The two types.**

- **`PresentedIdentity { iss: Option<String>, sub: String, tenant: Option<String> }`**
  - It is a distinct type in `security/presented.rs`.
  - It has no conversion into `VerifiedIdentity`, no role field, and no
    credential-kind, API-key or instance markers.
  - It is built by decoding the JWT payload without verification. Every other
    claim is dropped at decode time.
  - A malformed token, or one missing `sub`, is refused with 400.
- **`UserContext`** gains `presented: Option<PresentedIdentity>`.
  - `verified_identity()` stays `Some` only for `validation_hook` successes.
  - The sidecar launch principal is unchanged (`middleware.rs:39-83`).

**One intake function**, `intake(mode, headers | grpc metadata) -> UserContext`.
It replaces the per-interface paths in:

- `middleware.rs:195-310`;
- `a2a/grpc.rs:51-140` and `grpc_v1.rs:74`, where a bearer is no longer
  mandatory;
- ACP, A2A HTTP and `mcp_server.rs:71-87`.

What it does in each mode:

| Mode | Behaviour |
|---|---|
| `validation_hook` | Run validators in order. The first success gives a `VerifiedIdentity`. All failures give 401, or anonymous when `optional`. |
| `trusted_front` | Decode only. `presented = Some`, `verified = None`. A missing token is anonymous. |
| `single_user` | Tokens are **never** decoded for identity. Every request is the configured owner, as `presented`. A token is kept only for P5 custody (C2). |

**Authority reads require `VerifiedIdentity` (C1).** Each of these sites
changes to require it:

- roles (`effective_settings.rs:43`, `observers.rs:251-254`,
  `channel_observers.rs:150-151`, `agent_instances.rs:330-333`,
  `api_keys/policy.rs:82-87, 140`, `manager.rs:1709-1710`);
- `HostAuthority` (`claims.rs:121-130`);
- `uar_credential_kind` and `uar_api_key_id` (`verifier/mod.rs:53-61, 162`);
- `uar_instance_id` (`grpc.rs:122`, `a2a/handler.rs:97-99`, `a2a/v1.rs:214`,
  `routes.rs:1733`, `thread_service.rs:1382, 1437`, `team_execution.rs:261`);
- shared provider credentials (`credentials/policy.rs:111, 136`,
  `credentials/owner.rs:15-33`);
- `workspace_authorities` (`authority.rs:44-49`);
- `api_key_admin_principals` and `trusted_host_principals`
  (`config.rs:806-815`).

Outside `validation_hook`, the allowlist settings are ignored, with a startup
warning naming each one.

**A compile-time boundary.** Authority helpers take `&VerifiedIdentity`, never
`&UserContext`. Forgetting the check is then a type error, not a runtime
mistake.

### P3. Principal derivation that preserves keys (T3; G3; C3, C4, W1, W4) — owners: uar-trust-tools and uar-state

A new function, `CredentialPrincipal::for_owner(&UserContext, &IdentityConfig)`,
replaces `from_verified_context` for **ownership** uses only:

- **Verified identity:** exactly today's rule (`claims.rs:199-232`).
- **Presented identity:**
  1. `effective_iss = presented.iss.unwrap_or(primary_issuer)`.
  2. If `effective_iss == primary_issuer`, build today's principal for this
     node: kind `issuer` with the configured issuer when `jwt_issuer` is set,
     otherwise kind `local-jwt` with `urn:uar:local:jwt`. The tenant is
     unchanged, so the tenantless `local` forms carry over. **Keys are
     identical to today's**, so stored equality checks hold:
     - schedules and follow-ups (`claims.rs:321`);
     - memory (`memory/owned.rs:124`);
     - credential mappings (`principal_store.rs:151`);
     - cost budgets (`cost_budget.rs:951`);
     - approvals (`approval_decisions.rs:90`);
     - and every other §addendum site.
  3. Otherwise, build a new kind `presented` with `effective_iss`. These are
     new users, so no legacy rows exist for them.
  - The kind `issuer` produced by step 2 carries **no authority**, because
    authority reads `VerifiedIdentity` (P2).
- **Sidecar launch (`local-host`):** unchanged (`claims.rs:222-228`).

**Key B gains the issuer only for non-primary issuers.**
`presentation_owner_key` (`messages.rs:133-145`) and `principal_storage_key`
(`user_settings.rs:138-153`):

- keep today's `v1:t:…:s:` and `v1:s:` forms when `effective_iss ==
  primary_issuer`;
- emit `v2:i:{len}:{iss}:t:…|-:s:…` otherwise.

A primary user's settings, teams and observers keep their keys, and same-`sub`
users from different issuers no longer collide. Delegation contracts that
match `sub` only (`routes.rs:1731`, `thread_service.rs:1381`) switch to Key B.

**Scope (W4).** The 76 files in the assessment addendum form an audit
checklist. Each is classified as one of:

- "key only": no change, because the key function is unchanged;
- "authority": changes in P2;
- "derivation": changes here.

The classification is recorded in the change's `tasks.md`. Owners:

- uar-state: persistence and memory;
- uar-runtime: runtime and team execution;
- uar-trust-tools: security and API.

### P4. Primary-issuer change tool (T5; replaces revision 1's lazy fallback) — owner: uar-state

With P3, no migration is needed while `primary_issuer` stays the same. That is
the default, and the upgrade path for every existing node.

Changing `primary_issuer` on a node with data is the one case that needs a
rewrite. The tool is `uar identity rekey --from <iss> --to <iss>`:

- offline, with UAR stopped, against SurrealDB and PostgreSQL;
- `--dry-run` by default;
- it rewrites every stored owner key and every stored principal field in one
  transaction per store, with a receipt (rows per table, before and after);
- it never deletes a row.

At startup, UAR refuses to start if `primary_issuer` differs from the last
recorded value and no rekey receipt exists. The recorded value is a new
setting.

### P5. Token custody and opt-in forwarding (T4; G6; W5) — owners: uar-trust-tools and uar-runtime

**Custody.**

- `PresentedToken` lives in the run context for the **originating request's
  lifetime**.
- It is never serialized: no `Serialize`, and its Debug is redacted.
- The scrubber captures it (`credential_capture.rs:11-20`).

**Forwarding, opt-in per destination.**

- Settings: `mcp.servers[].forward_identity` and
  `a2a.peers[].forward_identity`.
- Values:
  - `none`, the default;
  - `exchange`, which calls the exchange hook;
  - `raw`, which needs an explicit owner setting, logs a startup warning,
    and is O5.

**Exchange hook.**

- Modelled on RFC 8693: subject token, audience and scope in; an
  audience-bound token out.
- Its endpoint is https or loopback only.
- **Every exchanged token is registered with the scrubber before use (W5).**

**Lifetime.**

- Team child runs and durable A2A sends started **within** the request inherit
  the custody handle while it is live.
- After the originating request ends, the handle is dropped. A forwarding
  destination then fails with `identity_unavailable`, never a silent
  credential-less call.
- Scheduled runs, and continuations started later, therefore cannot reach
  `exchange` or `raw` destinations. A continuation uses its own resuming
  request's token. This is a stated limit, not a defect, unless the owner hook
  issues a service identity (out of scope).

### P6. Authority replacements (T6; G5, G9; W6, W7) — owners: uar-trust-tools and uar-runtime

| Feature | In `validation_hook` | In `trusted_front` and `single_user` |
|---|---|---|
| Effective settings, observers, reconciliation | today's roles from `VerifiedIdentity` | `X-UAR-Admin-Key`, subject to `settings_mutation_auth_required` (as settings mutation is today, `api/settings.rs:396-404`), or the sidecar launch principal |
| `HostAuthority` | `trusted_host_principals` via `VerifiedIdentity`, or the sidecar launch | sidecar launch only |
| Governed A2A delegation (`uar_instance_id`) | verified peer token | **disabled**, with an explicit error: inbound peers cannot be authenticated without a validator |
| Shared provider credentials | verified issuer, tenant and subject (today) | **disabled** |
| `skill_required_roles` | today's check | not applicable. Skills that require roles are unavailable. Skill scope policy still applies. |
| API keys | `builtin_api_key` validator (O2) | not available |
| Sidecar delegation grants (`sidecar_guard.rs:303-`) | unchanged; these are host credentials, not user identity | unchanged |

**T1 is refined honestly (W7).** UAR issues no **user** identity. It still
issues host credentials (the sidecar launch token and delegation grants) and,
optionally, API-key JWTs when `builtin_api_key` is configured.

### P7. Local identity proxy (T7; G8; R5; W1) — owner: uar-lead (tools)

`tools/uar-jwt-proxy`:

- **Loopback only.** It refuses to listen on a non-loopback address, unless
  `PROXY_ALLOW_NON_LOOPBACK=1`, which tests never set.
- **Identity profiles.** It reads a non-secret `x-uar-test-identity: <profile>`
  header. Profiles live in the proxy's own config, each as `{ iss, sub,
  tenant, signing }`:
  - `signing=unsigned` for `trusted_front`;
  - `signing=shared_secret` for `validation_hook` fixtures;
  - `signing=forged` for refusal tests.
- **A stable default profile (R5).** It reproduces today's identity exactly:
  `sub=$PROXY_JWT_SUB`, `iss` equal to the configured `jwt_issuer`, the same
  tenant. Existing scenarios keep their keys under both P1 mappings.
- **No roles** in unsigned profiles. The default `admin,user` remains only for
  `shared_secret` profiles, which the current fixture uses unchanged.
- **gRPC.** An h2c listener injects the same profile into gRPC metadata, for
  I15-I18.

Tests send only the profile name. The proxy, as external infrastructure,
holds any secret.

### P8. Specifications and documentation — owner: uar-lead

- **OpenSpec changes, one per P1-P7.** Candidate capabilities:
  - `identity-trust-declaration`;
  - `presented-identity`;
  - `owner-key-derivation`;
  - `identity-rekey`;
  - `presented-identity-custody`;
  - `identity-authority`;
  - `local-identity-fixture`.

  Names are settled in Spec against `openspec/specs/`.
- **Documents.** Update:
  - `docs/operations/principal-provider-credentials.md`;
  - the security docs;
  - the new identity-modes page.
- **Decisions.** Record in `.prometheus/decisions.md` that "team APIs require
  a verified owner" now means a verified **or presented** owner. Do not
  rewrite the earlier entries.

### P9. Behavior scenarios (authored now, run only at the gate) — owner: uar-lead with the scenario authors

Each scenario gets its own feature file, steps and cucumber config under
`scripts/behaviors/`. They use only the proxy's profile header; no credentials.

| Scenario | Instance | Proves |
|---|---|---|
| `identity-two-subjects` (catalog E09) | main fixture switched to `trusted_front` | two subjects isolated: runs, events, cancel, sessions; a restart keeps recall |
| `identity-two-issuers` | main | the same `sub` under two issuers stays separate, including settings and teams (Key B `v2`) |
| `identity-legacy-continuity` | main | data created before the switch (a session, KB, memory, schedule, follow-up and observer) is listed and usable after switching to `trusted_front`. It writes new data first, then lists, which covers revision 1's C3. |
| `identity-unsafe-bind-refused` | disposable (c52 fixture) | `trusted_front` with a non-loopback registration and no attestation refuses to serve. `single_user` does the same. |
| `identity-forged-refused` | disposable, `validation_hook` | the `forged` profile is refused, and `shared_secret` succeeds |
| `identity-no-authority-from-presented` | main, `trusted_front` | a presented token carrying `roles:["admin"]` and `uar_credential_kind:"issuer"` (proxy profile) gets no admin view, no host authority and no shared credentials |
| `identity-forwarding` | main | `none` sends nothing; `exchange` calls a recording exchange endpoint and a recording MCP peer receives only the exchanged token; after the request ends, a resumed run fails with `identity_unavailable`; no token appears in events, history or logs |
| `identity-sidecar-host` | sidecar launch | `local-host` principal data and host features are unchanged |

**Regression:** one earlier owner-scoped scenario per family — C03, C07, C39,
C05, C36 — rerun once after the switch.

## Dependencies

- P1 → P2 → P3.
- P4 depends on P3.
- P5 and P6 depend on P2.
- P7 depends on P1, P2 and P3, because profiles must reproduce today's key.
- P9 depends on P1-P7.
- P8 runs alongside, closing with each change.
- Nothing here depends on the A2UI work.

## Gate (exact commands, after P1-P9 and operator approval, run serially)

1. `cargo build --locked --no-default-features --features server-full`
2. `cargo build --locked --manifest-path tools/uar-jwt-proxy/Cargo.toml`
3. For each identity change:
   `node <kbd-orchestrator>/shared/openspec/cli.mjs run --project "$PWD" -- validate <change-name>`
4. Restart the owned fixture and proxy with `security.identity.mode=trusted_front`
   (fixture command recorded in P9's support files).
5. For each P9 scenario, then each regression scenario:
   `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/<name>.cucumber.mjs`
6. **The Boss:** `pnpm build:mac:arm64` is required by project rule when the
   phase includes The Boss. P1 changes `bin/uar-sidecar.rs`. The script is
   **not defined** in this repository's root or `frontend/package.json`, so
   its location and owner must be confirmed (O7) before the gate is approved.

## Open decisions (operator)

| Id | Question | Recommended |
|---|---|---|
| O1 | Replaced by P3/P4. The only migration is the explicit rekey tool. Accept? | Yes |
| O2 | Keep API keys as an optional `builtin_api_key` validator, or retire them? | Keep as an optional validator |
| O3 | External validator: HTTP callback in server mode, and an in-process trait for embedded hosts? | Yes, both |
| O4 | Tenant in `trusted_front`: from the token claim, optional? | Yes |
| O5 | Allow `raw` forwarding at all? | Only with an explicit owner setting and a startup warning |
| O6 | Deprecation window for `jwt_required` | One release |
| O7 | Where does `pnpm build:mac:arm64` live, and who owns The Boss's gate? | Operator to name |
| O8 | Disable governed A2A delegation and shared provider credentials outside `validation_hook`? | Yes; no unauthenticated substitute exists |

## Out of scope

- An identity provider, logins, or a role system inside UAR.
- The A2UI surface work.
- San Saba's A2A task tenant partition. Re-check it after P3.
- Service identities for scheduled runs.

---

## Revision 2 independent review — 2026-10-28

A fresh-context `artifact-critic` reviewed this plan. Verdict: **defects found
— 3 critical, 10 warnings, 6 suggestions.** It checked about 45 citations and
found them correct.

The critic reports that one fixture YAML read put raw values into its own
working context. None are repeated in its report.

The lead re-verified these three claims:

- `CredentialOwnerBinding::bind` returns `IdentityRequired` for every kind
  except `issuer`, `local-jwt` and `local-host` (`credentials/owner.rs:13-23`).
- Verified identities take their issuer from `config.jwt_issuer`, not from the
  token (`verifier/mod.rs:331`).
- The Boss's local UAR payload loader refuses anything but darwin-arm64
  (`the-boss/scripts/local-uar-payload.cjs:42`).

### Critical

- **C1. The new `presented` kind breaks for non-primary-issuer users.**
  - **Model runs.** Credential binding rejects the kind (`owner.rs:15-22`;
    `policy.rs:133-138`; `manager.rs:887`; `server.rs:4948`).
  - **Schedules and follow-ups.** `ScheduleOwnerGrant::restore` rejects it
    (`claims.rs:302-307`).
  - **Observers.** They rebuild the owner without an issuer
    (`observer/delivery.rs:189`; `messages.rs:71-90`). Storing the issuer
    needs a schema change, so "no legacy rows" does not mean "no new fields".
  - **Coverage.** `identity-two-issuers` covers none of this.
- **C2. `(iss, sub)` is not enforced in `validation_hook` mode.**
  - Verified identities use the configured issuer
    (`verifier/mod.rs:331`). With several validators, same-`sub` users from
    two issuers collide.
  - `HostAuthority::for_service` (`claims.rs:121-129`) could match an
    issuer-B token against `trusted_host_principals`.
  - What the external validator must return is not defined.
- **C3. P6 conflicts with P9.** P6 gates admin features with the admin key
  only when `settings_mutation_auth_required=true`.
  - The fixture has that flag set to false, so in `trusted_front` every caller
    gets the admin view and `identity-no-authority-from-presented` fails.
  - It also widens access that `effective_settings.rs:36-37` deliberately
    refuses today.

### Warnings (summary)

- **W1. Authority is read from the principal.** "Kind `issuer` carries no
  authority" is false. Credential policy reads `CredentialPrincipal.issuer_kind()`
  and `issuer()` (`policy.rs:111, 121-125`; `owner.rs:15-16`) inside the run
  kernel, where no `VerifiedIdentity` exists. A verified bit changes
  `CredentialPrincipal` equality and `ScheduleOwnerGrant` serialization.
- **W2. Sidecar `host-session` access breaks.** `host-session` lives only in
  `claims.roles` (`middleware.rs:69-81`). Helpers that take only
  `&VerifiedIdentity` would drop sidecar and admin-key access.
- **W3. Nodes without `jwt_issuer` split their data.** Their keys are
  `local-jwt` today, and any IdP token with an `iss` would get a new key.
  `primary_issuer` and `jwt_issuer` can also disagree, and `rekey` semantics
  are then ill-posed.
- **W4. Assertable primary identity is not stated.** In `trusted_front`,
  omitting `iss`, or setting it to the primary issuer, makes the caller the
  existing user. On loopback that means any local process. The plan should
  say so plainly.
- **W5. P5's lifetime contradicts its scenario.** Dropping custody when the
  request ends breaks long runs partway through. `identity-forwarding`
  expects a resumed run to fail, while P5 says it uses its own token.
- **W6. P7 profiles can't express P9's test.** They cannot carry the forged
  claims `identity-no-authority-from-presented` needs, and "no host
  authority" passes trivially once allowlists are ignored.
- **W7. Scenario gaps.**
  - `identity-legacy-continuity` has no pre-switch data step.
  - Nothing tests the default upgrade mapping.
  - Nothing tests the rekey tool.
- **W8. Intake misses sites.** Missed: `memory_admin.rs:440`
  (`owner_bearer` in the body, `middleware.rs:148-158`),
  `api_keys/policy.rs:98, 121` and `middleware.rs:312`. Also,
  `VerifiedIdentity::for_scoped_delegation` (`claims.rs:72-81`) already
  converts a context into a verified identity.
- **W9. Bind enforcement has gaps.** The registry covers `primary-http`,
  `companion-http` and `a2a-grpc` (`server.rs:509, 525, 552`). Embedded hosts
  (G10) are not closed, and gRPC `:authority` is not checked.
- **W10. Gate commands are not exact.** They include a placeholder, and the
  restart and fixture launch commands are missing.

### Suggestions

- The sidecar path is `src/bin/uar-sidecar.rs`.
- `jwt_required` consumers are also at `config.rs:31, 850, 1634, 1735`.
- Store the recorded primary issuer in the database, and state the recovery
  path after a partial rekey.
- State that intake stores the derived principal.
- Say how token-absence checks learn which values to look for.

### The Boss facts (operator, 2026-10-28)

- **Repository:** `/Users/gqadonis/Projects/prometheus/the-boss` (HEAD
  `8e2160c21`).
- **This Mac is Intel** (x86_64, Xeon W-3235). The operator builds The Boss
  here with the Intel script.
- **x64 scripts:**
  - `build:mac:x64` sets `THE_BOSS_UAR_ENABLED=0`, so it has no UAR.
  - `build:mac:x64:release:uar` enables UAR with `THE_BOSS_UAR_LOCAL=0`, so
    it packages a released payload.
- **There is no x64 `:local` variant.** The local payload loader refuses
  non-arm64 (`local-uar-payload.cjs:42`). On this Mac, no Boss build packages
  the changed UAR sidecar.
- **The managed sidecar uses `host-asserted`.** The launch token is sent as
  the bearer, with `x-uar-principal` (`UarSidecarService.ts:672-675, 859`).
  P2 leaves that path unchanged.
- **External instances use `token-subject`** with a configured runtime bearer
  (`:1133, :1180`).
- **The Boss reads `authentication.principalMode`** from UAR capabilities
  (`capabilities.rs:322-336`), so it must stay stable.
- **Gate consequence (O7).** Validating the identity change inside The Boss on
  this Mac needs an x64 local-payload path in The Boss, or the Boss check
  moves to an arm64 machine. `build:mac:x64:release:uar` alone proves only
  that The Boss builds against a released UAR.

### Lead assessment

Two consecutive reviews found critical defects, and they share a root cause.
`CredentialPrincipal` is both the **ownership key** and an **authority input**:
credential policy reads its issuer kind. Splitting "key from presented,
authority from verified" therefore cannot be done at the type level without
changing the type whose equality every stored key depends on.

A revision 3 would need to:

1. Change `CredentialPrincipal` explicitly, with an authority source that
   takes part neither in equality nor in key derivation. Accept a versioned
   serialization for `ScheduleOwnerGrant` and observers.
2. Carry the token's `iss` in verified identities, accepting new keys for
   non-primary issuers on multi-validator nodes.
3. Give every consumer that validates kinds (credentials, schedules,
   observers) a defined rule for non-primary issuers.
4. Gate admin features independently of `settings_mutation_auth_required`.
5. Build fixtures for multi-issuer, no-`jwt_issuer` and upgrade paths, so the
   gate does not run only where the claim holds.

**Readiness: BLOCKED.**
