# Plan revision 3 (superseded 2026-10-28 by plan.md revision 4) — identity-pass-through

KBD Plan, 2026-10-28. Planning only. Nothing is implemented. The verification
gate stays CLOSED until P1-P10 are implemented and the operator approves the
gate commands.

Revisions 1 and 2, with their reviews, are in `plan-rev1.md` and
`plan-rev2.md`. This plan uses T1-T7, G1-G10 and R1-R5 from `assessment.md`,
including its addendum. Work is done by the `uar-core` team, one writer per
file, with the verification lock in every packet. Each change becomes one
OpenSpec change with at least one spec delta.

## What revision 3 changes

Revision 2 failed because `CredentialPrincipal` is both the ownership key and
an authority input: credential policy reads its `issuer_kind`. Revision 3
separates the two explicitly, following a pattern UAR already uses.
`ActorOwner` keeps `verified_roles` outside equality and hashing, with the
comment "Fresh host-captured authority is separate from stable ownership
identity" (`runtime/actor/messages.rs:12-31`).

1. **`CredentialPrincipal` becomes ownership only.** Its fields, kinds,
   `Eq`/`Hash` and `storage_key()` stay exactly as today, and nothing reads it
   to decide authority any more. **No new kind is introduced.** Revision 2's
   `presented` kind is dropped, because credentials, schedules and keys reject
   unknown kinds (`credentials/owner.rs:15-22`, `claims.rs:302-314`).
2. **A new `PrincipalAuthority` carries authority.** It is held beside the
   principal in `ActorOwner` (outside `PartialEq`/`Hash`, like
   `verified_roles`) and in the run context. It is excluded from every key.
