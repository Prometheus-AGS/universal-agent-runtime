# Private fixture and composition approval packet — source-only handoff

Phase: `uar-post-lifecycle-verification-readiness`; stage `user-outcome-implementation`; task2.9. Waypoint read at revision504 / plan135: private fixture/resource seed and derived-input inventory remain incomplete. Operator verification gate **CLOSED**. This packet is a draft, not permission, adoption, verification, or PASS.

Ownership: only this handoff. Other writers share the checkout; their edits are preserved. Commands used were source/document/JSON reads and this documentation write. No acceptance/helper import or invocation, compiler/build/check/test/format/lint/review gate, credential read/copy, service/provider operation, signing, or adoption ran. No hashes were recomputed. Digests below are copied from frozen metadata. No production edit was made.

Retain original epoch `uar-user-outcomes-original-20261024`, all prior consumption, 4000 calls / 2400 minutes / nine repairs consumed, separate USD10 Cohere + USD10 OpenAI embedding pools, and Liter zero incremental billing only under its explicit operator statement. No retry, tenth repair, reset, pool transfer, publication, or extra provider selection is implied.

## Frozen inputs already available

`user-outcome-source-010.json` records source identity `sha256:28137dc46cbb05aa9169f93b46232ea6aafc8d3f04e7c841274e4bbf4d109018`.

`user-outcome-installed-inventory-001.json` describes the P0.006 snapshot, adopted A12/binding14. Its preparation manifest is:

- path `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-006/artifacts/snapshot/preparation.json`
- digest `sha256:80d491ad67d09e2a98492bb413bd153438cc17d9f10879d48506fb23e8bb31a9`

Actual native artifact is `snapshot/native/payload/universal-agent-runtime`, digest `sha256:25151ae66312e41c99060971645b2414b28817669a164f064cd59d3b0576828c`. Actual `snapshot/native-build-receipt.json`, digest `sha256:d6c4c0ba459eff56c8e994feeb294c6e5487417e04b19f30bd357cca6a789e32`, records that same source/artifact identity, matching command/operator_approved_command, and exit_code0. These are prior producer receipts, not a new check here.

Frozen Python selected by installed metadata:
`/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-006/artifacts/snapshot/python_runtime/payload/bin/python3.12`.

Frozen preparer digest: `sha256:a928a3bffd19443d98c25e6a6f20fea5ecbb315164ddd1e11b5c2a96c41c1dfa`.
Frozen composer digest: `sha256:0873f35f9c5cdba114ab20afa30d1465687ee7fcfecc575c6b3a624364b042e2`.

The descriptive installed inventory is **not** the final composer schema. The full `preparation.json` supplies `installed`, `closures` with `files`/manifest references, `installed_artifact`, `build_receipt` and source denominator. Each closure manifest supplies its actual path/digest rows. Build a new additive final input inventory; do not rewrite source010, installed-inventory001 or producer receipts.

## Authority DAG and whether approval can be combined

1. Lead authors exact private config and matching preparation approval, freezes their bytes, and presents exact effects/argv. Config has current explicit JWT validity and an existing owned private parent. Approval projection excludes only `preparation_approval` to avoid self-reference.
2. Operator explicitly approves named private-preparation command/effects. Only then create the exclusive private scope, generate fixture credentials/tokens/datastore inputs, and privately retrieve exactly Cohere/Liter keys. Capture its redacted receipt. No env.json loading is required for composition.
3. Named metadata-only sealing records actual config / preparation approval / inputs.json / receipt.json identities; records redacted existing-service provenance, tools and configuration metadata; creates full-context charge documents, hosted embedding source authority, original-epoch accounting evidence and resource packet. This is input preparation, not new source or runtime evidence.
4. Explicitly authorized registered signer signs/adopts exact resource authority. Seed then references actual resource packet and signature hashes, retaining all prior counters/reservations. Signature hash cannot be inside its own signed preimage. No derived fixture hash can be in resource authority that the fixture itself references.
5. Author final bindings and final inventory with those actual references, seal them under the named metadata allowance, then execute the exact composer argv. Composer reads/hashes final source/installed/authority/private nonsecret inputs and writes a new private output directory; it does not read env.json or launch services/providers.
6. Seal new `derived-input-receipt.json` and its listed outputs externally. Present fresh whole-phase implementation-complete inventory, exact source identity, remaining tasks and exact V commands. Final derived adoption/signature and product V require explicit operator instruction. Neither receipt nor its future signature reenters fixture/configuration inputs.

