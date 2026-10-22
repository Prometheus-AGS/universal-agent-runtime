# Budget storage failure retention repair

Phase: `uar-post-lifecycle-verification-readiness`, Execute. Verification CLOSED
for this teammate. Only source inspection and the writes below were performed;
no checks, tests, query probes, builds, imports, or binary execution ran.

## Observed failure

`target/client-tools-local-gate-003/candidate.log:277` records model admission
refusal, `accounting_admission_unavailable`, `other_storage_error`, and detail
`Query`. The log does not retain the underlying query reason. This handoff does
not claim a root cause for the transaction failure or a successful admission.

## Source-demonstrated defect and change

Owned source: `src/uar/persistence/providers/accounting/surreal.rs`, function
`checked`. Its existing error selection returned the lowest statement-index
error. In pinned vendored SurrealDB 3.3.0,
`vendor/patches/surrealdb-lifecycle/surrealdb-core-3.3.0/src/dbs/executor.rs`
around lines 2023–2048 replaces earlier successful transaction results with
`QueryError::NotExecuted` when a later statement fails, then appends the actual
typed failure. Therefore minimum-index selection can discard the substantive
failure and return only a rollback placeholder.

The edited selection ranks `(is_not_executed, statement_index)` instead of only
`statement_index`. A substantive error wins when present; the original lowest
index still wins within each class. The SurrealDB error value and its chain are
returned intact. Existing accounting revision-conflict classification remains.
All errors still fail closed. There is no retry or budget bypass.

Exact edited source expression:

```rust
    // A failed transaction replaces earlier successful statement results with
    // NotExecuted. Prefer its actual failure over those rollback placeholders.
    if let Some((_,error)) = errors.into_iter().min_by_key(|(index,error)| (
        matches!(error.query_details(), Some(surrealdb::types::QueryError::NotExecuted)), *index,
    )) { return Err(error.into()); }
```

The durable admission query itself was not changed. Pinned
`src/legacy/expr/statements/update.rs:73–90` returns an array for UPDATE without
ONLY, so the current `array::len` CAS checks are not demonstrated to be the
failure. No speculative query rewrite was made.

## Remaining work and ownership

Lead owns the next explicitly authorized real integration gate. Ownership was
expanded to `src/uar/runtime/cost_budget.rs` **only** for
`storage_error_detail`. That helper now appends static subtype labels for known
Query, Validation, Configuration, Serialization, and NotFound variants and
retains all typed cause kinds in order. Examples are `Query:NotExecuted`,
`Query:TransactionConflict`, `Validation:Parse`, and `NotFound:Table`.
Absent subtype details produce `Unspecified`; unmatched variants retain their
outer kind. No message, SQL, owner identity, bound value, or detail field is
formatted. The underlying error chain remains unchanged.

API evidence is the pinned installed `surrealdb-types-3.3.0/src/error.rs`:
`details()` at 403, `ErrorDetails` at 551, typed subtype definitions at
776–931, and `cause()` already used by this helper. The vendored SDK reexports
the pinned types as `surrealdb::types` and its `Error` at `src/lib.rs:60,92`.

The public taxonomy has an uncomfortable limitation: it does not expose a
distinct type/schema/unsupported query subtype. Pinned `From<TypeError>` at
`error.rs:1299` maps to `Internal`; vendored execution error mapping also
collapses numerous execution errors to `Internal`. The helper cannot recover
that lost distinction safely. If the next real gate reports `Internal`, source
inspection of that newly narrowed failure and possibly approved upstream
field-free classification are still needed. Raw database error text can contain
bound data and should not be exposed through client responses.

The actual failing statement and durable admission root cause remain unknown.
Compilation and runtime behavior of this small selection repair are unverified.
`src/uar/persistence/accounting.rs` was inspected but unchanged. The only
production changes are the accounting error selection and the explicitly
authorized diagnostic helper. The only additional file is this required
handoff. Others share this checkout; preserve their edits.

## Gate005 follow-up: terminal usage is unpriced

This follow-up is source analysis and handoff only; no additional production
edit or execution was performed by this teammate.

Gate005 `candidate.log:223` reports `Execution completed with unresolved durable
accounting`, error `accounting_usage_unresolved`. The generated config's
`fixture` model `gpt-4o-mini` has no `pricing_identity`. Its `openai` provider
entry is separate and cannot confer a price on the `fixture` identity.
Gate005 has reached provider contacts without the prior accounting admission
failure. The original gate003 `Query` cause remains unreproduced and unknown;
the error-retention repair is not proof of that original transaction's cause.
The terminal Unresolved outcome below is expected fail-closed behavior until
the fixture has the correct validated catalog pricing binding.

Source mechanism:

- `turn/bindings.rs:217` defaults `BoundModel.pricing_model` to its model identity,
  here `fixture/gpt-4o-mini`. Lines 298–300 replace it only when validated
  `config.catalog_pricing_model` exists. Lines 387–390 bind that pricing identity
  into the budget driver independently of the endpoint driver/wire model.
- `llm/catalog.rs:282–286` looks up exactly the provider/model pair for pricing;
  it does not infer OpenAI prices from a custom provider's model spelling.
- `cost_budget.rs:1497` therefore produces no price. The terminal call at 1543
  passes `last_price=None`. `accounting::observe` deliberately does not settle
  terminal usage without a cost; `DurableCall::usage` at 1714 reports Unresolved
  when the returned row remains Pending. This warning alone is not evidence of
  a storage query failure.

Existing supported setup correction: retain custom provider identity `fixture`
and add this model metadata:

```json
"pricing_identity": {"provider_id": "openai", "model_id": "gpt-4o-mini"}
```

`ModelPricingIdentity::qualified_model` (`llm/registry.rs:123–140`) validates the
catalog entry and its finite nonnegative rates; registry resolution captures it
as `catalog_pricing_model` at 640. This uses the existing operator-declared
pricing mapping, with no fabricated rate and no accounting exemption. Merely
renaming the provider to `openai` would lose custom-provider coverage; the
separate pricing identity is the appropriate fixture representation. Setup
owner and lead were notified. Actual settlement after correction remains
unverified and belongs to the lead's next authorized gate.