3. **Every token-derived principal uses the token's own `iss`**, in kind
   `issuer`, including verified tokens (fixes revision 2's C2). The single
   exception is the **primary alias** (§P3), which maps one issuer onto the
   node's legacy principal so existing rows keep their keys.

## Changes, in order

### P1. Trust declaration on bound sockets (T2; G1)

**Owner:** uar-trust-tools.

**Setting:** `security.identity`.

- **`mode`**, one of:
  - `validation_hook`;
  - `trusted_front { upstream }`;
  - `single_user { owner_sub, owner_iss? }`.
- **`validators`** (an ordered list, used by `validation_hook`):
  - `builtin_shared_secret { issuer }`;
  - `builtin_jwks { issuer, jwks_url, audience? }`;
  - `builtin_api_key`;
  - `external_http { url }`, which must be https or loopback. It returns
    `{iss, sub, tenant?, roles?}` for a token, or refuses it. A `roles` value
    is trusted only from this validator.
- **`optional`**: when true, a request with no credential is anonymous.
- **`primary_alias { issuer }`**, optional (§P3).
- **`public_bind_attested { upstream, reason }`**, optional.

**Existing configs keep today's behavior.**

| Today | Maps to |
|---|---|
| `jwt_required=true` | `validation_hook` with today's verifier settings, as one `builtin_shared_secret` or `builtin_jwks` validator whose `issuer` is `jwt_issuer`; `optional=false` |
| `jwt_required=false` | the same mapping with `optional=true` |

- When `jwt_issuer` is set, `primary_alias.issuer` defaults to it.
- When it is not set, there is no alias. Today such nodes store `local-jwt`
  keys and never read `iss`. The mapped validator therefore keeps ignoring
  `iss` for those nodes, so their keys don't change.
- `trusted_front` and `single_user` are chosen only explicitly.

**`jwt_required` consumers that change:**

- `config.rs:31, 731, 802-841, 850, 1634, 1735`;
- `server.rs:493`;
- `a2a/handler.rs:87`;
- `a2a/grpc.rs:72`;
- `settings/manager.rs:1403-1425`;
- `src/bin/uar-sidecar.rs:20, 145-148`;
- the governance `JwtRequired` reason (`governance/runtime_control.rs:461-463`).
  `trusted_front` adds a new `IdentityTrustedFront` reason, so governance
  status still shows that identity is not verified.

The old keys stay readable for one release, with a warning (O6).

**Bound-socket enforcement.**

1. After `primary-http`, `companion-http` and `a2a-grpc` register
   (`server.rs:509, 525, 552`), read the registry
   (`runtime_control.rs:455-480`). No other TCP listener exists today. Any
   future listener must register, and a startup assertion fails if a bound
   socket is unregistered.
2. `trusted_front` and `single_user` require every registered address to be
   loopback. Otherwise `public_bind_attested` is required, and its presence
   logs a startup warning naming the upstream.
3. On loopback, HTTP `Host` and gRPC `:authority` must be `localhost`,
   `127.0.0.1`, `[::1]` or a configured name (the `sidecar_guard.rs:285-290`
   logic, extended to the gRPC interceptor).

**Embedded hosts (G10).** They have no sockets and no declaration. The host
supplies the principal and authority through the embedded API as
`PrincipalAuthority::HostAsserted`. This is documented as an implicit fourth
mode.

### P2. Intake and the presented-identity boundary (T1; G2, G4)

**Owner:** uar-trust-tools.

**`PresentedIdentity { iss: Option, sub, tenant: Option }`** (new
`security/presented.rs`):

- It is produced only by an unverified payload decode. All other claims are
  dropped at decode: roles, `uar_credential_kind`, `uar_api_key_id`,
  `uar_instance_id`.
- A malformed token, or one with no `sub`, gets 400.
- `UserContext.claims.roles` is **never** filled from a presented token.
- No conversion exists into `VerifiedIdentity`.
- `VerifiedIdentity::for_scoped_delegation` (`claims.rs:72-81`), which copies
  `claims.roles`, is restricted to host-launch contexts, where intake set the
  roles itself (`middleware.rs:69-81`).

**One `intake(mode, headers | metadata | body-bearer)` replaces:**

- `middleware.rs:148-158, 195-310, 312`;
- `a2a/grpc.rs:51-140`, `grpc_v1.rs:74`;
- ACP, A2A HTTP;
- `mcp_server.rs:71-87`;
- `memory_admin.rs:440`, the body `owner_bearer`, which in non-hook modes is
  read as presented and gives no host authority.

Per mode:

| Mode | Behavior |
|---|---|
| `validation_hook` | Run the validators in order. The first success gives a `VerifiedIdentity` carrying the **token's** `iss` (fixes `verifier/mod.rs:331`). All fail: 401, or anonymous when `optional`. |
| `trusted_front` | Decode only. A missing token is anonymous. |
| `single_user` | Tokens are never decoded. Every request is the configured owner. |
| Sidecar launch | Unchanged (`middleware.rs:39-83`), giving `PrincipalAuthority::HostLaunch`. |

`intake` stores both the derived `CredentialPrincipal` (P3) and the
`PrincipalAuthority` (P4) in `UserContext`. The 48 `session_owner_key()`
callers therefore stay unchanged.

### P3. Ownership derivation with a primary alias (T3; G3)

**Owners:** uar-trust-tools, with uar-state.

`CredentialPrincipal::derive(identity, &IdentityConfig)` handles both verified
and presented identities:

1. **Sidecar launch:** `local-host`, unchanged (`claims.rs:222-228`).
2. **`single_user`:** kind `issuer`, issuer `owner_iss`. With no `owner_iss`,
   kind `local-jwt` (`urn:uar:local:jwt`), which is today's key on an
   unconfigured node.
3. **Primary alias:** a token whose `iss` equals `primary_alias.issuer`, or
   that carries no `iss` while an alias is configured, gets the node's
   **legacy principal**:
   - kind `issuer` with that issuer when today's `jwt_issuer` is set;
   - kind `local-jwt` (`urn:uar:local:jwt`) when the operator set
     `primary_alias` on a node that never had `jwt_issuer`.

   Tenant is unchanged. Existing keys are identical, so the stored equality
   checks still hold (assessment addendum list).
4. **Any other `iss`:** kind `issuer`, issuer = the token's `iss`. This is the
   existing kind. Credential binding (`owner.rs:16`) and schedule restore
   (`claims.rs:311`) already accept it, and Key A already includes the issuer.
5. **No `iss` and no alias:**
   - in `trusted_front`, kind `local-jwt`;
   - in `validation_hook`, only nodes without `jwt_issuer` reach this, so it
     is today's key.

**Key B** is `presentation_owner_key` (`messages.rs:133-145`) and
`principal_storage_key` (`user_settings.rs:138-153`).

- It keeps `v1:t:…:s:` and `v1:s:` for local-host, local-jwt and
  primary-alias principals.
- Any other issuer gets `v2:i:{len}:{iss}:t:…|-:s:…`, so same-`sub` users
  from different issuers stay apart.
- Delegation contracts that match only `sub` (`routes.rs:1731`,
  `thread_service.rs:1381`) move to Key B.

**Stated policy (W4).**

- In `trusted_front`, anyone who can reach the node can claim the primary
  alias by omitting `iss` or sending that issuer.
- On loopback, "anyone" means any local process, not only the front.
- The identity-modes page states both, and recommends `validation_hook` where
  that matters.

**Audit.** The 76 files in the assessment addendum form a checklist recorded
in `tasks.md`, each classified as:

- "key only": no change, because intake stores the principal;
- "authority": P4;
- "derivation": here.

### P4. `PrincipalAuthority`, separate from ownership (T6; G5; fixes revision 2 W1, W2)

**Owners:** uar-trust-tools, with uar-runtime.

```
enum PrincipalAuthority {
    Verified { validator: ValidatorId, roles: BTreeSet<String> }, // from validation_hook only
    HostLaunch,                                                   // sidecar launch token
    AdminKey,                                                     // X-UAR-Admin-Key matched this request
    HostAsserted,                                                 // embedded host
    Presented,                                                    // trusted_front decode
    SingleUser,
    Anonymous,
}
```

**Where it lives.**

- **`ActorOwner`:** stored beside `verified_roles`, and excluded from
  `PartialEq`/`Hash` (`messages.rs:19-31`), so ownership equality is unchanged.
- **The run context:** it is passed from admission into the kernel. The sites
  that today hold only a `CredentialPrincipal` (`manager.rs:875-887`;
  `server.rs:4948`) receive it as an explicit parameter.

**Authority decisions read `PrincipalAuthority`, never `issuer_kind`.**

| Decision | Rule |
|---|---|
| Shared provider credentials (`credentials/policy.rs:111, 121-125`) | `Verified` only |
| Personal credential binding (`credentials/owner.rs:15-33`) | ownership as today. It is an owner's own stored credential, not an authority. |
| `HostAuthority` (`claims.rs:121-130`) | `Verified`, matched against `trusted_host_principals` using the **token** issuer; or `HostLaunch` |
| Effective settings (`effective_settings.rs:36-43`), observers (`observers.rs:251-254`, `channel_observers.rs:150-151`), reconciliation (`agent_instances.rs:330-333`) | see P5 |
| `skill_required_roles` (`manager.rs:1709-1710`) | `Verified` roles only |
| Governed A2A delegation (`uar_instance_id`, `grpc.rs:122`, `a2a/handler.rs:97-99`, `a2a/v1.rs:214`, `routes.rs:1733`, `thread_service.rs:1437`, `team_execution.rs:261`) | `Verified` only. Other modes return an explicit `peer_identity_unverified`. |
| `workspace_authorities` (`authority.rs:44-49`), `api_key_admin_principals`, `trusted_host_principals` (`config.rs:806-815`) | `Verified` only. Otherwise a startup warning says they are inactive. |
| API keys (`api_keys.rs:330-456`; `policy.rs:82-87, 98, 121, 140`; `middleware.rs:312`) | Exist only as the `builtin_api_key` validator (O2) |

**Persisted principals carry an authority class, versioned.**

- **`ScheduleOwnerGrant`** (`claims.rs:265-276`, `deny_unknown_fields`) gains
  `authority_class: Option<Verified|HostLaunch|Presented|SingleUser>` in a
  `v2` record.
  - On restore, a v1 record keeps today's behavior: `issuer` kind is treated
    as `Verified`, and local kinds as their host or local class.
  - A scheduled run never gets more authority than its recorded class, and
    `Presented` grants get no shared credentials.
  - New rows cannot be read by an older binary. That is the stated
    downgrade limit.
- **Observer rows** gain `owner_issuer: Option<String>`.
  `from_persisted_identity` (`messages.rs:71-90`; `observer/delivery.rs:189`)
  uses it to rebuild the principal. A null value means a legacy or primary
  owner, which is today's behavior.

### P5. Administrative gate, independent of the settings flag (fixes revision 2 C3)

**Owner:** uar-trust-tools.

Effective settings, observers, reconciliation, memory admin and accounting are
admitted for `Verified` with the existing roles, `HostLaunch` or `AdminKey`.

The new setting `security.admin.require_key_outside_hook` defaults to true.
When true, these features ignore `settings_mutation_auth_required`, which
continues to govern only settings mutation. `effective_settings.rs:36-37` says
"A settings key … cannot substitute". `AdminKey` is therefore a **new**
admission for effective settings, used only outside `validation_hook`; that
the operator accepts this is open decision O9.

The local fixture's proxy injects the admin key from its own config (P8). The
key is held by external infrastructure, so tests carry no credential.

### P6. Token custody and forwarding (T4; G6)

**Owners:** uar-trust-tools, with uar-runtime.

**Custody.**

- `PresentedToken` is held for **the run's lifetime in this process**, from
  admission to the run's terminal state or process exit.
- It is never serialized, its Debug output is redacted, and the scrubber
  captures it.
- Team children and A2A sends started by the run share it.

**Runs without a live token** are runs resumed after a restart, schedules and
follow-ups. They have no token. A continuation that a new request starts uses
that request's token, as new custody.

**Forwarding.** `mcp.servers[].forward_identity` and
`a2a.peers[].forward_identity` take one of:

- `none` (the default);
- `exchange`, which calls the exchange hook. It is modelled on RFC 8693 and
  must use https or loopback;
- `raw`, which needs an explicit owner setting and logs a startup warning
  (O5).

Every exchanged token is registered with the scrubber before use. A
forwarding destination reached by a run with no live token fails with
`identity_unavailable`.

**Leak checks run on the fixture side.** The recording exchange endpoint and
the recording MCP peer keep received token values only in their own process.
They expose `GET /leak-scan?run=<id>`, which reads the run's events, history
and logs and returns booleans. The test never sees a token value.

### P7. Rekey tool (T5)

**Owner:** uar-state.

- **Default upgrade.** Under P1 and P3 the default upgrade needs no rekey.
- **Command:** `uar identity rekey --from-alias <iss|none> --to-alias <iss|none> [--apply]`.
  - It is needed only when `primary_alias` changes on a node with data.
  - It runs offline, with UAR stopped.
  - The default is a dry run.
  - It rewrites owner keys and stored principal fields in Key A, Key B,
    schedule grants, observers, credential mappings and memory identifiers.
- **Journal and resume.** A per-store journal makes a partial run resumable
  across SurrealDB and PostgreSQL. A receipt is written at the end, and no row
  is deleted.
- **Recorded alias.** The applied alias is kept in a database row, not an
  editable setting.
- **Startup check.** UAR refuses to start when the configured alias differs
  from the recorded one and no completed receipt exists.

### P8. Local identity proxy (T7; G8)

**Owner:** uar-lead (tools).

`tools/uar-jwt-proxy`:

- **Loopback only.** It refuses any other listen address unless
  `PROXY_ALLOW_NON_LOOPBACK=1`, which tests never set.
- **Profiles.** A non-secret `x-uar-test-identity: <profile>` header selects
  one. Profiles live in the proxy's config file as
  `{ iss?, sub, tenant?, signing, extra_claims? }`. `signing` is one of:
  - `unsigned`;
  - `shared_secret`;
  - `jwks:<keyid>`, where the proxy serves a local JWKS at `/jwks.json`. This
    gives two issuers and real RS256 for `builtin_jwks`.
  - `forged`, a bad signature.

  `extra_claims` (for example `roles`, `uar_credential_kind`) exist so tests
  can prove they are stripped.
- **Default profile (R5).** It reproduces today's identity exactly: `sub`,
  `iss` = the configured `jwt_issuer`, tenant, `shared_secret`, roles
  `admin,user`.
- **Admin key.** It injects `X-UAR-Admin-Key` from its own config for
  `admin=true` profiles.
- **gRPC.** An h2c listener applies the same injection for gRPC.

### P9. Specifications and documentation

**Owner:** uar-lead.

- **OpenSpec changes**, one per P1-P8. The candidate capabilities are:
  - `identity-trust-declaration`;
  - `presented-identity`;
  - `owner-principal-derivation`;
  - `principal-authority`;
  - `admin-gate`;
  - `identity-custody`;
  - `identity-rekey`;
  - `local-identity-fixture`.
- **Documents.**
  - Update `docs/operations/principal-provider-credentials.md` and the
    security docs.
  - Add `docs/operations/identity-modes.md`, including the W4 statement and
    the downgrade limit.
- **Decisions.** Append to decisions.md that "team APIs require a verified
  owner" now means a verified **or presented** owner.

### P10. Fixtures and behavior scenarios (authored now, run only at the gate)

**Owner:** uar-lead, with the scenario authors.

**Fixture launcher.** Author
`scripts/manual-host/identity-fixture.mjs <fixture> [--stop]`. It is built on
the existing `c52-disposable-instance.mjs`. Each fixture starts its own pinned
SurrealDB, UAR and proxy on loopback, outside the repository's working
directory (gotcha 2026-10-20). Fixtures:

| Fixture | UAR config |
|---|---|
| `legacy` | **The pre-change UAR binary**, recorded by digest before P1, with today's `jwt_required=false` and `jwt_issuer` set |
| `upgraded` | The new binary on `legacy`'s data directory, with the same config (default mapping) |
| `front` | The same data directory, switched to `trusted_front` with `primary_alias` = the old `jwt_issuer` |
| `no-issuer` | A fresh store, legacy binary, no `jwt_issuer`. It is then upgraded and switched to `trusted_front` with no alias, then with `primary_alias` set and a rekey |
| `multi-issuer` | `validation_hook` with `builtin_jwks` validators for issuers A and B, both served by the proxy JWKS |
| `bind-refusal` | `trusted_front` bound to a non-loopback interface address, with no attestation; plus `single_user` with the same bind |

**Scenarios.** Each has a feature file, a steps file and a
`scripts/behaviors/identity-<name>.cucumber.mjs`. Tests send only profile
names.

| Scenario | Fixture | Proves |
|---|---|---|
| `identity-upgrade-continuity` | `legacy` → `upgraded` → `front` | Data created under `legacy` stays listed and usable after the upgrade and after the switch. The data is a session, KB, memory, schedule, follow-up, observer and a stored personal provider credential. New data is written first, then listed. |
| `identity-no-issuer-node` | `no-issuer` | Without an alias, IdP tokens get new keys. That is the documented split. With the alias plus `rekey`, the old data is reachable and the receipt matches the rows. |
| `identity-two-subjects` (catalog E09) | `front` | Two subjects are isolated in runs, events, cancel and sessions; recall survives restart. |
| `identity-two-issuers` | `multi-issuer` | The same `sub` from A and B stays separate in sessions, settings, teams, observers, model runs and schedules. B's schedule fires as B. |
| `identity-forged-refused` | `multi-issuer` | `forged` gets 401; `jwks:A` succeeds. |
| `identity-no-authority-from-presented` | `front` | A profile with `extra_claims` roles=admin, `uar_credential_kind=issuer` that matches the fixture's `trusted_host_principals` entry gets no effective settings, no host authority, no shared credentials and no delegation. Positive control: the same profile under `multi-issuer` as `jwks:A` gets host authority. |
| `identity-admin-gate` | `front` | Without the admin profile, effective settings, observers and reconciliation are refused even with `settings_mutation_auth_required=false`. With the admin profile they are allowed. |
| `identity-forwarding` | `front` plus recorders | `none` sends nothing. `exchange` calls the recording exchange endpoint, and the MCP peer gets only the exchanged token. A run resumed after a restart fails at the forwarding tool with `identity_unavailable`. A continuation started by a new request succeeds. `/leak-scan` is false everywhere. |
| `identity-bind-refused` | `bind-refusal` | Both configurations refuse to serve and name the reason. |
| `identity-sidecar-host` | the existing sidecar launch path | `local-host` data and host features are unchanged. `authentication.principalMode` is `hostAsserted` (`capabilities.rs:322-336`). |

**Regression** (main fixture, after switching it to `front`): C03, C07, C39,
C05 and C36, once each.

## Dependencies

- P1 → P2 → P3 → P4.
- P5 depends on P4.
- P6 depends on P2.
- P7 depends on P3 and P4.
- P8 depends on P1-P3 and P5.
- P10 depends on P1-P8.
- P9 runs alongside them.
- None of this depends on the A2UI work.

## Gate (exact commands; run serially after P1-P10 and operator approval)

1. **Record the legacy binary.** Before P1 lands:
   `shasum -a 256 target/debug/universal-agent-runtime > .prometheus/verification/identity-legacy-binary.sha256`.
   The binary is copied to `target/identity-legacy/` (a reversible copy).
2. `cargo build --locked --no-default-features --features server-full`
3. `cargo build --locked --manifest-path tools/uar-jwt-proxy/Cargo.toml`
4. For each change:
   `node /Users/gqadonis/Projects/prometheus/prometheus-skill-system/dist/plugins/claude/prometheus-skill-pack/skills/kbd-process-orchestrator/shared/openspec/cli.mjs run --project "$PWD" -- validate <change-name>`
5. For each fixture, in the order of the scenario table:
   `node scripts/manual-host/identity-fixture.mjs <fixture>`, then that
   fixture's scenarios with
   `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/identity-<name>.cucumber.mjs`,
   then `node scripts/manual-host/identity-fixture.mjs <fixture> --stop`.
6. Regression on the main fixture:
   `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/<c-scenario>.cucumber.mjs`
   for C03, C07, C39, C05 and C36.
7. **The Boss.**
   - **This Mac is Intel.** `pnpm --dir /Users/gqadonis/Projects/prometheus/the-boss build:mac:x64:release:uar`
     proves only that The Boss still builds against a **released** UAR. No
     x64 local-UAR payload exists: `local-uar-payload.cjs:42` refuses
     non-arm64.
   - Exercising the changed sidecar inside The Boss needs either
     `build:mac:arm64:local` on an Apple Silicon Mac, or a new x64
     local-payload path in The Boss (O7).

## Open decisions (operator)

| Id | Question | Recommended |
|---|---|---|
| O1 | No data rewrite by default; the rekey tool only when the alias changes. Accept? | Yes |
| O2 | Keep API keys only as the optional `builtin_api_key` validator? | Yes |
| O3 | External validator: an HTTP callback, plus an in-process trait for embedded hosts? | Yes |
| O4 | Tenant in `trusted_front` comes from the token claim, and is optional? | Yes |
| O5 | Allow `raw` forwarding? | Only with an explicit owner setting and a startup warning |
| O6 | Deprecation window for `jwt_required` | One release |
| O7 | The Boss gate on this Intel Mac: add an x64 local-UAR payload to The Boss, or run `build:mac:arm64:local` on an Apple Silicon Mac? | Add the x64 local payload. This is a change in the-boss, filed separately. |
| O8 | Disable governed A2A delegation and shared provider credentials outside `validation_hook`? | Yes |
| O9 | Admit `AdminKey` for effective settings outside `validation_hook`, which today refuses a settings key? | Yes, only with `require_key_outside_hook`, and documented |
| O10 | Accept the downgrade limit (older binaries cannot read v2 schedule grants)? | Yes, documented |

## Out of scope

- An identity provider, logins or a role system inside UAR.
- The A2UI surface work.
- Service identities for scheduled runs.
- San Saba's A2A task tenant partition. Re-check it after P3.

---

## Revision 3 independent review — 2026-10-28

A fresh-context `artifact-critic` reviewed this revision. Verdict: **defects
found — 2 critical, 13 warnings, 4 suggestions.** It checked about 40
citations and found them correct; three paths were imprecise.

The lead re-verified two claims:

- `target/debug/universal-agent-runtime` does not exist. The fixture runs
  `target/uar-team-concurrency-repair/debug/universal-agent-runtime`
  (`target/prompt-macos-launch.json`).
- The actor registry is keyed by `(ActorOwner, String)`
  (`actor/system.rs:47`).

### Critical

- **C1. The upgrade evidence can't be produced.** Gate step 1 hashes a binary
  that doesn't exist, and it must run "before P1" inside a gate that runs
  after P10. Once P1 lands, a pre-change binary needs a build of old source,
  which the closed gate forbids. Without it, the `legacy`, `upgraded` and
  `no-issuer` fixtures cannot exist.
- **C2. Key B v1 can't survive an alias change.** v1 Key B has no issuer
  (`messages.rs:133-144`, `user_settings.rs:138-153`).
  - After an alias change, the new alias's users inherit the old alias's
    settings, teams, MCP-bound runs, targets and observers, and a rekey can't
    tell the rows apart.
  - The no-issuer rule (map the alias to `local-jwt`, so keys don't change)
    contradicts P7's mandatory rekey and the `identity-no-issuer-node`
    expectation.

### Warnings (summary)

- **W1. Key B's v1/v2 choice needs configuration its builders don't have.**
  An `ActorOwner` without a principal falls back to v1, the primary's
  namespace. Rule 3 also stops working once `jwt_issuer` is deprecated.
- **W2. The shared-credential rules conflict.**
  - `policy.rs:111` is both the personal-binding check and an authority read.
  - O8 conflicts with restoring v1 schedules as `Verified`.
  - Pause and resume keep the old grant (`schedules/controller.rs:134-156`),
    so a presented impersonator can resume a v1 schedule that then fires as
    `Verified`.
- **W3. Role trust across validators is ambiguous.**
  - Upgraded `jwt_required=true` nodes would lose roles, or any added JWKS
    issuer could grant itself `admin` and `uar_instance_id`.
  - `HostAuthority` requires the private `uar_credential_kind` marker
    (`claims.rs:122`, `verifier/mod.rs:158`), and a real identity provider
    never sends it.
- **W4. P5 widens admission beyond O9.**
  - The sidecar launch secret opens effective settings, which
    `effective_settings.rs:35-39` refuses today.
  - Accounting cross-owner reads (`accounting.rs:44-58`) and memory-admin
    import (`memory_admin.rs:433`) move from `trusted_host_id` to roles or the
    admin key.
- **W5. One authority value can't describe The Boss's requests.** The Boss
  sends the launch token, the admin key and `x-uar-principal` together
  (`UarSidecarService.ts:649-675`). A single-valued `PrincipalAuthority`
  can't hold both, precedence is unstated, and delegation-grant bearers
  (`delegation_grants.rs:136-149`) have no variant.
- **W6. Actors can be reused across authority levels.** The registry is keyed
  by `ActorOwner`, and authority sits outside equality. A lower-authority
  request can reuse an actor spawned with higher authority
  (`actor/system.rs:47, 171`). Replace authority per message
  (`actor_host.rs:226`).
- **W7. `PrincipalAuthority` can be forged by construction.** It is a public
  plain enum. `VerifiedIdentity` and `HostAuthority` deliberately use private
  fields (`claims.rs:100-147`).
- **W8. Observers break an existing safeguard.** Rebuilding an observer's
  principal from a row contradicts `messages.rs:81` and `claims.rs:185`, and
  no authority class is recorded.
- **W9. Threading sites are missing.**
  - Not listed: `manager.rs:5356, 6114, 6149`, `api/credentials.rs:47`, and
    policy functions that take only `Option<&CredentialPrincipal>`.
  - `FollowupOwnerGrant` wraps the schedule grant (`claims.rs:281`), so the
    downgrade limit also covers follow-ups.
- **W10. `single_user` has no tenant or `owner_iss` default.** Existing rows
  split when a node switches to it.
- **W11. Gate gaps.**
  - The regression configs don't exist under C ids.
  - Switching the live fixture has no command, and it changes operator state.
  - The c52 fixture root is inside the repository (`c52-disposable-instance.mjs:14`).
  - The sidecar scenario has no fixture.
  - `multi-issuer` has no alias, so the v1/v2 boundary and "add a second
    validator to a node with data" go untested.
  - `/leak-scan` cannot see tokens the proxy mints.
- **W12. The plan conflicts with operator rules.**
  - The 2026-10-26 rule forbids test scripts that "mint … API authentication
    keys" and says "authentication plumbing must never become a test step,
    scenario". The fixture launcher generates signing keys and the admin key,
    and `identity-forged-refused` and `identity-admin-gate` are
    authentication scenarios. This needs an explicit ruling.
  - The project rule names `pnpm build:mac:arm64`. Swapping in x64 needs a
    waiver, and neither build exercises the changed UAR.
- **W13. API-key records are missing from the rekey list.**
  `api_keys/policy.rs:160` requires reissue when the issuer differs.

### Suggestions

- `IdentityTrustedFront` sets governance `may_disable=false`.
- T4 says "request only" while P6 says "run lifetime".
- `HostAsserted` collides with the wire value `PrincipalMode::HostAsserted`.
- The v1 `local-jwt` grant class is undefined.

### Lead assessment

There have been three revisions, each with critical findings. The defects are
now more specific (key B issuer, upgrade evidence, authority composition),
but they keep coming from the same constraint: preserving every existing key
while adding issuers and splitting authority.

Two choices would end the loop:

1. **Version the keys properly.** Do one explicit, tested migration to
   issuer-qualified v2 keys everywhere, with Key B included. Make the upgrade
   evidence a **data snapshot**: copy the current fixture's store before P1,
   so no old binary is needed.
2. **Narrow the first change.** Take `trusted_front` plus presented identity
   plus the proxy selector, with the alias fixed (alias changes refused),
   one validator, and no rekey. That covers E09 and most of San Saba's need.
   Defer multi-validator nodes, rekey and the authority redesign.

Either way, W12 needs an operator ruling on authentication scenarios.

**Readiness: BLOCKED.**
