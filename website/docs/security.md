---
sidebar_position: 8
title: Security
---

# Security

UAR treats authentication, tenant identity, and tool execution as trust
boundaries. Production deployments should terminate public traffic at an edge
gateway and keep UAR on a private service network.

## Authentication and tenancy

- JWT authentication is required by default.
- Startup fails when authentication is enabled with the published fallback
  secret. Set `UAR_SECURITY__JWT_SECRET` to a deliberate value, or configure
  authenticated JWKS verification.
- Configure issuer and audience validation for tokens minted outside UAR.
- Tenant identity is populated only after signature and claim verification.
  An unverified token string never becomes an isolation boundary.
- API-key exchange mints short-lived JWTs through the same configured provider.

For local development, the `uar-jwt-proxy` tool can mint and inject a token. It
is a local convenience and is not an internet-facing authentication gateway.

## Tool governance

MCP-discovered and native tools cross the same server-side schema, policy,
approval, and audit boundary. Cedar decisions can allow, require human
approval, or deny an action. A deny decision is final and cannot be overridden
by approval.

Keep provider credentials out of frontend code and persisted UI state. Use the
encrypted per-user credential store for multi-tenant deployments, configure
trusted origins, and leave file, terminal, web-fetch, and WASM capabilities
disabled unless the deployment explicitly needs them.

Report vulnerabilities through the repository's
[security policy](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/SECURITY.md).
