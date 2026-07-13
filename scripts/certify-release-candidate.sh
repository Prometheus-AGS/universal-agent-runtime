#!/usr/bin/env bash
set -euo pipefail

# Certify an installed release payload rather than a development checkout.
# Usage: scripts/certify-release-candidate.sh <artifact-directory> [results-directory]

artifacts="${1:?artifact directory is required}"
results="${2:-target/release-candidate-certification}"
archive_glob="${UAR_CANDIDATE_ARCHIVE_GLOB:-*linux-x64.tar.gz}"
port="${UAR_CANDIDATE_PORT:-1906}"
mock_port="${UAR_CANDIDATE_MOCK_PORT:-1907}"
parallel_requests="${UAR_PARALLEL_REQUESTS:-20}"
soak_interval_seconds="${UAR_SOAK_INTERVAL_SECONDS:-1}"
soak_p95_limit_ms="${UAR_SOAK_P95_LIMIT_MS:-2000}"
soak_memory_growth_limit_kib="${UAR_SOAK_MEMORY_GROWTH_LIMIT_KIB:-262144}"
previous_artifacts="${UAR_PREVIOUS_ARTIFACT_DIR:-}"
work="$(mktemp -d)"
server_pid=""
mock_pid=""
container_id=""
upgrade_journey=""

cleanup() {
  if [[ -n "$server_pid" ]]; then kill "$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true; fi
  if [[ -n "$mock_pid" ]]; then kill "$mock_pid" 2>/dev/null || true; wait "$mock_pid" 2>/dev/null || true; fi
  if [[ -n "$container_id" ]]; then docker rm -f "$container_id" >/dev/null 2>&1 || true; fi
  rm -rf "$work"
}
trap cleanup EXIT

mkdir -p "$results" "$work/installed" "$work/data"
archive="$(find "$artifacts" -maxdepth 1 -type f -name "$archive_glob" -print -quit)"
[[ -n "$archive" ]] || { echo "no candidate archive matches $archive_glob" >&2; exit 1; }

manifest="$artifacts/release-manifest.json"
if [[ -f "$manifest" ]]; then
  candidate_tag="$(jq -er '.release' "$manifest")"
  source_sha="$(jq -er '.source.sha' "$manifest")"
  soak_duration_seconds="${UAR_SOAK_DURATION_SECONDS:-10800}"
else
  candidate_tag="${UAR_CANDIDATE_TAG:?UAR_CANDIDATE_TAG is required without release-manifest.json}"
  source_sha="${UAR_CANDIDATE_SOURCE_SHA:?UAR_CANDIDATE_SOURCE_SHA is required without release-manifest.json}"
  soak_duration_seconds="${UAR_SOAK_DURATION_SECONDS:-60}"
fi
if [[ -n "${UAR_EXPECTED_CANDIDATE_TAG:-}" && "$candidate_tag" != "$UAR_EXPECTED_CANDIDATE_TAG" ]]; then
  echo "candidate manifest tag $candidate_tag does not match expected tag $UAR_EXPECTED_CANDIDATE_TAG" >&2
  exit 1
fi
[[ "$source_sha" =~ ^[0-9a-f]{40}$ ]] || { echo "candidate source SHA is invalid" >&2; exit 1; }
[[ "$soak_duration_seconds" =~ ^[0-9]+$ ]] || { echo "UAR_SOAK_DURATION_SECONDS must be a non-negative integer" >&2; exit 1; }
[[ "$parallel_requests" =~ ^[1-9][0-9]*$ ]] || { echo "UAR_PARALLEL_REQUESTS must be positive" >&2; exit 1; }
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  checkout_sha="$(git rev-parse HEAD)"
  [[ "$checkout_sha" == "$source_sha" ]] || {
    echo "candidate manifest/source SHA $source_sha does not match checkout $checkout_sha" >&2
    exit 1
  }
fi
archive_digest="$(sha256sum "$archive" | cut -d' ' -f1)"
if [[ -f "$manifest" ]]; then
  manifest_archive_digest="$(jq -er --arg name "$(basename "$archive")" '.artifacts[] | select(.name == $name) | .sha256' "$manifest")"
  [[ "$archive_digest" == "$manifest_archive_digest" ]] || {
    echo "candidate archive digest does not match release manifest" >&2
    exit 1
  }
fi