Private preparation and deterministic composition **can be requested in one bounded conditional approval** if that approval explicitly names steps1–5, exact argv, exact config/projection, source/snapshot identities, all intermediate metadata writes/hashes and signing/adoption effects. Random secrets mean final inputs/receipt hashes cannot be known in advance. Authorization must therefore permit only sealing the actual outputs of this one named preparer into the listed final slots, with stop-on-any-mismatch and no helper/code edits. Combining two argv alone does not authorize the intervening resource accounting/signature work. Final V and external derived adoption cannot be inferred from this preparation approval. A simple, concrete alternative is approve preparation first, then present actual sealed final bindings/inventory and resource authority for the next explicit approval.

Uncomfortable fact: proposal003 still has many null authority/resource fields. A combined packet cannot honestly promise immediate composition until they are resolved. Installed closure availability does not establish service identity, model availability, port availability, tariff correctness, native behavior or acceptance outcomes.

## Concrete private config draft

Proposed new filename (not created here):
`/Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-private-inputs-001.json`.

This draft is deliberately inert. Replace only executable, two validity values and actual preparation_approval reference after exact operator authority. Root/ports/selection below reproduce frozen local draft. If root/output selection changes, update both preparer and final composer argv before approval. Parent must exist mode0700; preparer refuses reuse of an existing epoch scope and never clears it.

```json
{
  "schema_version": 1,
  "executable": false,
  "epoch_id": "uar-user-outcomes-original-20261024",
  "scope_parent": "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/private",
  "composition_output_dir": "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/composed",
  "selected_embedding_services": ["cohere"],
  "validity": {"not_before_unix": null, "expires_unix": null},
  "ports": {
    "postgres_http": 49401, "postgres_grpc": 49402, "postgres": 49403,
    "surreal_http": 49411, "surreal_grpc": 49412, "surreal": 49413,
    "primary_proxy": 49421, "alternate_proxy": 49422, "structured_proxy": 49423,
    "cohere_embedding_proxy": 49424, "structured_metadata": 49425, "negative_401_peer": 49426
  },
  "existing_key_sources": {
    "COHERE_API_KEY": "/Users/gqadonis/.bash_profile",
    "LITER_LLM_MASTER_KEY": "/Users/gqadonis/.prometheus/kbd/secrets.env"
  },
  "source_contracts": {
    "claims": {"path": "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/uar/security/claims.rs", "sha256": "sha256:a269f6dc7c05f14c2419a7e70f0b0cda89c67a0d0cb4de3fe3210f0fd95a5ded"},
    "verifier": {"path": "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/uar/security/verifier/mod.rs", "sha256": "sha256:6f68f3f27618bfe5cb39ebedb9c6ce0204360bb0cc71651d5e89d2b9297ba50f"},
    "presentation_owner": {"path": "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/uar/runtime/actor/messages.rs", "sha256": "sha256:93196dea061aeafce7dd8fffc06da3ccf051be48acbad4d87859d7d9d9355b60"},
    "user_settings": {"path": "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/uar/api/user_settings.rs", "sha256": "sha256:be631a36446e0849af1496a3e29b2624567f1c515fb2bc36dade79dfe81f0346"},
    "collaboration_scope": {"path": "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/uar/api/collaboration.rs", "sha256": "sha256:ae91e1b57ee3e1c2254cd6f62fea42181e7bd3cf355a02688c94d297c2fe7157"},
    "session_storage": {"path": "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/uar/persistence/mod.rs", "sha256": "sha256:4e52f6480d5ec21eba3545c57e87b13d880c50cde068e813a3bd142ed52f8097"}
  },
  "preparation_approval": {"path": null, "sha256": null},
  "pending_operator_inputs": [
    "Preserve original adopted epoch and all prior consumption; no reset",
    "Actual resource authority, conservative charge ceilings and prior unknown reconciliation",
    "Actual resource signature and separately approved external derived adoption"
  ]
}
```

Preparer enforces current validity `start <= now < end`, `0 < end-start <=86400`. Select exact timestamps before approval; packet must finish before JWT expiry and original budget deadline. A runner wall limit86400 leaves no allowance for preparation/signing delay within a24h token window, so actual start/deadline scheduling must be reconciled in the exact packet rather than assuming the full nominal wall allowance is usable. Original epoch ends2026-10-27T13:38:17.685447Z; do not extend it.

The config itself is absolute regular nonsymlink owned mode0600 JSON <=1MiB. Preparation approval must equal the exact object below, with actual root/hash substituted; boolean alone grants no authority:

```json
{
  "operation": "prepare-uar-acceptance-private-inputs",
  "epoch_id": "uar-user-outcomes-original-20261024",
  "scope_root": "<scope_parent>/uar_fixture_inputs_<sha256(epoch UTF8) first20>",
  "preparation_projection_sha256": "<sha256 of compact sorted UTF8 config excluding preparation_approval>",
  "effects": [
    "create-exclusive-private-scope-and-state-parents",
    "generate-owned-hs256-jwt-and-admin-secrets",
    "mint-two-owned-fixture-user-tokens",
    "generate-shared-owned-ingress-raw-key",
    "generate-owned-postgres-and-surreal-credential-inputs",
    "privately-copy-two-selected-existing-provider-keys"
  ],
  "operator_approved": true
}
```

