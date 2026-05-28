#!/usr/bin/env bash
# scripts/worktree-list.sh — list this repo's worktrees that live under
# ~/.claude/worktrees/. Anything outside that root is hidden so the output
# stays focused on the supported convention.

set -euo pipefail

[[ -n "${HOME:-}" ]] || { printf 'worktree-list: HOME is not set\n' >&2; exit 1; }
root="$HOME/.claude/worktrees"

# git worktree list --porcelain produces blocks of:
#   worktree <path>
#   HEAD <sha>
#   branch <ref> (or "detached" / "bare")
#
# We surface only worktrees whose path starts with $root and pretty-print
# them as <path>\t<branch>\t<short-sha>.

git worktree list --porcelain | awk -v root="$root" '
  /^worktree / { path = substr($0, 10); next }
  /^HEAD /     { head = substr($0, 6); next }
  /^branch /   { branch = substr($0, 8); next }
  /^bare/      { branch = "(bare)"; next }
  /^detached/  { branch = "(detached)"; next }
  /^$/ {
    if (path != "" && index(path, root) == 1) {
      short = substr(head, 1, 8)
      printf "%s\t%s\t%s\n", path, branch, short
    }
    path = ""; head = ""; branch = ""
  }
  END {
    if (path != "" && index(path, root) == 1) {
      short = substr(head, 1, 8)
      printf "%s\t%s\t%s\n", path, branch, short
    }
  }
'
