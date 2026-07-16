#!/usr/bin/env bash
# Builds the MCP server sidecar binaries that src-tauri/tauri.conf.json's
# bundle.externalBin expects at src-tauri/binaries/<name>-<target-triple>.
#
# Not committed to git (see src-tauri/.gitignore) -- run this once per
# platform before `cargo tauri build`/`cargo tauri dev`, or wire it into CI
# for each supported target. Linux and macOS are Stable; Windows is
# Experimental (CLAUDE.md).
#
#   mcp-server-filesystem <- vendor/git/rust-mcp-filesystem (GQAdonis fork,
#                             vendored as a git submodule; upstream project
#                             is rust-mcp-stack/rust-mcp-filesystem)
#   mcp-server-fetch      <- tools/mcp-server-fetch (this repo's own Rust
#                             MCP server, no Python runtime dependency)
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bin_dir="$repo_root/src-tauri/binaries"
mkdir -p "$bin_dir"

target_triple="${TAURI_ENV_TARGET_TRIPLE:-$(rustc -vV | sed -n 's/^host: //p')}"
ext=""
if [[ "$target_triple" == *windows* ]]; then
  ext=".exe"
fi

echo "▶ Building mcp-server-filesystem sidecar for $target_triple…"
(cd "$repo_root/vendor/git/rust-mcp-filesystem" && cargo build --release --bin rust-mcp-filesystem)
# NOTE: the repo's root .cargo/config.toml sets a relative target-dir, which
# cargo resolves relative to that config file's directory (the repo root),
# not the crate's own directory -- so the vendored crate's build output lands
# in the repo-root target/, not vendor/git/rust-mcp-filesystem/target/.
cp "$repo_root/target/release/rust-mcp-filesystem$ext" \
  "$bin_dir/mcp-server-filesystem-$target_triple$ext"
echo "✔ $bin_dir/mcp-server-filesystem-$target_triple$ext"

echo "▶ Building mcp-server-fetch sidecar for $target_triple…"
(cd "$repo_root" && cargo build --release -p mcp-server-fetch)
cp "$repo_root/target/release/mcp-server-fetch$ext" \
  "$bin_dir/mcp-server-fetch-$target_triple$ext"
echo "✔ $bin_dir/mcp-server-fetch-$target_triple$ext"

echo "✅ Sidecar binaries ready in $bin_dir"
