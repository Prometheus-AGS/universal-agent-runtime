# Plan — identity-pass-through (revision 4, narrowed)

KBD Plan, 2026-10-28. Planning only. Nothing is implemented. The verification
gate stays CLOSED until N1-N6 are implemented and the operator approves the
gate commands.

Revisions 1-3 and their reviews are in `plan-rev1.md`, `plan-rev2.md` and
`plan-rev3.md`.

## Scope (operator, 2026-10-28)

Revision 4 covers only:

- `trusted_front` mode;
- the presented identity;
- the proxy's identity selector;
- **one fixed issuer**, where issuer changes are refused;
- **one validator**, today's verifier, unchanged;
- **no rekey**.

**Deferred** to later changes:

- multi-validator nodes and more than one issuer per node;
- the authority redesign (`PrincipalAuthority`);
- `single_user`;
- token forwarding and the exchange hook;
- the gRPC listener in the proxy (I15-I18);
- bind attestation for containers.

**What this means for the adopted decision** (decisions.md, 2026-10-28):

- **Q1 (trust declaration):** this delivers the `trusted_front` declaration
  only. The default stays today's verifier.
- **Q2 (`(iss, sub)` key):** this holds trivially, because one issuer is
  admitted per node.
- **Q3 (opt-in forwarding):** deferred. Nothing is forwarded, as today.
- **Q4 (data mapping):** no data is remapped, because no persisted format
  changes.
- **Q5 (role replacement):** deferred. Features that need roles are
  unavailable in `trusted_front`.
- **Q6 (test identities):** delivered as the proxy selector.

## Design: one issuer, today's keys, nothing persisted changes

- A `trusted_front` node accepts presented tokens whose `iss` is absent or
  equal to the node's **fixed issuer**. The fixed issuer is today's
  `security.jwt_issuer`, or none on nodes without it.
- Any other `iss` is refused. No second issuer can exist, so no new key form
  is needed.
- The principal built from a presented token is **exactly** today's principal
  formula (`claims.rs:199-232`):
  - kind `issuer` with the configured issuer when `jwt_issuer` is set;
  - otherwise kind `local-jwt` (`urn:uar:local:jwt`);
  - the tenant from the token's `tenant_id`, as today.

  Every key and every stored equality check is unchanged. No persisted format
  changes, so no migration and no old binary are needed for upgrade evidence.
- **Authority is not redesigned.** A presented identity carries **no**
  verified identity, no host authority, no roles and no private markers.
  Every authority site that reads those (assessment §3) therefore fails
  closed, unchanged. Two authority reads use the principal's `issuer_kind`
  rather than the verified identity. They are neutralized by **refusing the
  configurations** that would make them grant anything (N2).

## Changes, in order

### N1. Proxy identity selector (T7) — owner: uar-lead (tools); independent of N2-N4

- **The selector.** `tools/uar-jwt-proxy` reads a non-secret
  `x-uar-test-identity: <profile>` header. Unknown profile names get 400.
  Without the header, the current single identity is used, unchanged.
- **Profiles** live in the proxy's own config file
  (`PROXY_PROFILES_FILE`), each as
  `{ sub, tenant?, iss_override?, signing: shared_secret | unsigned, extra_claims? }`.
  - `shared_secret` is today's signing (`main.rs:343-355`), with `iss` from
    the configured `jwt_issuer` (`main.rs:205-208`).
  - `unsigned` is for `trusted_front` fixtures. It emits a JWT with an
    `alg:none` header.
  - `iss_override` exists only to prove that a foreign issuer is refused
    (O3).
  - `extra_claims` exist only to prove they are stripped.
- **No roles for named profiles.** Profiles carry no roles unless
  `extra_claims` adds them. The default identity keeps today's `admin,user`.
- **Loopback only.** The proxy refuses a non-loopback listen address
  (`main.rs:32` `PROXY_LISTEN`) unless `PROXY_ALLOW_NON_LOOPBACK=1`, which no
  fixture sets.
- **E09 needs nothing more.** The selector unblocks E09 on today's main
  fixture with no UAR change: two `shared_secret` profiles with different
  `sub` values. UAR already separates subjects (assessment §2).
- **Files:** `tools/uar-jwt-proxy/src/main.rs` and its README.

### N2. `trusted_front` configuration and startup guards (T2) — owner: uar-trust-tools

**Setting:** `security.identity.mode`.

