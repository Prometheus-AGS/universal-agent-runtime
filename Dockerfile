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
#   - wasmtime (CLI)  pinned with the runtime; discovered skills use plain WASM
#
# Build with:
#   docker buildx build --platform linux/amd64 -t uar:latest .
# Runtime image size is intentionally large (~3GB) - see README for a future
# "runtime-slim" variant that strips build tools.
# =============================================================================

# Toolchain pins — bump via a new KBD change so drift is auditable.
# Rust nightly is dated (matches rust-toolchain.toml at repo root).
# Go follows the latest stable per go.dev/doc/devel/release.
ARG RUST_TOOLCHAIN=nightly-2026-07-18
ARG NODE_MAJOR=24
ARG PYTHON_VERSION=3.13.18
ARG GO_VERSION=1.26.3
# Keep aligned with both Wasmtime packages resolved in Cargo.lock. Plain
# components are compiled by the runtime for its own engine/features/host target.
ARG WASMTIME_VERSION=50.0.1
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
# TARGETARCH is set automatically by BuildKit (`amd64` or `arm64`); see
# multi-arch branches further down in the Go / wasmtime / tinygo installs.
ARG TARGETARCH

ENV DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8 \
    CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/cargo/bin:/usr/local/go/bin:/root/.local/bin:/root/.bun/bin:$PATH

