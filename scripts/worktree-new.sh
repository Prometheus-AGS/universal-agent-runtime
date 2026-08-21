#!/usr/bin/env bash
# scripts/worktree-new.sh — create a git worktree under ~/.claude/worktrees/.
#
# Usage:
#   scripts/worktree-new.sh <name> [--base <ref>]
#
# Refuses if:
#   * $HOME is unset.
#   * <name> already exists at ~/.claude/worktrees/<name>.
#   * the resolved target would land inside the repo working tree.
#
# Seeds .claude/settings.local.json into the new worktree when the current
# checkout has one and the destination does not track that path. A tracked
# candidate copy is never overwritten, so immutable worktrees remain clean.

set -euo pipefail

die() { printf 'worktree-new: %s\n' "$*" >&2; exit 1; }

name="${1:-}"
[[ -n "$name" ]] || die "usage: $0 <name> [--base <ref>]"
shift

base="HEAD"
if [[ "${1:-}" == "--base" ]]; then
  base="${2:-}"
  [[ -n "$base" ]] || die "--base requires a ref"
  shift 2
fi
[[ $# -eq 0 ]] || die "unexpected extra args: $*"

case "$name" in
  */*|*..*|.) die "invalid name: $name (no slashes, no parent traversal)" ;;
esac

[[ -n "${HOME:-}" ]] || die "HOME is not set"

root="$HOME/.claude/worktrees"
target="$root/$name"

[[ -e "$target" ]] && die "target already exists: $target (try scripts/worktree-list.sh)"

repo_root="$(git rev-parse --show-toplevel 2>/dev/null)" || die "not inside a git repo"

# Canonicalize the parent of the target (parent must exist or be creatable)
mkdir -p "$root"
canon_root="$(cd "$root" && pwd -P)"
canon_target="$canon_root/$name"
canon_repo="$(cd "$repo_root" && pwd -P)"

case "$canon_target" in
  "$canon_repo"|"$canon_repo"/*)
    die "refusing to create worktree inside the repo tree: $canon_target"
    ;;
esac

git worktree add "$canon_target" "$base"

src_settings="$canon_repo/.claude/settings.local.json"
if [[ -f "$src_settings" ]]; then
  if git -C "$canon_target" ls-files --error-unmatch .claude/settings.local.json >/dev/null 2>&1; then
    printf 'retained tracked candidate settings: %s\n' "$canon_target/.claude/settings.local.json"
  else
    mkdir -p "$canon_target/.claude"
    cp "$src_settings" "$canon_target/.claude/settings.local.json"
  fi
fi

printf 'created: %s\n' "$canon_target"
