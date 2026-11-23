#!/usr/bin/env bash
# PreToolUse:Bash — dependency advisory only.
# Canonical KBD/VLBV owns execution admission and phase acceptance. Do not
# infer a second permission boundary from a filename, environment toggle,
# command regex, or generated waypoint status. This hook grants no authority.

set -uo pipefail

input="$(cat 2>/dev/null || true)"

if command -v jq >/dev/null 2>&1; then
  cmd="$(printf '%s' "$input" | jq -r '.tool_input.command // ""' 2>/dev/null || true)"
else
  cmd="$(printf '%s' "$input" | sed -n 's/.*"command"[[:space:]]*:[[:space:]]*"\(.*\)".*/\1/p' | head -1)"
fi
[[ -n "$cmd" ]] || exit 0

root="${CLAUDE_PROJECT_DIR:-$PWD}"

# --- coupled-dependency guard -------------------------------------------------
# Two defects on 2026-08-08/09 had the same shape: half of a coupled pair was
# merged and the other half was not, and the break surfaced as type errors
# naming crates that appear nowhere in this repo.
#
#   wasmtime 47 beside wasmtime-wasi 46  -> two Linker<T> types, server-full
#                                           stopped compiling on main
#   @assistant-ui/react 0.15.4 beside
#   react-markdown 0.14.8                -> peer demanded ^0.15, useMessage gone
#
# Neither was caught by review, because "MERGEABLE" and "minor-patch" both read
# as safe. A manifest edit is cheap to inspect; do it here rather than trusting
# anyone to remember. Advisory by design: this warns on stderr and exits 0,
# because a false positive that blocks a legitimate bump is worse than a
# reminder that is occasionally unnecessary.
if printf '%s' "$cmd" | grep -Eq '(cargo (add|update|upgrade)|pnpm (add|update|up)|npm (install|update))'; then
  pairs=""
  for f in "$root/Cargo.toml" "$root/frontend/package.json"; do
    [[ -f "$f" ]] || continue
    wt="$(grep -oE '^wasmtime = \{ version = "[0-9]+' "$f" 2>/dev/null | grep -oE '[0-9]+$' || true)"
    ww="$(grep -oE '^wasmtime-wasi = \{ version = "[0-9]+' "$f" 2>/dev/null | grep -oE '[0-9]+$' || true)"
    if [[ -n "$wt" && -n "$ww" && "$wt" != "$ww" ]]; then
      pairs="${pairs}  wasmtime=$wt but wasmtime-wasi=$ww  ($f)\n"
    fi
  done
  if [[ -n "$pairs" ]]; then
    printf 'COUPLED DEPENDENCY SKEW — these crates share a major version:\n\n' >&2
    printf '%b\n' "$pairs" >&2
    printf 'wasmtime-wasi re-exports wasmtime types. A skew puts two distinct\n' >&2
    printf 'Linker<T> in one build and server-full stops compiling. Move both, or\n' >&2
    printf 'neither. Verified pins live in versions.toml.\n\n' >&2
  fi
fi

exit 0
