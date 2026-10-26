# Cohere through the OpenAI embedding interface

Set `llm.embedding.backend: openai` and `send_dimensions: false` for Cohere's
`https://api.cohere.ai/compatibility/v1` endpoint. Keep the existing credential
reference and explicitly select model and expected `vector_dimension`.
[The compatibility API](https://docs.cohere.com/v2/docs/compatibility-api) supports
input/model/encoding_format for embeddings and does not support dimensions.
The companion YAML is an example, not a verified availability receipt.

`send_dimensions` defaults to true and preserves the established behavior: send
the optional field for non1536 configured widths. False suppresses it; it never
changes the expected width or embedding fingerprint. The OpenAI backend validates
every returned vector against configured dimension and returns DimensionMismatch
instead of silently padding/truncating. This applies to model pairs of equal
width as well: equal widths do not establish equal embedding spaces.

The setting is propagated from config through the actual RAG embedding backend,
seeded in typed settings, validated before live mutation, and boot-hydrated after
restart. Existing local-failover settings ownership still rejects live changes.
This option does not alter the separate memory-service embedding SDK contract.

Documented model/width choices are embed-english-v3.0 (1024),
embed-multilingual-v3.0 (1024), and embed-english-light-v3.0 (384). These names and
widths do not prove access, model compliance, retrieval quality or real provider
availability. The phase gate remains CLOSED: no import, test, build, check,
service probe, embedding or inference request was executed for this change.
