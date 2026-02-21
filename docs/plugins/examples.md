# UAR Plugin Architecture — Concrete Examples

_Last updated: 2026-02-21_

Two full worked examples of plugins built on the three core primitives: realtime, code runner, and LLM.

---

## Example 1: Live Video Transcription Plugin

**What it does:** Listens for WebRTC media sessions, spawns a Python microVM sandbox that runs Whisper in real-time, calls an LLM to diarize speakers, and streams each transcript segment back to the client via `uar-realtime`.

### Directory structure

```
plugins/transcription/
  plugin.toml    ← manifest
  main.py        ← entrypoint (receives events + tool calls)
  transcribe.py  ← sandbox worker code (runs in child sandbox)
  summarize.py   ← LLM summarization runner
```

### `plugin.toml`

```toml
[plugin]
name = "transcription"
version = "1.0.0"
type = "sandbox"

[sandbox]
image = "prometheus-ags/sandbox-python:latest"
language = "python"
entrypoint = "main.py"
memory_mib = 2048
cpus = 2
network = true
mode = "session"

[[subscriptions]]
topic = "session:*:media"
events = ["media:track:started", "media:track:ended"]

[[channels]]
name = "plugin:transcription:{session_id}"
events = ["transcript:segment", "transcript:completed", "transcript:summary", "job:started", "job:error"]

[[tools]]
name = "transcription_start"
description = "Start live transcription for the current video session"
[tools.parameters]
session_id  = { type = "string", required = true }
language    = { type = "string", default = "en" }
summarize   = { type = "boolean", default = true }

[[tools]]
name = "transcription_get_summary"
description = "Get the AI-generated meeting summary so far"
[tools.parameters]
session_id = { type = "string", required = true }

[capabilities]
llm = ["chat", "transcription"]
filesystem = "readwrite"
internal_apis = ["uar-realtime-publish", "uar-llm"]

[settings]
whisper_model    = { type = "string", default = "base", enum = ["tiny","base","small","medium","large"] }
summary_model    = { type = "string", default = "auto" }
diarization      = { type = "boolean", default = true }
```

### `main.py` — plugin entrypoint

```python
"""
Transcription plugin entrypoint.
Called by UAR runtime for each arriving event or tool call.
"""
import uar
import json, os

settings = uar.plugin.settings()
active_jobs: dict[str, dict] = {}


def on_event(event: dict):
    """Dispatch realtime events to handlers."""
    match event["event"]:
        case "media:track:started":
            _on_track_started(event["topic"], event["payload"])
        case "media:track:ended":
            _on_track_ended(event["topic"], event["payload"])


def _on_track_started(topic: str, payload: dict):
    """Spawn a transcription sandbox when audio track appears."""
    session_id = topic.split(":")[1]
    track_id   = payload["track_id"]
    track_kind = payload["kind"]   # "audio" | "video"

    if track_kind != "audio":
        return   # only transcribe audio tracks

    # Emit job-started event so frontend knows we're on it
    uar.realtime.emit(
        topic=f"plugin:transcription:{session_id}",
        event="job:started",
        payload={"track_id": track_id, "model": settings["whisper_model"]}
    )

    # Spawn child sandbox for heavy lifting
    sandbox = uar.sandbox.create(
        language="python",
        image="prometheus-ags/sandbox-python:latest",
        memory_mib=2048,
        network=True,
        session_id=session_id,
    )
    sandbox.write_file("/workspace/transcribe.py", _load_worker("transcribe.py"))
    job = sandbox.execute(
        command="python3 /workspace/transcribe.py",
        env={
            "SESSION_ID":        session_id,
            "TRACK_ID":          track_id,
            "WHISPER_MODEL":     settings["whisper_model"],
            "DIARIZATION":       str(settings["diarization"]).lower(),
        },
        # Output streams to uar-realtime automatically
        stream_topic=f"plugin:transcription:{session_id}",
    )
    active_jobs[session_id] = {"sandbox": sandbox, "job": job, "segments": []}


def _on_track_ended(topic: str, payload: dict):
    session_id = topic.split(":")[1]
    job_ctx = active_jobs.pop(session_id, None)
    if not job_ctx:
        return

    # Wait for sandbox to finish, then generate summary
    job_ctx["sandbox"].wait()
    transcript = _collect_transcript(session_id)
    _generate_summary(session_id, transcript)


def _generate_summary(session_id: str, transcript: str):
    response = uar.llm.chat(
        model=settings["summary_model"],
        messages=[
            {"role": "system", "content":
                "You produce concise, structured meeting summaries with action items."},
            {"role": "user", "content": f"Transcript:\n\n{transcript}"},
        ],
        temperature=0.2,
    )
    uar.realtime.emit(
        topic=f"plugin:transcription:{session_id}",
        event="transcript:summary",
        payload={
            "summary":      response.content,
            "word_count":   len(response.content.split()),
        }
    )


def on_tool_call(tool_name: str, args: dict, session_id: str):
    """Handle MCP tool calls from agents."""
    match tool_name:
        case "transcription_get_summary":
            return _get_summary_from_file(args["session_id"])
        case "transcription_start":
            return {"status": "listening", "note": "Waiting for audio track"}

def _load_worker(name: str) -> str:
    return open(os.path.join(os.path.dirname(__file__), name)).read()

def _collect_transcript(session_id: str) -> str:
    try:
        return open(f"/workspace/transcript_{session_id}.txt").read()
    except FileNotFoundError:
        return ""

def _get_summary_from_file(session_id: str) -> dict:
    try:
        return {"summary": open(f"/workspace/summary_{session_id}.txt").read()}
    except FileNotFoundError:
        return {"summary": None, "note": "Summary not yet available"}
```

