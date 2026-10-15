UAR v1.1.0 completes the nine #379 conformance items and four Kubo follow-up requirements. Source commit: `d815ef72ac7d2c57a9bd18b23fcc58ffb224f326`. Both native images were built and published by [the tag deployment workflow](https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/38780160573).

```bash
docker pull ghcr.io/prometheus-ags/universal-agent-runtime@sha256:064f5004fb3ba76506abce5cf81285b2a8aa3e54fa5a22e2c2bfc97b1211e43b
```

| Artifact | Digest |
|---|---|
| Multi-platform image | `sha256:064f5004fb3ba76506abce5cf81285b2a8aa3e54fa5a22e2c2bfc97b1211e43b` |
| linux/amd64 manifest | `sha256:94f78a7ed17197fc045baec514ed6ec30d3cf23433e8c26340ccfddb080fac90` |
| linux/arm64 manifest | `sha256:b3a3d8bdda9d919fa3489dded935c834256bf89984fa17b07a428acdb7a1c1ae` |

The complete local acceptance program passed against this pulled immutable digest, with no UAR source or binary patches: 12 shipped configuration profiles, 4 MCP/skills/AG-UI profiles, 12 elicitation decisions, 4 explicit approval denials, and 8 A2A transport/governance/approval cases. Authorization denial created no task; dynamic protected-skill activation remained ineligible; invocation counts were zero before approval, one after approval, and zero after denial. Exact skill upload/readback, binding/restart, and failed-persistence nonpublication passed. Public discovery/version, CORS, effective settings authorization/revisions and the five-minute default egress observation passed. Product checks ran locally; GitHub Actions built and published artifacts only.

[The pinned release contract and setup instructions](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/v1.1.0/docs/releases/v1.1.0-kubo.md) document operator skill roles, trusted-host credentials, per-agent `allowed_agent_ids`, request-time MCP headers, single-manifest remote skill import, and complete `tool_approval` / `mcp_elicitation` interrupt and resume shapes with `@ag-ui/core` 1.0.2. Required tool approval remains enforced with governance enabled or disabled.

Default startup requires authentication, binds loopback, selects no model and uses disabled/local embedding settings. Explicit deployment profiles retain their selected listeners and remote databases. Configure authentication, model/provider access and any deployment listener deliberately. Both MCP sharing modes support elicitation; restricted destinations require explicit authorized run grants.

Official A2A 1.0 JSON-RPC and `lf.a2a.v1.A2AService` are the supported 1.0 interfaces. Existing legacy discovery and `a2a.AgentService` remain documented compatibility surfaces. Archives and supporting skill assets are outside the single-SKILL.md upload contract.

Original execution acceptance ran on linux/arm64. Full follow-up acceptance also passed against the published linux/amd64 child manifest under emulation on the arm64 OrbStack host. Native amd64 hardware execution remains unverified. The egress receipt covers the observed 300-second interval, excludes documented interface control frames, and does not certify the launch-to-first-capture gap. Kubo's own real-service integration remains its downstream check.

Attached JSON receipts and `SHA256SUMS` retain the image/source/workflow provenance, complete acceptance result, egress observations, manifest and pinned protocol-client provenance.

## Published amd64 acceptance follow-up — 2026-10-15

The unchanged full acceptance program **passed** against the published linux/amd64 child manifest `sha256:94f78a7ed17197fc045baec514ed6ec30d3cf23433e8c26340ccfddb080fac90`: exit 0, `result: passed`, `full_acceptance: true`. Coverage: 12 shipped configuration projections, 4 MCP/skills/AG-UI profiles, 12 elicitation decisions, 4 explicit approval denials and all 8 A2A cases, including the 300-second default egress observation.

```bash
DOCKER_DEFAULT_PLATFORM= PYTHONUNBUFFERED=1 python3 scripts/release/uar_1_1_acceptance.py --image ghcr.io/prometheus-ags/universal-agent-runtime@sha256:94f78a7ed17197fc045baec514ed6ec30d3cf23433e8c26340ccfddb080fac90 --evidence /tmp/uar-v1.1.0-published-amd64-evidence-r2
```

Image metadata reports linux/amd64 and the UAR executable's ELF machine is x86-64 (62). Execution used amd64 emulation on the arm64 OrbStack host; this is not native amd64 hardware qualification. The pinned SurrealDB companion and grpcurl client were arm64. Runtime and fixture sources matched v1.1.0 before and after the run; no source or binary patches, rebuild or tag movement occurred. Existing capture-window, configuration-projection and review-independence limits remain.

An initial full attempt completed default checks but stopped at a companion Docker pull with `cannot overwrite digest`. Its incomplete receipt is retained. Removing the global platform override preserved the existing companion platform while UAR remained pinned to its immutable amd64 child digest; the second complete run passed. No assertions or timeouts were changed.

The supplemental `uar-v1.1.0-amd64-*` JSON assets and `SHA256SUMS-amd64` retain this result, execution provenance, egress observations, protocol-client provenance and the incomplete first attempt. The original arm64 assets and `SHA256SUMS` remain unchanged. The v1.1.0 multi-platform image digest remains `sha256:064f5004fb3ba76506abce5cf81285b2a8aa3e54fa5a22e2c2bfc97b1211e43b`.
