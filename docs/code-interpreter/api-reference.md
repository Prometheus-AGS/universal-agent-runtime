# UAR Code Interpreter — API Reference

_Last updated: 2026-02-21_

## Key Principle: Sandboxes Are First-Class Realtime Publishers

All sandbox execution output flows through `uar-realtime`. More importantly, **code running _inside_ a sandbox can emit events directly to `uar-realtime`** — through an injected HTTP client that talks to the internal publish API. This means:

- A plugin's sandbox job can publish status, results, progress, or arbitrary domain events to any realtime channel
- Clients (frontend, other plugins, external API callers) receive all real-time information from a **single place** — the `uar-realtime` WebSocket connection
- A video transcription job running in a sandbox can push each transcript segment as a `plugin:transcription:{session_id}:segment` event the moment it's produced, rather than buffering all output until the job completes

### Sandbox → realtime emit (injected SDK)

Each sandbox receives an injected environment variable `UAR_REALTIME_PUBLISH_URL` pointing to the `uar-realtime` internal publish endpoint, plus a `UAR_SANDBOX_TOKEN` for auth. The language-specific SDK wrappers make this ergonomic:

```python
# Python — injected into every sandbox at startup
import os, requests

class Realtime:
    _url = os.environ["UAR_REALTIME_PUBLISH_URL"]
    _token = os.environ["UAR_SANDBOX_TOKEN"]

    @staticmethod
    def emit(topic: str, event: str, payload: dict):
        requests.post(
            f"{Realtime._url}/internal/v1/publish",
            headers={"Authorization": f"Bearer {Realtime._token}"},
            json={"topic": topic, "event": event, "payload": payload},
            timeout=2,
        )

# Usage inside sandbox code:
Realtime.emit("plugin:transcription:sess_123", "transcript:segment", {
    "text": "The meeting has been called to order.",
    "timestamp_ms": 4200,
    "speaker": "Alice"
})
```

```javascript
// Node.js — auto-imported via global setup
const { emit } = require('uar-realtime');
emit('plugin:canvas:board_42', 'plugin:canvas:cursor:moved', { x: 140, y: 290 });
```

```rust
// Rust — via injected uar-realtime-core crate or HTTP
use uar_realtime_core::SandboxEmitter;
let emitter = SandboxEmitter::from_env();  // reads env vars
emitter.emit("plugin:my-plugin:results", "job:completed", json!({ "count": 42 })).await;
```

```bash
# Bash — curl wrapper injected into PATH
uar-emit "plugin:my-plugin:status" "job:progress" '{"pct": 75}'
```

`uar-code-interpreter` exposes two interfaces:

1. **MCP Tool Server** — primary interface for UAR agents (tool calls)
2. **HTTP API** — for direct integration, internal calls from UAR, and remote runner delegation

---

## 1. MCP Tool Interface

The code interpreter registers as an MCP server. UAR agents invoke its capabilities as standard MCP tool calls — no new agent protocol needed.

### Tool: `code_exec`

Execute code in an isolated sandbox.

