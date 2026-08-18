# Negative-control evidence

Date: 2026-08-15
Profile: `server-full` only. These results transfer to no other profile.

The reconciliation candidate before the first two inversions was:

```text
96920a864e47f267126849b245d77c1cfd8ff52b2fc99d5cc790de5b05a74472  src/uar/runtime/skills/service.rs
21e890e94eb60b7a0b731e9b3e86f6ee745a7a65d26e62a68b7a4a7be2b0eb6d  -
```

## Empty-source fail-safe inversion

The inversion removed only the branch that returns when the configuration source
is empty and durable `fs-skills` records exist.

Command:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --lib \
  uar::runtime::skills::service::tests::empty_source_fail_safe_preserves_every_skill_origin \
  -- --exact --test-threads=1
```

Actual output (exit 101):

```text
running 1 test
test uar::runtime::skills::service::tests::empty_source_fail_safe_preserves_every_skill_origin ... FAILED
thread 'uar::runtime::skills::service::tests::empty_source_fail_safe_preserves_every_skill_origin' panicked at src/uar/runtime/skills/service.rs:1327:9:
assertion failed: stored.iter().all(|skill| !skill.tombstoned)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 586 filtered out; finished in 0.20s
error: test failed, to rerun pass `-p universal-agent-runtime --lib`
```

## Provider discriminator inversion

The inversion removed only `.filter(|skill| skill.provider_id == "fs-skills")`
from the tombstone candidate iterator.

Command:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --lib \
  uar::runtime::skills::service::tests::api_skill_survives_empty_configuration_by_provider_id \
  -- --exact --test-threads=1
```

Actual output (exit 101):

```text
running 1 test
test uar::runtime::skills::service::tests::api_skill_survives_empty_configuration_by_provider_id ... FAILED
thread 'uar::runtime::skills::service::tests::api_skill_survives_empty_configuration_by_provider_id' panicked at src/uar/runtime/skills/service.rs:1370:9:
assertion failed: !api_skill.tombstoned
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 586 filtered out; finished in 0.17s
error: test failed, to rerun pass `-p universal-agent-runtime --lib`
```

## Reconciliation-candidate restoration

After both inversions were removed, the hashes returned exactly to:

```text
96920a864e47f267126849b245d77c1cfd8ff52b2fc99d5cc790de5b05a74472  src/uar/runtime/skills/service.rs
21e890e94eb60b7a0b731e9b3e86f6ee745a7a65d26e62a68b7a4a7be2b0eb6d  -
```

The affected service slice was then rerun:

```text
test uar::runtime::skills::service::tests::api_skill_survives_empty_configuration_by_provider_id ... ok
test uar::runtime::skills::service::tests::empty_source_fail_safe_preserves_every_skill_origin ... ok
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 576 filtered out; finished in 0.82s
```

The independent review then found four additional reachable visibility and
provenance failures. The corrected final candidate before those inversions was:

```text
9956eca135efc92ea4629269a14e04d7f5c83135c53a684f890635bd563c6a14  src/uar/runtime/skills/registry.rs
cd9324777bbfe070e5f48746e835bf91adc885442bb6d6de158c622bfcc82303  src/uar/runtime/skills/service.rs
dbb88829501b32924988b81a933153695927bfc3ba57953f546c472504f79585  src/uar/runtime/skills/storage/filesystem.rs
ef22852b7ed647a02669f591fa7b8c03d45381209c96ff911ee3aaa196c3ea64  -
```

## Vector candidate-limit inversion

The inversion restored the old fixed database limit of five before tombstone
filtering.

Command:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --lib \
  uar::runtime::skills::registry::tests::vector_candidates_include_tombstones_before_visibility_filtering \
  -- --exact
```

Actual output (exit 101):

```text
running 1 test
test uar::runtime::skills::registry::tests::vector_candidates_include_tombstones_before_visibility_filtering ... FAILED
thread 'uar::runtime::skills::registry::tests::vector_candidates_include_tombstones_before_visibility_filtering' panicked at src/uar/runtime/skills/registry.rs:355:9:
assertion `left == right` failed
  left: 5
 right: 6
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 590 filtered out; finished in 0.00s
error: test failed, to rerun pass `-p universal-agent-runtime --lib`
```

## Refresh visibility inversion

The inversion returned raw provider rows from `SkillService::refresh` instead of
filtering tombstones from the API-facing result.

Command:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --lib \
  uar::runtime::skills::service::tests::refresh_hides_tombstones_but_keeps_them_available_for_restore \
  -- --exact
```

Actual output (exit 101):

```text
running 1 test
test uar::runtime::skills::service::tests::refresh_hides_tombstones_but_keeps_them_available_for_restore ... FAILED
thread 'uar::runtime::skills::service::tests::refresh_hides_tombstones_but_keeps_them_available_for_restore' panicked at src/uar/runtime/skills/service.rs:969:9:
assertion `left == right` failed
  left: 2
 right: 1
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 590 filtered out; finished in 0.19s
error: test failed, to rerun pass `-p universal-agent-runtime --lib`
```

## Dynamic-storage boundary inversion

The inversion removed the provider-id check at the filesystem write boundary.

Command:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --lib \
  uar::runtime::skills::storage::filesystem::tests::dynamic_storage_rejects_non_api_skills \
  -- --exact
```

Actual output (exit 101):

```text
running 1 test
test uar::runtime::skills::storage::filesystem::tests::dynamic_storage_rejects_non_api_skills ... FAILED
thread 'uar::runtime::skills::storage::filesystem::tests::dynamic_storage_rejects_non_api_skills' panicked at src/uar/runtime/skills/storage/filesystem.rs:364:14:
configuration-managed skill must not enter dynamic storage: ()
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 590 filtered out; finished in 0.01s
error: test failed, to rerun pass `-p universal-agent-runtime --lib`
```

## Stale-dynamic precedence inversion

The inversion made a stale dynamic copy win over the real configuration source
regardless of filesystem traversal order.

Command:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --lib \
  uar::runtime::skills::storage::filesystem::tests::configuration_source_wins_over_a_stale_dynamic_copy \
  -- --exact
```

Actual output (exit 101):

```text
running 1 test
test uar::runtime::skills::storage::filesystem::tests::configuration_source_wins_over_a_stale_dynamic_copy ... FAILED
thread 'uar::runtime::skills::storage::filesystem::tests::configuration_source_wins_over_a_stale_dynamic_copy' panicked at src/uar/runtime/skills/storage/filesystem.rs:410:9:
assertion `left == right` failed
  left: "api"
 right: "fs-skills"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 590 filtered out; finished in 0.01s
error: test failed, to rerun pass `-p universal-agent-runtime --lib`
```

## Final-candidate restoration

After all four reviewer-found inversions were removed, the three source hashes
and their combined diff hash returned exactly to the values recorded above. The
complete skills slice then observed `46 passed; 0 failed; 545 filtered out`.

The later addition of a test-only tracing subscriber, required to observe the
already-implemented empty-source error log, did not alter any inverted guard.
The final candidate hashes are registry `9956eca1...c6a14`, service
`97432069...ca0a`, filesystem `dbb88829...79585`, and combined diff
`f6f9e872...ac72f`; the complete skills slice again observed 46 passing and 0
failed.