No OpenAI key is selected or copied. env.json contains generated secrets plus only COHERE_API_KEY/LITER_LLM_MASTER_KEY; never log/source/put values in argv. stdout and receipt are redacted. inputs.json declares source-derived owner keys, JWT claims/security patch, environment names and paths, not observed authentication outcomes.

## Exact proposed helper argv and effects

Private preparation command (not run; executable config and exact approval still absent):

```json
["/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-006/artifacts/snapshot/python_runtime/payload/bin/python3.12", "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/scripts/acceptance/prepare_private_inputs.py", "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-private-inputs-001.json"]
```

Effects: one exclusive mode0700 scope; postgres/surreal/packet state parents; exclusive mode0600 env.json, inputs.json and receipt.json; fresh fixture-only HS256/admin/ingress/datastore values; two fixture principals/tokens; literal retrieval of exactly two existing named keys. No profile editing, shell source/eval, datastore creation, service/model request, environment adoption or acceptance.

Composition command after exact new final bindings/inventory are authored/sealed (proposed new filenames, not existing artifacts):

```json
["/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-006/artifacts/snapshot/python_runtime/payload/bin/python3.12", "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/scripts/acceptance/compose_fixture.py", "--bindings", "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-bindings-final-001.json", "--inventory", "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-inventory-final-001.json", "--output", "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/composed", "--compose"]
```

Effects: read/hash declared final inventory files; derive real frozen configuration/schema identities; create exclusive mode0700 output with mode0600 fixture, both adapter runtime/startup/server/MCP configs, proxy/service/store/team configs, adoption.json and derived-input-receipt.json. Status is `COMPOSED_NOT_EXECUTED_NOT_GATE_AUTHORITY`;97cases/582rows. No provider, service, database, parent env loading or acceptance. Existing partial directory is a failure, never silently overwritten/retried.

## Final schema and remaining field mapping

Final inventory shape is `schema_version=1, finalized=true, full_closure=true, files=[{path,sha256}], source_files=[absolute paths], installed_artifact={path,sha256}, build_receipt={path,sha256}, closures={name:{complete:true,files:[paths],manifest:{path,sha256}}}`. Preserve all12 complete closures from preparation.json. Add union of source010 regular rows, closure manifest rows/files, artifact/build receipt, actual preparer config/approval/inputs/receipt, accounting evidence, resource packet/signature, charge documents, hosted embedding source approvals and existing-service authority/tool provenance. Every referenced file must be frozen. Source_files stays exactly source010's selected source denominator; later authority/private/build outputs never enter that build preimage. Bindings/inventory cannot self-hash inside their own content; composer adds their byte identities to final fixture/configuration closure itself. Do not include env.json in a file-read inventory: composer must not parse or hash secrets merely to establish nonsecret preparation linkage.

Final bindings start from proposal003 and substitute actual installed paths from preparation.json. They need `schema_version=1, executable=true, example=false, full_closure_approved=true`, actual operator_approval/operator_signature, private_inputs_config and private_inputs.inputs/receipt references. Composer replaces actual security patch, primary/history/subject identity, workspace, foreign auth ref, backend parents/env refs and datastore refs from inputs.json. Preserve complete approved native security policy outside that merged patch.

Resource approvals still needed:

- Three Liter binding charge documents, one each for `liter-primary`, `liter-alternate`, `liter-structured`; actual allowed models gpt-5.4/k3/gpt-5.4; positive full_context_tokens from actual approved model contract; basis `full-context-plus-output-ceiling`; output ceiling4096; request1048576/response8388608; explicit zero incremental charge and retry/fallback-chain coverage; spend_pool=liter/limit0; Authorization Bearer prefix and existing-service authority kind.
- One Cohere charge document for `cohere-embeddings`: all three declared embed-v3 aliases, exact compatibility URL, request1048576/response8388608, basis `embedding-request-ceiling`, proposed5000microunits full-request ceiling, spend_pool=cohere/limit10000000, explicit96 textual inputs x512 billable tokens/no-extra-charge assumption under operator0.10USD/million statement, actual guard, Bearer prefix. This ceiling is not tariff verification.
- Three hosted embedding source approval refs/model_source_identity values for A(multilingual1024), B(english1024), C(english-light384), one shared selected Cohere listener. No synthesized vector evidence or availability claim.
- Per-Liter-binding upstream approval exact object in composer lines634–641: kind/service_id/operator_approved/endpoint_url/epoch_id/binding_id/header_prefixes and exact launchagent/launch_script/service_config refs. The observer binds the same source bytes plus exact launchctl/ps/lsof refs, actual uid/argv/listener names/credential names and finite limits. No service observation occurs during composition.
- Original-epoch seed: epoch_id, approval_packet_sha256, approval_signature_sha256, prior_epoch_id, nonempty frozen prior_consumption_evidence, unresolved_reservations=[] only after real reconciliation, consumed with all six counters(model_attempts,embedding_attempts,generated_tokens,cost_microunits,request_bytes,response_bytes), all three spend_pool_consumed_microunits whose sum equals global cost, fixed original epoch_started_unix/deadline_unix. No zero genesis inferred from lack of product calls. Runtime budget code checks adopted bytes/relationships, not signature cryptographic validity; external signing/adoption is mandatory parent work.
- Remaining proposal003 nonsecret operational slots: exact existing-service observer values; browser static_directory/static_files from emitted assets including index; MIME policy and trusted peer list as actual source-backed choices; Surreal namespace/digest-pinned Args; backend profile IDs and actual image/resource metadata. Composer fills only documented prepared overrides; it does not fill all nulls automatically. No live probes are authorized to resolve them.

