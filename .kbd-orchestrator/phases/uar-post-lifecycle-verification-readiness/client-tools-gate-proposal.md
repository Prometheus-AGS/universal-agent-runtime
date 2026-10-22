# Caller-tools verification proposal

Phase `uar-post-lifecycle-verification-readiness`, Execute. Source inventory: `client-tools-implementation-inventory.json`. Original13 source tasks and14 caller-tools source tasks complete. No runtime acceptance is inherited.

Operator authorized one isolated source-only review. That review is in progress; its findings and corrections must be reconciled before build approval is requested. All executable gates remain CLOSED.

## Proposed first executable gate (not authorized)

From the UAR repository root, one build writer, no other commands implied:

```bash
CARGO_TARGET_DIR=target/uar-post-lifecycle-main cargo build --locked --no-default-features --features server-full,postgres-backend --bin universal-agent-runtime
```

This compiles the production binary and both persistence implementations. It does not prove tools work, run tests, boot a service or publish anything. Failure stops this gate; repairs require a refreshed source inventory and renewed approval for rerun. Expected artifact path is `target/uar-post-lifecycle-main/debug/universal-agent-runtime`; no artifact exists by this proposal.

## Later production gates

The authored `scripts/release/client_tools_acceptance.py` exercises real HTTP paths. Its exact invocations require actual candidate/provider URLs, separately scoped credential variable names, configured agent/model identifiers, a safe advertised host-tool declaration, durable DB lifecycle bindings, and graph/budget fixture identifiers. These inputs are not fabricated here. Matrix, cold restart, graph and cumulative-budget modes remain pending, as does the prior main-boundary scenario gate. Each complete executable command and service lifecycle action must be presented for authorization after these bindings are concrete. Nothing here approves hidden setup/preflight checks, product tests in GitHub Actions, or publication.

Frozen1.2.0 is unchanged. Any1.2.1 image requires acceptance and separate publication authorization.
