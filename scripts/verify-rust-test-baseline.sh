#!/usr/bin/env bash
set -euo pipefail

# #462: never certify a library harness compiled with temporary source patches.
die() { printf 'rust-test-baseline: %s\n' "$*" >&2; exit 1; }

mode=working-tree
case "${1:-}" in
  '') ;;
  --acceptance) mode=acceptance; shift ;;
  *) die "usage: bash scripts/verify-rust-test-baseline.sh [--acceptance]" ;;
esac
[[ $# -eq 0 ]] || die "unexpected arguments: $*"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/.." && pwd)"
cd "$repo_root"

# These reads must propagate errors, including inside command substitutions.
source_identity() {
  local state submodules line
  state="$(git status --porcelain --untracked-files=all --ignore-submodules=none)" || return
  [[ -z "$state" ]] || { printf 'acceptance requires a clean committed checkout\n' >&2; return 1; }
  submodules="$(git submodule status --recursive)" || return
  while IFS= read -r line; do
    case "$line" in
      [-+U]*) printf 'acceptance requires initialized recursive submodules at committed pins\n' >&2; return 1 ;;
    esac
  done <<< "$submodules"
  git submodule foreach --quiet --recursive '
    state=$(git status --porcelain --untracked-files=all --ignore-submodules=none) || exit
    test -z "$state" || { echo "dirty submodule: $displaypath" >&2; exit 1; }
  ' || return
  printf 'commit='
  git rev-parse --verify HEAD || return
  printf 'tree='
  git rev-parse --verify 'HEAD^{tree}' || return
  printf 'recursive_submodules:\n%s\n' "$submodules"
}

toolchain_identity() {
  cargo --version || return
  rustc --version --verbose || return
}

if [[ "$mode" == acceptance ]]; then
  initial_source="$(source_identity)" || die "source identity refused"
else
  printf '%s\n' 'WORKING_TREE_ONLY: compiles unstaged changes too; does not certify the staged commit.'
fi

# One invocation owns all three feature sets. Cargo also retains its own target
# directory locks; unrelated Cargo writers still require operator scheduling.
target_dir="${CARGO_TARGET_DIR:-target}"
mkdir -p "$target_dir"
target_dir="$(cd -- "$target_dir" && pwd -P)"
lock_dir="$target_dir/.uar-rust-test-baseline.lock"
mkdir "$lock_dir" || die "baseline already owns $lock_dir (remove only after confirming its owner has stopped)"
compile_log=''
cleanup() {
  if [[ -n "$compile_log" ]]; then rm -f -- "$compile_log"; fi
  rmdir -- "$lock_dir"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
compile_log="$(mktemp "${TMPDIR:-/tmp}/uar-rust-test-baseline.XXXXXX")"

initial_toolchain="$(toolchain_identity)" || die "toolchain identity unavailable"
printf 'mode=%s\nrepository=%s\ntarget_directory=%s\ncargo_profile=test\n' "$mode" "$repo_root" "$target_dir"
printf 'toolchain:\n%s\n' "$initial_toolchain"
if [[ "$mode" == acceptance ]]; then
  printf 'source_before:\n%s\n' "$initial_source"
fi

for features in minimal server-full server-full,postgres-backend; do
  printf 'command: CARGO_TERM_COLOR=never cargo test --locked --no-default-features --features %s --lib --no-run\n' "$features"
  CARGO_TERM_COLOR=never cargo test --locked --no-default-features --features "$features" --lib --no-run 2>&1 | tee "$compile_log"
  # Repository policy requires warning-free compiles, including dependencies.
  # Keep the full diagnostics visible; do not weaken lints or inject RUSTFLAGS.
  if LC_ALL=C grep -E '^warning(\[[^]]+\])?:' "$compile_log"; then
    die "warnings emitted for $features; compile is not accepted"
  else
    scan_status=$?
    [[ "$scan_status" -eq 1 ]] || die "warning diagnostic scan failed"
  fi
done

if [[ "$mode" == acceptance ]]; then
  final_source="$(source_identity)" || die "source identity changed or became dirty"
  [[ "$initial_source" == "$final_source" ]] || die "source identity changed during compilation"
  final_toolchain="$(toolchain_identity)" || die "final toolchain identity unavailable"
  [[ "$initial_toolchain" == "$final_toolchain" ]] || die "toolchain identity changed during compilation"
  printf 'source_after:\n%s\n' "$final_source"
  printf '%s\n' 'RUST_TEST_BASELINE_ACCEPTANCE_PASS: all three library harnesses compiled on unchanged committed source.'
else
  printf '%s\n' 'RUST_TEST_BASELINE_WORKING_TREE_PASS: all three library harnesses compiled; staged source is not certified.'
fi
