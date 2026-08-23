# UAR Cookbook

> **Current authority:** [Installation and source quickstart](/docs/installation).
> These examples are source-level demonstrations, not release certification.

This directory contains small examples that compile against the checked-in UAR
and SDK source.

## Inventory

| Directory | Contents | Runtime dependency |
|---|---|---|
| `runtime/` | Four Rust examples for configuration, native tools, and SSE | Two examples run locally; two exercise source contracts only |
| `sdk/rust/` | Client initialization and streaming examples | A configured UAR server when executed |
| `sdk/python/` | A chat request example | A configured UAR server when executed |
| `sdk/typescript/` | A response-handling example | TypeScript source only |
| `a2ui/` | Inventory and links to the maintained renderer examples | None |

Run the collection validator locally from the repository root:

```bash
bash tools/validate-cookbook.sh
```

The script compiles the examples it owns and runs only examples that do not
require external credentials, a model, or a live UAR server. Passing it does not
prove inference, deployment, or another runtime profile.