# Ubuntu omits manpages, but xz and GCC alternatives install links to them.
RUN if [ "$TARGETARCH" = amd64 ]; then \
        sources=/etc/apt/sources.list.d/ubuntu.sources; \
        grep -Fxq 'URIs: http://archive.ubuntu.com/ubuntu/' "$sources" \
            && sed -i 's|^URIs: http://archive\.ubuntu\.com/ubuntu/$|URIs: http://us.archive.ubuntu.com/ubuntu/|' "$sources"; \
    fi \
    && apt-get update && apt-get -o 'DPkg::Options::=--path-include=/usr/share/man/man1/*' \
        install -y --no-install-recommends \
        ca-certificates curl git build-essential patch pkg-config \
        libssl-dev cmake unzip xz-utils \
        gnupg lsb-release \
        protobuf-compiler libprotobuf-dev \
        # clang + libclang for bindgen (surrealdb-librocksdb-sys FFI bindings).
        clang libclang-dev \
        # WASM low-level tooling from apt (wasm-opt via binaryen,
        # wat2wasm/wasm2wat/wasm-objdump via wabt). Adding here keeps
        # the cargo install step focused on Rust-source tools only.
        binaryen wabt \
        # CPython standard-library modules and native extensions.
        libbluetooth-dev libbz2-dev libdb-dev libffi-dev libgdbm-dev \
        libgdbm-compat-dev liblzma-dev libncurses-dev libreadline-dev \
        libsqlite3-dev tk-dev uuid-dev zlib1g-dev patchelf \
    && rm -rf /var/lib/apt/lists/*

COPY vendor/patches/cpython/gh-127878-import-cache.patch /tmp/cpython-import.patch

# Noble lacks libmpdec-dev. Build the pinned upstream library before CPython.
# uv manages the system interpreter's packages. venv still uses bundled ensurepip.
RUN set -eu; \
    workdir="$(mktemp -d)"; \
    mkdir "$workdir/mpdecimal" "$workdir/python"; \
    cd "$workdir/mpdecimal"; \
    curl --proto '=https' -fsSLo mpdecimal.tar.gz \
        https://www.bytereef.org/software/mpdecimal/releases/mpdecimal-4.0.1.tar.gz; \
    echo '96d33abb4bb0070c7be0fed4246cd38416188325f820468214471938545b1ac8  mpdecimal.tar.gz' \
        | sha256sum -c -; \
    tar -xzf mpdecimal.tar.gz --strip-components=1; \
    ./configure --disable-cxx; \
    make -j "$(nproc)"; \
    make install; \
    ldconfig; \
    cd "$workdir/python"; \
    curl --proto '=https' -fsSLo python.tar.xz \
        "https://www.python.org/ftp/python/${PYTHON_VERSION}/Python-${PYTHON_VERSION}.tar.xz"; \
    echo 'abb0102e6d4ae027509bcb82327d9dd85515b44a3ba8ad538cfc5e9847029f68  python.tar.xz' \
        | sha256sum -c -; \
    tar -xJf python.tar.xz --strip-components=1; \
    patch --batch --forward --fuzz=0 -p1 < /tmp/cpython-import.patch; \
    rm /tmp/cpython-import.patch; \
    ./configure --enable-shared --enable-loadable-sqlite-extensions \
        --with-system-libmpdec --with-ensurepip=no; \
    make -j "$(nproc)"; \
    make install; \
    ldconfig; \
    ln -s python3 /usr/local/bin/python; \
    cd /; \
    rm -rf "$workdir"

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
# Pin the official cargo-binstall archive and verify it before extraction.
# This keeps the bootstrap independent of its source-build toolchain lints.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain ${RUST_TOOLCHAIN} --profile minimal \
                    --component rustfmt,clippy,rust-src \
    && rustup target add wasm32-wasip2 wasm32-wasip1 wasm32-unknown-unknown \
    && case "${TARGETARCH}" in \
        amd64) binstall_arch=x86_64; binstall_sha=9e0e064f189a9abb9f8df5fb93c3c810ef5cc19e7212d998c7a64c9e4d3527d5 ;; \
        arm64) binstall_arch=aarch64; binstall_sha=a235250cd684af9bf1e3a1e4c216b1d36e6c553553d5d789744f27689d81c2d6 ;; \
        *) echo "Unsupported cargo-binstall architecture: ${TARGETARCH}" >&2; exit 1 ;; \
    esac \
    && binstall_archive=/tmp/cargo-binstall.tgz \
    && curl --proto '=https' --tlsv1.2 -fsSL \
        "https://github.com/cargo-bins/cargo-binstall/releases/download/v1.26.2/cargo-binstall-${binstall_arch}-unknown-linux-musl.tgz" \
        -o "${binstall_archive}" \
    && printf '%s  %s\n' "${binstall_sha}" "${binstall_archive}" | sha256sum -c - \
    && tar -xzf "${binstall_archive}" -C "${CARGO_HOME}/bin" cargo-binstall \
    && rm "${binstall_archive}" \
    && cargo binstall --no-confirm --locked \
        cargo-component \
        wasm-tools \
        wit-bindgen-cli \
        wasm-bindgen-cli \
        twiggy \
    # cargo-wasi 0.1.28 (archived, last release) ships no prebuilt binaries, so
    # binstall compiles it from source, and its locked rustix no longer
    # compiles on the pinned nightly. Unlocked, it resolves a current rustix.
    && cargo binstall --no-confirm cargo-wasi \
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
    && corepack prepare pnpm@11.15.0 --activate \
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

# wasmtime CLI (operator tooling; no trusted native-cache loading is advertised).
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
# Stage 2: builder - compiles UAR, the SPA, and stages plain WASM skills.
# -----------------------------------------------------------------------------
FROM toolchain AS builder

ARG UAR_CARGO_BUILD_JOBS
ARG UAR_CARGO_CACHE_KEY=default

WORKDIR /src

# The build context must already include the pinned recursive submodule
# checkout. Git metadata is excluded from the context.
COPY . .
RUN for input in \
        models.dev/package.json \
        crates/prometheus-skill-system/skills/process/kbd-process-orchestrator/SKILL.md \
        frontend/packages/prometheus-entity-management/package.json \
        frontend/packages/prometheus-entity-management/packages/entity-graph-core/package.json \
        frontend/packages/prometheus-entity-management/packages/entity-graph-react/package.json \
        vendor/git/liter-llm/crates/liter-llm/Cargo.toml \
        vendor/git/rust-mcp-filesystem/Cargo.toml; do \
        if [ ! -f "$input" ]; then \
            printf 'Missing vendored build input: %s. Initialize pinned submodules recursively before building.\n' "$input" >&2; \
            exit 1; \
        fi; \
    done

# Frontend: use the same pinned pnpm and immutable frontend lockfile as release
# validation. The frontend remains an isolated workspace under `frontend/`.
RUN cd frontend \
    && pnpm install --frozen-lockfile \
    && (cd packages/prometheus-entity-management \
        && pnpm install --frozen-lockfile \
        && pnpm --filter @prometheus-ags/entity-graph-core build \
        && pnpm --filter @prometheus-ags/entity-graph-react build \
        && pnpm --filter @prometheus-ags/prometheus-entity-management build) \
    && pnpm build

# Backend: cargo build (Linux drops the `metal` feature; uses surrealkv embedded).
# The lockfile fetches the pinned xberg sibling crates from a public Git fork.
# Retain the optional github_token mount for private Git dependencies.
# Runtime-only Git configuration rewrites SSH to credential-free HTTPS URLs;
# the GitHub-scoped helper reads the mount only on get and never stores secrets.
RUN --mount=type=secret,id=github_token \
    --mount=type=cache,id=uar-target-${RUST_TOOLCHAIN}-${TARGETARCH}-server-full-postgres-${UAR_CARGO_CACHE_KEY},target=/src/target,sharing=locked \
    if [ -s /run/secrets/github_token ]; then \
      export GIT_CONFIG_COUNT=4 \
        GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf \
        GIT_CONFIG_VALUE_0=ssh://git@github.com/ \
        GIT_CONFIG_KEY_1=url.https://github.com/.insteadOf \
        GIT_CONFIG_VALUE_1=git@github.com: \
        GIT_CONFIG_KEY_2=credential.helper \
        GIT_CONFIG_VALUE_2= \
        GIT_CONFIG_KEY_3=credential.https://github.com.helper \
        GIT_CONFIG_VALUE_3='!f() { if [ "$1" = get ]; then printf "username=x-access-token\npassword="; cat /run/secrets/github_token || return; printf "\n"; fi; }; f'; \
    fi \
    && if [ -n "${UAR_CARGO_BUILD_JOBS}" ]; then export CARGO_BUILD_JOBS="${UAR_CARGO_BUILD_JOBS}"; fi \
    && CARGO_NET_GIT_FETCH_WITH_CLI=true cargo +"${RUST_TOOLCHAIN}" build --release --locked \
        --features "server-full,postgres-backend" \
        --bin universal-agent-runtime \
    && mkdir -p /out/bin \
    && cp target/release/universal-agent-runtime /out/bin/ \
    && for model in target/release/build/xberg-tesseract-*/out/eng.traineddata; do \
         if [ -f "$model" ] && printf '%s  %s\n' \
           7d4322bd2a7749724879683fc3912cb542f19906c83bcc1a52132556427170b2 "$model" \
           | sha256sum --check --status; then \
           install -D -m 0644 "$model" /out/tessdata/eng.traineddata || exit 1; \
           break; \
         fi; \
       done \
    && test -s /out/tessdata/eng.traineddata \
    && install -m 0644 "${CARGO_HOME}"/git/checkouts/xberg-*/ffb3b1a/crates/xberg-tesseract/licenses/tesseract-LICENSE \
         /out/tessdata/LICENSE

