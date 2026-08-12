#!/usr/bin/env bash
# PostToolUse:Edit|Write — warn when a second session writes the same tree.
#
# The tool has already run by the time this fires, so exit 2 cannot prevent
# the write. It surfaces the collision to the model loudly, which is the only
# useful action left at this point. Prevention belongs in worktrees.
#
# Lock is advisory and stale after 4 hours, so an abandoned session does not
# wedge the next one.

set -uo pipefail

root="${CLAUDE_PROJECT_DIR:-$PWD}"
lock="$root/.prometheus/.writer.lock"
me="${CLAUDE_SESSION_ID:-pid-$PPID}"
now="$(date +%s)"
stale_after=14400

# Only guard writes to the tree this hook protects.
#
# This check did not exist before 2026-08-12. Without it the hook fired on EVERY
# Write/Edit — including files in ~/.claude/plans/ that have nothing to do with
# this repo — so a whole planning session was spent reading a collision warning
# about a tree it was not touching. A guard that cries wolf on unrelated writes
# teaches people to ignore it, which is worse than having no guard.
target="$(
  cat 2>/dev/null | python3 -c 'import json,sys
try:
    d = json.load(sys.stdin)
    print(d.get("tool_input", {}).get("file_path", ""))
except Exception:
    print("")
' 2>/dev/null || true
)"
if [[ -n "$target" ]]; then
  case "$target" in
    "$root"/*) : ;;               # inside the guarded tree — check the lock
    *) exit 0 ;;                  # outside it — not this hook's business
  esac
fi

mkdir -p "$(dirname "$lock")" 2>/dev/null || exit 0

# A lock held by a dead process is not contention, it is litter.
#
# The 4-hour staleness window is a blunt proxy for "is anyone still there".
# When the holder encodes a PID we can answer that exactly, so a crashed session
# no longer blocks the next one for four hours. Reclaiming is logged, never
# silent: a lock that vanishes without explanation is its own confusion.
_holder_alive() { # <holder-token>
  local h="$1" pid
  case "$h" in
    pid-*) pid="${h#pid-}" ;;
    *) return 0 ;;                # non-PID holder (session id) — cannot probe
  esac
  [[ "$pid" =~ ^[0-9]+$ ]] || return 0
  kill -0 "$pid" 2>/dev/null
}

if [[ -f "$lock" ]]; then
  holder="$(head -1 "$lock" 2>/dev/null || true)"
  since="$(sed -n '2p' "$lock" 2>/dev/null || echo 0)"
  [[ "$since" =~ ^[0-9]+$ ]] || since=0
  age=$(( now - since ))

  if [[ -n "$holder" && "$holder" != "$me" ]] && ! _holder_alive "$holder"; then
    printf 'single-writer: reclaiming lock from dead holder %s (held %ss)\n' "$holder" "$age" >&2
    holder=""
  fi

  if [[ -n "$holder" && "$holder" != "$me" && "$age" -lt "$stale_after" ]]; then
    cat >&2 <<EOF
SINGLE-WRITER COLLISION — another session holds the writer lock on this tree.

  holder:  $holder
  held:    ${age}s ago
  lock:    $lock

Two agents writing one tree produce interleaved edits and a build that
belongs to neither. Stop and move to a git worktree with its own build
directory, or confirm the other session has ended and remove the lock.
EOF
    exit 2
  fi
fi

printf '%s\n%s\n' "$me" "$now" > "$lock" 2>/dev/null || true
exit 0
