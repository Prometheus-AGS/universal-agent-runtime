## Context

UAR has mature provider, skill, MCP, RAG, context, cancellation, and normalized-event subsystems, but they are connected through separate global or agent-only paths. `AgentSessionConfig` is process-local, the run API accepts no typed resource policy, and the chat handler consumes only the selected agent. Embedded hosts can supply one external LLM callback, but cannot advertise models, lifecycle, or capabilities.

## Goals / Non-Goals

**Goals:**

- Make UAR the single policy and execution owner for cloud, embedded-local, and agent-selected chat.
- Resolve global, agent, conversation, and turn settings deterministically and persist the immutable result with each run.
- Apply allowed skills, MCP servers, knowledge bases, context strategy, memory, approval policy, and provider/model route to actual execution.
- Let embedded hosts register capability-aware local model providers without bypassing UAR orchestration.
- Preserve backward request compatibility and normalized realtime observability.

**Non-Goals:**

- Move UI state or prompt construction into React, Flutter, or host adapters.
- Permit mobile hosts to spawn arbitrary stdio MCP sidecars.
- Add implicit local-to-cloud fallback.

## Decisions

1. Add `RunPolicy` and `EffectiveRunPolicy` domain types. Resource selections use `inherit`, `auto`, `all`, `none`, or `selected`. Resolution intersects eligibility at each scope; deny and disabled state always win.
2. Store conversation policies through `PersistenceLayer` using a versioned JSON record. In-memory, SurrealDB, and PostgreSQL providers implement the same contract. `AppState` keeps a read-through cache, not an independent source of truth.
3. Extend `RunManager::start_run` with a typed execution context rather than additional positional arguments. The manager resolves and stores policy before retrieval, skill matching, MCP filtering, context trimming, or driver construction.
4. Filter resource registries before prompt assembly. An empty selected KB set means no retrieval, never "search everything". An empty MCP selection exposes no external tools. Skill matching operates only over the eligible set.
5. Resolve context tokens from the selected model capability profile and persist the effective strategy; remove the hard-coded 128K run budget.
6. Add a `LocalModelProvider` registry above `ExternalLlmDriver`. Providers expose catalog, capability, lifecycle, diagnostics, prepare, stream, cancel, and unload operations. A local route is explicit and has no silent cloud fallback.
7. Emit `PolicyResolved` plus existing lifecycle, skill, retrieval, MCP/tool, reasoning, citation, and terminal events into replayable history. The event contains identifiers and provenance but no credentials or hidden reasoning.
8. Keep HTTP and embedded-library transports as thin adapters over the same service methods. Existing clients omitting policy receive legacy-equivalent defaults.

## Risks / Trade-offs

- [Migration changes the run signature] → retain compatibility wrappers while first-party clients migrate, then remove only after certification.
- [Persisted policies reference deleted resources] → resolve them as unavailable, emit a warning, and never silently broaden access.
- [All-resource selection can overflow model context] → run capability and token preflight and return an actionable error or require an explicit narrower selection.
- [A host local provider can misstate capabilities] → certify provider/model pairs and reject unsupported request parts before generation.
- [MCP transport differs by platform] → enforce runtime transport capabilities during policy resolution.

## Migration Plan

1. Add versioned types, persistence methods, and migrations with backward readers.
2. Populate conversation policy from existing agent-session configuration on first read.
3. Integrate policy resolution and audit events into the run path while preserving old request decoding.
4. Register embedded providers and migrate KnowMe desktop, web, and Flutter adapters.
5. Certify desktop, then physical Android; rollback by omitting policy and disabling embedded-provider registration while retaining readable records.

## Open Questions

None. Platform transport constraints and precedence rules are fixed by this design.
