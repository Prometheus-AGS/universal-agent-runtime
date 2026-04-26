# Dependency Management

This document describes how git-sourced dependencies are managed in UAR and provides the standard operating procedure for upgrading them.

## Why Git Dependencies Are Pinned

UAR uses several crates sourced directly from Git repositories rather than crates.io:

| Crate | Repository | Reason |
|-------|-----------|--------|
| `rmcp` | `modelcontextprotocol/rust-sdk` | MCP Rust SDK is pre-release; no stable crates.io version |
| `surreal-memory` | `Prometheus-AGS/surreal-memory-server` | Internal library, not published |
| `kreuzberg` | `GQAdonis/kreuzberg` | Default local document intelligence provider; follows the fork's main branch |
| `prometheus_parking_lot` | `Prometheus-AGS/prometheus-parking-lot-rs` | Internal library, not published |

Most git dependencies are **pinned to a specific commit SHA** via `rev = "..."` in `Cargo.toml`. `kreuzberg` intentionally tracks `branch = "main"` so UAR can consume the active document-intelligence fork. Pinning ensures:

- **Reproducible builds**: The same SHA is resolved every time regardless of upstream changes
- **CI stability**: The CI pipeline does not break due to unexpected upstream commits
- **Audit trail**: The `Cargo.toml` history shows exactly which upstream version was adopted and when

## Current Pinned Versions

```toml
rmcp          = rev "085470025f690050e8776ffa939e7ba71d3abc01"
surreal-memory = rev "c6f95c905c16907ad58ef9049f32dcc9531d40eb"
kreuzberg     = branch "main" on GQAdonis/kreuzberg
prometheus_parking_lot = rev "32b481d6c5694545d35789894f6feecf5ac4ca3e"
```

## Upgrade SOP

Follow these steps when upgrading a pinned git dependency:

### 1. Identify the target commit

```bash
# Get the latest commit SHA on the default branch
git ls-remote https://github.com/<org>/<repo>.git HEAD

# Or list tags if upgrading to a specific release
git ls-remote --tags https://github.com/<org>/<repo>.git
```

### 2. Update Cargo.toml

Change the `rev = "..."` field for the target crate to the new SHA:

```toml
rmcp = { git = "https://github.com/modelcontextprotocol/rust-sdk", rev = "<new-sha>", features = [...] }
```

### 3. Refresh Cargo.lock

```bash
cargo update -p rmcp
```

This updates only the named package in the lock file without touching other dependencies.

### 4. Verify the build

```bash
cargo build
```

Fix any compilation errors caused by upstream API changes before proceeding.

### 5. Run the test suite

```bash
cargo test --lib          # Unit tests (fast, no Docker)
cargo test                # Full suite (requires Postgres + Redis)
```

### 6. Commit both files together

Always commit `Cargo.toml` and `Cargo.lock` as a single atomic commit:

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(deps): upgrade rmcp to <new-sha>"
```

Including the SHA in the commit message provides a clear audit trail.

## Upgrading to crates.io Once Available

When a git-sourced crate publishes a stable crates.io release:

1. Remove the `git = "..."` and `rev = "..."` fields
2. Add `version = "x.y.z"` instead
3. Run `cargo update -p <crate>` to pull from the registry
4. Follow steps 4–6 above

## Security Advisories

Run `cargo audit` periodically to check for known vulnerabilities in all dependencies (including transitive ones):

```bash
cargo install cargo-audit
cargo audit
```

The CI pipeline runs `cargo audit` as part of the release workflow (`release.yml`). For non-release branches, run it manually before merging significant dependency changes.
