#!/usr/bin/env bash
set -euo pipefail

# validate-config.sh — YAML syntax + required-field check on generated config files
# Usage: validate-config.sh [config_path] [env_path]
# Exit 0 = valid, Exit 1 = errors found

CONFIG_PATH="${1:-./config.yaml}"
ENV_PATH="${2:-./.env}"

ERRORS=0
WARNINGS=0

echo "=== UAR Config Validation ==="

# --- YAML Syntax Check ---
if [ -f "$CONFIG_PATH" ]; then
  if python3 -c "import yaml; yaml.safe_load(open('$CONFIG_PATH'))" 2>/dev/null; then
    echo "✅ YAML syntax: $CONFIG_PATH"
  else
    echo "❌ YAML syntax error in $CONFIG_PATH:"
    python3 -c "import yaml; yaml.safe_load(open('$CONFIG_PATH'))" 2>&1 | head -5
    ERRORS=$((ERRORS + 1))
  fi
else
  echo "ℹ️  No config.yaml found at $CONFIG_PATH (will use env vars only)"
fi

# --- Required Fields Check ---
if [ -f "$CONFIG_PATH" ]; then
  python3 - <<PYEOF
import yaml, sys

errors = []
warnings = []

with open("$CONFIG_PATH") as f:
    cfg = yaml.safe_load(f) or {}

# Required: database_url
persistence = cfg.get("persistence", {})
if not persistence.get("database_url"):
    errors.append("persistence.database_url is required")

# Required: llm.model
llm = cfg.get("llm", {})
model = llm.get("model", "")
if not model:
    errors.append("llm.model is required")
elif "/" not in model:
    errors.append(f"llm.model must be 'provider/model' format, got: '{model}'")

# Security: default jwt_secret
security = cfg.get("security", {})
jwt_secret = security.get("jwt_secret", "")
if jwt_secret in ("secret_key_change_me", "fallback_secret_change_in_production", ""):
    errors.append("security.jwt_secret is the default placeholder — generate a real secret: openssl rand -base64 64")

# candle-vllm protocol check
providers = cfg.get("providers", [])
for p in providers:
    if p.get("id") == "candle-vllm" or "candle" in p.get("base_url", ""):
        if p.get("protocol") == "responses":
            errors.append(f"provider '{p.get('id')}': protocol must be 'chat' for candle-vllm, not 'responses'")

# Warn on missing optional but recommended fields
if not security.get("jwt_secret"):
    warnings.append("security.jwt_secret not set")

for e in errors:
    print(f"❌ ERROR: {e}")
for w in warnings:
    print(f"⚠️  WARNING: {w}")

sys.exit(len(errors))
PYEOF
  PYEXIT=$?
  ERRORS=$((ERRORS + PYEXIT))
fi

# --- .env Check ---
if [ -f "$ENV_PATH" ]; then
  echo ""
  echo "=== .env Check ==="

  # Check for legacy LLM_ vars
  if grep -qE "^LLM_API_KEY=" "$ENV_PATH" 2>/dev/null; then
    echo "⚠️  WARNING: LLM_API_KEY is deprecated. Use UAR_LLM__API_KEY instead."
    WARNINGS=$((WARNINGS + 1))
  fi
  if grep -qE "^LLM_MODEL=" "$ENV_PATH" 2>/dev/null; then
    echo "⚠️  WARNING: LLM_MODEL is deprecated. Use UAR_LLM__MODEL instead."
    WARNINGS=$((WARNINGS + 1))
  fi

  # Check for default JWT secret in env
  if grep -qE "^UAR_SECURITY__JWT_SECRET=secret_key_change_me" "$ENV_PATH" 2>/dev/null; then
    echo "❌ ERROR: UAR_SECURITY__JWT_SECRET is set to the default placeholder."
    ERRORS=$((ERRORS + 1))
  fi

  echo "✅ .env scanned"
fi

echo ""
echo "=== Summary: $ERRORS error(s), $WARNINGS warning(s) ==="

exit "$ERRORS"
