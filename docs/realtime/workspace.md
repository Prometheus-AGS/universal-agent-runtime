# UAR Realtime — Cargo Workspace Structure

_Last updated: 2026-02-21 | Status: Planned_

This document describes how to convert the `universal-agent-runtime` single-crate project into a **Cargo workspace monorepo** that houses both `universal-agent-runtime` and the new `uar-realtime` service as independent binaries, with all shared realtime code in a common library crate.

The same codebase supports three deployment modes controlled by Cargo features and runtime config — no forking, no code duplication.

---

## 1. Why a Cargo Workspace (not a separate repo)

`uar-realtime` shares core types with UAR that **must change together** atomically:

| Shared type | Why it can't drift between repos |
|---|---|
| `Envelope` | Same wire format — a change to a field breaks both sides simultaneously |
| `SubscriptionFilter` + predicate DSL | UAR validates filters at publish; `uar-realtime` evaluates them — they must agree |
| `UserContext` / JWT claims | Single auth contract |
| Topic pattern grammar | Same routing logic on both sides |
| `EventEmitter` API | UAR calls this to publish; it targets `uar-realtime` |

A cross-repo PR for a field rename in `Envelope` creates coordination overhead and version-skew risk. An atomic commit in a workspace eliminates both.

**Monorepo ≠ mono-binary.** The two binaries are compiled, Dockerized, and deployed completely independently.

---

## 2. Deployment Modes

The `EventEmitter` trait abstraction supports three deployment modes. **Call-site code is identical in all modes.**

| Mode | When | Emitter impl | Realtime process |
|---|---|---|---|
| **Tauri** | Desktop app (`--features tauri`) | `InProcessEmitter` | None — embedded in UAR binary |
| **Dev / single-node cloud** | No external URL configured | `InProcessEmitter` | Optional in-process WS endpoint |
| **Cloud (multi-node)** | `UAR_REALTIME__EXTERNAL_URL` set | `HttpEmitter` | Standalone `uar-realtime` process |

The Tauri and dev single-node cases both use `InProcessEmitter` — the difference is that in Tauri, the WebSocket server also runs **inside the same process** (served by the embedded Axum instance Tauri wraps). There is no separate `uar-realtime` binary to run, deploy, or manage.

---

## 3. Target Directory Layout

```
universal-agent-runtime/           ← git repo root
  Cargo.toml                       ← WORKSPACE root (modified)
  Cargo.lock                       ← single lockfile for all workspace members

  src/                             ← existing UAR source (unchanged)
    uar/
      realtime/
        mod.rs                     ← re-exports (thin shim; real code in uar-realtime-core)

  crates/
    uar-realtime-core/             ← NEW: shared library (used by BOTH binaries + Tauri)
      Cargo.toml
      src/
        lib.rs
        envelope.rs                ← Envelope struct + serde
        filter.rs                  ← SubscriptionFilter + predicate evaluator
        topic.rs                   ← topic pattern matching (glob)
        auth.rs                    ← JWT claim extraction
        emitter.rs                 ← EventEmitter trait + InProcessEmitter + HttpEmitter
        broker.rs                  ← RealtimeBroker (tokio::broadcast)
        presence.rs                ← Presence state types
        error.rs                   ← RealtimeError
        websocket.rs               ← WS handler (used in-process for Tauri + dev mode)

  uar-realtime/                    ← NEW: standalone binary (cloud mode only)
    Cargo.toml
    Dockerfile
    src/
      main.rs
      config.rs
      transport/
        mod.rs                     ← Transport trait
        websocket.rs               ← delegates to uar-realtime-core::websocket
        sse.rs
        webrtc/
          mod.rs
          signaling.rs
          data_channel.rs
          media/
            sfu.rs
            recording.rs
      broker/
        nats.rs                    ← NATS back-plane (cluster mode)
      internal/
        publish.rs                 ← POST /internal/v1/publish (UAR → uar-realtime)
```

---

## 3. Cargo.toml Changes

### 3.1 Root `Cargo.toml` — convert to workspace