| Value | Behavior |
|---|---|
| `verify` (default) | Today's behavior, driven by `jwt_required` and every existing setting. No consumer of `jwt_required` changes. |
| `trusted_front` | Presented identity (N3). Allowed only in the server binary. |

**`trusted_front` refuses to start**, with a named reason, when any of these
holds:

1. **The node is a sidecar or host-launched.** The sidecar binary
   (`src/bin/uar-sidecar.rs`) and sidecar-guard mode don't read this setting,
   and the config loader rejects it there.
2. **Shared provider credentials are configured** (`provider_credentials`
   shared mode or `shared_fallbacks`). The shared-mode check reads the
   principal's `issuer_kind` and `tenant` (`credentials/policy.rs:111`), so a
   presented token naming the fixed issuer and a tenant would pass it.
3. **A registered bound address is not loopback.** The check reads the
   governance bound-ingress registry after every listener binds
   (`primary-http`, `companion-http`, `a2a-grpc`; `server.rs:509, 525, 552`;
   `governance/runtime_control.rs:455-480`). Hostnames are never judged.
4. **`jwt_required=true`** is also set. The two are contradictory, so the
   operator must choose one.
5. **The fixed issuer changed** (N5).

**Allowlists stay inactive.** `trusted_host_principals`,
`api_key_admin_principals` and `workspace_authorities` stay configured and
inactive. They match only a `VerifiedIdentity` (`claims.rs:121-130`;
`authority.rs:44-49`), and none exists in this mode. A startup warning names
each one.

**DNS rebinding.** Every HTTP request's `Host` must be `localhost`,
`127.0.0.1`, `[::1]` or an entry in `security.identity.allowed_hosts`. This
reuses `sidecar_guard.rs` `authority_allowed`. gRPC binds only loopback under
guard 3, and its `:authority` check is deferred and stated.

**Governance status.** A new reason, `IdentityTrustedFront`, is added beside
`JwtRequired` (`runtime_control.rs:461-463`). It sets `may_disable=false`
(`:477`), as `JwtRequired` does today. The documentation states this.

**Files:**

- `src/config.rs`, the new `security.identity` section and its validation,
  next to `validate_identity_policy`;
- `src/server.rs` and `src/uar/governance/runtime_control.rs`, for the
  post-bind check;
- `src/uar/security/sidecar_guard.rs`, for the reused host check.

### N3. Presented-identity intake (T1) — owner: uar-trust-tools

**New file:** `src/uar/security/presented.rs`. It defines
`PresentedIdentity { iss: Option<String>, sub: String, tenant: Option<String> }`
and `decode_presented(token) -> Result<PresentedIdentity, PresentedError>`.

**`decode_presented`:**

- It base64url-decodes the payload and **never** verifies.
- It requires a non-empty `sub` that is not `anonymous`.
- It keeps only `iss`, `sub` and `tenant_id`. Everything else is discarded:
  `roles`, `uar_credential_kind`, `uar_api_key_id`, `uar_instance_id`, `exp`
  and `aud`.
- A malformed token gets 400 `presented_identity_malformed`.
- An `iss` that is present and differs from the fixed issuer gets 401
  `issuer_not_accepted`.

**Where it runs.** In `trusted_front` mode only, it replaces `verify_token`
at:

- the HTTP middleware bearer branch (`middleware.rs:195-310`), which covers
  native runs, AG-UI, A2A HTTP, `/v1` chat and messages, ACP and `/mcp/uar`;
- gRPC `authenticate` (`a2a/grpc.rs:51-140`; `grpc_v1.rs:74`).

**The resulting `UserContext`:**

- `claims` holds `sub` and `tenant_id`, with **empty** `roles` and no markers;
- `authority` is `None`;
- `host_authority` is `None`;
- `credential_kind` is `Unclassified`;
- `api_key_id` is `None`;
- `presented` is `Some(..)`.

Readers of `uar_instance_id` read `claims` (`grpc.rs:122`;
`a2a/handler.rs:97-99`), and the field is now empty, so governed delegation
is refused, as for any unverified caller.

**What stays the same:**

- **No token:** anonymous, as with `jwt_required=false` today.
- **`x-api-key`:** still runs through `validate_key` in this mode
  (`middleware.rs:264-281`). That keeps today's single validator for API
  keys, and API keys still carry verified identities.
- **The sidecar launch path** (`middleware.rs:39-83`) is unreachable, because
  of guard 1.

**Capture.** The presented token is still captured by `RunSecretScrubber`
(`credential_capture.rs:11-20`) and is never forwarded, as today.

