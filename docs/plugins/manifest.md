# UAR Plugin Architecture — Plugin Manifest

_Last updated: 2026-02-21_

Every plugin is described by a `plugin.toml` manifest file. This is the single source of truth for a plugin's identity, permissions, channels, tools, and sandbox configuration.

---

## Manifest Format

```toml
# plugin.toml — the complete plugin descriptor

[plugin]
# Unique name — becomes the channel namespace prefix: plugin:{name}:*
name = "transcription"
version = "1.0.0"
description = "Real-time audio/video transcription and summarization"
author = "Prometheus AGS"
license = "MIT"

# Plugin type: "wasm", "sandbox", or "external"
type = "sandbox"

# ── Sandbox configuration (type = "sandbox") ─────────────────────────────────
[sandbox]
# OCI image to use for the runner sandbox
image = "prometheus-ags/sandbox-python:latest"
# Language runtime
language = "python"
# Entry point — relative to plugin package root
entrypoint = "main.py"
# Resources
memory_mib = 1024
cpus = 2
# Network access required (to call uar LLM API + download models)
network = true
# Sandbox lifetime model
mode = "session"   # "ephemeral" | "session" | "persistent"

# ── Wasm configuration (type = "wasm") ───────────────────────────────────────
# [wasm]
# module = "plugin.wasm"
# memory_pages = 256  # 16 MB

# ── Channel subscriptions ─────────────────────────────────────────────────────
# Channels the plugin wants to receive events from.
# UAR validates these against the plugin's declared capabilities.
[[subscriptions]]
topic = "session:*:media"          # listen for media events in any session
events = ["media:track:started", "media:track:ended", "media:participant:joined"]

[[subscriptions]]
topic = "session:*:agent"          # react to agent outputs
events = ["agent:run:completed"]
# Optional server-side filter — only trigger if the session has transcription enabled
where = { "metadata.transcription_enabled" = { "$eq" = true } }

[[subscriptions]]
topic = "system:notifications"
events = ["system:*"]              # all system events

# ── Published channels ────────────────────────────────────────────────────────
# Channels the plugin is allowed to publish to.
# Must all start with plugin:{name}:
[[channels]]
name = "plugin:transcription:{session_id}"
description = "Per-session transcription events"
events = [
  "transcript:segment",     # individual transcript chunk
  "transcript:completed",   # full transcript ready
  "transcript:summary",     # LLM-generated meeting summary
  "job:started",
  "job:progress",
  "job:completed",
  "job:error",
]

[[channels]]
name = "plugin:transcription:global"
description = "Global transcription service events"
events = ["service:ready", "service:degraded"]

# ── MCP tools exposed to UAR agents ──────────────────────────────────────────
[[tools]]
name = "transcription_start"
description = "Start real-time transcription for a session"
[tools.parameters]
session_id = { type = "string", required = true }
language = { type = "string", default = "en" }
diarization = { type = "boolean", default = true }
summarize = { type = "boolean", default = true }

[[tools]]
name = "transcription_stop"
description = "Stop transcription and return final transcript"
[tools.parameters]
session_id = { type = "string", required = true }

[[tools]]
name = "transcription_get"
description = "Get the current transcript for a session"
[tools.parameters]
session_id = { type = "string", required = true }

# ── Required capabilities ─────────────────────────────────────────────────────
[capabilities]
# LLM access — model families the plugin may call
llm = ["chat", "transcription"]         # openai-compatible + whisper-compatible
# Filesystem access within sandbox
filesystem = "readwrite"
# Internal service access
internal_apis = ["uar-realtime-publish", "uar-llm"]

# ── Settings (user-configurable) ──────────────────────────────────────────────
[settings]
# Default language for transcription
default_language = { type = "string", default = "en", description = "BCP-47 language code" }
# Whisper model size
whisper_model = { type = "string", default = "base", enum = ["tiny", "base", "small", "medium", "large"] }
# Enable speaker diarization
diarization_enabled = { type = "boolean", default = true }
# LLM model to use for summarization
summarization_model = { type = "string", default = "auto" }
```

---

## Minimal Manifest (Simple Plugin)

For a simple event-transformer plugin with no sandbox:

```toml
[plugin]
name = "slack-notifier"
version = "0.1.0"
description = "Posts UAR system events to Slack"
type = "external"

[[subscriptions]]
topic = "system:notifications"
events = ["system:notification"]
where = { "level" = { "$in" = ["warning", "error", "critical"] } }

[[channels]]
name = "plugin:slack-notifier:status"
events = ["notification:sent", "notification:failed"]

[capabilities]
internal_apis = ["uar-realtime-publish"]
```

---

## Manifest Field Reference

### `[plugin]`

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | ✅ | Unique plugin identifier (lowercase, hyphens) |
| `version` | string | ✅ | SemVer |
| `description` | string | ✅ | Human-readable description |
| `type` | `"wasm"` \| `"sandbox"` \| `"external"` | ✅ | Plugin runtime type |
| `author` | string | ❌ | Author name or org |
| `license` | string | ❌ | SPDX license identifier |

### `[sandbox]` (type = "sandbox" only)

| Field | Type | Default | Description |
|---|---|---|---|
| `image` | string | — | OCI image for the sandbox |
| `language` | `"bash"` \| `"rust"` \| `"python"` \| `"node"` | — | Runtime language |
| `entrypoint` | string | — | Entry file relative to plugin package |
| `memory_mib` | integer | 512 | Memory limit |
| `cpus` | float | 1.0 | CPU limit |
| `network` | boolean | false | Network access |
| `mode` | `"ephemeral"` \| `"session"` \| `"persistent"` | `"session"` | Sandbox lifetime |

### `[[subscriptions]]`

| Field | Type | Required | Description |
|---|---|---|---|
| `topic` | string | ✅ | Topic pattern (supports `*` wildcard at end) |
| `events` | `string[]` | ✅ | Event name patterns (`*` = all events on topic) |
| `where` | object | ❌ | Server-side payload filter (subscription DSL predicate) |

### `[[channels]]`

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | ✅ | Channel name pattern (must start with `plugin:{name}:`) |
| `description` | string | ❌ | Human-readable description |
| `events` | `string[]` | ✅ | Event names the plugin will emit on this channel |

### `[[tools]]`

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | ✅ | MCP tool name (snake_case) |
| `description` | string | ✅ | Description visible to LLM tool selection |
| `parameters` | object | ❌ | JSON Schema-compatible parameter definitions |

### `[capabilities]`

| Field | Type | Description |
|---|---|---|
| `llm` | `string[]` | LLM capability families: `"chat"`, `"transcription"`, `"embedding"`, `"image"` |
| `filesystem` | `"none"` \| `"readonly"` \| `"readwrite"` | Sandbox filesystem access |
| `internal_apis` | `string[]` | Internal UAR APIs: `"uar-realtime-publish"`, `"uar-llm"`, `"uar-sessions"` |

### `[settings]`

User-configurable per-installation settings. Each field is a setting definition:

```toml
[settings]
my_setting = {
  type = "string" | "boolean" | "integer" | "float",
  default = <value>,
  description = "...",
  enum = [...]    # optional: restrict to these values
}
```