```json
{
  "tool": "code_exec",
  "arguments": {
    "language": "python",
    "code": "import pandas as pd\ndf = pd.DataFrame({'x': [1,2,3]})\nprint(df.describe())",
    "mode": "session",
    "session_id": "sess_abc123",
    "timeout_seconds": 60
  }
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|---|---|---|---|
| `language` | `"bash"` \| `"rust"` \| `"python"` \| `"node"` \| `"auto"` | ✅ | Runtime. `"auto"` triggers detection. |
| `code` | `string` | ✅ | Source code to execute |
| `mode` | `"ephemeral"` \| `"session"` \| `"project"` | ❌ | Default: `"ephemeral"` |
| `session_id` | `string` | Required for `session`/`project` mode | Sandbox lifetime tied to this session |
| `timeout_seconds` | `integer` | ❌ | Default: 300. Max: 86400 |
| `env` | `object` | ❌ | Environment variables to inject |
| `stdin` | `string` | ❌ | stdin to provide |

**Response (streaming via uar-realtime, then final MCP response):**

```json
{
  "exit_code": 0,
  "stdout": "   x\ncount  3.0\nmean   2.0\n...",
  "stderr": "",
  "execution_time_ms": 342,
  "sandbox_id": "sbx_xyz789"
}
```

---

### Tool: `shell_exec`

Run a shell command directly (always uses Bash).

```json
{
  "tool": "shell_exec",
  "arguments": {
    "command": "ls -la /workspace && cat README.md",
    "session_id": "sess_abc123"
  }
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|---|---|---|---|
| `command` | `string` | ✅ | Bash command or script |
| `session_id` | `string` | ❌ | Reuse existing session sandbox |
| `timeout_seconds` | `integer` | ❌ | Default: 60 |

---

### Tool: `file_write`

Write a file into the active sandbox filesystem.

```json
{
  "tool": "file_write",
  "arguments": {
    "session_id": "sess_abc123",
    "path": "/workspace/data.csv",
    "content": "name,score\nAlice,95\nBob,87\n"
  }
}
```

---

### Tool: `file_read`

Read a file from the sandbox filesystem.

```json
{
  "tool": "file_read",
  "arguments": {
    "session_id": "sess_abc123",
    "path": "/workspace/output/report.html"
  }
}
```

**Response:**
```json
{
  "content": "<!DOCTYPE html>...",
  "size_bytes": 4823,
  "encoding": "utf-8"
}
```

---

### Tool: `sandbox_create`

Explicitly create a session sandbox (optional — `code_exec` creates one lazily).

```json
{
  "tool": "sandbox_create",
  "arguments": {
    "session_id": "sess_abc123",
    "language": "rust",
    "mode": "project",
    "project_repo": "https://github.com/example/my-project.git"
  }
}
```

---

### Tool: `sandbox_destroy`

Destroy a session sandbox and free its resources.

```json
{
  "tool": "sandbox_destroy",
  "arguments": {
    "session_id": "sess_abc123"
  }
}
```

---

## 2. HTTP API

### Public API (port 5001)

Used by external API callers and the SDK clients.

#### `POST /api/v1/execute`

One-shot code execution.

```http
POST http://uar-code-interpreter:5001/api/v1/execute
Authorization: Bearer <jwt>
Content-Type: application/json

{
  "language": "python",
  "code": "print(2 + 2)",
  "mode": "ephemeral",
  "timeout_seconds": 30
}
```

Response:
```json
{
  "exit_code": 0,
  "stdout": "4\n",
  "stderr": "",
  "execution_time_ms": 187
}
```

---

#### `POST /api/v1/sandboxes`

Create a session sandbox.

```http
POST /api/v1/sandboxes
Authorization: Bearer <jwt>

{
  "session_id": "sess_abc123",
  "language": "rust",
  "memory_mib": 1024
}
```

Response:
```json
{
  "sandbox_id": "sbx_xyz789",
  "session_id": "sess_abc123",
  "status": "running",
  "created_at": "2026-02-21T09:42:00Z"
}
```

---

#### `POST /api/v1/sandboxes/{sandbox_id}/execute`

Execute in an existing sandbox. Output streams via `uar-realtime`.

```http
POST /api/v1/sandboxes/sbx_xyz789/execute
Authorization: Bearer <jwt>

{
  "language": "rust",
  "code": "fn main() { println!(\"Hello!\"); }",
  "stream_to_topic": "sandbox:sess_abc123"
}
```

Response (immediate, before execution completes):
```json
{
  "execution_id": "exec_111",
  "status": "running",
  "stream_topic": "sandbox:sess_abc123"
}
```

Then watch `uar-realtime` for:

```json
{ "topic": "sandbox:sess_abc123", "event": "sandbox:stdout",  "payload": { "data": "Hello!\n",   "execution_id": "exec_111" } }
{ "topic": "sandbox:sess_abc123", "event": "sandbox:completed","payload": { "exit_code": 0, "duration_ms": 2341, "execution_id": "exec_111" } }
```

---

#### `GET /api/v1/sandboxes/{sandbox_id}/files`

List files in the sandbox.

```http
GET /api/v1/sandboxes/sbx_xyz789/files?path=/workspace
```

Response:
```json
{
  "entries": [
    { "name": "main.rs", "type": "file", "size_bytes": 234 },
    { "name": "Cargo.toml", "type": "file", "size_bytes": 156 },
    { "name": "target", "type": "directory" }
  ]
}
```

---

#### `GET /api/v1/sandboxes/{sandbox_id}/files/{path}`

Download a file from the sandbox.

```http
GET /api/v1/sandboxes/sbx_xyz789/files/%2Fworkspace%2Foutput.csv
```

---

#### `DELETE /api/v1/sandboxes/{sandbox_id}`

Destroy a sandbox.

---

### Internal API (port 5002 — UAR → interpreter)

Not exposed outside the deployment. No TLS required — localhost/pod-internal only.

#### `POST /internal/v1/execute`

```http
POST http://localhost:5002/internal/v1/execute
Authorization: Bearer <internal-secret>

{
  "language": "python",
  "code": "print('hello')",
  "session_id": "sess_abc123",
  "realtime_topic": "sandbox:sess_abc123"
}
```

#### `GET /internal/v1/health`

Liveness probe for Kubernetes / Docker Compose.

```json
{ "status": "ok", "runner": "microsandbox", "active_sandboxes": 3 }
```

---

## 3. Realtime Event Reference

All sandbox execution events are published to `uar-realtime` on the `sandbox:{session_id}` channel.

| Event | Direction | Payload |
|---|---|---|
| `sandbox:created` | Server → Client | `{ sandbox_id, session_id, language }` |
| `sandbox:stdout` | Server → Client | `{ data: string, execution_id }` |
| `sandbox:stderr` | Server → Client | `{ data: string, execution_id }` |
| `sandbox:completed` | Server → Client | `{ exit_code, duration_ms, execution_id }` |
| `sandbox:error` | Server → Client | `{ code, message, execution_id }` |
| `sandbox:destroyed` | Server → Client | `{ sandbox_id, session_id }` |
| `sandbox:file:written` | Server → Client | `{ path, size_bytes }` |
| `sandbox:timeout` | Server → Client | `{ execution_id, timeout_seconds }` |

---

## 4. SDK Usage Examples

### From a UAR agent (via MCP tool call)

```typescript
// Automatically routed through MCP tool server
const result = await agent.callTool("code_exec", {
  language: "python",
  code: `
import numpy as np
data = np.random.randn(1000)
print(f"mean={data.mean():.3f} std={data.std():.3f}")
`,
  mode: "session",
  session_id: ctx.sessionId
});
```

### From frontend TypeScript (direct API)

```typescript
// Subscribe to realtime events first
realtimeClient.join(`sandbox:${sessionId}`);
realtimeClient.on("sandbox:stdout", (event) => {
  terminal.write(event.payload.data);
});
realtimeClient.on("sandbox:completed", (event) => {
  console.log(`Exit code: ${event.payload.exit_code}`);
});

// Then trigger execution
await fetch(`/api/v1/sandboxes/${sandboxId}/execute`, {
  method: "POST",
  headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
  body: JSON.stringify({
    language: "rust",
    code: `fn main() { println!("Hello from Rust!"); }`,
    stream_to_topic: `sandbox:${sessionId}`
  })
});
```

### Rust (within UAR subsystem)

```rust
// From anywhere that has AppState
state.interpreter().execute(ExecutionRequest {
    language: Language::Python,
    code: "print('from agent')".to_owned(),
    mode: ExecutionMode::Session { session_id: session_id.clone() },
    timeout_seconds: Some(30),
    ..Default::default()
}).await?;
```

---

## 5. Error Codes

| Code | HTTP | Description |
|---|---|---|
| `sandbox_not_found` | 404 | No sandbox with that ID exists |
| `sandbox_timeout` | 408 | Execution exceeded timeout limit |
| `runner_unavailable` | 503 | No sandbox runner available on this platform |
| `language_unsupported` | 400 | Language not supported by current runner |
| `quota_exceeded` | 429 | Too many concurrent sandboxes |
| `filesystem_error` | 500 | File read/write failed inside sandbox |
| `vm_launch_failed` | 500 | microVM failed to start |
| `auth_required` | 401 | Missing or invalid JWT |
| `forbidden_topic` | 403 | Session ID does not belong to this JWT principal |
