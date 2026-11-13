#!/bin/sh
set -eu

case "$1" in
    amd64)
        arch=x86_64
        tools_sha=21c036c1740a0552124c79c6e2170cb61c5b016005671a6d5bc06cf642165431
        wit_sha=bf4efe883ab70d34a4740253911dda20caf895d58a003af58369506b04684cbe
        bindgen_sha=9687f3bb551b9f66f37ed32a40d7913a00f7962db653054e56857bc306c3227f
        ;;
    arm64)
        arch=aarch64
        tools_sha=773b5e6bf957f0d93a9a337c9c5fc4c35509706301a80799f57a76fa367fd1ca
        wit_sha=1f9b4fb9f3af15374e2a2d026f6a1fcd8b48f37655f59f48a32cb421e169cbe4
        bindgen_sha=99ffc93ce80081b91632e2b539b782821a8f3439ab0fddc645287462b2f55d3d
        ;;
    *) printf 'Unsupported WASM toolchain architecture: %s\n' "$1" >&2; exit 1 ;;
esac

workdir=$(mktemp -d)
trap 'rm -rf "$workdir"' EXIT
cd "$workdir"

install_release() {
    url=$1
    sha=$2
    shift 2
    curl --proto '=https' -fsSLo release.tar.gz "$url"
    printf '%s  release.tar.gz\n' "$sha" | sha256sum -c -
    mkdir release
    tar -xzf release.tar.gz --strip-components=1 -C release
    for binary do
        install -m 0755 "release/$binary" "$CARGO_HOME/bin/$binary"
    done
    rm -r release release.tar.gz
}

install_release "https://github.com/bytecodealliance/wasm-tools/releases/download/v1.263.0/wasm-tools-1.263.0-${arch}-linux.tar.gz" \
    "$tools_sha" wasm-tools
install_release "https://github.com/bytecodealliance/wit-bindgen/releases/download/v0.63.0/wit-bindgen-0.63.0-${arch}-linux.tar.gz" \
    "$wit_sha" wit-bindgen
install_release "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/0.2.130/wasm-bindgen-0.2.130-${arch}-unknown-linux-musl.tar.gz" \
    "$bindgen_sha" wasm-bindgen wasm-bindgen-test-runner wasm2es6js

# The published cargo-component lock contains yanked dependencies.
cargo install --version '=0.21.1' cargo-component
cargo install --locked --version '=0.8.0' twiggy

if [ "$arch" = x86_64 ]; then
    # Install the executable that the cargo-wasi wrapper embeds on amd64.
    crate=cargo-wasi-exe-x86_64-unknown-linux-musl
    curl --proto '=https' -fsSLo cargo-wasi.crate \
        "https://static.crates.io/crates/$crate/$crate-0.1.28.crate"
    printf '%s\n' '3102f1edcd441f732b249f81c018b882f154a65953aaf90c153881884a88e5fd  cargo-wasi.crate' | sha256sum -c -
    tar -xOzf cargo-wasi.crate "$crate-0.1.28/src/binary" > "$CARGO_HOME/bin/cargo-wasi"
    chmod 0755 "$CARGO_HOME/bin/cargo-wasi"
else
    # Resolve a current rustix for the published source fallback on this nightly.
    cargo install --version '=0.1.28' cargo-wasi
fi
