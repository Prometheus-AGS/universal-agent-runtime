# Change 11 verification and integration notes

The KBD identifier contains a dot, which OpenSpec rejects. The valid OpenSpec
slug is therefore `sdk-typescript-1-0`; the proposal retains the exact KBD ID.

The branch starts at `b9a85515`, before uncommitted Changes 6–8. Integration
must reconcile any finalized authentication, error-envelope, configuration, or
Vault contracts from those changes. The TypeScript client currently preserves
arbitrary error details to make that reconciliation non-breaking where possible.

The committed server exposes run lifecycle, tool execution, chat completion,
knowledge, and ingest routes used here. It does not expose `/v1/embeddings` in
this checkpoint even though the phase plan requires embeddings in the 1.0 SDK
surface. The SDK implements the conventional OpenAI-compatible endpoint, but a
live-server embedding example/test is deferred until the server contract lands.

The Rust 1.0 branch is executing concurrently, so exact symbol-for-symbol
comparison cannot occur inside this isolated worktree. This SDK implements the
capability contract stated by plan.md (chat/streaming, tools, structured output,
embeddings, run create/stream/cancel/resume/checkpoints, knowledge CRUD/search,
and ingest). Integration should compare the three SDK branch manifests before
merge and resolve naming-only differences without removing capabilities.

Package publication, live GitHub Pages deployment, and live-server smoke tests
are deferred because this dispatch does not authorize external publication or
service mutation. Local TypeDoc generation and all offline focused gates pass.
