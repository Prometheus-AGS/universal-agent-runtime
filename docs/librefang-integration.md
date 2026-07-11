# LibreFang (bossfang) Integration Guide

CH-18 (`librefang-a2a-agui-bridge`, D-C scope). This document covers the
**UAR side only** — everything here is testable and true today without any
changes to the LibreFang/bossfang codebase. Cross-repo work (changes inside
`librefang`) is explicitly out of scope for this document and for the UAR
change that produced it; see `docs/uar-next-fable.md` §"Grounded bossfang
integration plan" for the fuller cross-repo picture.

## 1. Zero-code seam: UAR as a bossfang model provider

The fastest integration path needs **no new code on either side**: point a
bossfang provider entry at UAR's OpenAI-compatible endpoint.

```
POST http://<uar-host>:<port>/v1/chat/completions
```

This is a real, drop-in OpenAI Chat Completions-compatible endpoint — same
request/response shape (`messages[]` in, `choices[0].message.content` /
`chat.completion.chunk` SSE deltas out) that any OpenAI-client-shaped caller
already knows how to talk to. Configure it in bossfang the same way you'd
configure any other `provider_urls` override, pointing `base_url` at this
path. No UAR-specific request fields are required — `stream_mode` (see
§3 below) is optional and only matters if you want the AG-UI event stream
instead of/alongside plain OpenAI chunks.

Once wired, every bossfang agent/channel routed through this provider
transparently runs through UAR's model router, prompt-dialect engine,
memory, and governance layer — with zero bossfang-side code.

**Verified by:** `tests/integration/live/librefang_seam_cases.rs` ::
`zero_code_seam_v1_chat_completions_is_openai_compatible` — boots a real UAR
server and sends a bare OpenAI-shaped request (no extension fields) to
`/v1/chat/completions`, asserting an `object: "chat.completion"` response
with the expected `choices[0].message` shape.

## 2. A2A task intake contract

UAR exposes A2A (Agent2Agent) on two bindings, both delegating to the same
handler logic (`src/uar/api/a2a/handler.rs` for JSON-RPC,
`src/uar/api/a2a/grpc.rs` for gRPC — see CH-01):

- **JSON-RPC 2.0** — `POST /a2a/compiler`, methods `message/send`,
  `tasks/get`, `tasks/cancel`. Full request/response shapes:
  `docs/A2A_PROTOCOL.md`.
- **gRPC** — `AgentService` on the port configured by
  `UAR_SERVER__GRPC_PORT` (default `50051`; see `docs/configuration.md`),
  methods `MessageSend`, `TaskGet`, `TaskCancel`, `MessageStream`. Proto:
  `proto/a2a.proto`.
- **AgentCard discovery** — `GET /.well-known/agent.json`.

