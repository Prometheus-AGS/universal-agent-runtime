# R5 customer budget followup — authored, unexecuted

Phase: `uar-outstanding-issues-v1-1-1`, Execute. The full-phase verification
boundary remains closed. This handoff transfers authored fixtures to the lead;
it does not claim passing integration, independent review, or image acceptance.

## Observed requirement and source evidence

Re-read public issue comment with:

`gh api repos/prometheus-ags/universal-agent-runtime/issues/comments/6213136510`

The command returned the customer's two successive causes: remote Surreal was
not attested, then the exact `max_tokens_per_turn: 16000` ceiling could not admit
the conservative model reservation. Removing the ceiling allowed v1.1.0 calls
of 141–187 seconds under a 240-second timeout. Public source:
https://github.com/Prometheus-AGS/universal-agent-runtime/issues/385#issuecomment-6213136510

Source inspection of `build.rs`, `catalog/provider_catalog.json`, and
`src/uar/runtime/cost_budget.rs` establishes that the build embeds the catalog,
and `begin_call` without a final-wire contract uses `ModelCatalog::global()`.
The pinned `openai/gpt-4o-mini` entry has context 128000 and max output 16384,
for a 144384-token reservation. Its input/output prices are 0.15/0.6. Configured
provider metadata cannot substitute for this global-catalog bound. The fixture
does not override catalog limits, lower reservation, remove budget policy, or
claim a final-wire contract.

The uncomfortable correction: the former installed-image unsupported-storage
fixture selected PostgreSQL, but current `PostgresProvider` explicitly supports
durable accounting and reports `LocalDurable`. That claim was wrong and has been
removed from the mandatory image inventory and corrected in `R5-integration.md`.

## Exact changed paths and interfaces

- `scripts/release/uar_1_1_1_runtime.py`: removes the PostgreSQL unsupported row;
  adds mandatory `binding-16000-catalog-reservation-refusal` in `binding` and
  implements its actual public-host/store/provider assertions. Extends existing
  `binding-unattested-refusal` with a persisted bounded agent and startup warning.
- `tests/fixtures/uar_1_1_1_provider.py`: adds only `catalog_budget` OpenAI mode,
  complete text plus explicit terminal usage (one prompt token, one completion
  token) for durable settlement of the adequate-ceiling control.
- `tests/support/uar_1_1_1_runtime_fixture.rs` (new file from the original R5
  implementation): adds actual InMemoryProvider unsupported-storage execution to
  existing `binding_refusals`; feature availability is asserted, import is gated.
- `coordination/handoffs/R5-integration.md`: corrects the stale PostgreSQL claim.
- This handoff, `coordination/handoffs/R5-budget-followup.md`.

No production runtime, shared runner/support, dependency, pin, vendor or manifest
was edited in this followup. The provider has synthetic fixture credentials only;
retained JSON receipts are redacted and mode 0600 under the private evidence root.

## Mandatory assertion-backed behavior

`binding-16000-catalog-reservation-refusal` starts an actual installed runtime with
eligible attested remote Surreal and administratively verified controlled OpenAI
HTTP peer. It requires all of the following before emitting a passed row:

1. Public `/api/models` reports exact pinned limits 128000/16384 and positive
   prices; their sum is 144384, greater than 16000.
2. A finite 1000000-token positive control settles done, contacts the actual
   provider exactly once, and durably records reserved_tokens 144384 plus settled
   terminal usage of two tokens. The earlier Anthropic durable positive/restart
   control also remains in the mandatory inventory.
3. Real `POST /api/agents` preserves exactly `max_tokens_per_turn: 16000`.
   Real run admission/stream/status/AG-UI settles error. Public `agui.error` has
   code `provider_error` and safe message `budget_admission_failed`.
4. Trusted ERROR for that exact run has stage `model_call_admission`, category
   `budget_admission_failed`, typed cause `accounting_budget_exhausted`, a UUID
   diagnostic reference, source `catalog_context_and_output`, reserved_tokens
   144384 and configured_turn_token_limit 16000. Fixture key must be absent from
   public stream and logs. This consumes the lead's production diagnostic edit.
5. Provider request count does not increase. Actual scoped Surreal SQL shows no
   new accounting call, identical earlier call records and scope values, one new
   failed-run scope retaining limit 16000, and zero settled/reserved costs/tokens,
   pending calls and unknown calls in that scope.
6. Actual runtime restart leaves all accounting snapshots equal; real persisted
   agent GET still reports exact limit16000 and no refused request is replayed.

`binding-unattested-refusal` now persists an exact16000 agent using public HTTP,
restarts the actual runtime, reads the agent back unchanged, and requires a new
startup WARN with category `budget_durable_storage_unavailable` and positive
`bounded_agent_count`. It counts only warnings added by that restart. Startup
must not contact the provider. The existing safe binding stage/reference,
`remote_surreal_durability_not_attested` log and zero-contact assertions remain.

Unsupported storage is covered by the existing exact source case:

`phase_runtime_binding_refusals_use_real_host_store_and_provider`

It constructs actual `InMemoryProvider`, asserts its actual unsupported durable
eligibility, then runs the real manager with verified identity, finite budget and
real HTTP provider peer. Required outcome is error, safe `durable_accounting`
stage plus reference, no leaked marker/key and zero provider contact. The real
SurrealKV positive and other refusal controls remain. This is source-only evidence,
never a published-image row. No new case name or runner registration is required.

## Verification ownership and unresolved limits

No test, check, build, syntax/import probe, lint, formatter or review gate was run.
Only source reads, public issue retrieval and file authoring occurred. Runtime
behavior and compileability remain unverified; fresh independent review is pending.
The assertions are deliberately strict and may reveal an actual integration gap.

After the lead records full phase implementation completion, the existing local
six-group runner `scripts/release/uar_1_1_1_acceptance.py --manifest <lead manifest>`
will consume the updated closed inventory. Its binding supplemental command is:

```sh
UAR_LOCAL_ONLY=1 cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test bauar_full_harness_cursor phase_runtime_binding_refusals_use_real_host_store_and_provider -- --exact --test-threads=1
```

The lead runner also supplies its dedicated target directory and private resource/
evidence roots, and requires actual `1 passed; 0 failed`. This command is recorded
for the future gate and was not executed. Installed cases need the lead's actual
image, Docker, disposable Surreal service, RFC1918 relay and scoped fixture SQL;
there is no additional external account, paid model or real credential prerequisite.

No production diagnostic gap remains identified by source inspection: the lead's
current `server.rs` startup WARN and `cost_budget.rs` typed per-call ERROR expose
the fields consumed here, while the existing public safe message remains intact.
Actual observability is still subject to the mandatory real integration gate.
The previous summary-capture limitation remains unchanged: disabled production
summary inference is not claimed as helper-model execution.