Metadata-only sealing must be named explicitly in operator approval if it runs any file hashing/identity routines; do not sneak standalone hash preflights under either helper command. It may copy existing frozen digest rows without opening unrelated paths. Final resource-signing and external derived-adoption commands must be taken from the actual registered signer schema by the lead; this packet intentionally invents no signer command/API.

## Completion handoff

Changed file: this handoff only. No verification was run, and none is claimed. Source/metadata inspection established the required dependency order and schemas; actual private outputs, charge/accounting/resource authority, composed fixture and derived adoption remain absent/unverified. No guards were added. No unsolicited production changes were added. Exact parent decision needed before execution: staged approval versus explicitly bounded combined preparation/sealing/signature/composition approval; product V stays CLOSED in either case until the fresh inventory and exact V instruction.

## Follow-up metadata authoring — target007 final-bindings draft

Owned additions only:

- `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-bindings-final-001.draft.json`: cloned proposal003, preserved all budget vectors/limits/caps/ports and original epoch, retained `executable=false`, `example=true`, `full_closure_approved=false`. Installed paths/artifact/producer reference now copy sealed P0.006 metadata. Packet root/private/composed are target007; immutable artifact references remain target006. Concrete adapter profile labels, prepared credential environment aliases, Surreal namespace/daemon argv, observer tool paths/finite limits/credential names/wildcard listener selection, embedding ingress env alias and source-backed empty MIME/A2A choices are filled. Observer source executable changed from proposal's inert false to a correctly shaped pending frozen ref.
- `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-bindings-final-001.recipe.json`: typed authoring recipe, explicitly not a callable helper/API. Lists future private-generated overrides, exact existing-service provenance, byte sealing, resource/charge/hosted-source authority, original-epoch seed, final inventory and admission activation. No generated digest/authority was invented.

The draft sets static_directory to the already captured `target/uar-user-outcomes-006/artifacts/snapshot/static/payload` and enumerates `/assets/` path/URL rows with digest null. Source-inspected producer configuration includes this copied static tree but does not select it into any of its12 closure groups; therefore preparation.json closure manifests lack those rows. Lead must explicitly seal the captured entire static tree, including index.html/non-assets, into final input inventory and fill asset hashes under the named metadata allowance. This requires no new frontend build or browser probe.

Source choices and rationale: namespace `uar_acceptance` and Surreal daemon start/log/user/pass/bind/rocksdb argv come from runtime-observer.example.json, with preparer credential reference aliases. Empty MIME list means all supported types per src/config.rs; empty trusted A2A peers is its default trust policy. Existing-service observer README supplies `/bin/launchctl`, `/bin/ps`, `/usr/sbin/lsof`,30sec command/1MiB output/120sec wall bounds, the three exact credential names and selected default-host socket `*:4000`. That wildcard requires explicit future approval and real observation; no loopback-exclusive claim is made. UID/current resolved executable/source/tool hashes and exact argv remain lead-owned provenance slots. No live call was made to resolve them.

The new recipe and draft supersede target001 paths in the earlier proposed command/config section of this handoff. Lead's final command must name target007/composed and the actual finalized bindings/inventory, and actual private config must select target007/private and target007/composed. Earlier config/code-source digests remain frozen metadata facts, not new verification.

No tests/checks/builds/syntax/import/helper execution or source hash recomputation ran. Metadata authoring used inline standard-library JSON/path reads and writes only, with filesystem-name enumeration for captured static assets. One metadata read attempted to access a missing `path` field on a producer tree entry and returned KeyError; a later source read inspected that entry's actual `output` field. This was a read-only metadata inspection failure, not a gate or acceptance result. No secret files or values were read. No production/source edit, sign/adopt operation or service/provider operation occurred. No guards were added. All generated/private/authority fields and actual runtime compatibility remain unverified.
