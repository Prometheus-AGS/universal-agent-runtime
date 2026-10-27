# Provider spend pools — source implementation handoff

Gate CLOSED. Source inspection and authorized source/docs writes only. No imports,
tests, compiler/build/lint/format/check/schema validation, review verification,
ledger execution, service/Docker/provider or model calls ran. Shell source reads
and documentation writes used login:false. agent-runtime-security skill consulted;
parent permission and existing approved-input boundaries remain authoritative.

## Files and ownership released

- scripts/acceptance/budget_proxy.py: pool validation, provider binding, seed
  reconciliation and atomic persistent admission.
- scripts/acceptance/budget_proxy.schema.json: required explicit pool/binding
  fields and operator maxima.
- scripts/acceptance/budget_proxy.README.md: exact contracts, pricing basis and
  limitations; earlier lifecycle/provenance remains intact.
- scripts/acceptance/budget_proxy.example.json: inert explicit allocations and
  example binding pool. It remains deliberately nonexecutable with placeholders.

No composer, runtime observer, Rust, UI or private operator state edited.
structured_output_path received the contract and owns composer/template integration.
Root owns task2.12 state; this handoff alone does not complete that integration.

## Exact executable contract

Config.spend_pools must have exactly:

```
{"cohere":{"limit_microunits":10000000},
 "openai_embeddings":{"limit_microunits":10000000},
 "liter":{"limit_microunits":0}}
```

Explicit smaller limits are allowed, never a larger one. Each object has only
limit_microunits. Config.limits.cost_microunits must be explicit, <=20000000 and
<=sum of pool limits. The unused OpenAI pool is valid and cannot transfer to
Cohere. Source does not silently adopt a limit or refill/reset consumption.

Seed.spend_pool_consumed_microunits must have exactly cohere/openai_embeddings/liter
nonnegative integer entries, each <=its own limit. Sum must equal existing
seed.consumed.cost_microunits. Existing prior_epoch_id/prior_consumption_evidence,
explicit unresolved_reservations, packet/signature refs and fixed epoch deadlines
remain mandatory. Missing prior pool allocation blocks, rather than defaulting
unknown earlier charges to zero or reallocating them. Adopted seed/config bytes
remain in input_identity; prior evidence/signature validity remains the parent's
external adoption responsibility, not a cryptographic claim by this helper.

Binding.spend_pool is required. Charge approval additionally requires spend_pool
and spend_pool_limit_microunits matching config. Existing exact model/endpoint/
byte-bound/charge-ceiling approvals remain required. Cohere and OpenAI pools
require positive per-attempt ceilings. Liter requires zero and the already
mandatory explicit zero-charge approval.

Pool-to-route guard is concrete: cohere means an embedding binding to exact HTTPS
api.cohere.ai:443/compatibility/v1/embeddings; openai_embeddings means HTTPS
api.openai.com:443/v1/embeddings; liter means only a model binding through the
already approved existing loopback Liter service. Omitted default443 is accepted.
Charged embedding routes cannot select the free Liter model pool. Arbitrary
provider endpoints require a new explicitly scoped implementation/approval.

## Persistent admission and receipts

SQLite meta.spend_pool_totals stores the three cumulative reservation amounts.
New ledgers start from explicit prior pool allocations; existing ledgers must have
consistent pool metadata summing to global cost. A ledger without pool metadata
is refused; no automatic migration guesses allocations or resets prior charges.
Changed adopted inputs still reject existing epoch identity. A new epoch requires
explicit prior-consumption and unknown reconciliation, not a new zero assumption.

One BEGIN IMMEDIATE transaction checks all global counters and the selected
pool, increments both, and inserts the reservation before outbound. All listeners,
models, backend contexts and runtime restarts share one owner/ledger. Concurrency
cannot spend a separate balance per binding. Exhausted provider pool refuses
before forwarding even when another pool or global allowance remains.

Neither known success, failed response nor unknown outcome refunds charge
reservations. Existing settlement releases only known unused response bytes.
Stale reserved requests become unknown/blocked on restart and keep exposure.

Per-attempt receipt adds spend_pool, reserved_charge_microunits and
spend_pool_total_after_microunits. Snapshot/ready/terminal add
spend_pool_reserved_microunits and spend_pool_limits_microunits. Existing raw wire
SHA, approved authority, interval, actual response and usage receipts are preserved.

## Pricing and uncomfortable limit

Operator provided Coherev3 text pricing USD0.10/1million tokens. README records it
as operator-provided, not independently verified tariff data. No new tokenizer,
exact token count, automatic price or dynamic byte-charge estimator was invented.
Existing explicit approved full-request ceilings remain the charged reservation
amount. A UTF8-byte exposure basis can be adopted only as an explicit justified
assumption with input/framing/array bounds and rounding in charge approval; this
source does not claim that bytes are an exact or proved tokenizer bound. Supported
provider/model input limits may instead justify the approved worst-case ceiling.

The concrete guarantee is that admitted global/provider reservations stay inside
allocations. Actual spend is bounded only insofar as approved full-request charge
ceilings cover real billing, including internal gateway retries/fallbacks and any
additional fees. Pool counters are not verified invoices. Liter incrementalzero
is the operator's disposition, not an independently established underlying tariff
or physical provider-attempt count. Missing valid ceiling or unresolved earlier
consumption remains blocking. No actual availability, price, charging, concurrency,
restart, compilation or executable correctness was verified under the CLOSED gate.