### N4. Ownership from a presented identity (T3) — owners: uar-trust-tools, with uar-state

`CredentialPrincipal::from_verified_context` (`claims.rs:199-232`, 59
non-test callers) gains one branch. When `context.presented` is `Some`, it
builds the principal from the configured `jwt_issuer` (kind `issuer`) or
`local-jwt`, plus the presented `sub` and `tenant`. That is the same formula
as for a verified identity on this node.

- **Callers.** The 59 callers are unchanged. The function is renamed only in
  a later change.
- **Collaboration and teams** use `presentation_owner_key` and
  `principal_storage_key`, which take `user_id` and `tenant_id`
  (`messages.rs:133-145`; `user_settings.rs:138-153`). They get the same
  values. Today's "verified owner" check (`collaboration.rs:436-470`) accepts
  a presented owner, because it rejects only `anonymous`.
- **Authority reads on the principal.** Exactly two read `issuer_kind` (the
  assessment addendum and `credentials/owner.rs:15-33`):
  - **`policy.rs:111`, shared mode:** guard 2 refuses that configuration.
  - **`owner.rs:15-33` and `policy.rs:136`, personal binding:** ownership of
    the user's own stored credentials, as today.

**Consequence, stated in the documentation.** In `trusted_front`, anyone who
can reach the node can act as any `sub` under the fixed issuer, by sending an
unsigned token. On loopback that means any local process. Isolation depends
entirely on the front. Use `verify` mode where that matters.

### N5. Fixed-issuer record (no rekey) — owner: uar-state

- **Record.** On the first `trusted_front` start, UAR records the fixed issuer
  (the configured `jwt_issuer`, or the marker `none`). It goes in a new row in
  the existing durable settings store, `identity.fixed_issuer`, not an
  editable setting.
- **Later starts.** A later `trusted_front` start refuses to serve (guard 5)
  if the configured issuer differs. The message says issuer changes are
  unsupported and names the recorded value.
- **`verify` mode** ignores the record. Changing `jwt_issuer` there behaves as
  it does today, which is not this change's concern.
- **Memory store.** It has no durable row, so the check is skipped and the
  documentation says so.

### N6. Specifications and documentation — owner: uar-lead

- **OpenSpec changes:**
  - `identity-trusted-front`: N2-N5, with the capability `identity-trust-declaration`;
  - `local-identity-profiles`: N1, with the capability `local-identity-fixture`.

  Each has scenario-level spec deltas.
- **Documents.** Add `docs/operations/identity-modes.md`, covering the modes,
  the guards, the N4 consequence statement and the deferred list.
- **Decisions.** Append to decisions.md the narrowing and how Q1-Q6 map to
  this scope.

## Behavior scenarios (authored now, run only at the gate)

Tests send only profile names. The proxy, which is external infrastructure,
holds the signing secret, as it does today. No scenario sends, mints or reads
a credential.

| Scenario | Instance | Proves |
|---|---|---|
| `identity-profile-isolation` (catalog **E09**) | main fixture, today's `verify` mode, N1 only | Profiles `a` and `b` (`shared_secret`, different `sub`) see none of each other's runs, events or sessions; cancel of the other's run returns 404; recall survives an owned restart |
| `identity-front-isolation` | disposable, `trusted_front` | the same as E09 using `unsigned` profiles |
| `identity-front-existing-data` | disposable: first `verify`, then restarted as `trusted_front` with the same binary and store | a session, KB, memory record, schedule and follow-up created under `verify` with the default identity are listed and usable after the switch, and the schedule fires. Because no persisted format changes, this is the upgrade evidence. |
| `identity-front-foreign-issuer` | disposable, `trusted_front` | an `unsigned` profile with `iss_override` gets 401 `issuer_not_accepted` and creates nothing (O3) |
| `identity-front-no-authority` | disposable, `trusted_front` | an `unsigned` profile with `extra_claims` (`roles:["admin"]`, `uar_credential_kind:"issuer"`, `uar_instance_id`) gets 403 on effective settings, observers and reconciliation, no host authority, and is refused on governed delegation |
| `identity-front-guards` | disposable | `trusted_front` refuses to start with a non-loopback bind, with shared provider credentials configured, with `jwt_required=true`, and after `jwt_issuer` changes; each refusal names its reason |

