# Assessment — identity-pass-through

KBD Assess, 2026-10-28. This is a read-only source assessment. No code was
changed and no builds or tests were run (the verification gate is CLOSED).

Sources:

- the adopted decision (`.prometheus/decisions.md`, "Pass-through identity with
  a required trust declaration", 2026-10-28);
- the evaluation (`../../../../identity-pass-through-evaluation.md`);
- a uar-trust-tools source survey of the same date. Its file and line citations
  are reproduced below and were not re-verified one by one.

## Target state (adopted)

| Id | Requirement |
|---|---|
| T1 | UAR authenticates nothing and applies no roles of its own. |
| T2 | Every node declares where validation happens: `trusted-front` (a named upstream validates, and UAR binds loopback or a private network only), a validation hook (an owner plugin, or a built-in JWKS/issuer/audience check), or `single-user`. UAR refuses the unsafe combination of multi-user, no validator and a public bind. |
| T3 | The partition key is `(iss, sub)`, with tenant optional, the same on every interface. |
| T4 | Presented tokens are held in memory for the request only. They are forwarded only to destinations the owner opts in, preferably through a token-exchange hook. They are never persisted, and the scrubber stays. |
| T5 | Existing rows are mapped to the new key. |
| T6 | Features that depend on roles move to the admin key or host launch token for host features, and to the owner hook for the rest. |
| T7 | Local test identities come from a non-secret subject-selector header on the local identity proxy only. |

## 1. Credential intake today, per interface

| Interface | Intake | Validation | `jwt_required=false` |
|---|---|---|---|
| HTTP: native runs, AG-UI, A2A JSON-RPC, `/v1/chat/completions`, `/v1/messages`, `/mcp/uar` | `auth_middleware`, `src/uar/security/middleware.rs:195-310`, applied app-wide at `src/server.rs:2828` | A bearer runs through `verify_token`. `x-api-key` is checked only when there is no bearer (`:264-281`). | No token, or an invalid one, becomes `anonymous` (`:85, :111-116, :133-138`). |
| ACP | The same middleware, layered again (`server.rs:2839-2846`) | Same | Anonymous is allowed unless `acp.auth_required` is set (`acp/routes.rs:99,129`). |
| A2A gRPC, legacy and v1 | `authenticate`, `src/uar/api/a2a/grpc.rs:51-140`; `grpc_v1.rs:74` | A bearer is **always** required, then verified. A tenant is required when `jwt_required` is set (`:72`). There is no API-key path. | A token is still required. The policy diverges from HTTP. |
| A2A HTTP | `a2a/handler.rs:87-100` | Middleware | Anonymous requests reach the handler and then fail when the actor owner is built. |
| MCP server | `verified_owner`, `src/uar/mcp_server.rs:71-87` | Middleware context | Anonymous requests are rejected. |
| Sidecar launch | `sidecar_guard.rs:244-310`; `x-uar-principal` at `middleware.rs:39-83` | Constant-time launch token, not a JWT | Not applicable |
| API keys | `security/api_keys.rs:330-456` | Hash checked against storage. The key exchanges for a UAR-minted HS256 JWT marked `uar_credential_kind:"api_key"`. | Same |

**Verifier** (`security/verifier/mod.rs`):

- HS256 with a shared secret (`:154-161`), or JWKS with RS256/ES256/ES384 and a
  required `kid` (`:183, :260-273`).
- `exp` is required, with a 60 s leeway. `iss` and `aud` are checked only if
  configured (`:131-143`).
- The remote profile requires `sub` and `tenant_id` (`:325-329`) and full
  policy at configuration time (`src/config.rs:802-841`).

## 2. Partition keys today

- **Principal.** `CredentialPrincipal` (`claims.rs:199-242`) uses the
  **configured** issuer (or `urn:uar:local:jwt`), never the token's own `iss`.
- **Key A, `session_owner_key`** (`claims.rs:164-173`; `actor/messages.rs:126-129`)
  is kind + issuer + tenant + sub, stored as
  `v1:k:{kind}:i:…:{iss}:{t:…|local}:s:…:{sub}`. Its users:
  - sessions, conversations, runs and history;
  - AG-UI (`agui.rs:95,485`) and ACP (`acp/handler.rs:130`);
  - knowledge bases and memory (`memory/service.rs:1461-1463`);
  - approvals, client tools, `/v1` chat and messages (`server.rs:4985,6637`);
  - schedule and follow-up grants.
- **Key B, tenant + sub with no issuer.** `presentation_owner_key`
  (`messages.rs:133-145`) and `principal_storage_key`
  (`api/user_settings.rs:138-153`). Its users:
  - user settings;
  - collaboration and **teams** (`collaboration.rs:436-438`);
  - MCP-bound runs (`mcp_server.rs:85`);
  - prompt caching (`server.rs:5285-5293`).
- **Workspace.** `x-uar-workspace-id` is in neither key. It is admitted against
  `workspace_authorities` only in remote mode (`authority.rs:44-49`).

## 3. Authority derived from verified claims or roles

| Feature | Today | Source |
|---|---|---|
| Effective settings read | role `admin` | `api/effective_settings.rs:43` |
| Observers and channel observers | `operator`, `admin` or `host-session` | `observers.rs:251-254`; `channel_observers.rs:150-151` |
| Agent instance reconciliation | `operator`, `admin` or `host-session` | `agent_instances.rs:330-333` |
| API-key role delegation | a subset of the caller's roles, never `admin` or `host-session` | `api_keys/policy.rs:82-87`; `api_keys.rs:453` |
| Skill role gates | `skill_required_roles` checked against `verified_roles` | `runtime/manager.rs:1709-1710`; `messages.rs:54` |
| Host authority (memory admin, credentials, migration, accounting, supervisor, MCP host paths) | a token matching `trusted_host_principals` | `claims.rs:121-130`; `memory/service.rs:1468-1470` |
| A2A governed delegation | the `uar_instance_id` claim | `routes.rs:1733`; `thread_service.rs:1382,1437`; `team_execution.rs:261` |
| Shared provider credentials | a verified issuer, tenant and subject | `docs/operations/principal-provider-credentials.md:39-42` |
| Settings mutation | `X-UAR-Admin-Key`, not the JWT | `api/settings.rs:396-404` |
| Sidecar principal | a synthesized `host-session` role | `middleware.rs:77` |

## 4. Token custody today

- **Nothing is forwarded.** `AuthenticatedCredentialCapture`
  (`credential_capture.rs:1-29`) only feeds `RunSecretScrubber`, which removes
  the token from errors, events, history and checkpoints
  (`manager.rs:6281-6306`).
- **Outbound credentials come from configuration:**
  - MCP headers (`mcp/runtime.rs:106-125`);
  - A2A peer bearers (`a2a/peer.rs:52`);
  - paired admission (`tool_admission/http.rs:90-100`).
- **No persistence was found.** Not every `tracing` call site was audited.

## 5. Local test identity today

- `tools/uar-jwt-proxy` strips the client's `Authorization` header (`:394`).
- It mints an HS256 token on every request, signed with UAR's secret
  (`:44-47, :343-355`). The claims are `sub=$PROXY_JWT_SUB` (default `dev`), an
  optional `tenant_id`, and `roles` defaulting to `admin,user` (`:57-69`).
- It forwards only HTTP and websocket traffic. gRPC is not proxied.
- The live fixture listens on `127.0.0.1:60675`, with UAR on 53587 and gRPC on
  53588.

## 6. Gaps against the target

| Gap | Target | Evidence |
|---|---|---|
| G1 | No trust declaration exists. Profiles are `trusted_local` and remote, with `jwt_required` as a boolean. | T2 | `config.rs:731, 802-841` |
| G2 | Identity is only built from a verified token. There is no unverified decode path. | T1, T3 | `verifier/mod.rs:331`; `claims.rs:57-106` |
| G3 | Two key families (A with issuer, B without) and a configured, not token, issuer. | T3 | §2 |
| G4 | gRPC always requires a verified bearer; HTTP downgrades to anonymous. | T1, T3 | §1 |
| G5 | Ten features depend on roles or verified claims. | T6 | §3 |
| G6 | No per-destination forwarding, exchange hook, or in-memory custody beyond capture for scrubbing. | T4 | §4 |
| G7 | Existing rows are keyed `v1:k:…` and `v1:t:…` with the configured issuer. | T5 | §2 |
| G8 | The test proxy needs UAR's signing secret, has no selector header, and no gRPC. | T7 | §5 |
| G9 | API keys are a UAR-issued credential that UAR authenticates, which conflicts with T1. | T1 | `api_keys.rs:330-456` |
| G10 | Embedded hosts construct owners without a `UserContext`. Not traced. | T3 | `src/embedded.rs` |

## 7. Risks

- **R1. Forged identities.** In `trusted-front` mode every claim is asserted.
  Isolation is only as good as the declared front, which is the accepted
  trade-off, guarded by the T2 refusal.
- **R2. Silent data split.** A key change without a complete mapping hides
  users' existing sessions, memory and KBs. No data is deleted, but it becomes
  invisible.
- **R3. Feature loss.** Role-gated features lose their gate if the T6
  replacements are not complete before roles are removed.
- **R4. Forwarding.** MCP servers that follow the specification reject
  forwarded user tokens (MCP security best practices, "Token Passthrough"), so
  owners need the exchange hook for most tools.
- **R5. Test regressions.** Every passing behavior scenario uses the proxy. Its
  change must keep their identity stable (same `sub`, and an `iss` chosen so
  the mapped keys stay the same).

## 8. Prior context

The memory recall for this child was not run. No KBD lessons beyond this
session's records apply.

Knowledge gap: whether embedded hosts (`src/embedded.rs`) can carry a
principal at all (G10).

## Stage note

The operator asked for Assess and Plan only. Analyze (library and candidate
research) is skipped for this child by that instruction. The plan names one
external standard: token exchange, RFC 8693.

## Addendum — corrections from the plan revision 1 review (2026-10-28)

- **Fixture issuer.** The live fixture configures `security.jwt_issuer`
  (checked as a boolean only, value not read), with `jwt_required=false`,
  `settings_mutation_auth_required=false`, profile `trusted_local`, and host
  `127.0.0.1`. The proxy therefore emits `iss` equal to the configured issuer,
  and today's local rows are `issuer`-kind keys with that issuer.
- **Missing Key B form.** G7 also covers the tenantless Key B form `v1:s:`
  (`messages.rs:142`, `user_settings.rs:150`) and the tenantless Key A form
  `…:local:s:…` (`claims.rs:238`).
- **Key sites.** The key builders appear in 76 non-test source files: 48
  `session_owner_key()`, 43 `presentation_owner_key(`, 18
  `principal_storage_key(` and 135 `.storage_key()` uses. Stored values are
  compared for equality at least at:
  - `claims.rs:321`;
  - `memory/owned.rs:124`;
  - `memory/controls_store.rs:308`;
  - `credentials/principal_store.rs:151`;
  - `cost_budget.rs:951`;
  - `manager.rs:656, 9275`;
  - `native_skills/team_tools.rs:103`;
  - `mcp/interaction.rs:75`;
  - `api/accounting.rs:55`;
  - `api/discovery.rs:652`;
  - `persistence/approval_decisions.rs:90`.
- **`issuer_kind` as an authority marker.** It is read as one at
  `credentials/policy.rs:111, 136` and `credentials/owner.rs:15-33`.
- **Bound addresses.** The governance runtime already records actual bound
  ingress addresses and flags non-loopback ones
  (`governance/runtime_control.rs:455-480`).
