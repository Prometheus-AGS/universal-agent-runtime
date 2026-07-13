#!/usr/bin/env bash
set -euo pipefail

# Certify an installed release payload rather than a development checkout.
# Usage: scripts/certify-release-candidate.sh <artifact-directory> [results-directory]

artifacts="${1:?artifact directory is required}"
results="${2:-target/release-candidate-certification}"
archive_glob="${UAR_CANDIDATE_ARCHIVE_GLOB:-*linux-x64.tar.gz}"
port="${UAR_CANDIDATE_PORT:-1906}"
mock_port="${UAR_CANDIDATE_MOCK_PORT:-1907}"
work="$(mktemp -d)"
server_pid=""
mock_pid=""
container_id=""

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
tar -xzf "$archive" -C "$work/installed"
binary="$(find "$work/installed" -type f -name universal-agent-runtime -print -quit)"
[[ -n "$binary" ]] || { echo "candidate archive has no universal-agent-runtime binary" >&2; exit 1; }
chmod +x "$binary"
package_root="$(dirname "$binary")"
[[ -d "$package_root/static" ]] || { echo "candidate archive has no packaged React assets" >&2; exit 1; }

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
  UAR_SERVER__HOST=127.0.0.1 \
  UAR_STATIC_DIR="$package_root/static" \
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

"$binary" --version | tee "$results/version.txt"
grep -q 'universal-agent-runtime 1.0.0' "$results/version.txt"

# Explicit non-default path and port exercise the documented configuration and
# troubleshooting overrides. The same OpenAI-compatible call is BossFang's
# supported sidecar seam.
data_url="surrealkv://${work}/data/uar.db"
start_server "$data_url"
smoke_sidecar
stop_server

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
  docker stop --time 30 "$container_id" >/dev/null
  docker logs "$container_id" >"$results/container.log" 2>&1
  [[ "$(docker inspect -f '{{.State.ExitCode}}' "$container_id")" == 0 ]]
  docker rm "$container_id" >/dev/null
  container_id=""
  container_journey=',"non-root-container"'
fi

archive_digest="$(sha256sum "$archive" | cut -d' ' -f1)"
cat >"$results/results.json" <<JSON
{"schema_version":1,"outcome":"passed","archive":"$(basename "$archive")","archive_sha256":"$archive_digest","backup_sha256":"$source_digest","restored_sha256":"$restore_digest","port":$port,"journeys":["archive-install","version","packaged-react-app","configuration-override","health-readiness","bossfang-openai-sidecar","cold-backup-restore"$container_journey]}
JSON

echo "Candidate artifact certification passed; evidence: $results/results.json"
