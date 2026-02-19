#!/usr/bin/env bash
set -euo pipefail

: "${OPENAI_API_KEY:?OPENAI_API_KEY must be set}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

PORT=6565 \
UAR_SERVER__PORT=6565 \
LLM_BASE_URL='https://api.openai.com' \
LLM_MODEL='gpt-5.2' \
LLM_API_KEY="$OPENAI_API_KEY" \
UAR_SECURITY__JWT_REQUIRED=false \
UAR_PERSISTENCE__PROVIDER=surreal \
UAR_PERSISTENCE__DATABASE_URL='rocksdb://./data/uar.db' \
./target/release/universal-agent-runtime
