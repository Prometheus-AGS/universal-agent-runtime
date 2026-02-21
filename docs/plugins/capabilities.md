# UAR Plugin Architecture — Plugin Capabilities

_Last updated: 2026-02-21_

This document describes what plugins can do — the primitives available to every plugin regardless of type.

---

## The Three Core Capabilities

### 1. Listen to Anything via `uar-realtime`
### 2. Compute in `uar-code-interpreter` Sandboxes
### 3. Call LLM Models via UAR

These three capabilities compose into powerful workflows with no custom infrastructure.

---

## 1. Realtime — Subscribe and Publish

Every plugin can subscribe to any permitted channel and publish to its own namespace.

### Subscribing (from plugin host process or sandbox)

```python
# Inside a plugin sandbox — filter subscriptions are applied server-side by uar-realtime
# The plugin's event dispatch is defined in plugin.toml [[subscriptions]]
# When a matching event arrives, UAR routes it to the plugin's entrypoint as a call

def on_event(event: dict):
    """Called by the plugin runtime when a subscribed event arrives."""
    topic = event["topic"]
    name  = event["event"] 
    payload = event["payload"]
    
    if name == "media:track:started":
        handle_new_audio_track(payload)
    elif name == "agent:run:completed":
        generate_summary(payload)
```

### Publishing (from inside a sandbox — to any realtime channel)

```python
# Injected into every sandbox — no import needed
# Available as module `uar` or via the uar-emit cli tool

import uar

uar.realtime.emit(
    topic=f"plugin:transcription:{session_id}",
    event="transcript:segment",
    payload={
        "text": "The Q3 results show a 20% increase in revenue.",
        "timestamp_ms": 12400,
        "speaker": "CFO",
        "confidence": 0.97
    }
)
```

### Wildcard subscriptions

A plugin can subscribe to wildcard topics — UAR resolves these to concrete topic lists at registration:

```toml
# plugin.toml
[[subscriptions]]
topic = "agent:run:*"       # all agent runs
events = ["agent:run:started", "agent:run:completed"]

[[subscriptions]]
topic = "session:*:media"   # media events in any session
events = ["media:*"]        # all media events
```

### Available channels to subscribe to (with `realtime:subscribe` capability)

