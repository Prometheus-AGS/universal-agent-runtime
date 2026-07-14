#!/usr/bin/env bash
# Smoke-test every SDK example across sdks/{rust,python,typescript}/examples/.
#
# Every example talks to a live UAR server (http://localhost:1906 by
# default) and, for chat/agent examples, a configured LLM backend behind
# it - neither is available in a bare CI runner. So the default mode here
# is a *compile/typecheck* smoke test: every example must build cleanly
# against the current SDK surface, which is exactly the signal that
# catches API drift between the SDK and its own examples.
#
# The one fully self-contained example - sdks/rust/examples/error_handling.rs,
# which deliberately targets an unreachable port and asserts on the
# resulting miette diagnostic - is actually *run*, not just compiled.
#
# Set VALIDATE_EXAMPLES_LIVE=1 and UAR_BASE_URL to a real running server to
# additionally execute every example end-to-end (e.g. in a nightly job
# against a staging deployment). Compile-only mode is what CI runs on
# every PR.
#
# Usage: tools/validate-examples.sh [--rust-only|--python-only|--typescript-only]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

RUN_RUST=true
RUN_PYTHON=true
RUN_TS=true

for arg in "$@"; do
  case "$arg" in
  --rust-only)
    RUN_PYTHON=false
    RUN_TS=false
    ;;
  --python-only)
    RUN_RUST=false
    RUN_TS=false
    ;;
  --typescript-only)
    RUN_RUST=false
    RUN_PYTHON=false
    ;;
  *)
    echo "Unknown option: $arg" >&2
    exit 2
    ;;
  esac
done

LIVE="${VALIDATE_EXAMPLES_LIVE:-0}"
FAILURES=0
TOTAL=0

pass() {
  TOTAL=$((TOTAL + 1))
  echo -e "${GREEN}PASS${NC} $1"
}

fail() {
  TOTAL=$((TOTAL + 1))
  FAILURES=$((FAILURES + 1))
  echo -e "${RED}FAIL${NC} $1"
}

skip() {
  echo -e "${YELLOW}SKIP${NC} $1 (requires a live UAR server; set VALIDATE_EXAMPLES_LIVE=1 to run)"
}

section() {
  echo -e "${BLUE}== $1 ==${NC}"
}

validate_rust() {
  section "Rust SDK examples (sdks/rust/examples/)"
  local dir="$PROJECT_ROOT/sdks/rust"
  local examples
  examples=$(find "$dir/examples" -maxdepth 1 -name '*.rs' -exec basename {} .rs \; | sort)

  if ! (cd "$dir" && cargo build --examples --locked --quiet); then
    for ex in $examples; do
      fail "rust/$ex (workspace build failed)"
    done
    return
  fi

  for ex in $examples; do
    if [[ "$ex" == "error_handling" ]]; then
      # Fully self-contained: targets an unreachable port, always
      # completes and exits 0.
      if (cd "$dir" && cargo run --example error_handling --locked --quiet >/dev/null); then
        pass "rust/$ex (compiled + ran)"
      else
        fail "rust/$ex (ran but exited non-zero)"
      fi
      continue
    fi

    if [[ "$LIVE" == "1" ]]; then
      if (cd "$dir" && timeout 30 cargo run --example "$ex" --locked --quiet >/dev/null); then
        pass "rust/$ex (compiled + ran against live server)"
      else
        fail "rust/$ex (ran against live server and failed)"
      fi
    else
      pass "rust/$ex (compiled)"
      skip "rust/$ex"
    fi
  done
}

validate_python() {
  section "Python SDK examples (sdks/python/examples/)"
  local dir="$PROJECT_ROOT/sdks/python"
  local py="${PYTHON:-python3}"

  for ex in "$dir"/examples/*.py; do
    local name
    name="$(basename "$ex")"
    if "$py" -m py_compile "$ex"; then
      pass "python/$name (compiled)"
    else
      fail "python/$name (failed to compile)"
      continue
    fi
    if [[ "$LIVE" == "1" ]]; then
      if (cd "$dir" && timeout 30 "$py" "examples/$name" >/dev/null); then
        pass "python/$name (ran against live server)"
      else
        fail "python/$name (ran against live server and failed)"
      fi
    else
      skip "python/$name"
    fi
  done
}

validate_typescript() {
  section "TypeScript SDK examples (sdks/typescript/examples/)"
  local dir="$PROJECT_ROOT/sdks/typescript"

  if [[ ! -d "$dir/node_modules" ]]; then
    echo "Installing sdks/typescript dependencies..."
    (cd "$dir" && npm install --no-audit --no-fund --silent)
  fi

  # tsconfig.examples.json covers every file under examples/, including the
  # next.js example - one typecheck run validates the whole set against
  # the current SDK surface.
  if (cd "$dir" && npm run --silent typecheck); then
    pass "typescript/examples (typechecked via tsconfig.examples.json)"
  else
    fail "typescript/examples (typecheck failed)"
  fi

  for ex in "$dir"/examples/*.ts; do
    [[ -e "$ex" ]] || continue
    local name
    name="$(basename "$ex")"
    if [[ "$LIVE" == "1" ]]; then
      if (cd "$dir" && timeout 30 npx tsx "examples/$name" >/dev/null); then
        pass "typescript/$name (ran against live server)"
      else
        fail "typescript/$name (ran against live server and failed)"
      fi
    else
      skip "typescript/$name"
    fi
  done
}

$RUN_RUST && validate_rust
$RUN_PYTHON && validate_python
$RUN_TS && validate_typescript

echo
if [[ "$FAILURES" -eq 0 ]]; then
  echo -e "${GREEN}All $TOTAL example checks passed.${NC}"
  exit 0
else
  echo -e "${RED}$FAILURES of $TOTAL example checks failed.${NC}"
  exit 1
fi
