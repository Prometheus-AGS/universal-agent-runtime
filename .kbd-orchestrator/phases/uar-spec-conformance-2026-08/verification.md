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

Two intervening attempts are not conformance results: C-13 and C-10 each once
timed out waiting for their test server to become healthy before reaching a
capability assertion. Both cases passed on the unchanged-source retries.

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
| `shape_only_c12_persistence_config` | C-12 | shape-only | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
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
| `absent_c22_scheduled_runs` | C-22 | absent exclusion | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
| `absent_c23_peer_reachability` | C-23 | absent exclusion | pass | local pinned matrix; exit 0; 191.54s | 2026-08-09T17:56:19Z |
