# Round 3 handoff: uar-runtime (items 2b and 3)

Base c8eb9e23. Source inspection only. No Cargo, test, build, lint, server or browser was run.
Phase: implementation. Authorization class of every command: source inspection or implementation.

## Files changed

| Path | Change |
|---|---|
| `src/config.rs:1755` | `.set_default("resilience.burst_size", 100.0)` (was 20.0) |
| `src/uar/settings/resilience_policy.rs:33` | `ResiliencePolicy::default().burst_size` 100.0 (was 20.0) |
| `frontend/src/features/settings/ui/panels/resilience-preview.ts:4` | `burst_size: 100` (was 20) |
| `example.config.yaml:126-129` | `burst_size: 100.0` (was 10.0); comment `# Default: 100.0` |
| `tests/bdd/playwright.config.ts:143-146` | BDD app-server env: `UAR_SECURITY__LOCAL_LOOPBACK_OWNER: 'true'` plus a 2-line comment |
| this file | handoff |

Sustained `requests_per_second` untouched in all places. Limiter, frozen fixtures, other configs, tests untouched.
Interface effect: default burst 20 to 100 for any install that sets no `resilience.burst_size`; the BDD server
becomes the local owner for credential-free loopback requests. Product default of `local_loopback_owner` stays false.

## Check 1: the env key reaches the flag, and the preconditions hold

- Key mapping: `Environment::with_prefix("UAR").prefix_separator("_").separator("__").try_parsing(true)`
  (`src/config.rs:1963-1969`). `UAR_SECURITY__LOCAL_LOOPBACK_OWNER=true` becomes `security.local_loopback_owner`,
  parsed as bool. Field: `SecurityConfig.local_loopback_owner`, `#[serde(default)]` (`src/config.rs:781-782`).
  No `set_default` exists for it, none is needed. Same mechanism already used by the sibling `UAR_SECURITY__JWT_REQUIRED` entry.
- Decision: `trusted_local_loopback_context` (`src/uar/security/middleware.rs:174-184`) needs
  `local_loopback_owner && !jwt_required && !is_remote()`, loopback TCP peer from `ConnectInfo`, no `Authorization`
  and no `x-api-key` header, and `names_loopback_origin` (loopback Host; Origin absent or loopback http/https).
  Called at `middleware.rs:378-380`, only when no asserted principal, no Authorization header, no x-api-key.
- `is_remote()` is `deployment_profile == Remote` (`src/config.rs:824-826`); the profile defaults to `TrustedLocal`
  (`config.rs:667-673`) and the BDD env does not set it. `jwt_required` is `'false'` in the BDD env.
- `ConnectInfo` is wired: `src/server.rs:3510` and `3525` use `into_make_service_with_connect_info::<SocketAddr>()`.
- `server.host` default `127.0.0.1` (`config.rs:1747`); the BDD env sets only the port. Built page is served at
  `http://127.0.0.1:3002` (`tests/bdd/support/ports.ts:9-11`), so Host is `127.0.0.1:3002` and the peer is loopback.
- Node-side fetches (`tests/bdd/support/api.ts`, `s1.ts`, `steps/*`): `fetch(APP_BASE_URL + path)` with only
  `Content-Type`. No Authorization, no x-api-key, no Origin (Node fetch sends none), Host `127.0.0.1:3002`.
  All preconditions hold. (`api.ts:76,110,130,174,192,262,296,314,392,545`, `s1-ui.ts:425`.)
- Browser fetches: same-origin, so Host `127.0.0.1:3002`; Origin, when Chromium sends one on same-origin writes, is
  `http://127.0.0.1:3002`, which `is_loopback_authority` accepts.
- Does the frontend send an empty or placeholder Authorization header? No. Every site builds headers from
  `VITE_UAR_API_KEY ?? ""` and returns `{}` when empty: `presentations-api.ts:16-22`, `teams-transport.ts:41-45`,
  `presentation-assignments/api.ts:13`, `session-configuration/effective-model.ts:50-57`,
  `session-configuration/api/session-configuration-api.ts:10-15`, `user-jwt-settings-store.ts:11-24`
  (returns null unless the key starts with `ey`). No `.env*` with `VITE_UAR_API_KEY` exists (checked by count only;
  `ls` shows only `.env.example`), and the variable is not set in this shell. Not provable for a build made elsewhere:
  `pnpm test:bdd:prepare` builds with whatever `VITE_UAR_API_KEY` the build shell has. If it is set, the browser sends
  an `x-api-key` or Bearer on those calls and the local owner is defeated (an invalid `x-api-key` returns 401,
  `middleware.rs:355-373`). The committed `static/` bundle contains no `x-api-key` string at all, so it predates these
  transports; it was not evidence either way.
