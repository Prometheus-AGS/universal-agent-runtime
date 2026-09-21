# Plan research and concrete decisions

Date: 2026-09-16. This supplement resolves the five external-candidate survey prerequisites in the Analyze handoff. It does not rewrite that historical artifact. Candidate IDs cand-001–006 retain their earlier meaning; cand-007–011 are in plan-library-candidates.json.

## Build-versus-reuse outcomes

| Gap | External option compared with existing host | Decision and integration cost |
|---|---|---|
| F4 | Official A2A Python SDK versus UAR thread service and Memory/Postgres/Surreal providers | Reference cand-007, adapt cand-001. A second Python service would require another identity/credential/persistence boundary. Persist acceptance and correlation in existing host providers. |
| F6 | OpenTelemetry agent/tool conventions versus existing RuntimeStep and tracing/OTLP | Reference cand-008, adapt cand-001. Export conventions do not supply ordered lifecycle truth. Extend existing event records rather than install another service. |
| F5 | Anthropic skill-creator evaluation versus current activation/exhaustive oracle | Reference cand-009, adapt cand-001. Reuse experimental method, not Claude-specific runner scripts. Preserve UAR scopes, host permissions and existing Recall@10 gate. |
| F7 | Ragas versus current lexical verification and governed Liter | Reference cand-010, adapt cand-001. Claim-level support is useful; access at use and evidence versions belong to UAR. No Python package or alternate inference client. |
| F6/F8 | Inspect AI versus Rust completion/eval CLI | Reference cand-011, adapt cand-001. Add a production-host evaluation target and retain CLI/baseline format. An external runner is not required for the stated oracles. |

Primary sources and licenses are attached to each candidate. Repository metadata was read from the corresponding GitHub API: A2A, OTel semantic-conventions, Ragas report Apache-2.0; Inspect reports MIT. Anthropic root license metadata was null, so its skill-local LICENSE.txt was checked. No upstream code is copied, package installed, version changed or API compatibility promised. Current-main documentation is methodological evidence, not a pinned executable dependency contract. The moved OTel page and one failed GitHub page fetch were resolved through the official repository/source; no unsupported stability claim is made.

## Chosen mechanisms

1. **Durable lifecycle:** extend the existing persistence trait with an owner/agent/request-identity acceptance record and task/context/thread/run/artifact mapping. Use a unique owner-scoped message identity plus payload digest; same identity/different payload is conflict. Persist accepted intent before enqueue. Host execution claims it with revision compare-and-swap; restart reconciles intent versus existing run/receipt. An uncertain external effect remains recovery-required. Do not claim database transactions span a remote tool. Memory implements matching semantics but cannot certify restart durability; Postgres and Surreal must each pass crash fixtures.
2. **Graph observations:** extend existing NormalizedEvent::RuntimeStep projections with optional node/iteration/parent/child/tool correlation and committed sequence identity. Existing stream resume remains replay-only. Keep authoritative outcomes in host records; tracing/OTLP is a redacted projection, not a second state store.
3. **Skills:** retain scope resolution and explicit governed activation. Evaluate selection against exhaustive labeled candidates and success against task assertions; store immutable skill version/body for reattachment before final counting. Held-out certification cases are never used for description/ranker tuning.
4. **Evidence:** retain lexical overlap only as retrieval diagnostics. Through governed Liter, extract bounded atomic answer claims and request typed support/contradiction/unknown decisions against currently authorized source versions. Validate source IDs, offsets, quoted spans and version/access deterministically; invalid references or parser failure yield insufficient evidence. Conflicting evidence outranks an unqualified support claim; superseded evidence is stale. Semantic model judgments are explicitly advisory and method-labeled, not proof of truth. They cannot bypass access/integrity checks or independently pass the deterministic regression gate.
5. **Evaluation:** add a host-target adapter to the existing runner with recorded provider/tool fixtures, governed approvals and trace scorer inputs. Deterministic assertions decide hard regression outcomes. Live model judgments and measured quality/cost/latency are retained separately. Baseline updates are deliberate artifacts, not a certification shortcut.

## Preserved architectural reuse

library: cand-001 applies to every change. library: cand-002 applies to protected text counting, retaining tiktoken-rs0.12.0. Liter remains1.18.2 at the versions.toml pin; its actual request types are inspected before implementation. No arbitrary Jinja engine is selected. Context planning is pure; all summarizer calls and receipt writes occur in the trusted host.

## Limits that research cannot resolve by assertion

The installed YAML supplies six enabled provider defaults, but durable settings overrides and actual upstream bindings remain unverified. provider-profile-matrix.json names each observed deployment and its missing evidence; configured does not mean certified. The exact two historical routing failures were not identifiable in retained narrative reports; the three current routing cases are listed as a reproduction cohort, not asserted to be those two failures.

No runtime baseline was executed in Plan. The evaluation contract freezes source revision, input cases, labels, thresholds and measurement protocol. Baseline outputs must be captured from that frozen source before modifying the corresponding behavior. A failed or unavailable baseline is retained as evidence, never filled in with invented numbers.

The uncomfortable trade-off: these reference projects help choose the method but none proves the UAR-specific integration. Correct planning still requires production-path tests and may block enabling a deployment whose counting contract is unknown.
