# #463/#464 Linux amd64 local fixture routing research

Source research only. No runtime, network, container, credential, test, build, or verification operation was run. Behavior remains unverified.

## Finding

The repository contains a configurable `uar-jwt-proxy`, but source does not establish that a second proxy or candidate UAR instance is already provisioned. It supports a separate local route when its external owner runs/configures it with a distinct listen address and an explicit candidate upstream. The proxy accepts `PROXY_LISTEN` / `--listen` (default `127.0.0.1:8088`), `PROXY_UPSTREAM` / `--upstream`, `PROXY_JWT_TENANT_ID`, and UAR JWT claim settings; `Cli` and `AppState::mint_token` show the tenant claim is carried into minted identity. See `tools/uar-jwt-proxy/src/main.rs` (`Cli`, `Claims`, `AppState::mint_token`, `main`) and [`docs/dev-tools.md`](docs/dev-tools.md) (options table). The documentation's older generic statement describes JWT-required local development; the current operator boundary in root `AGENTS.md` says active UAR has `jwt_required=false`, team APIs still need verified owner context, and external `uar-jwt-proxy` supplies that owner and tenant. Tests must not implement auth plumbing.

A test can target an externally provided endpoint through `UAR_BASE_URL` and `UAR_WORKSPACE_ID`; `initialize` in `scripts/behaviors/support/capability-fixture.mjs` requires those plus an external manifest. `safeUrl` restricts the fixture endpoint and peer URLs to explicit HTTP(S) literal loopback URLs, without credentials/query/fragment; `http` attaches the nonsecret workspace header and forbids redirects. The fixture README requires the identity endpoint preserve a stable verified owner/tenant/workspace and have external authority for its documented API operations (`scripts/behaviors/fixtures/capability-dispatch/README.md`, “Exact remaining external prerequisites”).

## Independent routing prerequisites and ownership

- External host must own/provision a fresh candidate Linux amd64 UAR container, its isolated Surreal volume/settings namespace, and its local host port mapping. The fixture README expressly says the tests do not launch/restart containers and a pre-existing operator volume is not a disposable registry.
- External host must own a distinct loopback identity-proxy listener and point its upstream at that candidate's mapped UAR endpoint. Existing proxy CLI/environment has both listener and upstream selectors; no source indicates an already-running second instance or safe way to retarget another chat's instance.
- External host controls the matching candidate identity configuration and proxy tenant context. The proxy source exposes a tenant selector and secret selector; do not read, relay, or place secret values in fixture artifacts or test inputs.
- Set test `UAR_BASE_URL` to that externally owned proxy route and use a nonsecret `UAR_WORKSPACE_ID`. The fixture additionally needs its owned JSON manifest, exact source/image/config/contract identity receipts, controlled keyless provider registrations and peer topology as specified in its README. The host owns the peers, shared owned network namespace, and container-to-peer routing; container and host loopback are distinct.
- Preserve the other chat's current listener, upstream, process, container, and state. No evidence in these sources permits changing or borrowing them.

This establishes source-supported configuration interfaces only. It does not prove a second instance/route currently exists, that an endpoint is reachable, or that owner/tenant context works on any candidate.
