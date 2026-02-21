# ─────────────────────────────────────────────────────────────────
# Stage 1: Frontend build (Node / Bun)
# ─────────────────────────────────────────────────────────────────
FROM node:20-slim AS frontend-builder

WORKDIR /app

# Install bun
RUN npm install -g bun

# Copy frontend manifests first so dependency install is cached separately
COPY frontend/package.json frontend/bun.lock* ./frontend/

# Install dependencies
RUN cd frontend && bun install --frozen-lockfile

# Copy the rest of the source needed for the build
COPY frontend/ ./frontend/

# Build — Vite writes output to ../static (i.e. /app/static)
RUN cd frontend && bun run build

# ─────────────────────────────────────────────────────────────────
# Stage 2: Rust build
# ─────────────────────────────────────────────────────────────────
FROM rust:1.93-slim-bookworm AS builder

WORKDIR /app

# Skip ONNX model download and frontend build during Docker build
# (frontend assets are copied from Stage 1 below)
ARG SKIP_MODEL_BUILD=1
ARG SKIP_FRONTEND_BUILD=1
ENV SKIP_MODEL_BUILD=${SKIP_MODEL_BUILD}
ENV SKIP_FRONTEND_BUILD=${SKIP_FRONTEND_BUILD}

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    curl \
    gcc \
    g++ \
    clang \
    libclang-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "" > src/lib.rs && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY . .

# Copy pre-built frontend assets into static/ so the server can serve them
COPY --from=frontend-builder /app/static ./static

# Touch main.rs to ensure rebuild
RUN touch src/main.rs && cargo build --release

# ─────────────────────────────────────────────────────────────────
# Stage 3: Runtime
# ─────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies + Node.js 22 (required for npx-based MCP servers)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/universal-agent-runtime /app/server

# Copy static assets (served by the app at /static)
COPY --from=builder /app/static /app/static

# Copy tokenizer model files needed by VectorMatcher
COPY --from=builder /app/src/uar/runtime/matching/models/*.json /app/models/

# Copy MCP server configuration (loaded at startup by McpRegistry)
COPY mcp.json /app/mcp.json

# Create data directories used at runtime
RUN mkdir -p /data/ingest /app/skills /app/policies /app/models

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the binary
CMD ["/app/server"]
