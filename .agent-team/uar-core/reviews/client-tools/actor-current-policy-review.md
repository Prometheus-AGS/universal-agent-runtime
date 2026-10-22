# Caller-tool actor current-policy source review

Phase: Execute. Reviewed `src/uar/runtime/thread/actor_host.rs` current-policy diff and `.agent-team/uar-core/a2a-current-policy-repair.md`, plus actual manager root/policy resolution, policy fields/resource resolution, A2A execution-policy and owner comparison source. Own report only.

## Disposition

No blocking source finding in the scoped artifact-refresh/policy conversion change. Current-policy HTTP acceptance and executable behavior remain unverified.

- Noninstance caller-tool roots resolve the current registered artifact into `self.artifact`; the next request is built from that refreshed artifact. Lookup failures propagate before root creation. This is not a lookup whose result is discarded. Root scope checks still require the same artifact identity, owner and session.
- These roots do not populate `resolved_policy` from the frozen remote effective policy. `start_hosted_root_turn` supplies no inherited bindings, so the ordinary current global/agent/conversation resolver receives the refreshed artifact and host constraint rather than selecting the previous effective snapshot.
- With an A2A delegation, the ceiling comes from the retained contract's real `execution_policy()`, including the older-contract Presentation deny behavior. Otherwise the conversion explicitly supplies version, chat mode, optional agent/model, five resource selections, memory/cache booleans, context strategy and approval. It does not rely on RunPolicy defaults to replace an authority field. Provenance/warnings are audit fields rather than omitted authority.
- Frozen eligible resource IDs become Selected sets, including empty sets. Actual resource resolution intersects these with current universe and earlier scope restrictions, so newly available resources do not join that frozen delegated set. Memory false remains locked off and approval uses the stricter rank. Scalar route/context/cache fields retain the existing RunPolicy precedence semantics; this patch does not redefine every scalar as boolean conjunction or prove an independent scalar-ceiling algebra.
- Budget, cumulative usage, sandbox and presentation negotiation constraints remain attached. The instance path is excluded from the new refresh/conversion and retains bound artifact, epoch/representation, imported-history rules and execution-time collaboration/instance revalidation. No instance guard was removed.
- Fresh capture is passed into the request for each invocation. `refresh_owner` retains same-principal/namespace equality while replacing the owner value, so fresh verified roles are retained without allowing a different principal. This patch adds no environment credential fallback or cross-principal rebinding.

## Evidence and limits

Only source inspection (`cat`, `sed`, `rg`, scoped `git diff`) and this report write ran. No tests, builds, checks, imports, formatting, compiler probes, runtime or network commands ran. No production files were edited. Global reviewer verification remains CLOSED; the lead owns its scoped local functional test/fix loop. No compile, HTTP changed-policy acceptance, graceful shutdown, CI/merge or publication claim is made.

The separate A2A adapter preclaim current-agent check, ACP selection and persistence work are not accepted by this report. This is same-family isolated source review, not cross-model adversarial evidence.
