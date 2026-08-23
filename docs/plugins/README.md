# UAR Plugin Architecture

> **Historical — superseded 2026-08-23.** This directory describes a proposed
> universal plugin and realtime architecture. The current source retains a
> capability-denying WASM loader contract whose execution methods are not wired;
> see the [current tools authority](/docs/tools/overview).

_Last updated: 2026-02-21_

Plugins are the **extensibility backbone** of the Prometheus AGS ecosystem. A plugin is a first-class participant in the runtime — it can listen to everything happening in the system, execute long-running jobs in sandboxed microVMs with access to LLM models, and publish events back through `uar-realtime` to any client in the world.

---

## Documentation Index

| Document | Description |
|---|---|
| [overview.md](./overview.md) | Design goals, plugin types, ADRs |
| [manifest.md](./manifest.md) | Plugin descriptor format — channels, tools, capabilities |
| [capabilities.md](./capabilities.md) | What plugins can do — realtime, code runner, LLM, MCP |
| [lifecycle.md](./lifecycle.md) | Plugin registration, startup, event dispatch, shutdown |
| [channels.md](./channels.md) | Plugin channel taxonomy, event naming conventions |
| [examples.md](./examples.md) | Concrete plugin examples with full code |

---

## TL;DR — The Plugin Mental Model

```
                    ┌─────────────────────────────────────┐
                    │         uar-realtime                 │
                    │                                     │
  UAR emits ───────►│  system:notifications               │◄─── sandbox emits
  agent emits ─────►│  agent:run:{id}                     │◄─── plugin emits
  plugin emits ────►│  session:{id}                       │     
                    │  plugin:{name}:{scope}              │────► clients
                    └─────────────────────────────────────┘────► other plugins
                                        ▲▲
                    ┌───────────────────┘└──────────────────────┐
                    │  Plugin                                    │
                    │                                           │
                    │  1. LISTENS to any realtime channel       │
                    │  2. SPAWNS code runner sandboxes          │
                    │     (in microVMs with full Linux env)     │
                    │  3. CALLS LLM models via UAR              │
                    │  4. EMITS events back to realtime          │
                    │  5. EXPOSES MCP tools to agents           │
                    └───────────────────────────────────────────┘
```

### Example: Live Video Transcription Plugin

```
User starts a video call session
  │
  ├── plugin:transcription listens on session:{id}:media
  │
  ├── media:track:started event arrives →
  │     plugin spawns code runner sandbox
  │         (Python microVM + whisper + LLM summarization)
  │
  ├── audio frames flow into sandbox
  │     sandbox transcribes in real time
  │     sandbox calls UAR LLM API for speaker diarization
  │     sandbox emits: plugin:transcription:{id}:segment
  │                           → frontend shows live captions
  │
  └── session ends → plugin:transcription:{id}:summary emitted
                        → full meeting summary delivered to user
```

**All of this uses no custom infrastructure** — realtime is `uar-realtime`, compute is `uar-code-interpreter`, and LLM access is the existing UAR model routing layer.
