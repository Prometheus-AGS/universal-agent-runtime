# Source-backed native fixture policy draft

This is an inert source artifact: exampletrue, executablefalse. It has not been
loaded into UAR, composed, imported, validated, built or tested. The phase gate is
CLOSED. It contains no keys, active provider receipts, invented image hashes or
claims of usable model support. Source inspection used prometheus-rust-workspace
and relevant gotchas, including outer resilience vs lower-client retries and
credential verification before expensive fixture calls.

Composer consumes only native_configuration as the native policy overlay, then
merges the private preparer's actual security_patch. source_recipes is composition
metadata, never an AppConfig field. File names are native-policy.local-draft.json
and native-policy.local-draft.README.md; freeze both with the final source closure.

## Actual source basis and selected values

| Section | Source | Choice and purpose |
| --- | --- | --- |
| security | src/config.rs SecurityConfig; security/verifier/mod.rs; security/credentials/policy.rs | Required remote HS256/nbf and settings-key auth. Actual JWT issuer/audience/workspace mappings come from private preparation, not placeholders masquerading as identities. single_owner is the existing credential default; actual data still uses authenticated principal fences. |
| resilience | src/config.rs ResilienceConfig/default_*; config.embedded.yaml | Finite30s request/idle and15s stream-start; real production retries3/budget20s preserved. Rate limiting disabled as existing embedded example;100/200 values retain valid required fields. |
| runs | src/config.rs RunsConfig::default | Existing600s/1000/30s retained-state defaults. No fake immediate retention sweep or test-only hook. |
| sessions | src/config.rs SessionsConfig; observers_startup.README.md | Positive3600s persisted TTL admits real U21 aged-record maintenance while protecting ordinary case history. In-memory idle/cap remain disabled defaults. |
| file_processing | src/config.rs FileProcessingConfig::default | Real local kreuzberg,10files/50MiB each/100MiB total. Uploads stay in exact run-owned root via an already exported env name; no ambient temp/operator path. Empty MIME allowlist keeps supported formats. |
| vision | src/config.rs VisionConfig | auto_detecttrue; omitted optional model deserializes None. Actual provider vision capability/PNG interpretation needs real receipts. |
| intent_classifier | matching/intent/mod.rs ClassifierConfig | Explicit rules baseline;5/.5/.05 are default numeric values. Backend default is tfidf, so rules is a selected override. Cases exercise all mounted strategies later, not a static rules-only success claim. |
| harness | src/config.rs HarnessConfig/SkillReattachmentBudget | Actual typed assembly default, legacy_overlay activation default,5000/25000 bounded reattachment defaults. No invented token-pressure activation-denial policy. |
| llm | src/config.rs LlmConfig; selected bindings.local-draft model roles | Qualified fixture-primary/gpt-5.4, wire alias gpt-5.4, primary budget port49421. Raw owned ingress key, explicit chat protocol/parallel tools. Lower Liter client max_retries0 avoids an extra unobserved retry layer; outer governed retries stay bounded. Existing gateway retries remain separately disclosed. |
| llm.embedding | EmbeddingBackendConfig; rag/embeddings/{mod,openai}.rs | Actual user-selected multilingual-v3.0/1024, OpenAI interface through49424, batch32, send_dimensionsfalse. Returned width stays validated; no dimension coercion. |
| native_tools | src/config.rs NativeToolsConfig; tools fixture contracts | Real file tools restricted to exact tools-effects directory; default read/write limits. Session search enabled, terminal/web disabled because fixture instructions require only owned harmless file effects. |
| memory | src/config.rs MemoryConfig; memory/service.rs; vendored SDK embeddings/openai.rs | Enabled embedded retained store, auto_capture/injection enabled. English-v3.0/1024 via same budget listener49424, exact private memory path/ns/db. SDK already omits dimensions; do not invent memory.send_dimensions.7/.3/2000 and MCP settings are source defaults. |
| a2a | src/config.rs A2aConfig | Explicit owned instance label, no outgoing trusted peers. This does not disable supported incoming A2A API cases or invent trust for an external peer. |
| sycophancy | existing server fixture templates and server optional correction path | auto_correctfalse avoids a separately ungoverned private correction call; no claim that the optional path gained a finite output allowance. |

