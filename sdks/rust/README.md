# Universal Agent Runtime Rust SDK 1.0

> **Current authority:** [Rust SDK guide](/docs/sdk-rust/intro). The source
> package is checked in at version 1.0.0; registry availability is release evidence
> and is not inferred from this README.

The Rust SDK exposes a default HTTP client plus optional in-process runtime
features. Its client covers chat, tools, structured output, embeddings, runs and
checkpoints, knowledge bases, documents, search, and ingestion.

| Feature | Boundary |
|---|---|
| `http-client` | Default async HTTP/SSE client |
| `embedded` | Links UAR with host-persistence support |
| `embedded-mobile` | Adds the transport-free mobile composition |
| `server` | Links the `minimal` server composition |
| `full` | Combines the HTTP client and `server` features |

Build the independently locked source package:

```bash
cargo check --locked --manifest-path sdks/rust/Cargo.toml
UAR_BASE_URL=http://127.0.0.1:1906 \
  cargo run --locked --manifest-path sdks/rust/Cargo.toml --example chat
```

The network example requires a running UAR server and valid credentials. The
embedded builder requires host-supplied persistence and provider boundaries; it
does not silently start an HTTP listener.

`Cargo.toml` names the crate `universal-agent-runtime-sdk`. Before using a
registry dependency, verify the exact version, checksum, and publisher on
crates.io. The source path is the repository-verifiable installation path.