| Channel | Events | Use case |
|---|---|---|
| `system:notifications` | `system:notification` | React to system-level alerts |
| `agent:run:{id}` | `agent:*` | Monitor any specific agent run |
| `agent:run:*` | `agent:run:started`, `agent:run:completed` | Global agent run monitoring |
| `session:{id}` | `session:*` | Session lifecycle events |
| `session:{id}:media` | `media:*` | WebRTC media events (audio/video) |
| `session:{id}:agent` | `agent:*` | Agent events within a session |
| `plugin:{other}:*` | _(other plugin's public events)_ | Cross-plugin communication |
| `user:{id}:activity` | `user:*` | User activity stream |

---

## 2. Code Runner — Spawn Sandbox Jobs

Plugins can spawn `uar-code-interpreter` sandboxes for compute-heavy or long-running jobs.

### From inside a plugin sandbox (spawn a child sandbox)

```python
import uar

# Spawn a new sandbox for a parallel job
child = uar.sandbox.create(
    language="python",
    image="prometheus-ags/sandbox-python:latest",
    memory_mib=2048,
    session_id=session_id,  # child inherits session context
)

# Write job code to the child sandbox
child.write_file("/workspace/transcribe.py", TRANSCRIPTION_CODE)

# Execute — output streams to uar-realtime automatically
job = child.execute("/workspace/transcribe.py", env={
    "AUDIO_TRACK_URL": audio_url,
    "SESSION_ID": session_id,
})

# Non-blocking: the child sandbox emits events directly to uar-realtime
# Parent does not need to wait
```

### Parallel swarm (spawn N sandboxes for N tasks)

```python
import uar, asyncio

# Split task into segments
segments = split_video_into_segments(video_url, segment_seconds=30)

# Spawn one sandbox per segment
jobs = [
    uar.sandbox.create_and_execute(
        code=TRANSCRIBE_SEGMENT_CODE,
        env={"SEGMENT_URL": seg.url, "SEGMENT_INDEX": str(i)},
        session_id=session_id,
    )
    for i, seg in enumerate(segments)
]

# All sandboxes emit to uar-realtime simultaneously
# Results collected via plugin:transcription:{id}:segment events
uar.sandbox.wait_all(jobs)
```

---

## 3. LLM Access from Sandboxes

Plugin sandboxes can call the UAR LLM routing layer using the session's injected JWT.

### Chat completion

```python
import uar

# Call the best available model via UAR routing
response = uar.llm.chat(
    model="auto",    # UAR selects based on context, cost, and availability
    messages=[
        {"role": "system", "content": "You summarize meeting transcripts."},
        {"role": "user",   "content": f"Summarize:\n\n{full_transcript}"}
    ],
    temperature=0.3
)
summary = response.content
```

### Transcription (Whisper-compatible)

```python
# Audio transcription via the injected uar.llm module
result = uar.llm.transcribe(
    audio_file="/workspace/segment.wav",
    model="whisper-large-v3",
    language="en",
    task="transcribe"   # or "translate"
)
text = result.text
segments = result.segments   # word-level timestamps
```

### Embeddings

```python
# Generate embeddings for semantic search / RAG
embedding = uar.llm.embed(
    model="text-embedding-3-small",
    input=document_text
)
vector = embedding.data[0].embedding   # list[float]
```

### Streaming LLM output → realtime

```python
# Stream LLM output directly to the client via uar-realtime
for chunk in uar.llm.chat_stream(
    model="auto",
    messages=[{"role": "user", "content": prompt}]
):
    uar.realtime.emit(
        topic=f"plugin:my-plugin:{session_id}",
        event="llm:token:delta",
        payload={"delta": chunk.content, "index": chunk.index}
    )
```

---

## 4. MCP Tools — Expose Capabilities to Agents

Plugins can register MCP tools that UAR agents discover and call like any other tool.

```toml
# plugin.toml
[[tools]]
name = "transcription_start"
description = "Begin real-time transcription for the current session"
[tools.parameters]
language = { type = "string", default = "en" }
```

When an agent calls `transcription_start`, UAR routes the call to the plugin:

```python
# main.py — plugin entrypoint handles both events AND tool calls
def on_tool_call(tool_name: str, args: dict, session_id: str):
    if tool_name == "transcription_start":
        job = start_transcription_job(
            session_id=session_id,
            language=args.get("language", "en")
        )
        return {"job_id": job.id, "status": "started"}
```

---

## 5. Plugin Settings Access

Plugins can read their configured settings (set by the user at install time):

```python
import uar

settings = uar.plugin.settings()

whisper_model = settings.get("whisper_model", "base")
lang = settings.get("default_language", "en")
```

---

## 6. Filesystem Access (Sandbox)

Plugin sandboxes have a persistent workspace at `/workspace` for the duration of their sandbox mode:

```python
# Writing results to the sandbox filesystem
with open("/workspace/transcript.json", "w") as f:
    json.dump(transcript_data, f)

# Reading back in a later turn (session mode — persists between calls)
with open("/workspace/transcript.json") as f:
    transcript = json.load(f)
```

Files can be exported back to the user via `uar.files.upload()` or read via the `file_read` MCP tool.

---

## 7. Capability Summary

| Capability | WASM plugin | Sandbox plugin | External plugin |
|---|:---:|:---:|:---:|
| Subscribe to realtime channels | ✅ | ✅ | ✅ |
| Publish to `plugin:{name}:*` channels | ✅ | ✅ | ✅ |
| Spawn child sandboxes | ❌ | ✅ | ✅ |
| Call LLM models | ❌ | ✅ | ✅ (via API) |
| Run for hours | ❌ | ✅ | ✅ |
| Install packages | ❌ | ✅ | ✅ |
| Full filesystem access | ❌ | ✅ | N/A |
| Network access | ❌ | Optional | ✅ |
| Expose MCP tools | ✅ | ✅ | ✅ |
| In-process event latency | ✅ | ❌ | ❌ |
| Works in Tauri/desktop | ✅ | ✅ (sidecar) | Optional |
