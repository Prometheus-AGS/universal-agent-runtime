#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"

dockerfile="${1:-${repo_root}/Dockerfile}"
toolchain_file="${2:-${repo_root}/rust-toolchain.toml}"

fail() {
  echo "runtime-image toolchain pin check failed: $*" >&2
  exit 1
}

[[ -r "${dockerfile}" ]] || fail "cannot read Dockerfile: ${dockerfile}"
[[ -r "${toolchain_file}" ]] || fail "cannot read toolchain file: ${toolchain_file}"

docker_toolchain="$({
  sed -nE \
    's/^[[:space:]]*ARG[[:space:]]+RUST_TOOLCHAIN=([^[:space:]#]+).*$/\1/p' \
    "${dockerfile}"
} | sed -n '1p')"

repository_toolchain="$({
  sed -nE \
    's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"([^"]+)".*$/\1/p' \
    "${toolchain_file}"
} | sed -n '1p')"

if (( $# >= 3 )); then
  effective_toolchain="$3"
else
  effective_toolchain="${docker_toolchain}"
fi

[[ -n "${docker_toolchain}" ]] || fail "Dockerfile has no default RUST_TOOLCHAIN value"
[[ -n "${repository_toolchain}" ]] || fail "toolchain file has no channel value"
[[ -n "${effective_toolchain}" ]] || fail "effective RUST_TOOLCHAIN value is empty"

[[ "${docker_toolchain}" == "${repository_toolchain}" ]] || fail \
  "Dockerfile RUST_TOOLCHAIN=${docker_toolchain} does not match repository channel=${repository_toolchain}"

[[ "${effective_toolchain}" == "${repository_toolchain}" ]] || fail \
  "effective RUST_TOOLCHAIN=${effective_toolchain} does not match repository channel=${repository_toolchain}"

if grep -Eq 'cargo[[:space:]]+\+nightly([[:space:]]|$)' "${dockerfile}"; then
  fail "backend build selects the unqualified moving nightly channel"
fi

grep -Fq 'cargo +"${RUST_TOOLCHAIN}" build' "${dockerfile}" || fail \
  'backend build does not select cargo +"${RUST_TOOLCHAIN}"'

echo "runtime-image toolchain pin consistent: docker=${docker_toolchain} repository=${repository_toolchain} effective=${effective_toolchain}"