```toml
# Cargo.toml (root)

[workspace]
members = [
    ".",                           # universal-agent-runtime (existing package)
    "crates/uar-realtime-core",    # shared types
    "uar-realtime",                # standalone realtime service
]
resolver = "2"

# Shared dependency versions — avoids duplication across members
[workspace.dependencies]
axum          = { version = "0.8" }
tokio         = { version = "1",   features = ["full"] }
serde         = { version = "1",   features = ["derive"] }
serde_json    = "1"
tracing       = "0.1"
uuid          = { version = "1",   features = ["v4", "serde"] }
jsonwebtoken  = "10"
anyhow        = "1"
thiserror     = "2"
async-trait   = "0.1"

[package]
name    = "universal-agent-runtime"
version = "0.1.0"
edition = "2024"

[dependencies]
# existing deps unchanged — add:
uar-realtime-core = { path = "crates/uar-realtime-core" }

# Use workspace versions for shared deps:
axum       = { workspace = true, features = ["multipart"] }
tokio      = { workspace = true, features = ["full", "process"] }
serde      = { workspace = true }
serde_json = { workspace = true }
# ... rest of existing deps unchanged
```

### 3.2 `crates/uar-realtime-core/Cargo.toml`

```toml
[package]
name    = "uar-realtime-core"
version = "0.1.0"
edition = "2024"

[dependencies]
serde       = { workspace = true }
serde_json  = { workspace = true }
tokio       = { workspace = true }
tracing     = { workspace = true }
uuid        = { workspace = true }
jsonwebtoken = { workspace = true }
anyhow      = { workspace = true }
thiserror   = { workspace = true }
async-trait = { workspace = true }
axum        = { workspace = true, features = ["ws"] }  # WS handler runs in-process for Tauri + dev
reqwest     = { version = "0.12", features = ["json", "rustls-tls-native-roots"], optional = true }

[features]
# Include the HTTP emitter (UAR → uar-realtime HTTP publish) — cloud mode
http-emitter = ["dep:reqwest"]
# Tauri mode — identical to dev/in-process; flag allows conditional compilation in UAR
tauri        = []
```

### 3.3 `uar-realtime/Cargo.toml`

```toml
[package]
name    = "uar-realtime"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "uar-realtime"
path = "src/main.rs"

[dependencies]
uar-realtime-core = { path = "../crates/uar-realtime-core" }

axum        = { workspace = true, features = ["ws", "macros"] }
tokio       = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
tracing     = { workspace = true }
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
anyhow      = { workspace = true }
thiserror   = { workspace = true }
async-trait = { workspace = true }
uuid        = { workspace = true }
tower-http  = { version = "0.6", features = ["cors", "trace"] }
config      = "0.15"
mimalloc    = "0.1"

# WebRTC (P2 — enable when signaling is implemented)
webrtc = { version = "0.11", optional = true }

# NATS back-plane (P3 — enable for cluster mode)
async-nats = { version = "0.37", optional = true }

[features]
default     = []
webrtc      = ["dep:webrtc"]
cluster     = ["dep:async-nats"]
```

---

## 4. Shared Code — `uar-realtime-core`

### 4.1 `EventEmitter` as a trait

The key design is that `EventEmitter` is a **trait** in `uar-realtime-core`, with two implementations:

```rust
// crates/uar-realtime-core/src/emitter.rs

use async_trait::async_trait;
use serde_json::Value;

/// The single emit API used everywhere in UAR (and in uar-realtime internally).
/// Call-site code never changes regardless of which transport is active.
#[async_trait]
pub trait EventEmitter: Send + Sync {
    async fn emit(&self, topic: &str, event: &str, payload: Value) -> usize;

    // Typed helpers: delegate to emit()
    async fn agent_token_delta(&self, run_id: &str, delta: &str, index: u64) -> usize {
        self.emit(
            &format!("agent:run:{run_id}"),
            "agent:token:delta",
            serde_json::json!({ "delta": delta, "token_index": index }),
        ).await
    }

    async fn agent_run_completed(&self, run_id: &str, reason: &str, tokens: u32, ms: u64) -> usize {
        self.emit(
            &format!("agent:run:{run_id}"),
            "agent:run:completed",
            serde_json::json!({ "run_id": run_id, "finish_reason": reason, "total_tokens": tokens, "duration_ms": ms }),
        ).await
    }

    async fn system_notification(&self, level: &str, title: &str, body: &str) -> usize {
        self.emit(
            "system:notifications",
            "system:notification",
            serde_json::json!({ "level": level, "title": title, "body": body }),
        ).await
    }

    // ... all other typed helpers
}

// ── Implementation 1: In-process (Tier 1 / dev) ────────────────────────────

pub struct InProcessEmitter {
    pub broker: Arc<crate::broker::RealtimeBroker>,
}

#[async_trait]
impl EventEmitter for InProcessEmitter {
    async fn emit(&self, topic: &str, event: &str, payload: Value) -> usize {
        self.broker.publish(topic, event, payload).await
    }
}

// ── Implementation 2: HTTP to uar-realtime (Tier 2+) ──────────────────────

#[cfg(feature = "http-emitter")]
pub struct HttpEmitter {
    client: reqwest::Client,
    base_url: String,
    secret: String,
}

#[cfg(feature = "http-emitter")]
#[async_trait]
impl EventEmitter for HttpEmitter {
    async fn emit(&self, topic: &str, event: &str, payload: Value) -> usize {
        self.client
            .post(format!("{}/internal/v1/publish", self.base_url))
            .bearer_auth(&self.secret)
            .json(&serde_json::json!({ "topic": topic, "event": event, "payload": payload }))
            .send()
            .await
            .map(|_| 1)
            .unwrap_or(0)
    }
}
```

### 4.2 `AppState` in UAR

```rust
// src/lib.rs — AppState gains one field

pub struct AppState {
    // ... existing fields ...

    /// Realtime event emitter. Emits to in-process broker (Tier 1)
    /// or to uar-realtime via HTTP (Tier 2+), depending on config.
    pub realtime: Arc<dyn uar_realtime_core::emitter::EventEmitter>,
}

impl AppState {
    /// Emit a realtime event from anywhere with a `State<AppState>`.
    ///
    /// ```rust
    /// state.realtime().agent_token_delta(&run_id, &delta, idx).await;
    /// ```
    pub fn realtime(&self) -> &dyn uar_realtime_core::emitter::EventEmitter {
        self.realtime.as_ref()
    }
}
```

### 4.3 Emitter selection in `server.rs` — all three modes

```rust
// src/server.rs — choose emitter implementation from feature flags + config

let realtime: Arc<dyn EventEmitter> =

    // ── Tauri desktop mode ─────────────────────────────────────────────────
    // Compiled with `--features tauri`. Everything runs in the same process.
    // The Axum WebSocket endpoint /api/realtime is served by the embedded
    // Axum server that Tauri wraps — no external service needed.
    #[cfg(feature = "tauri")]
    {
        let broker = Arc::new(RealtimeBroker::new());
        // Register the in-process WS handler on the router (done in build_router)
        Arc::new(InProcessEmitter { broker })
    }

    // ── Cloud mode (multi-node) ────────────────────────────────────────────
    // UAR_REALTIME__EXTERNAL_URL is set — publish to standalone uar-realtime.
    #[cfg(not(feature = "tauri"))]
    if let Some(url) = &config.realtime.external_url {
        Arc::new(HttpEmitter::new(url, &config.realtime.internal_secret))
    }

    // ── Dev / single-node cloud ────────────────────────────────────────────
    // No external URL — in-process broker, same as Tauri.
    #[cfg(not(feature = "tauri"))]
    else {
        let broker = Arc::new(RealtimeBroker::new());
        Arc::new(InProcessEmitter { broker })
    };

let state = AppState {
    // ...
    realtime,
};
```

> **In all modes, every call-site is identical:**
> ```rust
> state.realtime().agent_token_delta(&run_id, &delta, idx).await;
> ```

---

## 5. Tauri Mode — In-Depth

In Tauri mode, `uar-realtime` is not a separate process — it is the **library crates from `uar-realtime-core`** embedded directly into the UAR binary, which Tauri wraps.

```
Tauri shell
  └── UAR binary (single process)
        ├── Axum HTTP server
        │     ├── /api/chat/completion   (LLM inference)
        │     ├── /api/realtime          (WebSocket endpoint — in-process broker)
        │     └── /api/...              (all other routes)
        ├── RealtimeBroker              (tokio::broadcast, in-memory)
        ├── InProcessEmitter            (emit() → broker.publish())
        └── RunManager, MemoryService, etc.
