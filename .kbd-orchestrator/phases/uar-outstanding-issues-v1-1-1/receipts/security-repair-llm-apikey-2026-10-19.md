# Security repair: llm.api_key plaintext exposure — 2026-10-19 19:17Z

**Defect (operator-confirmed root cause of this repair):** anonymous
`GET /api/uar/settings` on a running instance exposed the `llm.api_key` value
in plaintext, because the `llm` settings-type schema lacked a sensitive
marker for the `api_key` property.

## Patch (source-only, uncommitted)

- `src/uar/settings/manager.rs` — `build_core_schema`: the `llm.api_key`
  schema property now carries `"x-sensitive": true`. Two class-level
  regression tests added (`mod tests` at EOF):
  - `llm_api_key_schema_is_marked_sensitive`
  - `every_password_control_property_is_sensitive` (invariant: every
    password-type control property in the core schema is marked sensitive)
- `src/uar/api/settings.rs` — one regression test:
  - `llm_api_key_masks_and_preserves_with_patched_schema` (masked GET against
    the patched schema; masked-placeholder preserve and explicit-replacement
    behavior at the handler level, using the minimal `serde_json::from_value`
    config pattern). NOTE (corrected 19:31Z): this test does NOT instantiate
    or read the persisted settings store; it proves handler logic only.
    Persisted-store value proof is the separate assertion below.
- Upgrade path (no migration needed): `SettingsManager::initialize`
  (`manager.rs:203-246`) upserts the settings-type schema on every boot, so
  existing stores receive the patched schema on the next patched-binary boot
  while stored values are preserved. Verified at runtime (below).

## Harness-unblock repairs (test-only; pre-existing stale test code)

`cargo test --lib` did not compile for reasons unrelated to the patch. All
seven errors classified as **stale test initializers/call sites** (none are
production-profile failures; the production lib target compiles clean):

| # | Site | Cause | Fix |
|---|---|---|---|
| 1 | `api/a2a/handler.rs:337` (test) | `VectorMatcher::new` now takes `Arc<SwappableEmbeddingBackend>` | wrap backend |
| 2 | `runtime/presentation_history_tests.rs:34` | same embedding refactor | wrap backend |
| 3 | `runtime/presentation_history_tests.rs:54` | `RunStreamState` gained 6 fields | add `None`/`Instant::now()` fields |
| 4 | `runtime/thread/service_tests.rs:66` | same embedding refactor | wrap backend |
| 5 | `api/settings.rs:1368` (test) | `SettingsApiState` gained `embedding_backend`, `embedding_base_config` | add `None` fields |
| 6 | `rag/chunking.rs:315` (test) | `chunk()` takes `&str` | borrow |
| 7 | `runtime/thread/approval_tests.rs:50` | `request` renamed to `request_tool` with `ToolApprovalRequest` (committed 22088cee) | rebuild helper with exact old-arg mapping (admission_id, call_index 0, tool_call_id, name "write", arguments, risk_reason; owners PairedHost/PairedHost) |

Note: `cargo build` (all targets) also fails in the auxiliary bin
`seed-minimax-verification` (E0382, pre-existing bitrot, unrelated to the
patch). The main binary target builds clean; the aux bin was left as-is.

## Unit verification (operator-authorized targeted run)

`cargo test --lib -- uar::settings::manager uar::api::settings` → **EXIT 0,
9 passed / 0 failed** (752 filtered out). All three named regression tests
confirmed running and passing, plus the pre-existing
`settings_api_masks_and_preserves_sensitive_values` and 5 prompt-caching
admin-boundary tests. Log: `uar-r14/secrep-tests.log` (second run; first run
EXIT 101 = the seven stale-test compile errors above).

## Runtime proof (disposable loopback, synthetic sentinels only)

- Binaries: pre-patch = candidate v4 sha256 `96bf4868…5154`
  (`uar-v4-preserved.bin`); patched = dev-profile build sha256
  `0ecff6b2…d8d`, source = HEAD `38f0b5d8` + dirty tree containing this patch
  and the harness-unblock repairs. Old-binary endpoint behavior is recorded
  only as pre-patch-store setup, never as patch evidence.