**Disposable instances** use the existing
`scripts/manual-host/c52-disposable-instance.mjs` pattern: its own pinned
SurrealDB, UAR and proxy, under `target/c52-disposable` (gitignored, inside
the repository; the 2026-10-20 `.env`-ancestor gotcha applies only to the
sidecar binary, which is never started here). It is extended with a
`--identity-mode` and `--profiles` option.

**Regression** (main fixture, which stays in `verify` mode, because N3 and N4
touch shared middleware and principal code):

| Config | Feature | Catalog |
|---|---|---|
| `agent` | `persisted-agent` | C03 |
| `continuity` | `conversation-continuity` | C07 |
| `knowledge` | `math-knowledge` | C05 |
| `team-follow-up` | `team-follow-up` | C36 |
| `memory-lifecycle` | `memory-lifecycle` | C39 |

## Dependencies

- **N1 first.** It is independent, and unblocks E09 alone.
- Then N2 → N3 → N4 → N5.
- N6 runs alongside.
- Scenarios depend on N1-N5. None of this depends on the A2UI work.

## Gate (exact commands; serial; after N1-N6 and operator approval)

1. `cargo build --locked --no-default-features --features server-full`
2. `cargo build --locked --manifest-path tools/uar-jwt-proxy/Cargo.toml`
3. For each change:
   ```
   node /Users/gqadonis/Projects/prometheus/prometheus-skill-system/dist/plugins/claude/prometheus-skill-pack/skills/kbd-process-orchestrator/shared/openspec/cli.mjs run --project "$PWD" -- validate identity-trusted-front
   node /Users/gqadonis/Projects/prometheus/prometheus-skill-system/dist/plugins/claude/prometheus-skill-pack/skills/kbd-process-orchestrator/shared/openspec/cli.mjs run --project "$PWD" -- validate local-identity-profiles
   ```
4. Restart the main fixture's proxy with the new build and a profiles file
   (an operator-owned process change, O4), then:
   `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/identity-profile-isolation.cucumber.mjs`
5. For each `identity-front-*` scenario:
   `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/<scenario>.cucumber.mjs`.
   Each starts and stops its own disposable instance.
6. Regression: `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/<config>.cucumber.mjs`
   for `agent`, `continuity`, `knowledge`, `team-follow-up` and
   `memory-lifecycle`. The main UAR is restarted with the new binary first
   (owned restart, `restart-owned-uar.mjs`).
7. **The Boss.** N2 guard 1 keeps `trusted_front` out of the sidecar. The
   sidecar's behavior is unchanged, but its binary is rebuilt from changed
   shared code. The project rule's `pnpm build:mac:arm64` cannot exercise a
   local UAR on this Intel Mac (`local-uar-payload.cjs:42`) and packages a
   released payload anyway. O5 asks for the operator's ruling.

## Open decisions (operator)

| Id | Question | Recommended |
|---|---|---|
| O1 | Accept that `trusted_front` gives no isolation beyond the front, and that any local process on loopback can act as any `sub`? | Yes; stated in the docs |
| O2 | Accept that role-gated features (effective settings, observers, reconciliation, skill role gates) are unavailable in `trusted_front` until the authority redesign? | Yes |
| O3 | The 2026-10-26 rule: is a scenario that proves a foreign `iss` is refused (`identity-front-foreign-issuer`) acceptable? It sends only a profile name, but it tests identity admission. | Yes; it tests an ownership rule, not authentication |
| O4 | Restarting the main fixture's proxy with a profiles file (step 4) changes operator-owned state. Approve it at gate time? | Yes, with the original restored after |
| O5 | The Boss: waive `build:mac:arm64` for this change (sidecar behavior unchanged, and not buildable locally on Intel), or run `build:mac:x64:release:uar` as a build-only check? | Waive, with the reason recorded; a The Boss check belongs to the authority-redesign change |

## Out of scope (deferred)

- Multi-validator nodes and several issuers per node, with Key B issuer
  qualification.
- `PrincipalAuthority` and replacements for the role-gated features.
- `single_user`.
- Forwarding and the exchange hook.
- Bind attestation for non-loopback fronts.
- The gRPC proxy listener (I15-I18).
- A rekey tool.
- San Saba's RS256 need. It is served by `trusted_front` behind its BFF on
  the same host, or by `verify` mode once JWKS support is configured
  (`verifier/mod.rs:183`).

---

## Revision 4 independent review — 2026-10-28

A fresh-context `artifact-critic` reviewed this revision. Verdict: **defects
found — 2 critical, 9 warnings, 6 suggestions.**