# Ship the actual plain components from the pinned skill pack. Native caches
# are not a trusted distribution format: the runtime validates and compiles
# these sources with its own engine/features/host architecture on registration.
RUN mkdir -p /out/skills/wasm-builtin \
    && find crates/prometheus-skill-system/skills -name "skill.wasm" -type f 2>/dev/null \
       | while read -r wasm; do \
           name=$(basename "$(dirname "$wasm")"); \
           cp "$wasm" "/out/skills/wasm-builtin/${name}.wasm"; \
         done

# Stage outputs that runtime needs.
RUN mkdir -p /out/bin /out/static /out/skills/builtin \
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
    UAR_PERSISTENCE__PROVIDER=surreal \
    UAR_PERSISTENCE__DATABASE_URL=surrealkv:///var/lib/uar/data/uar.db \
    UAR_SKILLS_WASM_BUILTIN_DIR=/opt/uar/skills/wasm-builtin \
    UAR_SKILLS_USER_DIR=/var/lib/uar/skills-user \
    UAR_SKILLS_DERIVED_DIR=/var/lib/uar/skills-derived \
    HF_HOME=/var/lib/uar/cache/huggingface \
    CARGO_HOME=/var/lib/uar/cache/cargo \
    PNPM_STORE_DIR=/var/lib/uar/cache/pnpm

COPY --from=builder /out/bin/universal-agent-runtime /usr/local/bin/universal-agent-runtime
# The pinned xberg build verifies this English pack. Its Linux system lookup
# can read it without materializing a cache or downloading data during OCR.
COPY --from=builder /out/tessdata/ /usr/share/tesseract-ocr/5/tessdata/
# Preserve restrictive source modes while making bundled assets readable by
# the non-root runtime identity, including the mandatory governance policies.
COPY --chown=65532:65532 --from=builder /out/static /opt/uar/static
COPY --chown=65532:65532 --from=builder /out/skills /opt/uar/skills
COPY --chown=65532:65532 --from=builder /out/models /opt/uar/models
# Governed startup refuses to boot without a Cedar policy set; the engine
# reads the relative "policies" directory, i.e. /opt/uar/policies.
COPY --chown=65532:65532 --from=builder /src/policies /opt/uar/policies

RUN mkdir -p /var/lib/uar/skills-user \
        /var/lib/uar/skills-derived \
        /var/lib/uar/cache/huggingface \
        /var/lib/uar/cache/cargo \
        /var/lib/uar/cache/pnpm \
        /var/lib/uar/data \
    && chown -R 65532:65532 /var/lib/uar

# One volume avoids nested anonymous volumes masking a caller's /var/lib/uar
# bind mount. Numeric ownership keeps the image runtime-agnostic and allows
# Kubernetes/Docker to enforce the same non-root identity.
VOLUME ["/var/lib/uar"]

USER 65532:65532

EXPOSE 1906 50051

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD curl -fsS http://127.0.0.1:1906/health || exit 1

CMD ["universal-agent-runtime"]
