# UAR Plugin Architecture — Overview & Design

_Last updated: 2026-02-21_

---

## 1. Design Goals

| Goal | Detail |
|---|---|
| **Composable** | Plugins compose the three UAR services: realtime (events), code interpreter (compute), and LLM routing (intelligence) |
| **Event-driven** | Plugins react to events — not polled, not scheduled (though scheduling is possible via events) |
| **Sandboxed compute** | Plugin jobs run in isolated microVMs — a misbehaving plugin cannot affect the host |
| **LLM-aware** | Plugin sandboxes can call UAR's LLM routing layer — full model access within the sandbox |
| **Channel-isolated** | Plugins own the `plugin:{name}:*` namespace; they cannot write to other namespaces |
| **MCP-exposable** | Plugins can register MCP tools that UAR agents can call |
| **Scalable** | Plugin jobs run in `uar-code-interpreter` sandboxes — horizontally scalable without plugin-specific infra |
| **Discoverable** | Plugin manifest declares all capabilities, channels, and tools at registration time |

---

## 2. Plugin Types

### Type A: WASM In-Process Plugin

- Compiled to `wasm32-wasip2`
- Runs inside UAR's existing `wasmtime` sandbox (the `wasm-runtime` feature)
- Lowest latency — in-process, sub-millisecond event dispatch
- **Use for:** lightweight event handlers, data transformation, simple MCP tools
- **Cannot:** spawn long-running processes, install packages, access filesystem

### Type B: Code Runner Plugin

- Registered plugin descriptor + code that runs in `uar-code-interpreter` sandboxes
- The plugin's "brain" is code (Python, Rust, Node, Bash) running in a full microVM
- **Can:** run for hours, install packages, call LLM APIs, compile code, access filesystem
- **Use for:** video transcription, background analysis, data pipelines, CI/CD jobs
- This is the primary plugin type for compute-heavy or long-running work

### Type C: External Service Plugin

- A standalone service (any language) that registers with UAR at startup
- Communicates via `uar-realtime` (subscribe/publish) and UAR's internal APIs
- **Use for:** enterprise integrations, existing services that want UAR integration
- Same channel isolation rules apply

---

## 3. The Three Primitives Every Plugin Uses

### Primitive 1: `uar-realtime` — the message bus

Every plugin has access to `uar-realtime` as both a subscriber and publisher:

```
Subscribe:  plugin listens on any channel (with permission)
Publish:    plugin emits on plugin:{name}:* channels

Examples:
  listen on: session:{id}:agent   → react to every agent output
  listen on: system:notifications → react to system events
  listen on: agent:run:*          → react to any agent run
  emit on:   plugin:transcription:{id}:segment
```

### Primitive 2: `uar-code-interpreter` — scalable compute

Plugins can spawn sandboxes for jobs:

```
plugin receives event → spawns sandbox → sandbox runs for hours
  → sandbox accesses LLM → sandbox emits events back to realtime
  → sandbox produces artifacts
```

### Primitive 3: UAR LLM Routing — model access

From inside a sandbox, plugins can call LLMs through UAR's model routing:

```python
# Inside a sandbox — calls UAR's LLM API
import uar

response = uar.llm.chat(
    model="auto",   # UAR resolves to best available model
    messages=[{"role": "user", "content": f"Summarize: {transcript}"}]
)
summary = response.content
```

The sandbox uses the session's JWT for auth — no separate credentials needed.

---

## 4. Architectural Decisions

### ADR-001: Plugins own `plugin:{name}:*` channel namespace

**Decision:** Each plugin registers a unique name (e.g., `transcription`, `canvas`, `ci-runner`). It can only publish to `plugin:{name}:*` channels. It can subscribe to any channel it has permission for.

**Rationale:** Namespace isolation prevents plugins from interfering with each other or spoofing system events. The UAR broker enforces this at publish time — publishing to an unauthorized namespace returns an error.

---

### ADR-002: Plugin compute always runs in `uar-code-interpreter`

**Decision:** Plugin compute jobs (Type B) do not get their own process manager. They use `uar-code-interpreter` sandboxes as their execution environment.

**Rationale:**
- Reuses existing resource limits, security policy, and platform support
- Plugin jobs benefit from the code interpreter's microVM isolation immediately
- Horizontal scaling of plugin compute = scaling `uar-code-interpreter`
- Plugin code gets the same sandbox-to-realtime emit capability as agent code

---

### ADR-003: Plugins emit through `uar-realtime`, not through UAR directly

**Decision:** A plugin sandbox emits events directly to `uar-realtime`'s internal publish API — not via UAR.

**Rationale:**
- Eliminates UAR as a bottleneck in plugin event throughput
- Plugin sandboxes and UAR agents are peers on the same event bus
- Clients connect once to `uar-realtime` and receive events from UAR, plugins, and sandboxes without knowing which source produced them

---

### ADR-004: Plugin manifest is a TOML/YAML descriptor

**Decision:** Each plugin is described by a `plugin.toml` manifest that declares its name, channel subscriptions, MCP tools, sandbox configuration, and required capabilities.

**Rationale:** Static declaration allows UAR to validate permissions, set up channel subscriptions, and pre-allocate resources at registration time rather than at runtime.

---

### ADR-005: LLM access from sandboxes uses the session JWT

**Decision:** Plugin sandbox jobs inherit the session JWT and use it to call UAR's LLM API at `http://uar:8080/api/llm/*` (internal, not public).

**Rationale:** Reuses existing auth — no separate API key management for plugins. The UAR LLM router enforces model access policies on the JWT's `roles` claim as usual.