- Fixture: 127.0.0.1:1914 (grpc 50154), fresh surrealkv store, `env -i` with
  synthetic `LLM_API_KEY=synthetic-sentinel-ALPHA-0123456789ab` (37 chars),
  loopback mock base URL, no inherited provider credentials. Mock auth server
  on 1916 (token file ALPHA → BETA). Store scrubbed and listeners verified
  closed at 19:17Z. Evidence: `uar-r14/secrep/`, script
  `uar-r14/secrep-proof.sh`. Window 19:09:35Z–19:12Z.

Results:
1. **Schema refresh on pre-patch store: PASS.** v4 boot created the store
   with the old schema; patched binary booted the same store; `initialize`
   upsert applied the patched schema (stored values preserved) — subsequent
   responses mask.
2. **Saved/effective masking: PASS.** `GET /api/uar/settings/llm.api_key`:
   `saved`, `effective`, and `data` are all exactly 37 `*` (full mask, value
   length preserved); `application_status: effective`, `apply: next_turn`.
   `GET /api/uar/settings` list: row present, `data` fully masked; leak-guard
   found no sentinel substring in any response body.
3. **Masked-placeholder update cycle: PASS (settings layer).** PUT with the
   exact 37-`*` placeholder → 200; revision advanced
   (`…19:09:35.050…` → `…19:10:31.250…`); row still masks 37 — consistent
   with preservation. A 200 + revision change + unchanged mask length does
   NOT by itself prove the stored value (corrected 19:31Z); the persisted
   proof is item 6.
4. **Explicit synthetic replacement: PASS (settings layer).** PUT BETA
   (26 chars) → 200; row masks exactly 26 `*` — replacement reflected.
6. **Persisted-store assertions: PASS (the value proof).** On the isolated
   fixture (fresh store, env-seeded synthetic ALPHA, patched binary
   `0ecff6b2…d8d`, instance stopped before each out-of-band read to respect
   surrealkv single-writer): real masked-placeholder PUT → 200, then
   `SELECT key, data FROM settings WHERE key='llm.api_key'` via the surreal
   CLI against the on-disk store returned the row with data equal to ALPHA —
   `alpha_preserved_after_placeholder: true`. Then a real explicit-replacement
   PUT (BETA) → 200, and the persisted row's data equaled BETA —
   `beta_persisted_after_replacement: true`. Equality booleans only; the only
   values read were the known synthetics. Evidence:
   `uar-r14/persist-assert/`, script `uar-r14/persist-assert.sh`; window
   19:29–19:30:41Z. Table `settings`, namespace/db `uar`/`uar`
   (persistence defaults, `config.rs:979/983`). Boot sequencing relies on
   `initialize()`'s API-write preservation (`manager.rs:274-286`: rows with
   `updated_at` set survive reseed with drift marking).
5. **Functional chat oracle: NOT AVAILABLE — recorded as non-evidence, not
   failure.** `/v1/chat/completions` returned 422 `model_not_verified`
   (the lane requires a verified provider/model). Source inspection then
   established that in this build the `llm.api_key` row has **no live
   per-request consumer**: boot `LlmConfig` is cloned from config layering
   (`server.rs:491`) and providers snapshot credentials at seed time
   (`llm/registry.rs:288`, `seed_from_llm_config`); no runtime/llm code reads
   the stored row back. A chat probe would therefore exercise a provider's
   snapshot key, never the row — it cannot prove row preservation and was
   not pursued as evidence. Row value semantics (preserve on placeholder,
   update on explicit value) are proven by the persisted-store assertions in
   item 6; runtime masking/refresh/replacement reflection are proven by
   items 1–4.

## Boundaries honored

- Synthetic sentinels only; the real credential never entered any env,
  config, store, log, or receipt. Instances 1907/1908 untouched. No
  full-release build, no publication, no rotation (operator action).
- Prior receipts remain bound to their original binaries; only the checks
  affected by this patch (settings masking family) are evidenced against the
  patched binary above.

## Residual risk / operator actions

- The real key remains exposed on any running **unpatched** instance
  (including 1907) until the patched build replaces it — rotation after
  rollout is the operator's call.
- `apply: next_turn` on `llm.api_key` has no honoring consumer in this build
  (row changes do not reach the running driver without reboot) — reported as
  an observed behavior gap, not patched (out of this repair's scope).