**Confirmed core claims:**
- Presented identities reproduce today's principal exactly, because the issuer
  comes from config (`claims.rs:207-220`).
- Key B uses only `sub` and tenant.
- Guard 4 also excludes the remote profile (`config.rs:819-825`).
- All three listeners register (`server.rs:509, 525, 552`), and no other bind
  exists.

### Critical

- **C1. The upgrade scenario can pass without testing anything.**
  `c52-disposable-instance.mjs:85, 93-94, 137` and `restart-owned-uar.mjs:113-137`
  start the binaries named in the launch records
  (`target/uar-team-concurrency-repair/debug/…`). Gate step 1 builds into
  `target/debug`. `SecurityConfig` has no `deny_unknown_fields`
  (`config.rs:709-710`), so an old binary ignores `security.identity.mode`,
  and `identity-front-existing-data` passes trivially.
  - **Fix:** the fixture takes an explicit binary path and digest, and every
    `trusted_front` scenario first asserts that governance reports
    `IdentityTrustedFront`.
- **C2. Signed profiles can carry forged authority claims.** N1 lets
  `shared_secret` profiles carry `extra_claims`. A signed
  `uar_credential_kind:"issuer"` is the precondition for `HostAuthority`
  (`claims.rs:121-130`), and `uar_instance_id` unlocks governed delegation
  (`routes.rs:1733`). On the verify-mode main fixture, a non-secret header
  could then yield a signed host or instance credential.
  - **Fix:** `extra_claims` and `iss_override` are allowed only with
    `signing: unsigned`. The proxy strips `x-uar-test-identity` before
    forwarding.

### Warnings (summary)

- **Wrong expected status.** Effective settings returns 401, not 403
  (`effective_settings.rs:42`).
- **The Host check can't be reused as written.** `authority_allowed` is
  private and compares exact host:port strings, without `[::1]`
  (`sidecar_guard.rs:198, 210, 227-242`). The plan also drops the guard's
  Origin refusal (`:287`). CORS-allowed browser origins can therefore assert
  any `sub` with an unsigned bearer. This needs its own open decision.
- **API keys keep their roles in `trusted_front`.** An `x-api-key` still
  yields a `VerifiedIdentity` with stored roles (`api_keys.rs:389-411`).
  "Role features are unavailable" is therefore false. Presented owners also
  can't manage their own keys (`api_keys/policy.rs:93-110`). This is a missing
  decision.
- **N5 misses an issuer change at the switch.** Switching from verify (issuer
  A) to `trusted_front` (issuer B) records B silently, and A's data goes
  invisible. Other problems with N5:
  - The `none` marker can collide with a real issuer of that name.
  - Nothing protects the record row from the settings API.
  - The row contradicts "no persisted format change".
- **The presented capability is weaker than `VerifiedIdentity`.**
  - `UserContext` fields are `pub`, and the branch has no mode check or
    `matches()` binding.
  - About 17 `UserContext {` literals need edits.
  - `credential_kind` and `api_key_id` exist only inside `VerifiedIdentity`.
- **Unsigned profiles on a verify node become anonymous, not 401**
  (`middleware.rs:133-138`). A misrouted profile collapses into the anonymous
  partition.
- **Gate steps 4 and 6 are not runnable as written.**
  - `restart-owned-uar.mjs` is a module, not a CLI.
  - `assertOwnedEndpoint` pins the proxy pid and command line, so a proxy
    restart breaks regression scenarios unless the launch records are
    rewritten.
  - c52 expects exactly 7 proxy arguments.
- **gRPC regression is missing.** Add the `a2a`, `acp` and `agui` configs.

### Suggestions

- **Sidecar guard.** Detect sidecar mode by `launch_token.is_some()`, because
  the config loader is shared.
- **Websocket selector.** The websocket bridge doesn't carry the selector
  header.
- **Unsigned tokens.** `jsonwebtoken` can't encode `alg:none`, so encode them
  by hand.
- **Caller count.** The "59 callers" count is unverified; there are 21 direct
  call lines.
- **Decision record.** Record the narrowing in decisions.md.

### Lead assessment

This is the first review where the design's core claims were confirmed in
source. The critical findings are about the fixture and the proxy, and both
fixes are specific. The warnings add three operator decisions:

- API keys in `trusted_front`;
- browser origins as identity sources;
- the issuer change at the verify → `trusted_front` switch.

A revision 5 would be a small edit, not a redesign.

**Readiness: BLOCKED** until those fixes and decisions are made.
