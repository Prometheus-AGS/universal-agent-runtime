# Spec conformance verification

Command used for every matrix result:

```bash
UAR_LIVE_INTEGRATION_BACKEND=recorded cargo test --locked \
  --no-default-features --features server-full --test integration \
  live::capability_cases -- --test-threads=1
```

The evidence is limited to the `server-full` profile, the recorded in-process
LLM stub, and the test database created by the live integration harness. It is
not a runtime-level verdict.

Final baseline Tier 2 confirmation: the unchanged, formatted source completed
the pinned matrix with exit 0 and all 20 cases passing in 196.61s at
2026-08-09T18:16:05Z.

Final conformance Tier 2 confirmation: the formatted source completed the
pinned matrix with exit 0 and all 29 cases passing in 288.73s at
2026-08-10T14:44:56Z.

Two intervening attempts are not conformance results: C-13 and C-10 each once
timed out waiting for their test server to become healthy before reaching a
capability assertion. Both cases passed on the unchanged-source retries.

## Evidence-label audit

| case | before | after | basis |
|---|---|---|---|
| `l3_c03_provider_registry` | L3 | L3 (unchanged) | Lists the runtime-owned provider registry with no model fixture. |
| `l3_c05_knowledge_base_catalog` | L3 | L3 (unchanged) | Lists the runtime-owned knowledge catalog with no model fixture. |
| `l3_c08_tools_registry` | L3 | L3 (unchanged) | Lists the runtime-owned tool catalog with no model fixture. |

No case changed labels in this audit.

## Per-capability result

| capability | evidence level | result | evidence and limit |
|---|---|---|---|
| C-01 | L2 | pass | Run lifecycle shares the recorded-provider stream case; fixture-authored model output. |
| C-02 | L2 | pass | AG-UI stream shape shares the recorded-provider run case; fixture-authored model output. |
| C-03 | L3 | pass | Runtime-owned provider catalog; no model fixture. |
| C-04 | L3 | pass | Real credential handler enforces its unauthenticated guard; encrypted multi-user behavior is not claimed. |
| C-05 | L3 | pass | Runtime-owned knowledge-base catalog; retrieval relevance is not claimed. |
| C-06 | shape-only | pass | Memory stats handler shape on a fresh temporary store; retention is not claimed. |
| C-07 | L3 | pass | Runtime-owned skills catalog under `server-full`; no transfer to `embedded-mobile`. |
| C-08 | L3 | pass | Runtime-owned tools catalog; model tool selection is not claimed. |
| C-09 | L3 | pass | Compiler spec/session catalogs return correct empty collections on a fresh database. |
| C-10 | L3 | pass | Runtime settings handler returns a JSON settings surface. |
| C-11 | L3 | pass | Runtime-owned A2UI schema catalog. |
| C-12 | L4 | pass | A knowledge-base resource survives a normal shutdown and cold process restart on the same SurrealKV path; a different-path negative control returns 404. |
| C-13 | absent + durability exclusion | pass | The legacy sessions route is retired; the current `X-UAR-Session-ID` chat contract works, but its in-memory `SessionStore` does not survive a cold process restart. |
| C-14 | L2 | pass | OpenAI-compatible response shape against the recorded provider; no live-provider claim. |
| C-15 | L3 | pass | Static well-known agent descriptor contract. |
| C-16 | L2 | pass | Cedar governance middleware with repository policy fixtures. |
| C-17 | L3 | pass | Configured authentication posture on a real protected handler. |
| C-18 | L3 | pass | Exact text extraction through the real multipart upload handler; binary OCR is not claimed. |
| C-19 | L2 | pass | Shipped eval suite through the real runner with a test-authored completion provider. |
| C-20 | L3 | pass | Real health, readiness, and metrics handlers. |
| C-21 | excluded | pass | Harness cannot create two tenant identities or target the same resource across them. |
| C-22 | absent exclusion | pass | Scheduled/event-initiated runs are documented absent; caller-initiated runs only. |
| C-23 | absent exclusion | pass | Peer reachability is documented absent in UAR. |
| C-24 | excluded | pass | Peer mesh requires two independently-addressable devices; this harness boots one. |
| C-25 | excluded | pass | UAR has no `frf-did` dependency or node-identity call path. |
| C-26 | excluded | pass | UAR consumes neither `frf-did` nor `frf-wallet`; offline verification is unreachable. |
| C-27 | excluded | pass | UAR has no `frf-wallet` dependency or delegation call path. |