### `transcribe.py` — sandbox worker (runs in child microVM)

```python
"""
Runs inside a child microVM sandbox.
Streams audio from the session, transcribes with Whisper,
emits segments to uar-realtime.
"""
import os, time, json, whisper, uar

session_id   = os.environ["SESSION_ID"]
track_id     = os.environ["TRACK_ID"]
model_size   = os.environ.get("WHISPER_MODEL", "base")
diarization  = os.environ.get("DIARIZATION", "true") == "true"

model = whisper.load_model(model_size)
writer = open(f"/workspace/transcript_{session_id}.txt", "w")

segment_index = 0

def transcribe_chunk(audio_path: str):
    global segment_index
    result = model.transcribe(audio_path, language=os.environ.get("LANGUAGE", "en"))
    
    for seg in result["segments"]:
        text = seg["text"].strip()
        if not text:
            continue

        # Speaker diarization via LLM (simple heuristic prompt)
        speaker = "Speaker"
        if diarization:
            resp = uar.llm.chat(
                model="auto",
                messages=[{"role":"user",
                    "content": f"Who is speaking in: '{text}'? Reply with 'Speaker A' or 'Speaker B' only."}],
                max_tokens=10
            )
            speaker = resp.content.strip()

        payload = {
            "text":         text,
            "speaker":      speaker,
            "start_ms":     int(seg["start"] * 1000),
            "end_ms":       int(seg["end"] * 1000),
            "segment_index": segment_index,
            "confidence":   round(1.0 - seg.get("no_speech_prob", 0), 3),
        }
        
        # Emit to uar-realtime — client sees it immediately
        uar.realtime.emit(
            topic=f"plugin:transcription:{session_id}",
            event="transcript:segment",
            payload=payload
        )
        
        writer.write(f"{speaker}: {text}\n")
        writer.flush()
        segment_index += 1

# Main loop: receive audio chunks from track stream
for audio_chunk_path in uar.sandbox.receive_audio_chunks(track_id):
    transcribe_chunk(audio_chunk_path)

uar.realtime.emit(
    topic=f"plugin:transcription:{session_id}",
    event="transcript:completed",
    payload={"total_segments": segment_index}
)
```

---

## Example 2: Collaborative Canvas Plugin

**What it does:** A realtime collaborative whiteboard — clients emit cursor moves and drawing operations; the plugin broadcasts them to all session participants and persists state.

