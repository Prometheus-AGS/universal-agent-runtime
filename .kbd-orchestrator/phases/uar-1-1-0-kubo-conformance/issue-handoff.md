Published and accepted: [UAR v1.1.0](https://github.com/Prometheus-AGS/universal-agent-runtime/releases/tag/v1.1.0).

`OG-4: v1.1.0, ghcr.io/prometheus-ags/universal-agent-runtime@sha256:064f5004fb3ba76506abce5cf81285b2a8aa3e54fa5a22e2c2bfc97b1211e43b`

Source/tag commit: `d815ef72ac7d2c57a9bd18b23fcc58ffb224f326`. [Native amd64/arm64 publication workflow](https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/38780160573) succeeded. Architecture image manifests:

- linux/amd64: `sha256:94f78a7ed17197fc045baec514ed6ec30d3cf23433e8c26340ccfddb080fac90`
- linux/arm64: `sha256:b3a3d8bdda9d919fa3489dded935c834256bf89984fa17b07a428acdb7a1c1ae`

The complete conformance program passed locally against the pulled multi-platform digest with no UAR source or binary patches: public discovery/version/CORS/effective settings; five-minute default egress, loopback and no-model checks; 12 configuration projections derived from shipped manifests; 4 MCP/skills/AG-UI profiles across both sharing modes and peer topologies; 12 elicitation decisions; 4 explicit approval denials; and all 8 A2A transport/governance/decision cases. Required tools ran zero times before approval, once after approval and zero after denial. Missing skill roles created no task; dynamic activation remained ineligible after approval using the same limited credential. Exact skill upload/readback/binding/restart and failed-persistence nonpublication passed.

The release includes the nine original conformance items and the four follow-ups. [Pinned setup, compatibility and interrupt/resume contract](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/v1.1.0/docs/releases/v1.1.0-kubo.md) covers trusted-host request credentials, per-agent restrictions, single-manifest upload and globally registered elicitation in both sharing modes. Official A2A 1.0 is served alongside documented legacy compatibility endpoints. Default startup remains loopback/authenticated/model-free; explicit deployment listeners and remote databases follow the approved deployment distinction.

The release attaches the complete acceptance JSON, manifest, publication/source/workflow receipt, egress observations, protocol-client provenance and SHA256SUMS. Execution acceptance was linux/arm64; amd64 was built natively but not executed locally. The packet receipt records the launch-to-first-capture gap and documented interface-control exclusions. Kubo can pin this tag/digest and perform its downstream real-service check.
