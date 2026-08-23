---
sidebar_position: 8
title: Security
description: Entry point for UAR authentication, credential, tenancy, and governance boundaries.
---

# Security

UAR treats authentication, tenant identity, provider credentials, and tool execution as separate trust boundaries. No one control implies the others.

Start with [Authenticate Requests](/docs/security/authentication), the current authority for JWT, JWKS, API keys, anonymous mode, and probe exceptions. Then use:

- [Manage Provider Credentials](/docs/security/credentials) for encrypted, user-scoped provider keys;
- [Understand Tenant Boundaries](/docs/tenancy/overview) for the current A2A-only partition claim;
- [Apply Governance Policies](/docs/governance/overview) for Cedar coverage and its present fallback;
- [Resolve Tool Approvals](/docs/governance/approvals) for permitted-but-sensitive tool calls.

Deployments still own TLS termination, edge authentication, secret custody, storage access, and external service policy. Report vulnerabilities through the repository's [security policy](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/SECURITY.md).
