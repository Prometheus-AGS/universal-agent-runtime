# RAG

The UAR RAG subsystem provides retrieval-augmented generation with pluggable embedding backends, citation streams, and a frozen golden-set evaluation harness.

## Topics

- Citation stream UX (`[1]`, `[2]` markers in SSE events)
- Embedding backends (FastEmbed, candle, OpenAI, Voyage, Cohere)
- RAGAS + DeepEval evaluation
- Golden set management in `evals/rag-golden-set/`

## Related documents

- [RAG benchmark results](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/rag-benchmark/README.md)
- [Citation stream ADR](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/adr/0008-rag-citation-stream.md)
