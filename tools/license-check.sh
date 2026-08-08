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
  check "root Cargo.toml license == MIT (found: ${root_license:-<missing>})" \
    "$([[ "$root_license" == "MIT" ]] && echo 0 || echo 1)"
fi
check "root LICENSE (MIT) exists" "$([[ -f LICENSE ]] && echo 0 || echo 1)"
check "root LICENSE-CC-BY-4.0.md exists" "$([[ -f LICENSE-CC-BY-4.0.md ]] && echo 0 || echo 1)"

# The runtime relicensed AGPL-3.0-only -> MIT (2026-08-07). These files existed
# only to support the copyleft + commercial-exception model and must NOT return:
# a stray LICENSE-COMMERCIAL.md would advertise terms that no longer apply.
check "root LICENSE-COMMERCIAL.md absent" "$([[ ! -f LICENSE-COMMERCIAL.md ]] && echo 0 || echo 1)"
check "root LICENSE contains MIT text" \
  "$(grep -q '^MIT License' LICENSE 2>/dev/null && echo 0 || echo 1)"
check "root LICENSE is not AGPL" \
  "$(grep -qi 'AFFERO' LICENSE 2>/dev/null && echo 1 || echo 0)"

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

# --- Rust SDK: MIT ---
# Was "MIT OR AGPL-3.0". The dual form existed only because the runtime was
# AGPL; with an MIT runtime there is nothing for a consumer to choose between.
rust_sdk_toml="sdks/rust/Cargo.toml"
if [[ -f "$rust_sdk_toml" ]]; then
  rust_sdk_license=$(grep -m1 '^license = ' "$rust_sdk_toml" | sed -E 's/^license = "(.*)"/\1/')
  check "sdks/rust/Cargo.toml license == MIT (found: ${rust_sdk_license:-<missing>})" \
    "$([[ "$rust_sdk_license" == "MIT" ]] && echo 0 || echo 1)"
else
  check "sdks/rust/Cargo.toml exists" 1
fi
check "sdks/rust/LICENSE-MIT exists" "$([[ -f sdks/rust/LICENSE-MIT ]] && echo 0 || echo 1)"
check "sdks/rust/LICENSE-AGPL absent" "$([[ ! -f sdks/rust/LICENSE-AGPL ]] && echo 0 || echo 1)"

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

# --- Whole workspace: no crate may declare a non-MIT license ---
# This check exists because an earlier version of this script passed while
# tools/uar-jwt-proxy and tools/mcp-server-fetch were still AGPL-3.0-only: it
# only inspected the root manifest and the SDKs, so it validated exactly what
# had already been fixed and missed everything else. Ask cargo, not a file list.
if command -v cargo >/dev/null 2>&1 && command -v python3 >/dev/null 2>&1; then
  non_mit=$(cargo metadata --no-deps --format-version 1 \
    | python3 -c 'import sys, json
d = json.load(sys.stdin)
bad = [p["name"] + "=" + str(p.get("license")) for p in d["packages"] if p.get("license") != "MIT"]
print(" ".join(bad))')
  check "all workspace crates declare MIT (offenders: ${non_mit:-none})" \
    "$([[ -z "$non_mit" ]] && echo 0 || echo 1)"
else
  echo "SKIP: workspace-wide license check (cargo or python3 unavailable)"
fi

if [[ "$fail" -ne 0 ]]; then
  echo "" >&2
  echo "license-check.sh: one or more license checks failed." >&2
  exit 1
fi

echo ""
echo "license-check.sh: all checks passed."