tar -xzf "$archive" -C "$work/installed"
binary="$(find "$work/installed" -type f -name universal-agent-runtime -print -quit)"
[[ -n "$binary" ]] || { echo "candidate archive has no universal-agent-runtime binary" >&2; exit 1; }
chmod +x "$binary"
package_root="$(dirname "$binary")"
[[ -d "$package_root/static" ]] || { echo "candidate archive has no packaged React assets" >&2; exit 1; }
[[ -d "$package_root/skills/builtin" ]] || { echo "candidate archive has no packaged built-in skills" >&2; exit 1; }
[[ -d "$package_root/models" ]] || { echo "candidate archive has no packaged model inputs" >&2; exit 1; }

cat >"$work/mock-mcp.py" <<'PY'
import json
import sys
import time

for line in sys.stdin:
    try:
        request = json.loads(line)
    except json.JSONDecodeError:
        continue
    request_id = request.get("id")
    if request_id is None:
        continue
    method = request.get("method")
    if method == "initialize":
        result = {
            "protocolVersion": request.get("params", {}).get("protocolVersion", "2024-11-05"),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "uar-resilience-mcp", "version": "1.0.0"},
        }
    elif method == "tools/list":
        result = {"tools": [{
            "name": "echo",
            "description": "Operational resilience process-boundary fixture",
            "inputSchema": {
                "type": "object",
                "properties": {"mode": {"type": "string"}},
                "required": ["mode"],
            },
        }]}
    elif method == "tools/call":
        mode = request.get("params", {}).get("arguments", {}).get("mode", "echo")
        if mode == "crash":
            sys.exit(23)
        if mode == "timeout":
            time.sleep(35)
        result = {"content": [{"type": "text", "text": f"mcp-{mode}"}], "isError": False}
    else:
        print(json.dumps({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "method not found"}}), flush=True)
        continue
    print(json.dumps({"jsonrpc": "2.0", "id": request_id, "result": result}), flush=True)
PY
cat >"$work/mcp.json" <<JSON
{"mcpServers":{"resilience":{"command":"python3","args":["$work/mock-mcp.py"]}}}
JSON