LibreFang already implements A2A client-side (`librefang-runtime/src/a2a.rs`,
per docs/uar-next-fable.md's validated source-tree check) — task delegation
from a bossfang orchestrator to a UAR compiler agent is conformance testing
against these two bindings, not new UAR-side work. `tests/test_a2a_grpc.rs`
(CH-01) exercises the gRPC binding end-to-end; the JSON-RPC binding's
existing coverage is `tests/test_a2a_client.rs`.

## 3. AG-UI stream consumption contract

Set `stream_mode: "agui_spec"` on a streaming `/api/chat/completion` or
`/v1/chat/completions` request (`"stream": true` required) to receive the
**official AG-UI protocol event vocabulary** instead of UAR's legacy
`agui.*`-named events:

| SSE `event:` | Emitted for |
|---|---|
| `RUN_STARTED` | Run begins |
| `TEXT_MESSAGE_CONTENT` | Assistant text delta |
| `REASONING_MESSAGE_CONTENT` | Extended-thinking/reasoning delta |
| `TOOL_CALL_START` | First delta or start of a given tool call (`call_index`/`id`; synthesized once per tool call — no dedicated UAR event exists for it) |
| `TOOL_CALL_ARGS` | Incremental tool-call argument JSON while the model is still generating |
| `TOOL_CALL_END` | Tool call's name+arguments are fully known, ready to execute |
| `TOOL_CALL_RESULT` | Tool finished executing (content + success flag) |
| `STATE_DELTA` | State patch / context update |
| `CUSTOM` | UAR-specific signals with no AG-UI equivalent (citations, skill activation, guardrail flags, memory events) — payload shape unchanged from the legacy event, only the wire name differs |
| `RUN_FINISHED` | Run completes |
| `RUN_ERROR` | Run errors or is cancelled |

This is what CopilotKit, Microsoft Agent Framework, and Oracle A2UI clients
expect on the wire (docs/uar-next-fable.md §7 R6) — `stream_mode: "agui"`
(legacy) and `"dual"` (legacy `agui.*` + OpenAI chunks together) are
unaffected and continue emitting UAR's original event names for existing
consumers; `"agui_spec"` is a fully independent mode, not a replacement, so
adopting it is opt-in per caller.

**Verified by:** `tests/integration/live/librefang_seam_cases.rs` ::
`agui_spec_mode_emits_official_event_vocabulary` — asserts `RUN_STARTED`,
`TEXT_MESSAGE_CONTENT`, `RUN_FINISHED` appear and that legacy `agui.*` names
and raw OpenAI chunks do not.

### Example client

A minimal event-vocabulary-aware SSE consumer, for reference:

```bash
curl -N -X POST http://localhost:3001/api/chat/completion \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "anthropic/claude-sonnet-5",
    "messages": [{"role": "user", "content": "hello"}],
    "stream": true,
    "stream_mode": "agui_spec"
  }' | while IFS= read -r line; do
    case "$line" in
      event:\ RUN_STARTED) echo "[run started]" ;;
      event:\ TEXT_MESSAGE_CONTENT) echo -n "" ;;  # next `data:` line has the delta
      data:*) echo "${line#data: }" | jq -r '.text_delta // empty' ;;
      event:\ RUN_FINISHED) echo "[run finished]" ;;
      event:\ RUN_ERROR) echo "[run error]" ;;
    esac
  done
```

## 4. Shared memory substrate

Both UAR and bossfang depend on the Prometheus-AGS `surreal-memory` server.
Standardizing scope-naming conventions so a bossfang workspace and a UAR
agent can address the same memory graph is cross-repo coordination work, out
of scope for this document — tracked in docs/uar-next-fable.md's integration
plan, not this change.

## 5. Skill bridge

UAR-compiled WASM skills are already deployable to bossfang via the skill
pack's `librefang-wasm-skill` (generates WASM-ABI crates for bossfang's
`WasmSkillSandbox`) and `upload-to-bossfang` (SSRF-guarded POST to
`/skills/install`) skills — no UAR runtime change needed; this is a
skill-pack-level bridge that already exists today.

## 6. Deployment decision: library or supervised service

For the current production architecture, bossfang should consume UAR through a
**supervised out-of-process boundary**, not link the full
`universal-agent-runtime` crate into the bossfang process. The existing
OpenAI-compatible `provider_urls` seam is the lowest-risk first step; A2A and
AG-UI add richer task and event semantics without changing the ownership
boundary.

| Criterion | Current UAR library | Supervised UAR service/sidecar |
|---|---|---|
| Integration effort | High: bossfang must reconcile Tokio, features, native dependencies, and exact Cargo pins | Low: supported `provider_urls` configuration plus stable HTTP/SSE contracts |
| Fault and security isolation | Shared process, allocator, runtime, and crash domain | Separate crash/restart and least-privilege boundary |
| Upgrade and rollback | Coupled bossfang/UAR release and rebuild | Independently versioned and reversible deployment |
| Latency and footprint | Avoids IPC, but embeds UAR's broad server/runtime dependency graph | Small local IPC cost; one reusable runtime process |
| Portability | Rust-only ABI/source integration | Language- and host-neutral OpenAI/A2A/AG-UI protocols |
| Ownership fit | Overlaps bossfang kernel responsibilities | Bossfang owns workflows/channels/approvals; UAR owns model routing and execution policy |

This recommendation applies to the **current monolithic crate**. A future
library option can make sense for offline/mobile packaging or a measured
latency-critical path after UAR extracts a narrow, dependency-light runtime
kernel (for example typed events, routing, and dialect policy) with no server,
UI, database, or process-lifecycle coupling. That extraction should be driven
by profiling rather than assumed IPC cost. Until then, embedding the full crate
would trade a mature protocol seam for a coupled build and failure domain.

For bossfang specifically, “sidecar” means a generic child process or managed
local service, not necessarily the existing Electron-oriented `uar-sidecar`
binary. The supervisor contract should include authenticated loopback or Unix
socket transport, readiness and health checks, protocol/profile negotiation,
bounded restart policy, structured logs, and clean shutdown. This preserves a
later migration to a narrow in-process library without changing bossfang's
agent, workflow, or channel contracts.

The decision is consistent with LibreFang's documented layered
kernel/runtime/API architecture and its supported OpenAI-compatible
`provider_urls` overrides:

- [LibreFang architecture](https://docs.librefang.ai/architecture)
- [LibreFang configuration](https://docs.librefang.ai/configuration)
- [LibreFang documentation](https://docs.librefang.ai/)