```toml
[plugin]
name = "canvas"
version = "0.1.0"
description = "Real-time collaborative whiteboard"
type = "wasm"                  # WASM — pure event relay, no heavy compute

[wasm]
module = "canvas_plugin.wasm"  # compiled from Rust

[[subscriptions]]
topic = "plugin:canvas:board:*"
events = ["canvas:cursor:moved", "canvas:stroke:draw", "canvas:element:add",
          "canvas:element:move", "canvas:element:delete"]

[[channels]]
name = "plugin:canvas:board:{board_id}"
events = ["canvas:state:sync", "canvas:cursor:moved", "canvas:stroke:draw",
          "canvas:element:add", "canvas:element:move", "canvas:element:delete",
          "canvas:participant:joined", "canvas:participant:left"]

[[tools]]
name = "canvas_create"
description = "Create a new collaborative canvas board"
[tools.parameters]
name  = { type = "string" }
board_id = { type = "string", description = "Optional board ID; generated if omitted" }

[[tools]]
name = "canvas_invite"
description = "Invite a user to collaborate on a canvas"
[tools.parameters]
board_id = { type = "string", required = true }
user_id  = { type = "string", required = true }
```

```rust
// canvas_plugin/src/lib.rs — compiled to wasm32-wasip2
use uar_plugin_sdk::{Event, ToolCall, ToolResult, emit};
use std::collections::HashMap;

static mut BOARD_STATE: Option<HashMap<String, serde_json::Value>> = None;

#[no_mangle]
pub extern "C" fn on_event(event_ptr: *const u8, event_len: usize) {
    let event: Event = deserialize(event_ptr, event_len);
    let board_id = extract_board_id(&event.topic);
    
    match event.name.as_str() {
        "canvas:cursor:moved" | "canvas:stroke:draw" => {
            // Relay to all board participants immediately
            emit(&event.topic, &event.name, &event.payload);
        }
        "canvas:element:add" | "canvas:element:move" | "canvas:element:delete" => {
            // Update in-memory state + relay
            update_board_state(&board_id, &event);
            emit(&event.topic, &event.name, &event.payload);
        }
        _ => {}
    }
}
```

---

## Example 3: CI/CD Runner Plugin

**What it does:** Listens for commits pushed to a connected repository, spawns a sandbox that runs the full test suite, and streams results back.

```toml
[plugin]
name = "ci-runner"
version = "0.1.0"
type = "sandbox"

[sandbox]
image = "prometheus-ags/sandbox-universal:latest"  # Rust + Node + Python
language = "bash"
entrypoint = "ci.sh"
memory_mib = 4096
cpus = 4
network = true
mode = "ephemeral"         # fresh sandbox per CI run

[[subscriptions]]
topic = "plugin:github-connector:events"
events = ["github:push", "github:pull_request:opened"]

[[channels]]
name = "plugin:ci-runner:run:{run_id}"
events = ["ci:started", "ci:step:started", "ci:step:completed",
          "ci:step:failed", "ci:completed", "ci:failed"]
```

```bash
#!/bin/bash
# ci.sh — runs inside the ephemeral microVM

set -euo pipefail
RUN_ID="${CI_RUN_ID}"
REPO_URL="${REPO_URL}"
COMMIT="${COMMIT_SHA}"

uar-emit "plugin:ci-runner:run:${RUN_ID}" "ci:started" \
  "{\"commit\": \"${COMMIT}\", \"repo\": \"${REPO_URL}\"}"

# Clone repo
git clone "${REPO_URL}" /workspace/repo
cd /workspace/repo && git checkout "${COMMIT}"

# Run steps
run_step() {
    local name="$1"; shift
    uar-emit "plugin:ci-runner:run:${RUN_ID}" "ci:step:started" "{\"step\": \"${name}\"}"
    if "$@" 2>&1; then
        uar-emit "plugin:ci-runner:run:${RUN_ID}" "ci:step:completed" "{\"step\": \"${name}\"}"
    else
        uar-emit "plugin:ci-runner:run:${RUN_ID}" "ci:step:failed" "{\"step\": \"${name}\"}"
        uar-emit "plugin:ci-runner:run:${RUN_ID}" "ci:failed" "{\"step\": \"${name}\"}"
        exit 1
    fi
}

run_step "cargo test"  cargo test --all 2>&1
run_step "cargo clippy" cargo clippy -- -D warnings 2>&1

uar-emit "plugin:ci-runner:run:${RUN_ID}" "ci:completed" \
  "{\"commit\": \"${COMMIT}\", \"status\": \"passed\"}"
```