- An empty `Authorization` value still counts as present: `auth_header` is `Some("")` (`middleware.rs:340-343`), so no
  local owner, and `resolve_authenticated_context` returns anonymous (`middleware.rs:107-121`). The cross-screen
  scenario that sends `Authorization: ''` therefore still gets anonymous and its 401.
- Why this fixes R3-3: chat admission needs a verified principal. `chat_turns.rs:249,294` and `agui.rs:264,381`
  call `CredentialPrincipal::from_verified_context`, which bails for `user_id == "anonymous"` (`claims.rs:268-271`).
  The local owner carries `captured_principal = local_host("local-user")` (`claims.rs:237-248`), so it passes.
  `authorize_context` returns Ok for non-remote profiles (`authority.rs:20-26`).

## Check 2: effect of anonymous becoming the local owner on the S1 steps and features

Both Node seeding and the browser now resolve to the same principal (`local-host`, subject `local-user`), as both
were the same `anonymous` before. Cross-principal splits cannot occur inside one run.

- Global, identity-independent: agents (`/api/agents*`: `discovery.rs:119,342,429`, no user), skills (`/api/skills`,
  `/api/agents/{id}/skills`), providers (`providers.rs:59-77` admin-key gate only; the key is not required in the BDD
  config, and BDD providers carry no `api_key`, so "stored provider credentials need `installation_id`" does not
  apply), settings (`settings.rs` `require_admin_key`, skipped because `SETTINGS_MUTATION_AUTH_REQUIRED=false`; identity
  only matters for governance status). No change for these.
- Owner-scoped, same owner on both sides: knowledge bases (`api/knowledge.rs` uses `session_owner_key()` on every
  route; the runtime retrieval uses `session_owner`, `manager.rs:5744-5748`), sessions and agent-config
  (`discovery.rs:918-921`, `1453-1520`), session effective config. The browser reads what Node seeded because the key
  is identical.
- `/api/uar/sync/stream` (`server.rs:4813-4818`): `session_owner_key()` is Ok for both anonymous and local owner.
- Chat and history: anonymous was refused (above); the local owner is admitted. This is the intended change, and it is
  the first time the nine other chat scenarios and the S1 chat steps get past setup, so unknown faults may appear.
- Policy universe: with a verified owner `eligible_presentations` runs and the KB allow-list is read under the owner key
  (`manager.rs:3057-3075`, `discovery.rs:824-835`); anonymous got none of the verified-owner presentation set. Could change
  the resolved presentation list for a session. The S1 steps do not assert presentations except through
  `/api/uar/settings/presentation-policy` (by key, identity independent).
- S1 steps that could pass or fail for a changed reason: none found by reading. Specific ones checked:
  `directRequest` refusals (live route 404/409, override 422, `llm.default_provider` settings write): all product rules,
  identity independent. `local-first-resilience` "fresh context does not inherit": the frontend lists threads from PGlite
  only and fetches a server session only by id (`services/chat-sessions-api.ts:29`; no list call in `frontend/src`), so
  a fresh browser context under the same server-side owner still sees no thread.
- Feature text: `live-model-choice.feature:17-18` and `live-model-choice-controls.feature:13` say "auth-off test
  configuration" and "send no credentials". Still true (no credentials sent). "Auth-off" is now only partly accurate
  (identity is the local owner). Comment wording only; not edited (not mine).
- Not S1, outside the ten chat scenarios, flagged only: `cross-screen-security` "Global, agent, and user memory" opens
  `/mcp/memory` without a credential. Before: `bind_http_owner` refuses anonymous with 401 (`memory/mcp_server.rs:960-966`).
  After: initialize succeeds as the local owner, then `memory_add` with `user_id: 'screen-owner-a'` is refused because
  `OwnedMemory::check_filter` requires the owner's own subject (`memory/owned.rs:39-45`). It fails both ways, at a
  different step; not a pass-to-fail flip.
- Data note (docs/security/jwt-validation.md "Data note"): anonymous-owned rows become invisible to the local owner. The
  BDD store is a fresh `randomUUID` SurrealKV path per run (`playwright.config.ts:44-45`), so there is no residue.
  Caveat: `reuseExistingServer: !process.env.CI` (`playwright.config.ts:153`). A server already on 3002 without the env
  entry would be reused silently; `test:bdd` sets `CI=1`, so a CI-style run starts its own.
