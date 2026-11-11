# jsonwebtoken RustCrypto security backports

The two production-library source trees come from the MIT-licensed crates.io
release archives. Keep each package version and its provider, feature names,
and JWT validation behavior. The workspace patches both versions because UAR
and its identity proxy use 11.1.0 while SurrealDB uses 10.4.0, including the
RustCrypto backend in its WebAssembly graph.

| Package | Original release source commit | SHA-256 of the crates.io archive |
| --- | --- | --- |
| jsonwebtoken 10.4.0 | `69a8fbf40a83c3d87301e75148e02b2090e4feed` | `eba32bfb4ffdeaca3e34431072faf01745c9b26d25504aa7a6cf5684334fc4fc` |
| jsonwebtoken 11.1.0 | `4c0ae752e9acc108c8e2c4c8ed8128dc66014210` | `e75fe14a82d81e5f5af639997db37d8b96045938a7ac6ab18cdbe1c7467e05e1` |

[Keats/jsonwebtoken#536](https://github.com/Keats/jsonwebtoken/pull/536), head
`36234be048bac3574f12dd40102b8e0a3a5bcde7`, moves the coordinated RustCrypto
dependency stack away from the affected crypto-bigint line. The four updated
11.1.0 backend files match that revision byte-for-byte. The 10.4.0 port keeps
its own provider and JWK interfaces and applies the same primitive API changes.
Both manifests explicitly enable getrandom 0.4's browser backend for WebAssembly
targets without a native OS RNG. The source exports omit the upstream standalone
test and benchmark targets, development dependencies, and lockfiles.

The application still owns explicit RustCrypto initialization and rejects an
already occupied provider slot. Retire both source patches when all upstream
consumers resolve fixed releases and the workspace release gate passes.
