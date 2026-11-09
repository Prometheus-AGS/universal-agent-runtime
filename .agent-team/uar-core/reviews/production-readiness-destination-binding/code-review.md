# Trusted destination binding source review

Scope: the unstaged diff against the index in `src/uar/runtime/manager.rs` and `src/uar/runtime/turn/bindings.rs`, plus the exact registry, credential, constructor and inherited-binding callers needed to trace it. This is the newly authored destination-binding artifact only. Earlier phase findings and producer handoffs were not consulted.

## Findings

[MEDIUM] Preserve the request-credential fallback path before registry resolution

File: `src/uar/runtime/manager.rs:6817`

The new branch selects registry resolution whenever the ordinary artifact has fallbacks, including when `run_credentials` supplies the destination. With a healthy registered primary and an artifact fallback `private-alias/model` supplied by validated run credentials but absent from the global provider registry, `resolve_to_llm_config` returns `None` and the new `continue` silently removes the authorized fallback before `config_for` runs. If the registry itself is absent, the new `ok_or_else` rejects the entire binding instead. Before this diff, the fallback configuration reached `RunCredentials::config_for`, which supplied its own endpoint, provider kind, wire model and key without requiring global registration.

This is an existing supported distinction in the surrounding code: primary resolution explicitly tolerates an unresolved catalog assignment when request credentials exist (`manager.rs:6475`), and `RunCredentials::from_inputs` validates provider IDs without requiring registry membership. `config_for` is also the existing refusal point for a missing request-credential provider (`turn/host/credentials.rs:166`). The later credential-policy decision cannot restore a fallback already pruned here.

Minimal fix: retain the run-credential configuration path before the new ordinary registry-only branch; resolve ordinary registered fallback configuration when no request credentials provide connection authority. Continue capturing the resulting full configuration and credential decision once. Preserve explicit credential-policy denial rather than falling through to a global source.

Falsifier for the authorized integration gate: configure a healthy registered primary and a request-credential-backed fallback alias absent from the registry; induce the permitted pre-output primary failure. The fallback must remain captured and receive the request at its request-supplied endpoint. A companion missing-credential-provider case must retain its explicit refusal. Neither scenario was executed in this review.

## Limits and command record

All commands were source inspection (`cat`, `sed`, `rg`, `nl`, `pwd`, and scoped `git diff`); the only write is this review artifact. No tests, compilation, lint, formatting, import probes, builds, services, network calls or acceptance scripts ran. Runtime behavior, compilation and selected-container delivery remain unverified. The lead retains the serialized gate.

The review used a fresh teammate context without producer history or prior reviews. It remains same-family model isolation, not an independent cross-family assessment. No public schema or secret values were added to this artifact.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 1 | info |
| LOW | 0 | pass |

Verdict: APPROVE under the severity rubric, with one actionable request-credential regression to resolve before claiming compatibility.
