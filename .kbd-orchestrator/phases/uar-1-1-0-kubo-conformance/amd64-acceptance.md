# Published amd64 acceptance

## Published amd64 acceptance follow-up — 2026-10-15

The unchanged full acceptance program **passed** against the published linux/amd64 child manifest `sha256:94f78a7ed17197fc045baec514ed6ec30d3cf23433e8c26340ccfddb080fac90`: exit 0, `result: passed`, `full_acceptance: true`. Coverage: 12 shipped configuration projections, 4 MCP/skills/AG-UI profiles, 12 elicitation decisions, 4 explicit approval denials and all 8 A2A cases, including the 300-second default egress observation.

```bash
DOCKER_DEFAULT_PLATFORM= PYTHONUNBUFFERED=1 python3 scripts/release/uar_1_1_acceptance.py --image ghcr.io/prometheus-ags/universal-agent-runtime@sha256:94f78a7ed17197fc045baec514ed6ec30d3cf23433e8c26340ccfddb080fac90 --evidence /tmp/uar-v1.1.0-published-amd64-evidence-r2
```

Image metadata reports linux/amd64 and the UAR executable's ELF machine is x86-64 (62). Execution used amd64 emulation on the arm64 OrbStack host; this is not native amd64 hardware qualification. The pinned SurrealDB companion and grpcurl client were arm64. Runtime and fixture sources matched v1.1.0 before and after the run; no source or binary patches, rebuild or tag movement occurred. Existing capture-window, configuration-projection and review-independence limits remain.

An initial full attempt completed default checks but stopped at a companion Docker pull with `cannot overwrite digest`. Its incomplete receipt is retained. Removing the global platform override preserved the existing companion platform while UAR remained pinned to its immutable amd64 child digest; the second complete run passed. No assertions or timeouts were changed.

The supplemental `uar-v1.1.0-amd64-*` JSON assets and `SHA256SUMS-amd64` retain this result, execution provenance, egress observations, protocol-client provenance and the incomplete first attempt. The original arm64 assets and `SHA256SUMS` remain unchanged. The v1.1.0 multi-platform image digest remains `sha256:064f5004fb3ba76506abce5cf81285b2a8aa3e54fa5a22e2c2bfc97b1211e43b`.

Raw receipt: receipts/uar-v1.1.0-amd64-acceptance.json. Execution and hashes: receipts/uar-v1.1.0-amd64-execution.json and SHA256SUMS-amd64. The full implementation boundary was complete before these runs; only environment selection changed between attempts.
