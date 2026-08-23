---
sidebar_position: 1
title: Apply Governance Policies
description: Understand Cedar coverage, runtime decisions, and present fail-open limits.
source_records:
  - openspec/specs/skill-governance/spec.md
  - openspec/specs/tool-approval-workflow/spec.md
current_authority: /docs/governance/overview
---

# Apply Governance Policies

UAR can evaluate Cedar policies at HTTP and tool-execution boundaries, then produce allow, approval-required, or deny decisions.

:::danger Boundary statement
Governance coverage depends on the build profile and request path. The current server also has a documented permit-all fallback when policy files fail to load. Do not present the runtime as universally fail closed.
:::

## Build-profile behavior

| Profile | Cedar engine | Branded governance UI | Current boundary |
|---|---|---|---|
| `server-full` | Included | Included with the admin application | HTTP requests carrying `X-Agent-Id`, orchestrated tool calls, and explicit skill-mutation checks |
| `minimal` | Not compiled | Not included | Generic actions, tool decisions, and skill mutation use a capability-disabled permit baseline; runtime risk can still require approval |
| `embedded-mobile` | Not compiled | Not included | The embedding host owns policy enforcement outside any shared runtime interfaces |

These are profile limits, not equivalent governance certifications.

## Policy directory and startup behavior

The packaged `server-full` process loads `*.cedar` files from the `policies` directory. A directory with no policy files produces an empty policy set, and Cedar denies evaluated requests because no permit applies.

The uncomfortable case is a read or parse error: server composition logs the failure and constructs a permit-all fallback. This preserves startup but weakens enforcement. Deployment monitoring must treat that warning as a security event. A successful process start is not proof that the intended policies loaded.

The engine supports replacing its active policy set through its reload method. The packaged HTTP server does not expose a dedicated public policy-reload endpoint, so operators should treat restart with validated files as the normal activation path unless their embedding integration explicitly invokes reload.

## HTTP governance boundary

The middleware evaluates only requests that carry `X-Agent-Id`. Requests without that header pass through this agent-level policy layer. For evaluated requests, UAR derives an action from the method and route and a resource from the path. A Cedar denial returns HTTP 403 with `GOVERNANCE_DENIED`.

This is not a replacement for [authentication](/docs/security/authentication). The header identifies the agent to the policy layer; its mere presence is not a verified tenant credential.

## Tool decisions

At the orchestrator boundary, a tool decision has three outcomes:

- **Allow**: execute immediately when policy permits and runtime risk does not require review.
- **Require approval**: pause and ask a human when policy permits but risk or the effective run policy requires review.
- **Deny**: reject immediately. Human approval cannot reverse it.

When Cedar is absent, the disabled facade permits the policy portion while retaining risk-based approval. See [tool approvals](/docs/governance/approvals) for the exact wait and terminal behavior.

## Skill mutation

The Cedar-enabled engine exposes an explicit skill mutation check with environment, confidence, validation, human-review, and governance context. A build without Cedar permits this policy check. The existence of the method does not prove that every path that can modify a file or external skill registry passes through it; coverage must be confirmed at each caller.

## Audit limits

Policy decisions are visible through logs and runtime events at the wired boundaries. Those signals are operational evidence, not an immutable, signed audit ledger. Retention and export depend on the deployment's log and telemetry systems. A policy file on disk, a loaded policy count, and an observed denial answer three different questions; retain evidence for each requirement you claim.

## Profile limits

Only `server-full` carries the Cedar governance claim. `minimal` and `embedded-mobile` are explicit exclusions. Even in `server-full`, requests without `X-Agent-Id`, the policy-load permit-all fallback, and ungoverned external systems remain outside the claim.

Next, learn how [approvals](/docs/governance/approvals) resolve a permitted-but-sensitive tool call.
