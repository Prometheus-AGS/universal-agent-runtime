# Design

`TeamHostContext` currently stores a `McpRunResources` whose cloned runtime shares one cache and connector. `apply_host_context` gives that runtime to every team attempt. `RunDelegationLifetime` closes request-owned MCP runtime resources on normal root exit, including a successful `team_wait` yield. A worker or continuation consequently receives the closed coordinator cache.

Keep the existing host-authenticated attachment and canonical binding checks. Capture the `RunMcpServers` already returned by ordinary admission and the resulting immutable owner/environment, rather than retaining its executable cache. For each attempt, invoke `RunMcpServers::resources` with those exact captured inputs and canonical directory. Its existing constructor creates a fresh request-owned cache/connector while preserving server definitions, private authentication and credential lease constraints. Each attempt retains ordinary root-owned cleanup.

No manager, scheduler, global MCP cleanup, policy selection, approval, effect-time fence or durable schema changes are needed. Host-context loss at process restart still requires explicit reattachment. The uncomfortable case remains expired or revoked captured credentials: fresh resource construction is not credential renewal or authority to substitute another destination.
