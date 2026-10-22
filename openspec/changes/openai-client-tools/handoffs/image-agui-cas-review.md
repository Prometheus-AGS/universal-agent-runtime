# Published-image AG-UI CAS source diagnosis

Execute. Source/evidence inspection only; unrelated verification gate CLOSED. No tests, probes, builds, imports, network, database queries or runtime actions. Only this handoff was written.

## Exact observed failure

Inspected retained `matrix.json` and `candidate.log` beneath `target/client-tools-image-bundle-v1.2.2/target/client-tools-published-image-001/`, plus source at published revision `c065bb0798f5bbbd4a878a0c41c04cb4e8a75ad6` using `git show` where relevant.

`agui:reject:partial` failed setup, before the rejection assertion. Wire run `9aa09a19-193b-44f5-adbf-737c84c2538f` emitted the two calls from batch `594edbdd-94dd-44ec-b789-f9dbbc15f630`, then `RUN_ERROR host_cleanup_unconfirmed`. Exact call IDs correlate native run `f936321b-4e89-4b15-aafc-3b035e6c1830` with the 23:12:39.026763 warning `caller_tool_settlement_failed`. The warning at 23:12:37.599622 belongs to another run and does not identify this failure's underlying operation.

## What concurrency does and does not explain

AG-UI starts a retained observer, then the stream. Both call `reconcile`; both can load a nonterminal binding and reach `seal_frontend_handoff` concurrently. Neither an initial binding read nor the eventual terminal-binding CAS serializes continuation settlement confirmation.

However, that race alone is **not** a demonstrated defect in continuation settlement. `confirm_handoff_for_run` accepts an already-settled loaded row; otherwise it joins the producer, creates the monotonic settled version, and performs full-row CAS. The Surreal provider at c065 converts a typed query `TransactionConflict` from `response.check()` to a CAS miss. A miss triggers an exact authenticated reread, validation, and acceptance of `source_settled:true`. An ordinary losing writer whose winner is observed is handled successfully. No evidence says which operation instead failed for this batch.

The manager at c065 already retains the producer mutex while joining and, after join, reads terminal run status without a `try_lock` settlement probe. Therefore the previously repaired concurrent-join `try_lock` explanation does not apply to this published source. There is no basis to weaken actual producer joining.

The terminal-binding CAS happens after confirmation and cannot itself produce the observed warning: that warning is emitted only inside the confirmation error arm. A terminal seal failure would propagate through the separate reconcile/stream error path.

## Causal discriminator

The missing discriminator is the exact failing confirmation stage and safe typed error category:

1. Initial exact-row load/deserialize/validation: distinguishes missing/binding/representation failures before physical joining.
2. Root producer join and returned status (and child join when execution_source exists): distinguishes actual cleanup/status failure from persistence failure.
3. Settlement transition validation, query transport, query response, or result decoding: distinguishes an immutable-state failure or fatal backend error from a normal CAS miss.
4. CAS-miss reread/validation and observed source_settled boolean: distinguishes a successfully observed winner from a reread failure or an unchanged pending row.

The current generic label collapses every item above. A final true row, even if independently established later, would show that some writer acknowledged settlement; it would not establish what another concurrent confirmer returned. Conversely, one false CAS is not proof of failed cleanup. Retained VLOG bytes are not a committed-state query and do not identify the failing stage.

The other investigator's proposed bounded static stage plus typed category retention is causally sufficient to split these alternatives without logging payloads or weakening admission. A semantic retry, serialization rewrite, relaxed settlement requirement, or broader storage change is not justified by the retained evidence. No such production change was made in this review.

## Disposition

Root cause remains unknown. Source supports concurrent callers and a fail-closed terminal winning publication, but does not establish why confirmation returned an error. The existing CAS winner reread rules out treating concurrency alone as the cause. This report adds no acceptance credit; published-image setup remains failed until the cause is established and a separately authorized corrected artifact passes its required path.

## Exact retained numeric representation follow-up

Read printable bytes with `strings` filtered to the exact batch ID from `store/uar.db/vlog/00000000000000000001.vlog`; no engine opened and no JSON parser or numeric program ran. Two full serialized versions are visible, one with source_settled false and one true. Both contain `0.00015539999999999982` in Agent/Global cost_usd and `2.1e-6` in Run/Session/cumulative cost_usd. No numeric spelling difference is visible between these versions. They remain raw log versions, not proof of committed state.

The manifest declares serde_json without float_roundtrip explicitly. That alone does not establish the published binary's unified dependency features. Inspected cached serde_json 1.0.151 `de.rs` non-float_roundtrip branch: `f64_from_parts` casts an integer significand to f64, then multiplies/divides a power-of-ten value. This makes numeric conversion a concrete stage to discriminate in a later authorized serialization observation, but source inspection here does not establish that the retained decimal changes bits or spelling on the exact deployed parser. No numeric drift was demonstrated. No float feature or money representation change is justified as the diagnosed repair on this evidence alone.

This follow-up preserves the unknown-cause disposition. The minimal stage/error evidence remains necessary unless independently authorized exact-artifact evidence establishes a specific representation mismatch. No checks or probes executed.

## Dependency-feature discriminator

Inspected c065 Cargo.toml and Cargo.lock plus the cached registry manifest. Root jsonschema 0.49.4 is an unconditional normal dependency with default-features false; its own unconditional serde_json dependency explicitly enables float_roundtrip. Disabling jsonschema default features does not remove that dependency feature. The locked dependency is 0.49.4, and Dockerfile's server-full/postgres-backend build retains this root dependency. No macOS-only cfg accounts for the native float_roundtrip feature. A host-build serde_json fingerprint without that feature belongs to a separate compilation context and does not establish Linux runtime feature selection.

Thus the proposed native-versus-Linux absence of float_roundtrip is unsupported by the inspected dependency path and cannot justify a numeric parser repair. No numeric or feature probe ran. Lead reports the bounded diagnostic repair now preserves the original error unchanged; this source diagnosis did not edit that implementation. Unknown-cause disposition stands pending stage-specific evidence.

## Earlier warning exact correlation

Native run `7fcf1355-d842-4776-9f6c-5720496e3286` at 23:12:37.599622 is the mixed host/caller scenario, not a negative rejection or another failed setup. Its log names host `fixture-host-call` and caller `uar_client_fa7f9c5b-21a2-41f9-8b60-096d55567aab_98abc792-03f1-48fb-98c5-2ecffcd36c64`. Matrix evidence around lines 46500–46570 contains the same exact caller ID under wire run `4e48f58b-1054-4135-968f-36c4813d913e`, thread `99bc1cbb-e801-40ec-9459-1a430f110830`; it emits the host result and RUN_FINISHED success with that pending ID. The following trace identifies scenario mixed. The summary marks `agui:mixed-host-caller` passed with two provider contacts, one handoff and zero retry contacts.

Thus a confirmation error warning coexisted with successful terminal publication for this exact batch. Source explains the compatible interleaving: independent observer/stream reconcilers can confirm concurrently; a success can seal first and the error branch's later seal can lose CAS and load the successful binding. This is evidence that warning count does not equal failed wire runs. It also narrows the investigation toward one confirmer failing while another succeeds, without proving which confirmation operation returned the suppressed error. Main native f936/wire 9aa09 failure remains distinct and failed setup. No new plan, production fix or executable verification was started.
