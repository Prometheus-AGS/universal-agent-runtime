Prometheus Meta‑Prompting Orchestration (PMPO)

Version: v1.0 (UAR Integration Edition)
Date: 2026‑02‑18
Applies to: UAR, UAR‑AGENT‑MD v1.1, UAR‑GOV v1.0, and the uar compile agent.md pipeline

⸻

1. What PMPO Is (Precise Definition)

Prometheus Meta‑Prompting Orchestration (PMPO) is a repeatable, specification‑driven control loop for transforming human-authored intent artifacts (documents, specs, templates) into validated, executable system outputs through a disciplined sequence of:
	1.	Spec — Normalize intent into a strict, machine-checkable artifact with explicit contracts.
	2.	Plan (Tasks) — Decompose work into an ordered, typed task graph with deterministic acceptance criteria.
	3.	Execute — Perform tasks using tools/runtimes with strong invariants and audit trails.
	4.	Reflect — Verify outcomes against acceptance criteria, analyze failures, and produce deltas for the next loop.

PMPO is not “prompt engineering.” It is a meta‑orchestration methodology that:
	•	treats documents as inputs to compilation
	•	requires formal constraints and validation gates
	•	produces actionable artifacts (binaries, configs, signed descriptors)
	•	closes the loop with measured outcomes and structured improvement

In the UAR context, PMPO is the methodology that turns UAR‑AGENT‑MD documents into running, governed agents via a compiler pipeline.

⸻

2. Why PMPO Matters for UAR

UAR is explicitly spec‑first: behavior is described in documents (UAR‑AGENT‑MD, UAR‑GOV) and enforced by a runtime (actors, Wasm, Cedar PEPs). PMPO is the operational bridge between:
	•	human-authored specifications (agent.md)
	•	deterministic enforcement (Cedar + PEP)
	•	executable runtime entities (AgentActor + endpoints + skills)

PMPO ensures the conversion is:
	•	repeatable (same input → same compiled descriptor)
	•	auditable (every decision logged)
	•	secure (fail-closed gates)
	•	composable (task graph, not ad-hoc steps)

⸻

3. PMPO Canonical Loop

3.1 Spec

Objective: Convert ambiguous intent into a strict artifact with contracts.

Inputs: agent.md (+ referenced schemas/policies)

Outputs:
	•	a parsed and normalized AgentDescriptor IR (Intermediate Representation)
	•	a list of referenced dependencies and required resolution (A2UI schemas, JSON Schemas, MCP endpoints)
	•	a strict Validation Contract: what must be true for compilation to proceed

3.2 Plan (Tasks)

Objective: Produce a deterministic task plan with acceptance criteria.

Outputs:
	•	a task graph (ordered steps)
	•	each task has:
	•	required inputs
	•	tool/routine used
	•	acceptance checks
	•	failure modes and remediation hints

3.3 Execute

Objective: Run the plan against the artifact.

Outputs:
	•	compiled Cedar policy
	•	validated IO schemas
	•	resolved MCP config
	•	registered A2A endpoints
	•	installed PEP bindings
	•	emitted signed runtime descriptor

3.4 Reflect

Objective: Verify and improve.

Outputs:
	•	pass/fail per stage
	•	detected drift (spec vs runtime)
	•	recommended changes to:
	•	the agent.md
	•	the compiler
	•	schema registries
	•	governance rules

Reflection feeds the next PMPO loop.

⸻

4. Applying PMPO to UAR‑AGENT‑MD → Actionable UAR Agent

This section describes precisely how PMPO processes agent.md (UAR‑AGENT‑MD v1.1) into a running agent.

4.1 Inputs
	•	agent.md (UAR‑AGENT‑MD v1.1)
	•	referenced A2UI schemas (by ID/version)
	•	referenced JSON Schemas (input/output)
	•	embedded Cedar policy text
	•	MCP server declarations (with auth references)
	•	runtime profile selection (optional)

4.2 Outputs
	•	agent.descriptor.json (normalized, runtime-ready)
	•	policy.compiled (Cedar authorizer artifacts or cached compiled representation)
	•	routes.a2a.json (actor routing registrations)
	•	pep.bindings.json (capability → enforcement map)
	•	agent.descriptor.sig (signature)
	•	compile.report.json (audit, stage results)

⸻

5. PMPO → UAR Compilation Pipeline Mapping

UAR‑AGENT‑MD §19 defines 8 stages. PMPO wraps these as a strict orchestration loop.

5.1 Spec Phase → Stages 01–02 (Normalization & Contracts)

Spec is “make it unambiguous.” In UAR compilation this means:
	•	Stage 01: Validate Frontmatter
	•	Normalize versioning, schema compatibility, runtime constraints
	•	Produce IR header and compatibility contract
	•	Stage 02: Validate A2UI Schemas
	•	Resolve ui.artifacts[*].schema against registry
	•	Resolve ui.actions[*].input_schema/output_schema
	•	Produce UI contract: actions are typed and bindable

Spec Output Artifact: agent.ir.json (normalized IR + unresolved references)

5.2 Plan Phase → Create Deterministic Task Graph

PMPO converts the 8 stages into tasks with explicit acceptance criteria.

