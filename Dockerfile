# syntax=docker/dockerfile:1.7
# =============================================================================
# Universal Agent Runtime — polyglot multi-stage Dockerfile
#
# The runtime image doubles as a build host so skills/plugins authored in any
# language UAR supports can be compiled in-container. Five toolchains are
# present in both the `toolchain` and `runtime` stages:
#
#   - Rust nightly  (+ wasm32-wasip2, wasm32-wasip1, wasm32-unknown-unknown)
#   - Node.js LTS >= 24  with npm + pnpm + bun
#   - Python 3.13  with uv + maturin + pyo3-build-config
#   - Go (latest)
#   - wasmtime (CLI)  for AOT compilation of WASM components via Cranelift
#
# Build with:
#   docker buildx build --platform linux/amd64 -t uar:latest .
# Runtime image size is intentionally large (~3GB) - see README for a future
# "runtime-slim" variant that strips build tools.
# =============================================================================

# Toolchain pins — bump via a new KBD change so drift is auditable.
# Rust nightly is dated (matches rust-toolchain.toml at repo root).
# Go follows the latest stable per go.dev/doc/devel/release.
ARG RUST_TOOLCHAIN=nightly-2026-05-01
ARG NODE_MAJOR=24
ARG PYTHON_VERSION=3.13
ARG GO_VERSION=1.26.3
ARG WASMTIME_VERSION=27.0.0
# TinyGo is the only Go path to WASI P2 / Component Model. Pinned alongside Go.
ARG TINYGO_VERSION=0.34.0

# -----------------------------------------------------------------------------
# Stage 1: toolchain - five language toolchains. Cache-friendly; rarely changes.
# -----------------------------------------------------------------------------
FROM ubuntu:24.04 AS toolchain

ARG RUST_TOOLCHAIN
ARG NODE_MAJOR
ARG PYTHON_VERSION
ARG GO_VERSION
ARG WASMTIME_VERSION
ARG TINYGO_VERSION
# TARGETARCH is set automatically by buildx (`amd64` or `arm64`); see
# multi-arch branches further down in the Go / wasmtime / tinygo installs.
ARG TARGETARCH=amd64