```

### Why this is correct for Tauri

| Concern | Cloud | Tauri |
|---|---|---|
| Scaling | Need separate process per resource type | Single user on one machine — no scale needed |
| Restarts | UAR restarts must not drop WS connections | The whole app restarts; user expects it |
| Network | Inter-process HTTP adds latency | No network hop at all — in-process is zero-cost |
| Complexity | Complexity is warranted by scale requirements | Simplicity wins — single binary, single deploy |
| WebRTC | `uar-realtime` handles signaling server | Tauri's WebView provides native WebRTC APIs directly; in-process signaling over the embedded WS |

### Cargo build commands

```bash
# Build for Tauri (desktop) — includes in-process realtime, no http-emitter
cargo build --features tauri -p universal-agent-runtime

# Build for cloud — includes http-emitter, no in-process broker in UAR
cargo build --features http-emitter -p universal-agent-runtime

# Build uar-realtime standalone service (cloud deployment)
cargo build -p uar-realtime
```

### `tauri.conf.json` — no realtime service in the build

The Tauri build does not include `uar-realtime` as a sidecar. The feature flag ensures it is not compiled in:

```json
// tauri.conf.json
{
  "bundle": {
    "externalBin": []
  }
}
```

---

## 5. Build & Docker

### Building locally

```bash
# Build just UAR
cargo build -p universal-agent-runtime

# Build just uar-realtime
cargo build -p uar-realtime

# Build both (cargo workspace builds all members)
cargo build

# Test everything
cargo test
```

### Dockerfiles

**UAR** (`Dockerfile` — already exists, no changes needed):
```dockerfile
# Already targets the `universal-agent-runtime` binary
COPY . .
RUN cargo build --release -p universal-agent-runtime
```

**uar-realtime** (`uar-realtime/Dockerfile` — new):
```dockerfile
FROM rust:1.85-slim AS builder
WORKDIR /build
COPY . .
RUN cargo build --release -p uar-realtime

FROM debian:bookworm-slim
COPY --from=builder /build/target/release/uar-realtime /usr/local/bin/
EXPOSE 4001 4002
CMD ["uar-realtime"]
```

### Docker Compose (dev)

```yaml
# docker-compose.dev.yaml (additions)
services:
  uar:
    build:
      context: .
      dockerfile: Dockerfile
    environment:
      UAR_REALTIME__EXTERNAL_URL: "http://uar-realtime:4002"
      UAR_REALTIME__INTERNAL_SECRET: "${INTERNAL_SECRET}"

  uar-realtime:
    build:
      context: .
      dockerfile: uar-realtime/Dockerfile
    ports:
      - "4001:4001"
    environment:
      URT_PUBLIC_PORT: 4001
      URT_INTERNAL_PORT: 4002
      URT_JWT_SECRET: "${JWT_SECRET}"
      URT_INTERNAL_SECRET: "${INTERNAL_SECRET}"
```

---

## 6. Migration Steps (When Ready to Implement)

1. **Add `[workspace]` table** to root `Cargo.toml`
2. **Create `crates/uar-realtime-core/`** — move `Envelope`, `SubscriptionFilter`, `EventEmitter` trait from `src/uar/realtime/` into the shared crate
3. **Update root `Cargo.toml` deps** to use `workspace = true` for shared versions
4. **Create `uar-realtime/`** — new crate with binary entry point, transports
5. **Add `uar-realtime-core` dep** to both root and `uar-realtime/Cargo.toml`
6. **Update `AppState`** to use `Arc<dyn EventEmitter>`
7. **Wire `server.rs`** to select in-process vs HTTP emitter from config
8. **Add `uar-realtime/Dockerfile`**
9. **Update `docker-compose.dev.yaml`**
10. Run `cargo build` and `cargo test` — workspace ensures everything compiles together
