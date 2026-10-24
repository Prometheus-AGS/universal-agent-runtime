# UAR 1.3.0 fixture-reference recovery — source only

Phase `uar-post-lifecycle-verification-readiness`, Execute. Waypoint source revision299 / planRevision100 records packet preparation only and tasks5.1/5.2 pending. Verification gate CLOSED. This handoff neither opens a gate nor claims fixture readiness. Others share the checkout; this task owns only this handoff and preserves their edits.

## Recovered references

All paths below are relative to `/Users/gqadonis/Projects/prometheus/universal-agent-runtime` unless absolute. File references were inspected read-only; no referenced fixture was started or reconfigured.

| Existing artifact | Safe source facts | Reuse limit |
|---|---|---|
| `target/client-tools-local-gate-019/config.yaml` | Present. Historical main loopback port53191; provider peer loopback53187/v1. Provider ID `fixture`, model `gpt-4o-mini`, qualified route `fixture/gpt-4o-mini`; separate `openai` provider for catalog-budget controls. Embedding backend `openai`, model `text-embedding-3-small`, vector dimension384 and same local peer `/v1`. | Concrete historical isolated config reference, not an approved1.3.0 config. Old ports are historical; no availability assertion. |
| Same config persistence | Provider `surreal`, embedded `surrealkv://…/target/client-tools-local-gate-019/store/uar.db`; namespace/database `client_tools_gate`; dimension384; memory disabled. | Incompatible unchanged input for post-lifecycle helpers, which require loopback WebSocket and dedicated `uar_post_lifecycle_` namespace/database. No mode substitution or old-store mutation authorized. |
| `target/client-tools-local-gate-019/model-verification.json` | Retained `ok=true`, provider `fixture`, model `gpt-4o-mini`. | Historical1.2.3 route proof only; exact config/endpoint/store fingerprint differs from proposed1.3.0 fixture. Do not author a verification status into new state. |
| `target/client-tools-local-gate-019/budget-model-verification.json` | Retained `ok=true`, provider `openai`, model `gpt-4o-mini`. | Same historical limitation. |
| `target/client-tools-local-gate-019/agent.json`, `budget-agent.json`, `graph-worker-after.json` | Existing artifacts identify the same fixture/OpenAI model routes. | Historical exact records; do not select their IDs as new owned rows or inherit mutation authority. |
| `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/client-tools-local-gate-019-receipt.json` | Version1.2.3, SurrealKV, owned controlled LLM/MCP peer, retained core95/safety60/cancel5 results and extended/cold recovery groups; exit2 explicitly means remaining scope excluded by finite contract; receipt records clean shutdown. | Historical receipt facts, not rerun or1.3.0 acceptance. |
| `target/client-tools-local-gate-019/local-gate.json` | Historical evidence root and `owned_shutdown_clean=true`. | No live-process/service claim. |
| `scripts/release/client_tools_local_gate.py` | Source-backed isolated config constructor `configuration()`, clean child environment and owned lifecycle supervisor. Constants `PROVIDER_ID=fixture`, `MODEL_ID=gpt-4o-mini`; embedding dimension384. Uses private HOME/TMPDIR, `UAR_LOCAL_ONLY`, `MCP_CONFIG_PATH`; copies shipped policies and provisions fresh isolated state. | Different gate and authority model: generates isolated auth/config and embedded store, actually performs provider verification, and drives all-five acceptance. Do not invoke it as an unapproved preflight or transplant its provisioning into the main-boundary packet. |
| `openspec/changes/openai-client-tools/handoffs/local-gate.md` | Records exact setup source, model selector repair, controlled provider proof, dimension384, owned cleanup and explicit noncertification limits. | Useful design/reference source, not current permission. |
| `/Users/gqadonis/.uar/config.yaml` | Present, mode0600. Only safe anchored model/ns/db fields were inspected: namespace/database `uar`, existing model `groq/llama-3.3-70b-versatile`. Historical phase analysis separately attributes loopback WebSocket endpoint127.0.0.1:28000. | Operator config, not an isolated fixture or authority to modify it. Current endpoint/durability/config identity was not recovered by reading credential values or probing service. Existing real provider must not receive fixture traffic or new spend. |

The last source reference reduces embedding dimension ambiguity for the old controlled fixture to384. It does not prove that384 is the operator-selected dimension for the current post-lifecycle config; that choice must remain consistent with its exact existing KB configuration and controlled peer.

## Existing variable names, without values

The current post-lifecycle recipes name `UAR_BOUNDARY_OWNER_AUTHORIZATION`, `UAR_BOUNDARY_OTHER_OWNER_AUTHORIZATION`, `UAR_BOUNDARY_EXISTING_HOST_AUTHORIZATION`, `UAR_BOUNDARY_EXISTING_WORKSPACE`. The older caller-tools supervisor passes different child-only names, `UAR_CLIENT_TOOLS_GATE_OWNER` and `UAR_CLIENT_TOOLS_GATE_OTHER`; it generates private isolated auth rather than demonstrating current boundary-name availability. No aliasing, token copy, reuse or fresh issuance was performed or inferred.

`src/config.rs` provides `surreal_user`/`surreal_pass` fields, documented `UAR_MEMORY__SURREAL_ENDPOINT`, `UAR_MEMORY__SURREAL_USER`, `UAR_MEMORY__SURREAL_PASS`, and explicit `UAR_PERSISTENCE__DATABASE_URL` override handling. These are source names, not evidence any value is set or suitable. The main-boundary manifest's `store.userEnv`/`passwordEnv` still need the actual operator-approved existing env names. No environment inventory or credential-value read was run. Source `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED` exists; this handoff does not assert its value, change it, or authorize an attestation substitute.

## Real unresolved operator/lead inputs

1. Identify the unchanged existing loopback WebSocket store endpoint, actual permitted dedicated namespace/database and existing credential env names. The old embedded-store fixture does not satisfy this contract.
2. Select an exact permitted existing owner/other-owner/workspace/host identity context; confirm how names in current recipes resolve without reading or copying token values. Existing isolated generated credentials are not automatically reusable or current authority.
3. Bind an actual isolated main config and source-supported provider/model/embedding route to the owned controlled peer. Old `fixture/gpt-4o-mini` plus dimension384 is a concrete historical reference, not proof that this route is currently registered in the selected store/config. No new provider registration/default mutation is implicit.
4. Retain the existing contract body recovery blocker and final whole-phase inventory requirements. Old route proofs cannot fill a missing adopted contract or approve execution.

No additional product source gap was established by these file inspections. If adapting the old setup requires new provisioning or a store mode change, stop for explicit scope decision; do not hide it inside a readiness operation.

## Handoff accounting

Only `.agent-team/uar-core/handoffs/v130-fixture-recovery.md` was written. No credentials file, token value, database file, process log body or provider trace body was opened. Metadata listings showed historical config/receipt/state locations; safe field-filtered reads supplied the table above. Source/file inspection commands were `cat`, `rg`, `ls`; the initial attempted `client-tools-local-gate-019.json` read/search named a nonexistent path, corrected to the existing `client-tools-local-gate-019-receipt.json`. No verification command, network/service/port/environment/import probe, Cargo, test, build, Docker or external action ran. Current setup, route availability, owner authority, durable eligibility and1.3.0 behavior remain unverified because the gate is CLOSED.