Omitted fields are optional or use the actual documented serde defaults. There
are no null native leaf objects to be mistaken for usable settings. Any further
case-specific override must be represented in source and the frozen packet, not
injected by an unapproved setup service.

## Routine team recipe

source_recipes.teams supplies the actual registered settings-only profile
uar.openai-compatible-chat.settings-v1/revision1/settingsRevision1 with off reasoning
by source default. It does not claim an exact fit contract or known live capacity.
The opaque credential/storage URI labels follow deployment binding's actual
protected-credential:// and protected-connection:// grammar (see bindings.rs and
tests/fixtures/collaboration/deployment-binding.json). They contain no secrets
and do not by themselves prove a stored credential or a connection exists. The
team case binds actual provider/model and catalogStorage/backend from authenticated
runtime capabilities; host provider capture resolves credentials separately.

The finite team reservation32768tokens/USD1/180s and team maximum262144tokens/
USD10/600s are inert planning bounds, not additional paid inference authorization
or a refill of the independent provider pools. Actual admissions/accounting and
child joins must satisfy the case predicates. Foreign JWT/workspace/owner fields
come from private preparation. These numeric proposals introduce no unknown
credential, endpoint or operator-only value requiring manual invention.

## Strategy and U13 capability boundaries

The actual server-full Cargo feature includes local-models and wasm-runtime;
postgres-backend is additive for the selected Postgres profile. Installed frozen
classifier WASM, real model assets, model capability metadata and transitive
artifact closure remain producer outputs, not values this JSON fabricates.
ClassifierConfig.wasm_component_path is optional; the strategy case sets its
actual frozen installed component through the mounted setting. Strategy names
alone do not prove lexical/semantic ranking, cold context, local inference,
truncation or protected multimodal preservation.

source_recipes.protected_overflow is a derivation recipe, not a completed vector.
Composer/producer must use the actually selected primary model's known context
window and the mounted HTTP/body limits to materialize a finite protected user
message. The proposed ASCII repeat unit and multiplier create an aggressive
candidate; they are not an exact tokenizer proof. It is accepted only through the
real run's protected_overflow/before-dispatch receipt. An HTTP413, proxy byte
refusal, unsupported model, unknown context, helper error or missing run receipt
cannot substitute for that outcome. No exact-boundary or guaranteed-fit row is
fabricated. If the resulting candidate cannot fit the mounted public request
bound, the producer must use another source-supported protected envelope or
report that concrete blocker; do not silently count an unmaterialized recipe.

The uncomfortable limit is that fully populated configuration is not runtime
capability evidence. Genuine model compliance, strategy ranking, owner isolation,
protected budgeting, all actual startup/retention and datastore effects remain
unverified. No new true operator input is introduced by these routine leaf
choices. Existing external obligations remain final charge/input-exposure approval,
prior consumption/unknown reconciliation, exact artifact/source inventory and
signed scope/adoption. Identity/token validity is generated by the approved private
preparer, not manually entered for each leaf. Ports match the current inert local
recipe and remain subject to the later explicitly approved packet.

The selected source catalog openai/gpt-5.4 context is1050000. The protected-overflow
candidate now repeats the four-byte ASCII unit twice per context token:8400000
text bytes. Four repeats would produce16800000bytes and exceed the mounted10MiB
request-body limit. This is source arithmetic, not a tokenizer execution or a
refusal proof. The proposal producer must independently bound the entire serialized
JSON request (including metadata/envelope overhead) below10MiB before emitting
the actual vector. Materialization remains assigned to structured_output_path;
only an actual owned host protected_overflow/before-dispatch receipt can satisfy
U13. Exact token counts, exact boundary and successful refusal remain unverified.