- Memory is enabled in the BDD env (`UAR_MEMORY__ENABLED=true`, local embedding). A verified owner can reach the memory
  paths that an anonymous caller never did; capture needs explicit owner consent (`controls.rs:31-37`) so it should stay
  off, but whether any memory recall or embedding call (`BAAI/bge-small-en-v1.5`, may need a model download) now runs
  on chat turns and adds stub requests was not traced. Unverified.

## Check 3: what the limiter uses at runtime

Boot config only. `server.rs:1986-1991` builds `AppRateLimiter::new(config.resilience.requests_per_second, burst)` once
from the loaded `Config`; `rate_limit_middleware` reads `state.config.resilience.rate_limit_enabled`
(`security/rate_limit.rs:71-76`). The persisted setting (`resilience.burst_size`, `persisted_resilience.rs:26`) feeds only
`resolve_persisted_resilience_policy` (per-run timeouts/retries, `server.rs:3669-3676`, `manager.rs:4676`); the limiter
never reads it. Consequences:
- An install that stored burst 20 as a setting is unaffected either way; the limiter did not use it and is not changed by it.
- An install whose `config.yaml` or env sets `burst_size` keeps that value. Only installs that set none get 100, and only
  after a restart. `packaging/native/default-config.yaml:19` ships `burst_size: 20.0` explicitly, so native installs
  from that file keep 20 and still 429 (not in my ownership).
- The settings UI default shown comes from boot config (`settings/manager.rs:1594`), so it follows the new default.

## Check 4: places that assert or state the old default (report only, not edited)

Stale after the change:
- `frontend/src/features/settings/ui/panels/resilience-settings-panel.tsx:222` `defaultValue="20"` for Burst Size.
  The visible hint will say 20 while the default is 100. Real mismatch; owner uar-console.
- `packaging/native/default-config.yaml:19` `burst_size: 20.0` (explicit, overrides the default for native installs).
- `docs/config/home.uar.embedded-openai.example.yaml:34` `burst_size: 10.0`.
- `.agent/skills/uar-config-wizard/references/config-reference.md:33` states default `20.0`;
  `.../env-vars.md:29` states default `20.0`.
- `openspec/changes/add-configurable-resilience-policies/proposal.md:32` `rate_limit.burst_size: 20`.
- `example.config.yaml:122-124` still says `# Default: 5.0` / `requests_per_second: 5.0`; the real default is 10.0
  (`config.rs:1754`). Pre-existing, left alone because the sustained rate must not change.
- `scripts/behaviors/steps/compiler-owner-isolation.steps.mjs:49-51` comment "(10/s, burst 20)"; it sets its own
  200 burst, comment only.
- `openspec/changes/uiux-schema-controls-and-pickers`: `numeric-bounds-oracle.json:179-181` and `design.md:324`
  use burst_size only as a lower bound (`below 0`, minimum 1, maximum null); unaffected by the default.

Not stale (explicit values, no default asserted): `tests/ingestion_admission.rs:92` (20.0, rate limit disabled),
`tests/settings_persistence.rs:182` (10.0), `src/uar/settings/manager.rs:3674` and `src/uar/api/providers.rs:757` test
literals (10.0), `config.test.yaml` (50), `config.embedded.yaml` (200), `config.remote.*.yaml` (40), `tests/common/test_server.rs:37` (1000).
No Rust, TypeScript or schema test asserts the 20.0 default (grepped `src`, `tests`, `frontend/src`, `scripts`). There is no
upper bound on burst_size in the schema (`manager.rs:1529`), so 100 is valid.

## Risks

- Exposed-server trade-off (operator accepted): up to 100 requests admitted at once before throttling at 10/s. The limiter is
  one process-wide bucket.
- If `VITE_UAR_API_KEY` is set in the BDD build shell, the local owner is defeated (see check 1).
- The local owner has no host authority, no roles, no tenant: any BDD step relying on admin-only host endpoints would be
  refused. None found in S1 or the chat features.
- The Providers page still makes 5 fetches of `/api/uar/providers` per load (noted in the plan, not fixed).

## Unverified

Nothing was compiled or run. Not confirmed: that `config.rs` and `resilience_policy.rs` still compile (single literal
changes), that the env entry yields the owner at runtime, that the nine chat scenarios and S1 steps pass, that memory
paths add no stub traffic. All of that is for the B5 gate.
