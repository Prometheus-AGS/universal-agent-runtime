#!/usr/bin/env bash
# Verifies that every crate/package manifest declares a license field and
# that it matches the expected value for its component, and that the
# corresponding LICENSE file(s) exist. See
# openspec/changes/license-dual-license-agpl-mit/proposal.md.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

fail=0

check() {
  local description="$1"
  local condition="$2"
  if [[ "$condition" != "0" ]]; then
    echo "FAIL: $description" >&2
    fail=1
  else
    echo "OK:   $description"
  fi
}

# --- Root runtime crate: AGPL-3.0-only ---
if [[ ! -f Cargo.toml ]]; then
  echo "FAIL: root Cargo.toml missing" >&2
  fail=1
else
  root_license=$(grep -m1 '^license = ' Cargo.toml | sed -E 's/^license = "(.*)"/\1/')
  check "root Cargo.toml license == AGPL-3.0-only (found: ${root_license:-<missing>})" \
    "$([[ "$root_license" == "AGPL-3.0-only" ]] && echo 0 || echo 1)"
fi
check "root LICENSE (AGPL-3.0) exists" "$([[ -f LICENSE ]] && echo 0 || echo 1)"
check "root LICENSE-COMMERCIAL.md exists" "$([[ -f LICENSE-COMMERCIAL.md ]] && echo 0 || echo 1)"
check "root LICENSE-CC-BY-4.0.md exists" "$([[ -f LICENSE-CC-BY-4.0.md ]] && echo 0 || echo 1)"

# --- Python SDK: MIT ---
py_toml="sdks/python/pyproject.toml"
if [[ -f "$py_toml" ]]; then
  if grep -q 'license = { text = "MIT" }' "$py_toml" || grep -q 'license = "MIT"' "$py_toml"; then
    check "sdks/python/pyproject.toml declares MIT" 0
  else
    check "sdks/python/pyproject.toml declares MIT" 1
  fi
else
  check "sdks/python/pyproject.toml exists" 1
fi
check "sdks/python/LICENSE exists" "$([[ -f sdks/python/LICENSE ]] && echo 0 || echo 1)"

# --- Rust SDK: MIT OR AGPL-3.0 ---
rust_sdk_toml="sdks/rust/Cargo.toml"
if [[ -f "$rust_sdk_toml" ]]; then
  rust_sdk_license=$(grep -m1 '^license = ' "$rust_sdk_toml" | sed -E 's/^license = "(.*)"/\1/')
  check "sdks/rust/Cargo.toml license == MIT OR AGPL-3.0 (found: ${rust_sdk_license:-<missing>})" \
    "$([[ "$rust_sdk_license" == "MIT OR AGPL-3.0" ]] && echo 0 || echo 1)"
else
  check "sdks/rust/Cargo.toml exists" 1
fi
check "sdks/rust/LICENSE-MIT exists" "$([[ -f sdks/rust/LICENSE-MIT ]] && echo 0 || echo 1)"
check "sdks/rust/LICENSE-AGPL exists" "$([[ -f sdks/rust/LICENSE-AGPL ]] && echo 0 || echo 1)"

# --- TypeScript SDK: MIT ---
ts_pkg="sdks/typescript/package.json"
if [[ -f "$ts_pkg" ]]; then
  ts_license=$(node -e "console.log(require('./$ts_pkg').license || '')" 2>/dev/null || echo "")
  check "sdks/typescript/package.json license == MIT (found: ${ts_license:-<missing>})" \
    "$([[ "$ts_license" == "MIT" ]] && echo 0 || echo 1)"
else
  check "sdks/typescript/package.json exists" 1
fi
check "sdks/typescript/LICENSE exists" "$([[ -f sdks/typescript/LICENSE ]] && echo 0 || echo 1)"

if [[ "$fail" -ne 0 ]]; then
  echo "" >&2
  echo "license-check.sh: one or more license checks failed." >&2
  exit 1
fi

echo ""
echo "license-check.sh: all checks passed."
