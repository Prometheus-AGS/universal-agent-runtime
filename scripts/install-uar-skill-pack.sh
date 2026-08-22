#!/usr/bin/env bash
set -euo pipefail

readonly PACK_REPOSITORY="https://github.com/Prometheus-AGS/prometheus-skill-system.git"
readonly PACK_COMMIT="c25561548aeb9ca656fdb942ab34378beedc2fe2"

prefix="${HOME}/.config/uar/skills"
source_dir=""

usage() {
    cat <<'EOF'
Install the UAR-pinned Prometheus Skill Pack from its public HTTPS repository.

Usage:
  scripts/install-uar-skill-pack.sh [--prefix <dir>] [--source-dir <dir>]

Options:
  --prefix <dir>      UAR skill-pack cache root.
                      Default: ~/.config/uar/skills
  --source-dir <dir>  Build from an existing checkout of the pinned commit.
                      Intended for offline installation and deterministic tests.
  -h, --help          Show this help.

The installer requires Git plus a Rust stable toolchain (`cargo` and `rustc`).
Install Rust with rustup from https://rustup.rs/ if either command is missing.
EOF
}

fail() {
    printf 'skill-pack install: %s\n' "$*" >&2
    exit 1
}

require_executable() {
    candidate="$1"
    label="$2"
    if [[ "$candidate" == */* ]]; then
        [[ -x "$candidate" ]] || fail "$label not found at $candidate"
    else
        command -v "$candidate" >/dev/null 2>&1 || fail "$label is required"
    fi
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --prefix)
            [[ $# -ge 2 ]] || fail "--prefix requires a directory"
            prefix="$2"
            shift 2
            ;;
        --source-dir)
            [[ $# -ge 2 ]] || fail "--source-dir requires a directory"
            source_dir="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            fail "unknown argument: $1"
            ;;
    esac
done

readonly cargo_bin="${CARGO:-cargo}"
readonly rustc_bin="${RUSTC:-rustc}"
require_executable git Git
require_executable "$cargo_bin" Cargo
require_executable "$rustc_bin" rustc

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/uar-skill-pack.XXXXXX")"
staging_root=""
cleanup() {
    rm -rf "$work_dir"
    if [[ -n "$staging_root" && -e "$staging_root" ]]; then
        rm -rf "$staging_root"
    fi
}
trap cleanup EXIT HUP INT TERM

if [[ -n "$source_dir" ]]; then
    source_dir="$(cd "$source_dir" && pwd)"
    [[ -f "$source_dir/.claude-plugin/plugin.json" ]] \
        || fail "source checkout is missing .claude-plugin/plugin.json"
    [[ -d "$source_dir/skills" ]] || fail "source checkout is missing skills/"
    [[ -z "$(git -C "$source_dir" status --porcelain)" ]] \
        || fail "source checkout must be clean"
else
    source_dir="$work_dir/source"
    git init --quiet "$source_dir"
    git -C "$source_dir" remote add origin "$PACK_REPOSITORY"
    git -C "$source_dir" fetch --quiet --depth 1 origin "$PACK_COMMIT"
    git -C "$source_dir" checkout --quiet --detach FETCH_HEAD
    git -C "$source_dir" submodule update --quiet --init --recursive --depth 1
fi

actual_commit="$(git -C "$source_dir" rev-parse HEAD)"
[[ "$actual_commit" == "$PACK_COMMIT" ]] \
    || fail "source commit $actual_commit does not match UAR pin $PACK_COMMIT"
if git -C "$source_dir" submodule status --recursive | grep -Eq '^[+-]'; then
    fail "source checkout has missing or mismatched submodules"
fi

manifest="$source_dir/.claude-plugin/plugin.json"
pack_name="$(awk -F'"' '/"name"[[:space:]]*:/ { print $4; exit }' "$manifest")"
pack_version="$(awk -F'"' '/"version"[[:space:]]*:/ { print $4; exit }' "$manifest")"
[[ "$pack_name" == "prometheus-skill-pack" ]] \
    || fail "unexpected plugin name in $manifest: $pack_name"
[[ -n "$pack_version" ]] || fail "plugin version is missing from $manifest"

cli_manifest="$source_dir/tools/prometheus-cli/Cargo.toml"
[[ -f "$cli_manifest" ]] || fail "canonical prometheus-cli manifest is missing"

printf 'Building prometheus-cli from verified commit %s...\n' "$PACK_COMMIT"
build_target="${CARGO_TARGET_DIR:-$work_dir/cargo-target}"
CARGO_TARGET_DIR="$build_target" \
    "$cargo_bin" build --locked --release --manifest-path "$cli_manifest" -p prometheus-cli

built_cli="$build_target/release/prometheus"
[[ -x "$built_cli" ]] || fail "prometheus-cli build did not produce $built_cli"

plugin_root="$prefix/prometheus-skill-pack"
install_root="$plugin_root/$pack_version"
staging_root="$prefix/.prometheus-skill-pack-${pack_version}.staging.$$"
previous_root="$plugin_root/.${pack_version}.previous.$$"

mkdir -p "$plugin_root" "$staging_root/.claude-plugin" "$staging_root/bin"
cp -R "$source_dir/skills" "$staging_root/skills"
cp "$manifest" "$staging_root/.claude-plugin/plugin.json"
if [[ -f "$source_dir/.mcp.json" ]]; then
    cp "$source_dir/.mcp.json" "$staging_root/.mcp.json"
fi
if [[ -f "$source_dir/scripts/skill-collision-allowlist.json" ]]; then
    mkdir -p "$staging_root/scripts"
    cp "$source_dir/scripts/skill-collision-allowlist.json" "$staging_root/scripts/"
fi
install -m 0755 "$built_cli" "$staging_root/bin/prometheus"
printf '%s\n' "$actual_commit" >"$staging_root/UAR_PACK_COMMIT"

if [[ -e "$install_root" ]]; then
    mv "$install_root" "$previous_root"
fi
if ! mv "$staging_root" "$install_root"; then
    if [[ -e "$previous_root" ]]; then
        mv "$previous_root" "$install_root"
    fi
    fail "could not activate $install_root"
fi
staging_root=""
if [[ -e "$previous_root" ]]; then
    rm -rf "$previous_root"
fi

skill_count="$(find "$install_root/skills" -name SKILL.md -type f | wc -l | tr -d ' ')"
printf 'Installed %s %s at %s\n' "$pack_name" "$pack_version" "$install_root"
printf 'Verified commit: %s\n' "$actual_commit"
printf 'Installed SKILL.md manifests: %s\n' "$skill_count"
printf 'Restart UAR; its installed-plugin precedence will select this version.\n'