cat >"$work/mock-openai.py" <<'PY'
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        body = {"object": "list", "data": [{"id": "gpt-4o", "object": "model", "owned_by": "certification"}]}
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(body).encode())

    def do_POST(self):
        length = int(self.headers.get("content-length", "0"))
        request = json.loads(self.rfile.read(length) or b"{}")
        request_text = json.dumps(request)
        messages = request.get("messages", [])
        if any(message.get("role") == "tool" for message in messages):
            if request.get("stream"):
                chunks = [
                    {"id": "candidate-certification", "object": "chat.completion.chunk", "created": 0, "model": request.get("model", "gpt-4o"), "choices": [{"index": 0, "delta": {"content": "mcp-recovered"}, "finish_reason": None}]},
                    {"id": "candidate-certification", "object": "chat.completion.chunk", "created": 0, "model": request.get("model", "gpt-4o"), "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}]},
                ]
                self.send_response(200)
                self.send_header("Content-Type", "text/event-stream")
                self.end_headers()
                for chunk in chunks:
                    self.wfile.write(("data: " + json.dumps(chunk) + "\n\n").encode())
                self.wfile.write(b"data: [DONE]\n\n")
                return
            body = {
                "id": "candidate-certification",
                "object": "chat.completion",
                "created": 0,
                "model": request.get("model", "gpt-4o"),
                "choices": [{"index": 0, "message": {"role": "assistant", "content": "mcp-recovered"}, "finish_reason": "stop"}],
                "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2},
            }
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(body).encode())
            return
        mcp_modes = {
            "mcp-echo": "echo",
            "mcp-crash": "crash",
            "mcp-timeout": "timeout",
        }
        for marker, mode in mcp_modes.items():
            if marker in request_text:
                if request.get("stream"):
                    chunks = [
                        {"id": "candidate-certification", "object": "chat.completion.chunk", "created": 0, "model": request.get("model", "gpt-4o"), "choices": [{"index": 0, "delta": {"tool_calls": [{"index": 0, "id": "resilience-call", "type": "function", "function": {"name": "resilience__echo", "arguments": json.dumps({"mode": mode})}}]}, "finish_reason": None}]},
                        {"id": "candidate-certification", "object": "chat.completion.chunk", "created": 0, "model": request.get("model", "gpt-4o"), "choices": [{"index": 0, "delta": {}, "finish_reason": "tool_calls"}]},
                    ]
                    self.send_response(200)
                    self.send_header("Content-Type", "text/event-stream")
                    self.end_headers()
                    for chunk in chunks:
                        self.wfile.write(("data: " + json.dumps(chunk) + "\n\n").encode())
                    self.wfile.write(b"data: [DONE]\n\n")
                    return
                body = {
                    "id": "candidate-certification",
                    "object": "chat.completion",
                    "created": 0,
                    "model": request.get("model", "gpt-4o"),
                    "choices": [{"index": 0, "message": {"role": "assistant", "content": None, "tool_calls": [{
                        "id": "resilience-call",
                        "type": "function",
                        "function": {"name": "resilience__echo", "arguments": json.dumps({"mode": mode})},
                    }]}, "finish_reason": "tool_calls"}],
                    "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2},
                }
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps(body).encode())
                return
        if "provider-outage" in request_text:
            self.send_response(503)
            self.end_headers()
            return
        if "rate-limit" in request_text:
            self.send_response(429)
            self.send_header("Retry-After", "1")
            self.end_headers()
            return
        if "malformed-provider-stream" in request_text:
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.end_headers()
            self.wfile.write(b"data: {not-json}\n\n")
            self.wfile.write(b"data: [DONE]\n\n")
            return
        if request.get("stream"):
            body = {
                "id": "candidate-certification",
                "object": "chat.completion.chunk",
                "created": 0,
                "model": request.get("model", "gpt-4o"),
                "choices": [{"index": 0, "delta": {"content": "candidate-certified"}, "finish_reason": None}],
            }
            done = {
                "id": "candidate-certification",
                "object": "chat.completion.chunk",
                "created": 0,
                "model": request.get("model", "gpt-4o"),
                "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}],
            }
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.end_headers()
            self.wfile.write(("data: " + json.dumps(body) + "\n\n").encode())
            self.wfile.write(("data: " + json.dumps(done) + "\n\n").encode())
            self.wfile.write(b"data: [DONE]\n\n")
            return
        body = {
            "id": "candidate-certification",
            "object": "chat.completion",
            "created": 0,
            "model": request.get("model", "gpt-4o"),
            "choices": [{"index": 0, "message": {"role": "assistant", "content": "candidate-certified"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2},
        }
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(body).encode())

ThreadingHTTPServer(("127.0.0.1", int(sys.argv[1])), Handler).serve_forever()
PY
python3 "$work/mock-openai.py" "$mock_port" >"$results/mock-provider.log" 2>&1 &
mock_pid=$!

start_server() {
  local data_url="$1"
  UAR_LLM__MODEL=openai/gpt-4o \
  UAR_LLM__BASE_URL="http://127.0.0.1:${mock_port}/v1" \
  UAR_LLM__API_KEY=candidate-certification \
  UAR_SECURITY__JWT_REQUIRED=false \
  UAR_SECURITY__SETTINGS_MUTATION_AUTH_REQUIRED=false \
  UAR_SERVER__HOST=127.0.0.1 \
  UAR_STATIC_DIR="$package_root/static" \
  UAR_BUILTIN_SKILLS_DIR="$package_root/skills/builtin" \
  UAR_MODELS_DIR="$package_root/models" \
  MCP_CONFIG_PATH="$work/mcp.json" \
  UAR_PERSISTENCE__PROVIDER=surreal \
  UAR_PERSISTENCE__DATABASE_URL="$data_url" \
    "$binary" --port "$port" >"$results/server.log" 2>&1 &
  server_pid=$!
  for _ in $(seq 1 90); do
    curl --fail --silent "http://127.0.0.1:${port}/readyz" >/dev/null && return
    if ! kill -0 "$server_pid" 2>/dev/null; then cat "$results/server.log" >&2; exit 1; fi
    sleep 1
  done
  cat "$results/server.log" >&2
  echo "candidate readiness timeout" >&2
  exit 1
}

stop_server() {
  kill -TERM "$server_pid"
  wait "$server_pid"
  server_pid=""
}

smoke_sidecar() {
  curl --fail --silent "http://127.0.0.1:${port}/healthz" >/dev/null
  curl --fail --silent "http://127.0.0.1:${port}/readyz" >/dev/null
  curl --fail --silent "http://127.0.0.1:${port}/" >"$results/index.html"
  grep -qi '<html' "$results/index.html"
  curl --fail --silent "http://127.0.0.1:${port}/v1/models" >"$results/models.json"
  curl --fail --silent "http://127.0.0.1:${port}/v1/chat/completions" \
    -H 'content-type: application/json' \
    -d '{"model":"openai/gpt-4o","stream":false,"messages":[{"role":"user","content":"candidate certification"}]}' \
    >"$results/chat.json"
  grep -q 'candidate-certified' "$results/chat.json"
}

chat_request() {
  local prompt="$1"
  local stream="${2:-false}"
  local output="$3"
  curl --silent --show-error --max-time 45 \
    -o "$output" -w '%{http_code} %{time_total}' \
    "http://127.0.0.1:${port}/v1/chat/completions" \
    -H 'content-type: application/json' \
    -d "{\"model\":\"openai/gpt-4o\",\"stream\":${stream},\"messages\":[{\"role\":\"user\",\"content\":\"${prompt}\"}]}"
}

certify_failure_recovery() {
  local failure code body metrics="$results/failure-recovery.jsonl"
  : >"$metrics"
  for failure in provider-outage rate-limit malformed-provider-stream; do
    set +e
    read -r code _ < <(chat_request "$failure" true "$results/${failure}.json")
    request_status=$?
    set -e
    body="$(tr '\n' ' ' <"$results/${failure}.json")"
    if [[ $request_status -eq 0 && "$code" == 2* && "$body" != *error* ]]; then
      echo "$failure was accepted as a successful provider response" >&2
      return 1
    fi
    printf '{"source_sha":"%s","candidate_tag":"%s","failure":"%s","http_status":"%s","surfaced":true}\n' \
      "$source_sha" "$candidate_tag" "$failure" "${code:-transport-error}" >>"$metrics"
  done
  read -r code _ < <(chat_request recovery false "$results/recovery.json")
  [[ "$code" == 2* ]] && grep -q candidate-certified "$results/recovery.json"
}

certify_mcp_process_boundary() {
  local attempt code health_code health_status
  health_code="not-requested"
  health_status=1
  for attempt in $(seq 1 15); do
    set +e
    health_code="$(curl --silent --show-error --max-time 5 \
      -o "$results/mcp-health.json" -w '%{http_code}' \
      "http://127.0.0.1:${port}/api/uar/mcp/health")"
    health_status=$?
    set -e
    if [[ $health_status -eq 0 && "$health_code" == 200 ]] && grep -q 'resilience' "$results/mcp-health.json"; then
      break
    fi
    if ! kill -0 "$server_pid" 2>/dev/null; then
      echo "candidate exited while waiting for MCP health" >&2
      cat "$results/server.log" >&2
      return 1
    fi
    sleep 1
  done
  if [[ $health_status -ne 0 || "$health_code" != 200 ]] || ! grep -q 'resilience' "$results/mcp-health.json"; then
    echo "MCP health did not expose the resilience fixture (curl=$health_status http=$health_code)" >&2
    cat "$results/mcp-health.json" >&2 || true
    return 1
  fi

  read -r code _ < <(chat_request mcp-echo false "$results/mcp-echo.json")
  [[ "$code" == 2* ]] && grep -q mcp-recovered "$results/mcp-echo.json"

  set +e
  read -r crash_code _ < <(chat_request mcp-crash false "$results/mcp-crash.json")
  crash_request_status=$?
  set -e
  if [[ $crash_request_status -eq 0 && "$crash_code" == 2* ]] && grep -q mcp-recovered "$results/mcp-crash.json"; then
    echo "crashed MCP call was replayed or reported as successful" >&2
    return 1
  fi
  read -r code _ < <(chat_request mcp-echo false "$results/mcp-after-crash.json")
  [[ "$code" == 2* ]] && grep -q mcp-recovered "$results/mcp-after-crash.json"

  timeout_started="$(date +%s)"
  set +e
  read -r timeout_code _ < <(chat_request mcp-timeout false "$results/mcp-timeout.json")
  timeout_request_status=$?
  set -e
  timeout_elapsed=$(( $(date +%s) - timeout_started ))
  if [[ $timeout_request_status -eq 0 && "$timeout_code" == 2* ]] && grep -q mcp-recovered "$results/mcp-timeout.json"; then
    echo "timed-out MCP call was replayed or reported as successful" >&2
    return 1
  fi
  [[ $timeout_elapsed -ge 30 && $timeout_elapsed -lt 45 ]]
  read -r code _ < <(chat_request mcp-echo false "$results/mcp-after-timeout.json")
  [[ "$code" == 2* ]] && grep -q mcp-recovered "$results/mcp-after-timeout.json"

  cat >"$results/mcp-process-boundary.json" <<JSON
{"source_sha":"$source_sha","candidate_tag":"$candidate_tag","stdio_discovery":true,"tool_call":true,"configured_crash_exit_code":23,"transport_loss_surfaced":true,"reconnected_after_crash":true,"tool_timeout_seconds":30,"observed_timeout_seconds":$timeout_elapsed,"timed_out_call_replayed":false,"reconnected_after_timeout":true}
JSON
}

certify_parallel_load() {
  local load_dir="$work/load" failures=0
  local -a pids=()
  mkdir -p "$load_dir"
  for request in $(seq 1 "$parallel_requests"); do
    (
      read -r code elapsed < <(chat_request "parallel-${request}" false "$load_dir/${request}.json")
      [[ "$code" == 2* ]]
      grep -q candidate-certified "$load_dir/${request}.json"
      printf '%s\n' "$elapsed" >"$load_dir/${request}.latency"
    ) &
    pids+=("$!")
  done
  for pid in "${pids[@]}"; do
    wait "$pid" || failures=$((failures + 1))
  done
  [[ $failures -eq 0 ]] || { echo "$failures parallel candidate requests failed" >&2; return 1; }
  awk -v sha="$source_sha" -v tag="$candidate_tag" \
    '{sum += $1} END {printf "{\"source_sha\":\"%s\",\"candidate_tag\":\"%s\",\"requests\":%d,\"failures\":0,\"mean_latency_ms\":%.3f}\n", sha, tag, NR, (sum / NR) * 1000}' \
    "$load_dir"/*.latency >"$results/parallel-load.json"
}

rss_kib() {
  if [[ -r "/proc/${server_pid}/status" ]]; then
    awk '/^VmRSS:/ {print $2}' "/proc/${server_pid}/status"
  else
    ps -o rss= -p "$server_pid" | tr -d ' '
  fi
}

certify_streaming_soak() {
  local started_epoch deadline request=0 errors=0 duplicates=0 max_rss initial_rss final_rss p95_ms final_growth peak_growth
  local soak_dir="$work/soak"
  mkdir -p "$soak_dir"
  started_epoch="$(date +%s)"
  deadline=$((started_epoch + soak_duration_seconds))
  initial_rss="$(rss_kib)"
  max_rss="$initial_rss"
  while (( $(date +%s) < deadline )); do
    request=$((request + 1))
    set +e
    read -r code elapsed < <(chat_request "streaming-soak-${request}" true "$soak_dir/${request}.sse")
    request_status=$?
    set -e
    if [[ $request_status -ne 0 || "$code" != 2* ]] || ! grep -q candidate-certified "$soak_dir/${request}.sse"; then
      errors=$((errors + 1))
    else
      occurrences="$(grep -o candidate-certified "$soak_dir/${request}.sse" | wc -l | tr -d ' ')"
      if [[ "$occurrences" -ne 1 ]]; then duplicates=$((duplicates + occurrences - 1)); fi
    fi
    awk -v value="$elapsed" 'BEGIN {printf "%.0f\n", value * 1000}' >>"$soak_dir/latency-ms"
    current_rss="$(rss_kib)"
    if (( current_rss > max_rss )); then max_rss="$current_rss"; fi
    if (( soak_interval_seconds > 0 )); then sleep "$soak_interval_seconds"; fi
  done
  final_rss="$(rss_kib)"
  final_growth=$((final_rss - initial_rss))
  peak_growth=$((max_rss - initial_rss))
  p95_ms="$(sort -n "$soak_dir/latency-ms" | awk -v count="$request" 'NR == int((count * 95 + 99) / 100) {print; exit}')"
  cat >"$results/soak.json" <<JSON
{"schema_version":1,"source_sha":"$source_sha","candidate_tag":"$candidate_tag","configured_duration_seconds":$soak_duration_seconds,"observed_duration_seconds":$(($(date +%s) - started_epoch)),"stream_requests":$request,"errors":$errors,"duplicate_events":$duplicates,"p95_latency_ms":${p95_ms:-0},"initial_rss_kib":$initial_rss,"final_rss_kib":$final_rss,"max_rss_kib":$max_rss,"final_rss_growth_kib":$final_growth,"peak_rss_growth_kib":$peak_growth,"thresholds":{"errors":0,"duplicate_events":0,"p95_latency_ms":$soak_p95_limit_ms,"peak_rss_growth_kib":$soak_memory_growth_limit_kib}}
JSON
  [[ $request -gt 0 && $errors -eq 0 && $duplicates -eq 0 ]]
  [[ ${p95_ms:-0} -le $soak_p95_limit_ms ]]
  [[ $peak_growth -le $soak_memory_growth_limit_kib ]]
}

certify_upgrade_journey() {
  if [[ -z "$previous_artifacts" ]]; then
    [[ "${UAR_REQUIRE_UPGRADE_JOURNEY:-0}" != 1 ]] || {
      echo "UAR_PREVIOUS_ARTIFACT_DIR is required for the upgrade journey" >&2
      return 1
    }
    printf '{"status":"not-requested"}\n' >"$results/upgrade.json"
    return
  fi

  local previous_archive previous_binary previous_root candidate_binary candidate_root previous_manifest previous_identity
  previous_archive="$(find "$previous_artifacts" -maxdepth 1 -type f -name "$archive_glob" -print -quit)"
  [[ -n "$previous_archive" ]] || { echo "no previous archive matches $archive_glob" >&2; return 1; }
  previous_manifest="$previous_artifacts/release-manifest.json"
  previous_identity="$previous_artifacts/previous-identity.json"
  if [[ -f "$previous_manifest" ]]; then
    previous_tag="$(jq -er '.release' "$previous_manifest")"
    previous_sha="$(jq -er '.source.sha' "$previous_manifest")"
    expected_previous_digest="$(jq -er --arg name "$(basename "$previous_archive")" '.artifacts[] | select(.name == $name) | .sha256' "$previous_manifest")"
    previous_build_kind=published-release-artifact
  elif [[ -f "$previous_identity" ]]; then
    previous_tag="$(jq -er '.source_ref' "$previous_identity")"
    previous_sha="$(jq -er '.source_sha' "$previous_identity")"
    expected_previous_digest="$(jq -er '.archive_sha256' "$previous_identity")"
    [[ "$(jq -er '.archive' "$previous_identity")" == "$(basename "$previous_archive")" ]] || {
      echo "controlled previous archive name does not match its identity" >&2
      return 1
    }
    previous_build_kind="$(jq -er '.build_kind' "$previous_identity")"
    [[ "$previous_build_kind" == controlled-source-rebuild ]] || {
      echo "unsupported previous artifact identity kind" >&2
      return 1
    }
  else
    echo "previous artifact needs release-manifest.json or previous-identity.json" >&2
    return 1
  fi
  [[ -f "$previous_identity" ]] || {
    echo "previous artifact requires workflow-resolved previous-identity.json" >&2
    return 1
  }
  [[ "$(jq -er '.source_ref' "$previous_identity")" == "$previous_tag" ]]
  [[ "$(jq -er '.source_sha' "$previous_identity")" == "$previous_sha" ]]
  [[ "$(jq -er '.verified_public_release' "$previous_identity")" == true ]]
  previous_release_url="$(jq -er '.release_url' "$previous_identity")"
  previous_published_at="$(jq -er '.published_at' "$previous_identity")"
  previous_tag_object_sha="$(jq -er '.tag_object_sha' "$previous_identity")"
  previous_tag_object_type="$(jq -er '.tag_object_type' "$previous_identity")"
  [[ "$previous_tag_object_sha" =~ ^[0-9a-f]{40}$ ]]
  [[ "$previous_tag_object_type" == tag || "$previous_tag_object_type" == commit ]]
  [[ "$previous_tag" != "$candidate_tag" && "$previous_sha" != "$source_sha" ]] || {
    echo "upgrade source must be a distinct previous release" >&2
    return 1
  }
  previous_archive_digest="$(sha256sum "$previous_archive" | cut -d' ' -f1)"
  [[ "$previous_archive_digest" == "$expected_previous_digest" ]] || {
    echo "previous archive digest does not match its release manifest" >&2
    return 1
  }
  mkdir -p "$work/previous" "$work/upgrade-data"
  tar -xzf "$previous_archive" -C "$work/previous"
  previous_binary="$(find "$work/previous" -type f -name universal-agent-runtime -print -quit)"
  [[ -n "$previous_binary" ]] || { echo "previous archive has no universal-agent-runtime binary" >&2; return 1; }
  chmod +x "$previous_binary"
  previous_root="$(dirname "$previous_binary")"
  [[ -d "$previous_root/static" ]] || { echo "previous archive has no packaged React assets" >&2; return 1; }
  [[ -d "$previous_root/skills/builtin" ]] || { echo "previous archive has no packaged built-in skills" >&2; return 1; }
  [[ -d "$previous_root/models" ]] || { echo "previous archive has no packaged model inputs" >&2; return 1; }

  candidate_binary="$binary"
  candidate_root="$package_root"
  binary="$previous_binary"
  package_root="$previous_root"
  "$binary" --version >"$results/upgrade-from-version.txt"
  start_server "surrealkv://${work}/upgrade-data/uar.db"
  smoke_sidecar
  continuity_id="upgrade_continuity.marker"
  continuity_marker="retained-from-${previous_sha}"
  jq -n '{name:"Upgrade continuity certification",key:"upgrade_continuity",schema:{type:"object",properties:{marker:{type:"string"}}}}' \
    >"$results/upgrade-setting-type-request.json"
  previous_type_create_code="$(curl --silent --show-error -o "$results/upgrade-setting-type-created.json" -w '%{http_code}' \
    -X POST "http://127.0.0.1:${port}/api/uar/settings/types" \
    -H 'content-type: application/json' --data-binary @"$results/upgrade-setting-type-request.json")"
  [[ "$previous_type_create_code" == 200 ]]
  jq -n --arg marker "$continuity_marker" '{value:$marker}' >"$results/upgrade-record-request.json"
  previous_create_code="$(curl --silent --show-error -o "$results/upgrade-record-created.json" -w '%{http_code}' \
    -X PUT "http://127.0.0.1:${port}/api/uar/settings/${continuity_id}" \
    -H 'content-type: application/json' --data-binary @"$results/upgrade-record-request.json")"
  [[ "$previous_create_code" == 200 ]]
  previous_read_code="$(curl --silent --show-error -o "$results/upgrade-record-previous.json" -w '%{http_code}' \
    "http://127.0.0.1:${port}/api/uar/settings/${continuity_id}")"
  [[ "$previous_read_code" == 200 ]]
  previous_record_digest="$(jq -S '{key,data}' "$results/upgrade-record-previous.json" | sha256sum | cut -d' ' -f1)"
  [[ "$(jq -r '.data' "$results/upgrade-record-previous.json")" == "$continuity_marker" ]]
  stop_server
  cp "$results/server.log" "$results/upgrade-previous.log"
  tar -czf "$work/upgrade-backup.tar.gz" -C "$work/upgrade-data" .
  upgrade_backup_digest="$(tree_digest "$work/upgrade-data")"

  binary="$candidate_binary"
  package_root="$candidate_root"
  start_server "surrealkv://${work}/upgrade-data/uar.db"
  smoke_sidecar
  candidate_read_code="$(curl --silent --show-error -o "$results/upgrade-record-candidate.json" -w '%{http_code}' \
    "http://127.0.0.1:${port}/api/uar/settings/${continuity_id}")"
  [[ "$candidate_read_code" == 200 ]]
  candidate_record_digest="$(jq -S '{key,data}' "$results/upgrade-record-candidate.json" | sha256sum | cut -d' ' -f1)"
  [[ "$candidate_record_digest" == "$previous_record_digest" ]]
  stop_server
  cp "$results/server.log" "$results/upgrade-candidate.log"

  mkdir -p "$work/rollback-data"
  tar -xzf "$work/upgrade-backup.tar.gz" -C "$work/rollback-data"
  rollback_restored_digest="$(tree_digest "$work/rollback-data")"
  [[ "$rollback_restored_digest" == "$upgrade_backup_digest" ]]
  binary="$previous_binary"
  package_root="$previous_root"
  start_server "surrealkv://${work}/rollback-data/uar.db"
  rollback_read_code="$(curl --silent --show-error -o "$results/upgrade-record-rollback.json" -w '%{http_code}' \
    "http://127.0.0.1:${port}/api/uar/settings/${continuity_id}")"
  [[ "$rollback_read_code" == 200 ]]
  rollback_record_digest="$(jq -S '{key,data}' "$results/upgrade-record-rollback.json" | sha256sum | cut -d' ' -f1)"
  [[ "$rollback_record_digest" == "$previous_record_digest" ]]
  smoke_sidecar
  stop_server
  cp "$results/server.log" "$results/upgrade-rollback.log"
  binary="$candidate_binary"
  package_root="$candidate_root"
  cat >"$results/upgrade.json" <<JSON
{"status":"passed","source_sha":"$source_sha","candidate_tag":"$candidate_tag","previous_source_sha":"$previous_sha","previous_ref":"$previous_tag","previous_tag_object_sha":"$previous_tag_object_sha","previous_tag_object_type":"$previous_tag_object_type","previous_build_kind":"$previous_build_kind","previous_public_release_verified":true,"previous_release_url":"$previous_release_url","previous_published_at":"$previous_published_at","previous_archive":"$(basename "$previous_archive")","previous_archive_sha256":"$previous_archive_digest","candidate_archive":"$(basename "$archive")","candidate_archive_sha256":"$archive_digest","continuity_record_kind":"durable-setting","continuity_record_id":"$continuity_id","continuity_marker":"$continuity_marker","previous_type_create_http_status":$previous_type_create_code,"previous_create_http_status":$previous_create_code,"previous_read_http_status":$previous_read_code,"candidate_read_http_status":$candidate_read_code,"rollback_read_http_status":$rollback_read_code,"previous_record_sha256":"$previous_record_digest","candidate_record_sha256":"$candidate_record_digest","rollback_record_sha256":"$rollback_record_digest","pre_upgrade_backup_tree_sha256":"$upgrade_backup_digest","rollback_restored_tree_sha256":"$rollback_restored_digest","upgrade_database_url":"surrealkv://<work>/upgrade-data/uar.db","rollback_database_url":"surrealkv://<work>/rollback-data/uar.db"}
JSON
  upgrade_journey=',"prior-version-upgrade"'
}

"$binary" --version | tee "$results/version.txt"
grep -q 'universal-agent-runtime 1.0.0' "$results/version.txt"

# Explicit non-default path and port exercise the documented configuration and
# troubleshooting overrides. The same OpenAI-compatible call is BossFang's
# supported sidecar seam.
data_url="surrealkv://${work}/data/uar.db"
start_server "$data_url"
smoke_sidecar
certify_failure_recovery
certify_mcp_process_boundary
certify_parallel_load
certify_streaming_soak
stop_server
cp "$results/server.log" "$results/installed-runtime.log"

# Exercise the documented cold-copy backup and restore path from the installed
# artifact. Digest equality is checked before the restored process can mutate
# its datastore during startup.
tar -czf "$work/uar-backup.tar.gz" -C "$work/data" .
mkdir "$work/restored"
tar -xzf "$work/uar-backup.tar.gz" -C "$work/restored"
tree_digest() {
  local directory="$1"
  (cd "$directory" && find . -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum | cut -d' ' -f1)
}
source_digest="$(tree_digest "$work/data")"
restore_digest="$(tree_digest "$work/restored")"
[[ "$source_digest" == "$restore_digest" ]]
start_server "surrealkv://${work}/restored/uar.db"
smoke_sidecar
stop_server
printf '{"source_sha":"%s","candidate_tag":"%s","startup":true,"readiness":true,"sigterm_exit_code":0,"restart":true}\n' \
  "$source_sha" "$candidate_tag" >"$results/lifecycle.json"

certify_upgrade_journey

container_journey=""
if [[ -n "${UAR_CANDIDATE_IMAGE:-}" ]]; then
  mkdir -p "$work/container-data"
  chmod 0777 "$work/container-data"
  container_id="$(docker run -d --network host --user 65532:65532 \
    -e UAR_LLM__MODEL=openai/gpt-4o \
    -e UAR_LLM__BASE_URL="http://127.0.0.1:${mock_port}/v1" \
    -e UAR_LLM__API_KEY=candidate-certification \
    -e UAR_SECURITY__JWT_REQUIRED=false \
    -e UAR_SERVER__HOST=127.0.0.1 \
    -e UAR_SERVER__PORT="$port" \
    -e UAR_PERSISTENCE__DATABASE_URL=surrealkv:///var/lib/uar/data/uar.db \
    -v "$work/container-data:/var/lib/uar" \
    "$UAR_CANDIDATE_IMAGE")"
  for _ in $(seq 1 90); do
    curl --fail --silent "http://127.0.0.1:${port}/readyz" >/dev/null && break
    if [[ "$(docker inspect -f '{{.State.Running}}' "$container_id")" != true ]]; then
      docker logs "$container_id" >&2
      exit 1
    fi
    sleep 1
  done
  smoke_sidecar
  container_uid="$(docker exec "$container_id" id -u)"
  [[ "$container_uid" -ne 0 ]]
  docker stop --time 30 "$container_id" >/dev/null
  docker logs "$container_id" >"$results/container.log" 2>&1
  container_exit_code="$(docker inspect -f '{{.State.ExitCode}}' "$container_id")"
  [[ "$container_exit_code" == 0 ]]
  find "$work/container-data" -mindepth 1 -print -quit | grep -q .
  cat >"$results/non-root-container.json" <<JSON
{"source_sha":"$source_sha","candidate_tag":"$candidate_tag","uid":$container_uid,"writable_persistence":true,"health":true,"sigterm_exit_code":$container_exit_code}
JSON
  docker rm "$container_id" >/dev/null
  container_id=""
  container_journey=',"non-root-container"'
fi

cat >"$results/results.json" <<JSON
{"schema_version":2,"outcome":"passed","source_sha":"$source_sha","candidate_tag":"$candidate_tag","archive":"$(basename "$archive")","archive_sha256":"$archive_digest","backup_sha256":"$source_digest","restored_sha256":"$restore_digest","port":$port,"soak_duration_seconds":$soak_duration_seconds,"evidence":{"lifecycle":"lifecycle.json","failure_recovery":"failure-recovery.jsonl","mcp_process_boundary":"mcp-process-boundary.json","parallel_load":"parallel-load.json","soak":"soak.json","upgrade":"upgrade.json"},"journeys":["archive-install","version","packaged-react-app","configuration-override","health-readiness","bossfang-openai-sidecar","provider-failure-recovery","mcp-crash-reconnect-timeout","parallel-load","streaming-reconnect-soak","cold-backup-restore"$upgrade_journey$container_journey]}
JSON

echo "Candidate artifact certification passed; evidence: $results/results.json"
