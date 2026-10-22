# Fresh agent policy for caller-tool actor roots

Task 13.15; source-only bounded repair. Changed only src/uar/runtime/thread/actor_host.rs.

Plain noninstance ActorThreadSession caller-tool roots now resolve the current registered artifact at every new root, including a warm mailbox continuation. A missing/deleted catalog artifact fails before that root starts. Existing instance-bound artifact, epoch, representation and collaboration revalidation remain on their dedicated path.

For a noninstance caller-tool root with remote constraints, request.resolved_policy is no longer populated from the frozen previous effective policy. The real current policy resolver receives a host_policy_constraint ceiling instead. Where present, this uses the actual retained A2A contract.execution_policy(). Other governed roots convert the captured EffectiveRunPolicy explicitly: version, chat mode, agent/model route, finite eligible skill/tool/MCP/knowledge/presentation IDs, memory/cache enablement, context strategy, and tool approval are retained. Finite eligible IDs become Selected ceilings, so new catalog resources do not gain delegated authority. Provenance/source audit labels have no RunPolicy authority field; their captured eligibility outcome is retained in those finite ceilings. Existing host budgets, cumulative usage grant, sandbox and presentation constraints remain attached.

A2A adapter writer was asked to verify current registered-agent existence before claiming the caller batch; adapter file remains their ownership. ACP selected-agent routing was reported separately and assigned by the lead. Production HTTP changed-agent-policy acceptance is specified in remaining-safety-plan.md and sent to the acceptance writer. This source patch does not claim that acceptance passed.

No tests, builds, checks, imports, probes, formatting, network calls or runtime commands were run. Lead owns the operator-authorized local functional loop and source/security review. Frozen and published release tags remain unchanged.
