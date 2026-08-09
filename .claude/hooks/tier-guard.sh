#!/usr/bin/env bash
# PreToolUse:Bash — block Tier 3 commands outside a deliberate release gate.
#
# Contract: exit 0 allows, exit 2 blocks and feeds stderr back to the model.
# Any other exit is a hook error and does not block, so every failure path
# below exits 0 deliberately. A guard that blocks because jq is missing is
# worse than one that does not fire.
#
# Waypoint schema note (measured across the estate 2026-08-09): `.phase` holds
# a phase IDENTITY ("uar-uiux-full-migration-2026-08"), not a lifecycle state.
# `.status` holds the lifecycle ("running", "execute_ready", "completed").
# An earlier version matched .phase against milestone|release|certify, which no
# real waypoint could ever satisfy — it blocked Tier 3 unconditionally with no
# reachable unblock path. Read .status, and provide explicit opt-ins.

set -uo pipefail

input="$(cat 2>/dev/null || true)"

if command -v jq >/dev/null 2>&1; then
  cmd="$(printf '%s' "$input" | jq -r '.tool_input.command // ""' 2>/dev/null || true)"
else
  cmd="$(printf '%s' "$input" | sed -n 's/.*"command"[[:space:]]*:[[:space:]]*"\(.*\)".*/\1/p' | head -1)"
fi
[[ -n "$cmd" ]] || exit 0

root="${CLAUDE_PROJECT_DIR:-$PWD}"
waypoint="$root/.kbd-orchestrator/current-waypoint.json"

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

t3='cargo build.*--release|cargo test.*--release|tauri build|flutter build (ios|apk|appbundle)|go test.*-race|wasm-pack test --headless|playwright test'
printf '%s' "$cmd" | grep -Eq "$t3" || exit 0

# --- explicit opt-ins, checked first: both are deliberate and reversible ---
[[ "${PROMETHEUS_TIER3:-0}" == "1" ]] && exit 0
[[ -f "$root/.kbd-orchestrator/tier3.allow" ]] && exit 0

status=""; phase=""
if [[ -f "$waypoint" ]] && command -v jq >/dev/null 2>&1; then
  status="$(jq -r '.status // ""' "$waypoint" 2>/dev/null || true)"
  phase="$(jq -r '.phase // ""' "$waypoint" 2>/dev/null || true)"
fi

# Lifecycle states in which Tier 3 is the expected gate rather than a violation.
if printf '%s' "$status" | grep -qiE '^(completed|complete|release|releasing|certify|certified|milestone|delivery)'; then
  exit 0
fi

cat >&2 <<EOF
TIER VIOLATION — Tier 3 command outside a release gate.

  command: $cmd
  status:  ${status:-<unknown>}
  phase:   ${phase:-<unknown>}
  source:  $waypoint

Tier 3 runs at milestone, release, or certification. A release build during
implementation invalidates the incremental cache and pays full optimization
for code that is about to change.

Run the Tier 2 equivalent instead, or open the gate deliberately:

  PROMETHEUS_TIER3=1 <command>          # one command
  touch .kbd-orchestrator/tier3.allow   # this session; delete when done

Do not edit the waypoint to get past this — it is the position record, not a
permission switch.
EOF
exit 2
