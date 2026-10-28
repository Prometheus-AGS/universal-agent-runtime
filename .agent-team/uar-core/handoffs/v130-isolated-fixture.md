# UAR 1.3.0 new isolated fixture handoff

Phase `uar-post-lifecycle-verification-readiness`, Execute; waypoint revision299. Product verification CLOSED. Source inspection and authorized fixture/config/document authorship only. No checks/tests/builds/validators/preflights/service/port/network/Docker operation, token issuance, signing or review gate ran. Others share the checkout; existing edits preserved. Lead owns canonical manifest, inventory and approval packet.

## Authored files

- `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/v1.3.0-fixtures/local-main.yaml`: complete source-backed config selecting main49310, companion gRPC49313, controlled provider/embedding peer49311, OpenAI-compatible384-dimensional embeddings, separate new loopback WS store49312, exact public synthetic JWT issuer/tenant and trusted-host principal mapping, disabled shared MCP connections, memory and risky native tools.
- `.../v1.3.0-fixtures/local-store.yaml`: standalone Compose project/container `uar-post-lifecycle-v130-001`, repository-pinned Surreal3.3.0 digest from production Compose, host loopback49312, `rocksdb:/data/surreal.db`, new owned bind path `target/uar-post-lifecycle-v130-001/store`. No healthcheck/restart/shared service/env file/volume purge.
- `.../v1.3.0-fixtures/local-fixture.env.example`: public nonoperator fixture variables, empty unissued JWT slots, no automatic durability attestation.
- `.../v1.3.0-isolated-fixture-approval.md`: concrete manifest bindings, exact separate setup/start/clean-environment JWT+boundary/stop/down proposals, effects and truthful durability limits, remaining operator decision.
- This handoff.

No product Rust, existing proposed manifest, canonical waypoint/tasks/inventory, historical receipt or operator config was edited. No hashes/check commands were run; lead may bind these bytes as part of authorized inventory authorship.

## Key source findings and integration actions

Read waypoint, historical fixture-recovery handoff/requirements, `scripts/post_lifecycle/main_boundary.py`, current manifest/scenario recipe, `tests/post_lifecycle_main.rs`, `tests/support/post_lifecycle_store.rs`, `src/config.rs`, `src/main.rs`, `src/server.rs`, `src/uar/security/{claims,middleware,sidecar_guard,authority}.rs`, `src/uar/persistence/providers/surreal.rs`, `src/uar/api/{providers,knowledge,accounting}.rs`, `src/uar/domain/knowledge.rs`, `src/llm/local_only.rs`, MCP resolver, `scripts/release/client_tools_local_gate.py`, repo Compose pins, and `.prometheus/gotchas.md`. Source inspection only; incorrect exploratory globs returned path-not-found, with no execution effects.

The existing public fixture `jwt()` helper has canonical issuer/audience/tenant, HS256 claim format and6-hour expiry. Dedicated synthetic owner `uar-post-lifecycle-owner`, foreign owner `uar-post-lifecycle-other`, host `uar-post-lifecycle-host` can be issued only during explicitly approved future launch. Host authority is not an admin role: configured exact `trusted_host_principals` mapping is supported by `HostAuthority::for_service`; no new JWT feature is needed. All values are public synthetic fixture data, no operator credential is needed or generated.

Lead should bind main config and peer dimensions384; store endpoint `ws://127.0.0.1:49312`, namespace/db `uar_post_lifecycle_v130_001`, env names `UAR_BOUNDARY_STORE_USER` / `UAR_BOUNDARY_STORE_PASSWORD`; provider/model `fixture` / `gpt-4o-mini`; KB config and source-framed owner string are provided exactly in approval doc. Do not alter D/E bytes: existing files already carry required `UAR_HOLD:kb-race:` markers.

Append one actual setup request AFTER main health: `POST /api/providers/fixture/test` with bearer owner plus `X-UAR-Admin-Key` from `UAR_BOUNDARY_SETTINGS_ADMIN_KEY`, body `{"model":"gpt-4o-mini"}`, status200 and exact `ok`, `received_text`, provider/model assertions. Route is `/api/providers`, not `/api/uar/providers`. Actual current model fingerprint persistence is required; do not synthesize a verified flag. This request contributes one controlled peer model request, after the initial zero-request readiness observation. Runtime behavior unverified.

Both current Python runner and Rust process launcher inherit environment. Proposed future command constructs the equivalent of `env -i` as an explicit Python allowlist, creates JWTs only in memory from existing helper source, and `execve`s the boundary. It admits only PATH/LANG/LC_ALL and Cargo/Rustup locations; sets private fixture HOME/TMPDIR and `UAR_LOCAL_ONLY`; excludes inherited provider credentials, proxies, config selectors, .env, exporter state and launch credentials. Retain exact helper source binding. No new product environment seam is required.

Remote WS store is ineligible until `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1`. Packet explicitly proposes trusted supervisor attestation for the NEW known rocksdb-backed launch; it does not infer persistence from WS or pretend a runtime durability result. Operator must approve that assertion with the launch. Do not set it globally or borrow any historical attestation.

## Unresolved decisions and unverified state

No product/config seam blocker was established. Remaining operator choice: approve or reject the exact new isolated fixture, its public synthetic local authority, attestation and bounded effects/commands A–E. Lead must reconcile whole-phase state, replacement contract/source/image and exact input bindings, then request approval; keep manifest approval refs/cleanupAuthorized/implementationComplete blocking until that process completes. Existing operator-auth/context absence is no longer a setup blocker for this proposed new fixture.

Port availability, Docker/image availability, parsing, actual auth/provider fingerprint/embedding/KB/CAS/accounting behavior, compilation, process exit and cleanup remain unverified. Gate CLOSED. No acceptance credit or true approval boolean is claimed. Do not run even a config validator or source-review gate without fresh named operator instruction.
