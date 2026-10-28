# Task2.15 local strategy resource separation

Phase user-outcome implementation, gate CLOSED. Source authored, no acceptance
claim. This fixes the reported local-label/hosted-Cohere routing defect; it does
not reinterpret LocalEmbedding as permitting remote inference.

## Files changed and source behavior

1. `src/server.rs`: registers host `local-skill-embedding` lifecycle with a
   retained weak-slot finalizer calling the actual swappable backend shutdown.
   Builds a separate strict FastEmbed member, fixed packaged BGE-small-en-v1.5
   identity, known384dimension, from `config.models.models_dir`. This uses the
   existing local backend factory, with no credentials or hosted endpoint.
   Missing/uncompiled assets record resource failure and leave the optional
   local matcher unavailable. Initialization remains lazy; no model probe is
   introduced. Supplies the same local matcher through additive service and
   manager builders. Existing global embedding backend, RAG vector matcher,
   hosted model/credential configuration and lifecycle remain intact.
2. `src/uar/runtime/manager.rs`: adds optional local_skill_matcher with an
   additive `with_local_skill_matcher` builder; existing public constructors
   keep their argument lists and initialize it absent. Local intent selection
   now explicitly requires that resource and pins it for the classifier. It
   no longer passes `self.vector_matcher.pin_embedding()` from the global RAG
   backend. Non-local classifier construction is unchanged.
3. `src/uar/runtime/skills/service.rs`: optional dedicated matcher plus additive
   builder; carries it into checked matching. Splits LocalEmbedding from the
   Embedding branch. Local matching ranks already scoped eligible skills with
   the local classifier, then uses existing threshold/margin/top-k resolution.
   Hosted embedding and hybrid continue using their original registry matcher;
   keyword, governed LLM binding and selection/mutation policy are preserved.
4. `src/uar/runtime/skills/service/selection.rs`: exact snapshot Arc initializer
   captures a new wrapper over the dedicated backend's actual pin, retaining
   coherent resource identity across the captured run. Existing selection
   reconciliation, registry snapshot and agent bindings are preserved.
5. `src/uar/runtime/matching/intent/local_embedding.rs`: constructor signature
   retained, but captures one actual backend Arc for index and query. Before
   either inference operation requires fastembed identity, no remote endpoint
   and actual local asset provenance. A hosted matcher or unavailable resource
   therefore errors before any embed call. Common local index/ranking helpers
   allow service-scoped candidates without reapplying only global enabled flags
   or letting ineligible rows consume its top-k. Existing IntentClassifier
   methods delegate using registry-enabled skills. No keyword fallback remains
   for a missing nonempty index; empty skill input remains an empty result.
   Actual batch-length mismatch propagates an error rather than zip truncation.

There are five production paths listed above (manager/server/local classifier/
service/selection); the task packet described “six owned files” but those named
paths total five. No sixth production file was invented. This new handoff is
the sixth changed file in this packet. No unrelated file was edited.

## Integration inputs and compatibility

Root owns composer derivation of `server.models.models_dir` from the actual
completed local_embedding_assets closure. It must reference the snapshot
containing these five names: bg-small-en-v1.5.onnx, tokenizer.json, config.json,
special_tokens_map.json, tokenizer_config.json. The server's strict local resource
ignores ambient discovery; selecting hosted Cohere for global RAG does not alter
this resource. The ordinary llm.embedding.models_dir config remains its separate
backend input. The resource model/dimension are the known packaged BGE shape,
not an arbitrary fallback context/capacity.

Embedded hosts using existing constructors now must explicitly attach a genuine
local matcher to enable LocalEmbedding. Absent resource yields the checked error
instead of silently borrowing a possibly hosted matcher. Existing constructors,
LocalEmbeddingClassifier::new, matching method labels and result contracts
remain compatible. No new activation policy or model selection feature.

## Evidence and limits

Read Rust workspace/best-practices/async skills and relevant gotchas; source
reads traced actual factory, server lifecycle, registry matching and classifier
contracts. Only source inspection and authorized edits ran with login:false.
No Cargo, compiler, tests, import, model construction/inference, tokenizer parse,
format/lint, review-verification, service or external provider call ran. No PASS
or phase-complete assertion. Actual compilation, lifecycle cleanup, hosted-call
absence and useful local ranking remain unverified under CLOSED.

Guards trace to the actual reported boundary: a LocalEmbedding selection could
inherit hosted Cohere, and missing index/resources could look like successful
empty matching. Pinned serving-instance and strict assets prevent that drift;
metadata labels are not used as substitutes for execution evidence. No fallback
or broad registry refactor was added. The uncomfortable thing is that source
separation still needs future real-path proof that no hosted request occurs and
both selected local paths produce useful candidates. Root must renew inventory
and keep the gate closed until exact operator approval. Ownership released.
