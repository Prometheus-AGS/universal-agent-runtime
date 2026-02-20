#!/usr/bin/env bash
set -euo pipefail

: "${OPENAI_API_KEY:?OPENAI_API_KEY must be set}"

# Get the directory where this script is located (universal-agent-runtime root)
UAR_REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$UAR_REPO"

PORT=6565 \
UAR_SERVER__PORT=6565 \
LLM_BASE_URL='https://api.openai.com' \
LLM_MODEL='gpt-5.2' \
LLM_API_KEY="$OPENAI_API_KEY" \
UAR_SECURITY__JWT_REQUIRED=false \
UAR_PERSISTENCE__PROVIDER=surreal \
UAR_PERSISTENCE__DATABASE_URL='rocksdb://./data/uar.db' \
UAR_PERSISTENCE__VECTOR_DIMENSION=384 \
./target/release/universal-agent-runtime
