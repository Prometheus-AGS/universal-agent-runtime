#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:1906}"
PROMPT="${PROMPT:-Reply with exactly: openai-ok}"
MODEL="${MODEL:-gpt-5.2}"
WAIT_SECONDS="${WAIT_SECONDS:-3}"
STREAM_TIMEOUT="${STREAM_TIMEOUT:-20}"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "ERROR: required command not found: $1" >&2
    exit 1
  fi
}

need_cmd curl
need_cmd sed
need_cmd grep

extract_json_field() {
  local json="$1"
  local field="$2"
  echo "$json" | sed -n "s/.*\"$field\":\"\\([^\"]*\\)\".*/\\1/p"
}

echo "== UAR chat test =="
echo "BASE_URL=$BASE_URL"
echo "PROMPT=$PROMPT"
echo "MODEL=$MODEL"
echo

echo "== Case 1: non-streaming (stream=false) =="
REQ1=$(printf '{"model":"%s","message":"%s","stream":false,"temperature":0.2}' "$MODEL" "$PROMPT")
RESP1=$(curl -sS -i -H 'content-type: application/json' -d "$REQ1" "$BASE_URL/api/chat/completion")
SID1=$(echo "$RESP1" | sed -n 's/^[Xx]-[Uu][Aa][Rr]-[Ss]ession-[Ii][Dd]:[[:space:]]*\(.*\)$/\1/p' | tr -d '\r' | tail -n 1)
BODY1=$(echo "$RESP1" | awk 'BEGIN{p=0} /^\r?$/{p=1;next} {if(p) print}')
if [[ -z "$SID1" ]]; then
  SID1=$(extract_json_field "$BODY1" "session_id")
fi

if [[ -z "$SID1" ]]; then
  echo "FAIL: non-streaming request did not return session ID"
  echo "Response: $RESP1"
  exit 1
fi

sleep "$WAIT_SECONDS"
FOLLOW1=$(curl -sS -H 'content-type: application/json' \
  -H "X-UAR-Session-ID: $SID1" \
  -d "{\"model\":\"$MODEL\",\"message\":\"What exact phrase did you just output? Reply with only that phrase.\",\"stream\":false}" \
  "$BASE_URL/api/chat/completion")
echo "POST response (headers+body): $RESP1"
echo "Follow-up response: $FOLLOW1"

if echo "$FOLLOW1" | grep -q 'openai-ok'; then
  echo "PASS: non-streaming session continuity works"
else
  echo "FAIL: non-streaming session continuity failed"
  exit 1
fi

echo
echo "== Case 2: streaming (stream=true) =="
REQ2=$(printf '{"model":"%s","message":"%s","stream":true}' "$MODEL" "$PROMPT")
STREAM_RESPONSE=$(curl -i -N -sS --max-time "$STREAM_TIMEOUT" -H 'content-type: application/json' -d "$REQ2" "$BASE_URL/api/chat/completion" || true)
SID2=$(echo "$STREAM_RESPONSE" | sed -n 's/^[Xx]-[Uu][Aa][Rr]-[Ss]ession-[Ii][Dd]:[[:space:]]*\(.*\)$/\1/p' | tr -d '\r' | tail -n 1)
STREAM_EVENTS=$(echo "$STREAM_RESPONSE" | awk 'BEGIN{p=0} /^\r?$/{p=1;next} {if(p) print}')

if [[ -z "$SID2" ]]; then
  echo "FAIL: streaming request did not return session ID header"
  echo "Streaming response: $STREAM_RESPONSE"
  exit 1
fi

sleep "$WAIT_SECONDS"
FOLLOW2=$(curl -sS -H 'content-type: application/json' \
  -H "X-UAR-Session-ID: $SID2" \
  -d "{\"model\":\"$MODEL\",\"message\":\"What exact phrase did you just output? Reply with only that phrase.\",\"stream\":false}" \
  "$BASE_URL/api/chat/completion")

echo "Streaming response (headers+events):"
echo "$STREAM_RESPONSE"
echo "Stream events:"
echo "$STREAM_EVENTS"
echo "Follow-up response: $FOLLOW2"

if echo "$STREAM_EVENTS" | grep -q '"chat.completion.chunk"' && echo "$STREAM_EVENTS" | grep -q '\[DONE\]'; then
  echo "PASS: streaming endpoint emitted delta and done events"
else
  echo "FAIL: streaming endpoint did not emit expected events"
  exit 1
fi

if echo "$FOLLOW2" | grep -q 'openai-ok'; then
  echo "PASS: streaming session continuity works"
else
  echo "FAIL: streaming session continuity failed"
  exit 1
fi

echo
echo "ALL TESTS PASSED"
