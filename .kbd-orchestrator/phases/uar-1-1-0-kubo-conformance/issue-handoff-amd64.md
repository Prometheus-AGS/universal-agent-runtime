## Published v1.1.0 amd64 follow-up: full acceptance PASS

The unchanged full acceptance program exited **0**, with `result: passed` and `full_acceptance: true`, against the published linux/amd64 image:

`ghcr.io/prometheus-ags/universal-agent-runtime@sha256:94f78a7ed17197fc045baec514ed6ec30d3cf23433e8c26340ccfddb080fac90`

Coverage passed: all **12** shipped configuration projections, **4** MCP/skills/AG-UI profiles, **12** elicitation decisions, **4** explicit false approvals, **8** A2A transport/governance/decision cases, and the **300-second** default capture. The program used the real runtime, persistence, protocol clients and observable MCP peer.

```bash
DOCKER_DEFAULT_PLATFORM= PYTHONUNBUFFERED=1 python3 scripts/release/uar_1_1_acceptance.py --image ghcr.io/prometheus-ags/universal-agent-runtime@sha256:94f78a7ed17197fc045baec514ed6ec30d3cf23433e8c26340ccfddb080fac90 --evidence /tmp/uar-v1.1.0-published-amd64-evidence-r2
```

Execution was **amd64 emulation on the arm64 OrbStack host**, not native amd64 hardware. Image metadata reported linux/amd64, and the UAR executable was x86-64 ELF machine62. The pinned SurrealDB companion and grpcurl client were arm64. Native amd64 hardware execution, the startup capture gap, actual infrastructure rollouts, and the prior review-independence limit remain unverified.

The first attempt completed defaults but stopped at a companion Docker pull with `cannot overwrite digest`. Its incomplete receipt is retained. Removing the global platform override preserved the existing companion while UAR stayed pinned to its amd64 child digest. The second full program passed; no runtime/fixture source, assertion, timeout or binary was patched.

The [Release](https://github.com/Prometheus-AGS/universal-agent-runtime/releases/tag/v1.1.0) now contains six supplemental amd64 assets: [full receipt](https://github.com/Prometheus-AGS/universal-agent-runtime/releases/download/v1.1.0/uar-v1.1.0-amd64-acceptance.json), [execution provenance](https://github.com/Prometheus-AGS/universal-agent-runtime/releases/download/v1.1.0/uar-v1.1.0-amd64-execution.json), egress and client provenance, the incomplete first attempt and [checksums](https://github.com/Prometheus-AGS/universal-agent-runtime/releases/download/v1.1.0/SHA256SUMS-amd64). All six uploaded SHA256 digests matched local files. The original six arm64 assets and their hashes remain unchanged. Source/tag commit remains `d815ef72ac7d2c57a9bd18b23fcc58ffb224f326`; the tag was not moved.

OG-4 remains: `v1.1.0, ghcr.io/prometheus-ags/universal-agent-runtime@sha256:064f5004fb3ba76506abce5cf81285b2a8aa3e54fa5a22e2c2bfc97b1211e43b`.
