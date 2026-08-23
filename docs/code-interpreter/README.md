# UAR Code Interpreter

> **Historical — superseded 2026-08-23.** This directory describes a proposed
> standalone `uar-code-interpreter` service that is not a Cargo workspace member.
> Current UAR provides governed in-process Wasmtime and remote sandbox tools; see
> the [current tools authority](/docs/tools/overview).

_Last updated: 2026-02-21_

`uar-code-interpreter` is a standalone Rust service (Cargo workspace member) that provides **secure, sandboxed code execution** for the Prometheus AGS ecosystem — the UAR equivalent of [e2b](https://e2b.dev) or Kimi's built-in code runner, designed specifically for this project's agent workflows, plugin system, and Tauri desktop deployment.

---

## Documentation Index

| Document | Description |
|---|---|
| [overview.md](./overview.md) | Design goals, prior art (e2b, microsandbox, Kimi), ADRs |
| [architecture.md](./architecture.md) | System architecture, execution modes, workspace structure |
| [runners.md](./runners.md) | Sandbox runner backends — microsandbox, Firecracker, Wasmtime, remote |
| [languages.md](./languages.md) | Language support — Bash, Rust, Python, Node.js |
| [platforms.md](./platforms.md) | Platform matrix — Linux, macOS, Windows, iOS, Android |
| [api-reference.md](./api-reference.md) | HTTP API, MCP tool interface, streaming output |

---

## TL;DR

```
Agent request: "Write and run a Rust program that calculates primes"
    │
    ▼
UAR agent calls MCP tool: code_exec
    │
    ▼
uar-code-interpreter
    ├─ spins up a microsandbox microVM (Linux/macOS)
    │   or Wasmtime sandbox (iOS/Android/restricted)
    ├─ writes code to /workspace/main.rs
    ├─ runs: cargo run
    └─ streams stdout/stderr back via uar-realtime events
    │
    ▼
Client sees output in real-time
```

## Key Features

- **microVM isolation** (microsandbox/libkrun) — hardware-level security, dedicated kernel per sandbox
- **Four languages** — Bash, Rust, Python, Node.js — all with persistent workspace support
- **Five platforms** — Linux, macOS, Windows, iOS, Android — via tiered runner selection
- **Three execution modes** — ephemeral, session-persistent, project (full repo checkout)
- **Streaming output** — stdout/stderr streamed via `uar-realtime` events
- **MCP tool server** — `code_exec`, `shell_exec`, `file_read`, `file_write` tools
- **Tauri mode** — runs as a bundled sidecar on desktop (same binary, no cloud dependency)
- **Kimi-style session sandboxes** — sandbox persists across agent turns; state accumulates
