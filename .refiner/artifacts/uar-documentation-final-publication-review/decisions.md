# Decisions

## Converge after one iteration

All blocking constraints pass observed deterministic checks against the complete
production artifact. Further content or visual changes would be speculative.
Live Pages validation remains a required publication action after push and does
not justify another local refinement cycle.

## Keep Rustdoc public-library scoped

The public reference documents `universal-agent-runtime` under `server-full`.
The internal `mcp-server-fetch` binary is neither a public library API nor a
reason to patch product code during a documentation publication phase.

## Remove generated Rust source browsing

Generated protobuf source locations inherit absolute build-cache paths. Staging
removes source pages and only their machine-local source anchors, then verifies
that no macOS, Linux, or Windows home path remains. Public item documentation and
generated API structure remain intact.