ENV DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8 \
    CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/cargo/bin:/usr/local/go/bin:/root/.local/bin:/root/.bun/bin:$PATH

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates curl git build-essential pkg-config \
        libssl-dev cmake unzip xz-utils \
        software-properties-common gnupg lsb-release \
        protobuf-compiler libprotobuf-dev \
        # WASM low-level tooling from apt (wasm-opt via binaryen,
        # wat2wasm/wasm2wat/wasm-objdump via wabt). Adding here keeps
        # the cargo install step focused on Rust-source tools only.
        binaryen wabt \
        # Python build deps for componentize-py / pyo3 native extensions.
        libffi-dev patchelf \
    && rm -rf /var/lib/apt/lists/*

# Python 3.13 from deadsnakes
RUN add-apt-repository -y ppa:deadsnakes/ppa \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        python${PYTHON_VERSION} python${PYTHON_VERSION}-dev python${PYTHON_VERSION}-venv \
    && rm -rf /var/lib/apt/lists/* \
    && ln -sf /usr/bin/python${PYTHON_VERSION} /usr/local/bin/python3 \
    && ln -sf /usr/bin/python${PYTHON_VERSION} /usr/local/bin/python

# uv (Astral) and Python helpers + componentize-py for Python→Component path.
# NOTE: `pyo3-build-config` is a RUST crate (crates.io), not a PyPI package —
# `uv pip install pyo3-build-config` always fails with "not found in the package
# registry". `maturin` is the actual pyo3/Rust↔Python build tool and is sufficient.
RUN curl -LsSf https://astral.sh/uv/install.sh | sh \
    && mv /root/.local/bin/uv /usr/local/bin/uv \
    && uv pip install --system --no-cache-dir \
        maturin \
        componentize-py \
        wasmtime

# Rust nightly + WASM targets + cargo-installed WASM ecosystem tools.
# `cargo-binstall` is installed first so subsequent `cargo binstall` calls
# pull prebuilt binaries instead of compiling from source where possible.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain ${RUST_TOOLCHAIN} --profile minimal \
                    --component rustfmt,clippy,rust-src \
    && rustup target add wasm32-wasip2 wasm32-wasip1 wasm32-unknown-unknown \
    && cargo install --locked cargo-binstall \
    && cargo binstall --no-confirm --locked \
        cargo-component \
        wasm-tools \
        wit-bindgen-cli \
        wasm-bindgen-cli \
        cargo-wasi \
        twiggy \
    && rm -rf /usr/local/cargo/registry /usr/local/cargo/git

# Node.js + npm + pnpm + bun + JS/TS Component-Model toolchain.
# - typescript@6 pins the major version explicitly (apt/pnpm would otherwise
#   pull whatever the workspace requests).
# - jco + componentize-js are the bytecodealliance toolchain for
#   JS/TS → WebAssembly Component Model.
# - assemblyscript is the TS-flavored DSL → core WASM compiler.
# - javy is Shopify's QuickJS-based JS → single-file WASM builder
#   (installed via npm wrapper rather than the standalone binary so it's
#   architecture-agnostic at this layer; arch-specific binary install
#   can land in a follow-up if size matters).
RUN curl -fsSL https://deb.nodesource.com/setup_${NODE_MAJOR}.x | bash - \
    && apt-get update && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/* \
    && corepack enable \
    && corepack prepare pnpm@latest --activate \
    && curl -fsSL https://bun.sh/install | bash \
    && ln -sf /root/.bun/bin/bun /usr/local/bin/bun \
    && npm install -g --no-fund --no-audit \
        typescript@6 \
        @bytecodealliance/jco \
        @bytecodealliance/componentize-js \
        assemblyscript \
        javy-cli

# Go (stock) — supports GOOS=js and GOOS=wasip1. Multi-arch via TARGETARCH.
RUN set -eux; \
    case "${TARGETARCH}" in \
        amd64) GO_ARCH=amd64 ;; \
        arm64) GO_ARCH=arm64 ;; \
        *) echo "unsupported TARGETARCH: ${TARGETARCH}"; exit 1 ;; \
    esac; \
    curl -fsSL "https://go.dev/dl/go${GO_VERSION}.linux-${GO_ARCH}.tar.gz" \
        | tar -C /usr/local -xz

# TinyGo — the only Go path to WASI P2 / Component Model
# (`tinygo build -target=wasip2 ...`). Ships as a .deb upstream.
RUN set -eux; \
    case "${TARGETARCH}" in \
        amd64) TG_ARCH=amd64 ;; \
        arm64) TG_ARCH=arm64 ;; \
        *) echo "unsupported TARGETARCH: ${TARGETARCH}"; exit 1 ;; \
    esac; \
    curl -fsSL -o /tmp/tinygo.deb \
        "https://github.com/tinygo-org/tinygo/releases/download/v${TINYGO_VERSION}/tinygo_${TINYGO_VERSION}_${TG_ARCH}.deb" \
    && apt-get update && apt-get install -y --no-install-recommends /tmp/tinygo.deb \
    && rm -f /tmp/tinygo.deb && rm -rf /var/lib/apt/lists/*

# wasmtime CLI (provides `wasmtime compile` AOT path via Cranelift).
# Multi-arch: amd64 → x86_64, arm64 → aarch64 in the upstream tarball names.
RUN set -eux; \
    case "${TARGETARCH}" in \
        amd64) WT_ARCH=x86_64 ;; \
        arm64) WT_ARCH=aarch64 ;; \
        *) echo "unsupported TARGETARCH: ${TARGETARCH}"; exit 1 ;; \
    esac; \
    curl -fsSL "https://github.com/bytecodealliance/wasmtime/releases/download/v${WASMTIME_VERSION}/wasmtime-v${WASMTIME_VERSION}-${WT_ARCH}-linux.tar.xz" \
        | tar -C /usr/local/bin --strip-components=1 -xJ \
            "wasmtime-v${WASMTIME_VERSION}-${WT_ARCH}-linux/wasmtime" \
    && chmod +x /usr/local/bin/wasmtime

# Sanity probe - fail the build early if any toolchain is missing.
RUN set -eux; \
    rustc --version; \
    cargo --version; \
    cargo component --version; \
    wasm-tools --version; \
    wit-bindgen --version; \
    wasm-bindgen --version; \
    wasm-opt --version; \
    node --version; \
    npm --version; \
    pnpm --version; \
    bun --version; \
    tsc --version; \
    jco --version; \
    asc --version; \
    python3 --version; \
    uv --version; \
    maturin --version; \
    componentize-py --help > /dev/null; \
    go version; \
    tinygo version; \
    wasmtime --version

# -----------------------------------------------------------------------------
# Stage 2: builder - compiles UAR, the SPA, and AOT-precompiles WASM skills.
# -----------------------------------------------------------------------------
FROM toolchain AS builder

WORKDIR /src

# Initialize submodules first so cargo's network step finds the vendored
# skill-system + entity-management contents.
COPY . .
RUN git config --global --add safe.directory '*' \
    && git submodule update --init --recursive --depth 1 || true

# Frontend: pnpm workspace install + build. The pnpm workspace root is `frontend/`
# (it has pnpm-workspace.yaml + pnpm-lock.yaml; the repo root uses bun and has no
# pnpm lockfile, which caused ERR_PNPM_NO_LOCKFILE when installing from /src).
RUN cd frontend \
    && pnpm install --no-frozen-lockfile \
    && pnpm -r --filter "./packages/*" build \
    && pnpm build

# Backend: cargo build (Linux drops the `metal` feature; uses surrealkv embedded)
RUN cargo +nightly build --release \
        --features "memory-palace,wasm-runtime,surreal-memory/embedded" \
        --bin universal-agent-runtime

# AOT-precompile any shipped WASM component skills. The skill-system repo
# doesn't ship .wasm files today; this loop is a no-op until authors add them,
# but it keeps the contract documented and exercised.
RUN mkdir -p /out/skills/wasm-builtin \
    && find crates/prometheus-skill-system/skills -name "skill.wasm" -type f 2>/dev/null \
       | while read -r wasm; do \
           name=$(basename "$(dirname "$wasm")"); \
           echo "AOT compiling $wasm -> /out/skills/wasm-builtin/${name}.cwasm"; \
           wasmtime compile -o "/out/skills/wasm-builtin/${name}.cwasm" "$wasm"; \
         done

# Stage outputs that runtime needs.
RUN mkdir -p /out/bin /out/static /out/skills/builtin \
    && cp target/release/universal-agent-runtime /out/bin/ \
    && cp -R static/. /out/static/ \
    && cp -R crates/prometheus-skill-system/skills/. /out/skills/builtin/ \
    && cp -R src/uar/runtime/matching/models /out/models

# -----------------------------------------------------------------------------
# Stage 3: runtime - polyglot image (keeps toolchains resident for user builds)
# -----------------------------------------------------------------------------
FROM toolchain AS runtime

WORKDIR /opt/uar

ENV UAR_STATIC_DIR=/opt/uar/static \
    UAR_MODELS_DIR=/opt/uar/models \
    UAR_BUILTIN_SKILLS_DIR=/opt/uar/skills/builtin \
    UAR_SKILLS_WASM_BUILTIN_DIR=/opt/uar/skills/wasm-builtin \
    UAR_SKILLS_USER_DIR=/var/lib/uar/skills-user \
    UAR_SKILLS_DERIVED_DIR=/var/lib/uar/skills-derived \
    HF_HOME=/var/lib/uar/cache/huggingface \
    CARGO_HOME=/var/lib/uar/cache/cargo \
    PNPM_STORE_DIR=/var/lib/uar/cache/pnpm

COPY --from=builder /out/bin/universal-agent-runtime /usr/local/bin/universal-agent-runtime
COPY --from=builder /out/static /opt/uar/static
COPY --from=builder /out/skills /opt/uar/skills
COPY --from=builder /out/models /opt/uar/models

VOLUME ["/var/lib/uar/skills-user", \
        "/var/lib/uar/skills-derived", \
        "/var/lib/uar/cache/huggingface", \
        "/var/lib/uar/cache/cargo", \
        "/var/lib/uar/cache/pnpm", \
        "/var/lib/uar/data"]

EXPOSE 1906 50051

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD curl -fsS http://127.0.0.1:1906/health || exit 1

CMD ["universal-agent-runtime"]