Task Graph (canonical):
	1.	Parse markdown → structural sections present
	2.	Validate frontmatter compatibility
	3.	Merge deployment profile overrides
	4.	Resolve A2UI schema IDs → schema documents
	5.	Resolve schema files → JSON Schema parse
	6.	Validate MCP server config + auth references
	7.	Validate A2A endpoints reference valid schemas
	8.	Compile Cedar policy + validate entity/action references
	9.	Bind endpoints into actor routing table
	10.	Derive PEP bindings for every declared capability surface
	11.	Emit descriptor + signature + compile report

Each task produces a small artifact and a success predicate.

5.3 Execute Phase → Stages 03–08 (Build the Runtime Agent)
	•	Stage 03: Validate MCP Server Config
	•	Verify endpoints, auth types, env var presence (or secret provider mapping)
	•	Ensure tool capability cross-refs match declared MCP servers
	•	Stage 04: Validate A2A JSON Schemas
	•	Parse input/output JSON Schema
	•	Enforce structural constraints (no ambiguous any / unknown formats if disallowed)
	•	Record schema fingerprints for audit and caching
	•	Stage 05: Compile Cedar Policy
	•	Parse embedded Cedar
	•	Type-check entity/action/resource usage
	•	Compile to authorizer-ready representation
	•	Ensure policy covers all required sensitive actions (or fail-closed)
	•	Stage 06: Register Actor Endpoints
	•	Bind a2a.endpoints[*].id into the actor routing registry
	•	Attach schema IDs and timeouts
	•	Attach caller restrictions
	•	Stage 07: Install PEP Enforcement
	•	For each capability surface (LLM, tools, file, net, plugin load, a2a.call):
	•	install a PEP gate
	•	ensure requests build (principal, action, resource, context)
	•	Attach obligations (budgets, rate limits, allowlists)
	•	Stage 08: Emit Signed Descriptor
	•	Emit canonical JSON descriptor (stable ordering)
	•	Sign descriptor + referenced fingerprints
	•	Emit compile report with stage results

5.4 Reflect Phase → Verification & Improvement

Reflection is where PMPO becomes a continuous quality engine rather than a one-off pipeline.

Reflection checks include:
	•	Determinism: same inputs → same descriptor hash
	•	Completeness: every declared capability has a corresponding PEP binding
	•	Governance coverage: all sensitive actions have explicit policy rules (or explicit deny)
	•	Schema coherence:
	•	UI actions ↔ skills/A2A endpoints must bind
	•	A2A endpoints ↔ schema files must exist and validate
	•	Operational readiness:
	•	MCP server auth references resolvable
	•	profile overrides do not violate max budgets

Reflection output:
	•	compile.report.json with:
	•	stage-by-stage verdicts
	•	reason codes
	•	remediation suggestions
	•	optional patch suggestions to agent.md (non-authoritative)

⸻

6. PMPO Acceptance Criteria (What “Done” Means)

An agent is considered actionable when all criteria are satisfied:
	1.	Descriptor validity: canonical descriptor parses and matches UAR domain types.
	2.	UI contract validity: every A2UI reference resolves; every UI action has typed IO.
	3.	MCP readiness: all declared MCP servers are structurally valid and capability-matched.
	4.	A2A contract validity: every endpoint has valid JSON Schema IO.
	5.	Governance compiled: Cedar policy compiles and references known actions/resources.
	6.	PEP installed: all sensitive boundaries are guarded.
	7.	Signature present: descriptor and fingerprints are signed.
	8.	Runtime load test: agent can be loaded and registered without warnings.

⸻

7. What the PMPO Loop Looks Like in Practice

7.1 Example: Converting agent.md into a Running Agent

Spec
	•	UAR parses agent.md → IR
	•	Resolves schema references and policy text

Plan
	•	Emits compile.plan.json with tasks & acceptance tests

Execute
	•	Runs stages 01–08
	•	Emits descriptor + signatures

Reflect
	•	Generates a compile report
	•	Suggests minimal changes (e.g., missing policy rule for net.http.request)

Then the next PMPO loop begins with updated inputs.

⸻

8. PMPO Deliverables for the UAR Tooling Surface

PMPO becomes concrete inside UAR via CLI and APIs.

8.1 CLI Commands
	•	uar compile agent.md — runs PMPO over the spec and emits signed descriptor
	•	uar compile --plan-only agent.md — emits task graph without execution
	•	uar compile --reflect agent.md — runs verification against prior artifacts

8.2 Debug / Governance
	•	uar policy simulate agent.md --action llm.invoke --resource model:gpt-4o
	•	uar policy explain  agent.md --action net.http.request --resource https://example.com

8.3 Artifact Outputs
	•	dist/<agent-id>/<version>/descriptor.json
	•	dist/<agent-id>/<version>/descriptor.sig
	•	dist/<agent-id>/<version>/compile.report.json
	•	dist/<agent-id>/<version>/fingerprints.json

⸻

9. Summary

PMPO is the operational methodology that makes UAR “specification-driven” in practice.
	•	The spec phase ensures the agent document is unambiguous and contract-complete.
	•	The plan phase turns compilation into a deterministic task graph with acceptance criteria.
	•	The execute phase runs the UAR‑AGENT‑MD pipeline stages 01–08 to produce a signed descriptor.
	•	The reflect phase verifies determinism, governance coverage, and bindability, feeding improvements into the next loop.

This is how a UAR‑AGENT‑MD document becomes an actionable, governed AgentActor in the Universal Agent Runtime.