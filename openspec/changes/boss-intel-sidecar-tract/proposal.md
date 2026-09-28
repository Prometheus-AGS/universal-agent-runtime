# Intel macOS sidecar with local embeddings

The Boss sidecar release for `darwin-x64` fails in `ort-sys 2.0.0-rc.13`: upstream no longer publishes an Intel macOS ONNX Runtime binary. The release must retain local embedding support and expose the actual feature set in its payload record.

Use the `ort-tract` CPU backend only for the Intel sidecar build. The other three Boss sidecar platforms retain their existing `server-full` build. This change covers build selection, early backend initialization, lockfile, and payload provenance. It does not change runtime ownership, model assets, or the other platform binaries.

Runtime UX, provider compatibility, and realtime event contracts stay the same. The packaged Intel sidecar uses the existing local model path with a different inference backend. The active Agent Fabric Convergence KBD release ledger records the Intel artifact after its native build; this change does not move its waypoint.

## Capabilities

- `boss-sidecar-artifacts`: native sidecar feature selection and truthful release records.
