#!/usr/bin/env bash
# Validate the UAR cookbook examples.
#
# Compiles every example and runs the ones that are fully self-contained.
# SDK examples that talk to a live UAR server are typechecked/compiled and
# skipped at runtime. A2UI examples are intentionally skipped until Changes
# 21–22 land.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
COOKBOOK="$PROJECT_ROOT/docs/cookbook"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

LIVE="${VALIDATE_COOKBOOK_LIVE:-0}"
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
    echo -e "${YELLOW}SKIP${NC} $1"
}

section() {
    echo -e "${BLUE}== $1 ==${NC}"
}

# Runtime examples are built as examples of the root crate.
validate_runtime() {
    section "Runtime examples (docs/cookbook/runtime/)"
    local bins=("cookbook_01_start_server" "cookbook_02_load_config" "cookbook_03_mcp_tool_call" "cookbook_04_streaming_sse")

    if ! (cd "$PROJECT_ROOT" && cargo build --examples --locked --quiet); then
        for bin in "${bins[@]}"; do
            fail "runtime/$bin (build failed)"
        done
        return
    fi

    for bin in "${bins[@]}"; do
        if (cd "$PROJECT_ROOT" && timeout 60 cargo run --example "$bin" --locked --quiet >/dev/null); then
            pass "runtime/${bin#cookbook_} (built + ran)"
        else
            fail "runtime/${bin#cookbook_} (built but failed to run)"
        fi
    done
}

# Rust SDK examples under docs/cookbook/sdk/rust/
validate_sdk_rust() {
    section "Rust SDK examples (docs/cookbook/sdk/rust/)"
    local dir="$COOKBOOK/sdk/rust"
    local bins=("01_init" "04_subscribe")

    if ! (cd "$dir" && cargo build --locked --quiet); then
        for bin in "${bins[@]}"; do
            fail "sdk/rust/$bin (build failed)"
        done
        return
    fi

    for bin in "${bins[@]}"; do
        if [[ "$bin" == "01_init" ]]; then
            # Fully self-contained: only constructs a client.
            if (cd "$dir" && timeout 30 cargo run --bin "$bin" --locked --quiet >/dev/null); then
                pass "sdk/rust/$bin (built + ran)"
            else
                fail "sdk/rust/$bin (built but failed to run)"
            fi
        elif [[ "$LIVE" == "1" ]]; then
            if (cd "$dir" && timeout 30 cargo run --bin "$bin" --locked --quiet >/dev/null); then
                pass "sdk/rust/$bin (built + ran against live server)"
            else
                fail "sdk/rust/$bin (ran against live server and failed)"
            fi
        else
            pass "sdk/rust/$bin (built)"
            skip "sdk/rust/$bin (requires a live UAR server; set VALIDATE_COOKBOOK_LIVE=1)"
        fi
    done
}

# Python SDK examples under docs/cookbook/sdk/python/
validate_sdk_python() {
    section "Python SDK examples (docs/cookbook/sdk/python/)"
    local dir="$COOKBOOK/sdk/python"
    local py="${PYTHON:-python3}"
    local py_path="$PROJECT_ROOT/sdks/python/src"

    for ex in "$dir"/examples/*.py; do
        [[ -e "$ex" ]] || continue
        local name
        name="$(basename "$ex")"
        if PYTHONPATH="$py_path" "$py" -m py_compile "$ex"; then
            pass "sdk/python/$name (compiled)"
        else
            fail "sdk/python/$name (failed to compile)"
            continue
        fi

        if [[ "$LIVE" == "1" ]]; then
            if (cd "$dir" && PYTHONPATH="$py_path" timeout 30 "$py" "examples/$name" >/dev/null); then
                pass "sdk/python/$name (ran against live server)"
            else
                fail "sdk/python/$name (ran against live server and failed)"
            fi
        else
            skip "sdk/python/$name (requires a live UAR server; set VALIDATE_COOKBOOK_LIVE=1)"
        fi
    done
}

# TypeScript SDK examples under docs/cookbook/sdk/typescript/
validate_sdk_typescript() {
    section "TypeScript SDK examples (docs/cookbook/sdk/typescript/)"
    local dir="$COOKBOOK/sdk/typescript"

    if [[ ! -d "$PROJECT_ROOT/sdks/typescript/node_modules" ]]; then
        echo "Installing sdks/typescript dependencies..."
        (cd "$PROJECT_ROOT/sdks/typescript" && npm install --no-audit --no-fund --silent)
    fi

    # Typecheck against the SDK source using the SDK's installed TypeScript.
    if (cd "$PROJECT_ROOT/sdks/typescript" && npx tsc -p "$dir/tsconfig.json"); then
        pass "sdk/typescript/examples (typechecked)"
    else
        fail "sdk/typescript/examples (typecheck failed)"
        return
    fi

    for ex in "$dir"/examples/*.ts; do
        [[ -e "$ex" ]] || continue
        local name
        name="$(basename "$ex")"
        if [[ "$LIVE" == "1" ]]; then
            if (cd "$dir" && timeout 30 npx -y tsx "examples/$name" >/dev/null); then
                pass "sdk/typescript/$name (ran against live server)"
            else
                fail "sdk/typescript/$name (ran against live server and failed)"
            fi
        else
            skip "sdk/typescript/$name (requires a live UAR server; set VALIDATE_COOKBOOK_LIVE=1)"
        fi
    done
}

validate_a2ui() {
    section "A2UI examples (docs/cookbook/a2ui/)"
    skip "A2UI examples are placeholders until Changes 21–22 land"
}

validate_runtime
validate_sdk_rust
validate_sdk_python
validate_sdk_typescript
validate_a2ui

echo
if [[ "$FAILURES" -eq 0 ]]; then
    echo -e "${GREEN}All $TOTAL cookbook checks passed.${NC}"
    exit 0
else
    echo -e "${RED}$FAILURES of $TOTAL cookbook checks failed.${NC}"
    exit 1
fi