| case | capability | evidence level | result | evidence | timestamp |
|---|---|---|---|---|---|
| `l3_c20_health_readiness_metrics` | C-20 | L3 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l3_c07_skills_catalog` | C-07 | L3 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l3_c10_settings_surface` | C-10 | L3 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l3_c11_a2ui_schema_registry` | C-11 | L3 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l3_c08_tools_registry` | C-08 | L3 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l3_c03_provider_registry` | C-03 | L3 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l2_c14_openai_compatible_surface` | C-14 | L2 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l2_c01_c02_run_stream_shape` | C-01 / C-02 | L2 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l4_c12_persistence_round_trip` | C-12 | L4 | pass | local pinned matrix; exit 0; 29/29; 288.73s; same-path resource matched after two normal child-process boots; caller token stopped HTTP before SIGTERM completed unchanged root shutdown | 2026-08-10T14:44:56Z |
| `l4_c12_persistence_round_trip__different_path_negative_control` | C-12 | L4 negative control | expected fail | targeted local command with `UAR_L4_NEGATIVE_CONTROL_DIFFERENT_PATH=1`; exit 101; second boot returned 404 instead of 200 for the created resource | 2026-08-10T14:44:56Z |
| `l3_c04_credentials_listing` | C-04 | L3 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l3_c05_knowledge_base_catalog` | C-05 | L3 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `shape_only_c06_memory_stats` | C-06 | shape-only | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l3_c09_compiler_specs` | C-09 | L3 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l3_c09_uar_compiler_specs` | C-09 | L3 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l3_c09_compiler_sessions` | C-09 | L3 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l3_c15_agent_descriptor_well_known` | C-15 | L3 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l3_c17_security_posture` | C-17 | L3 | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `absent_c13_sessions_retired` | C-13 | absent exclusion | expected fail | local pinned matrix; exit 101; 19/20; deliberate expected 418, actual 404 | 2026-08-09T17:39:53Z |
| `absent_c13_sessions_retired` | C-13 | absent exclusion | pass | local pinned matrix after revert; exit 0; 20/20; 191.54s | 2026-08-09T17:56:19Z |
| `excluded_c13_session_continuity_is_not_durable` | C-13 | durability exclusion | pass | local pinned matrix; exit 0; 29/29; current chat contract succeeded before and after reboot, while context stats returned 404 after the same-path cold restart because `SessionStore` is in-memory | 2026-08-10T14:44:56Z |
| `absent_c22_scheduled_runs` | C-22 | absent exclusion | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `absent_c23_peer_reachability` | C-23 | absent exclusion | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `l2_c16_governance_middleware` | C-16 | L2 | pass | local pinned matrix; exit 0; 28/28; 274.70s; repository Cedar policy fixture | 2026-08-10T10:16:23Z |
| `l3_c18_text_file_processing` | C-18 | L3 | pass | local pinned matrix; exit 0; 28/28; 274.70s; exact text extraction through upload handler | 2026-08-10T10:16:23Z |
| `l2_c19_eval_runner` | C-19 | L2 | pass | local pinned matrix; exit 0; 28/28; 274.70s; shipped suite through real runner with test provider | 2026-08-10T10:16:23Z |
| `excluded_c21_tenant_isolation_no_cross_read_surface` | C-21 | excluded | pass | local pinned matrix; exit 0; 28/28; 274.70s; harness cannot create two tenant identities or target one resource across them; real credential guard discriminated | 2026-08-10T10:16:23Z |
| `excluded_c24_peer_mesh_requires_two_devices` | C-24 | excluded | pass | local pinned matrix; exit 0; 28/28; 274.70s; requires two independently-addressable devices; real health handler discriminated only | 2026-08-10T10:16:23Z |
| `excluded_c25_node_did_not_consumed_by_runtime` | C-25 | excluded | pass | local pinned matrix; exit 0; 28/28; 274.70s; UAR declares no frf-did dependency or node-identity surface | 2026-08-10T10:16:23Z |
| `excluded_c26_did_resolution_and_vc_verification_not_consumed` | C-26 | excluded | pass | local pinned matrix; exit 0; 28/28; 274.70s; UAR consumes neither frf-did nor frf-wallet | 2026-08-10T10:16:23Z |
| `excluded_c27_wallet_not_consumed_by_runtime` | C-27 | excluded | pass | local pinned matrix; exit 0; 28/28; 274.70s; UAR declares no frf-wallet dependency or delegation surface | 2026-08-10T10:16:23Z |
