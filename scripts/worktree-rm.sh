#!/usr/bin/env bash
# scripts/worktree-rm.sh — remove a worktree from ~/.claude/worktrees/.
#
# Usage:
#   scripts/worktree-rm.sh <name> [--force]
#
# Refuses if the resolved target is not a descendant of ~/.claude/worktrees/.

set -euo pipefail

die() { printf 'worktree-rm: %s\n' "$*" >&2; exit 1; }

name="${1:-}"
[[ -n "$name" ]] || die "usage: $0 <name> [--force]"
shift

force_flag=()
if [[ "${1:-}" == "--force" ]]; then
  force_flag=(--force)
  shift
fi
[[ $# -eq 0 ]] || die "unexpected extra args: $*"

case "$name" in
  */*|*..*|.|..) die "invalid name: $name" ;;
esac

[[ -n "${HOME:-}" ]] || die "HOME is not set"

root="$HOME/.claude/worktrees"
target="$root/$name"

[[ -d "$target" ]] || die "no such worktree: $target"

canon_root="$(cd "$root" && pwd -P)"
canon_target="$(cd "$target" && pwd -P)"

case "$canon_target" in
  "$canon_root"/*) : ;;
  *) die "refusing to remove path outside $canon_root: $canon_target" ;;
esac

git worktree remove "${force_flag[@]}" "$canon_target"
printf 'removed: %s\n' "$canon_target"
