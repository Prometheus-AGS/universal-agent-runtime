RFC-0001 — UAR Agent Definition Specification Standard

Status: Draft
Author: Prometheus Architecture Group
Created: 2026-02-18
Applies To: Universal Agent Runtime (UAR)
Supersedes: None
Related Documents: UAR-AGENT-MD v1.1, UAR-GOV v1.0, PMPO v1.0

⸻

1. Abstract

This RFC formally defines the standard specification format for agent definition within the Universal Agent Runtime (UAR). The standard describes the structure, semantics, validation rules, and compilation requirements for agent artifacts written in Markdown under the UAR-AGENT-MD schema.

The objective of this RFC is to:
	•	Establish a stable, versioned specification contract for agent artifacts
	•	Define deterministic compilation semantics
	•	Formalize governance, UI, MCP, and A2A contract requirements
	•	Enable interoperability across runtimes, registries, and tooling

This RFC does not define runtime implementation details. It defines the specification standard that runtimes MUST follow.

⸻

2. Terminology

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, and MAY are to be interpreted as described in RFC 2119.

Agent Artifact — A Markdown document conforming to the UAR-AGENT-MD schema.

Descriptor — The normalized, canonical JSON representation produced by compilation.

Compilation — The deterministic transformation of an Agent Artifact into a signed Descriptor.

Capability Surface — Any runtime boundary requiring policy enforcement (LLM, tool, file, net, plugin, A2A).

⸻

3. Goals
	1.	Provide a deterministic agent definition format.
	2.	Enable governance-first execution via Cedar policies.
	3.	Support declarative UI contracts (A2UI).
	4.	Support explicit MCP infrastructure dependencies.
	5.	Support typed Agent-to-Agent contracts.
	6.	Enable portable, local-first deployment.

⸻

4. Non-Goals
	•	Defining the internal architecture of UAR
	•	Defining UI rendering implementations
	•	Defining MCP protocol details
	•	Replacing external IAM systems

⸻

5. Specification Structure Requirements

An Agent Artifact MUST contain the following top-level sections in Markdown:

# Agent: <name>

## Metadata
## Identity
## UI (A2UI)
## Capabilities
## Skills
## Tools
## MCP Servers
## Knowledge Base
## Memory Model
## A2A Contracts
## Governance
## Budgets & Constraints
## Execution Model
## Observability
## Deployment Profiles

All sections are REQUIRED.

⸻

6. Metadata Requirements

The ## Metadata section MUST contain valid YAML frontmatter with:
	•	id (string, globally unique)
	•	version (semver)
	•	runtime (compatibility constraint)
	•	schema (UAR-AGENT-MD version)

Optional fields MAY include owner, license, checksum, ui_schema_version.

Compilation MUST fail if required fields are missing or incompatible.

⸻

7. Identity Requirements

The ## Identity section MUST define a principal object mapping to governance entities.

Minimum fields:
	•	type
	•	namespace

Identity attributes MUST be compatible with Cedar entity schemas.

⸻

8. UI (A2UI) Requirements

The ## UI (A2UI) section MUST:
	•	Declare artifacts referencing versioned A2UI schema IDs
	•	Declare UI actions with JSON Schema input_schema and output_schema

Compilation MUST verify:
	•	All A2UI schemas resolve via registry
	•	All schema references are valid JSON Schema
	•	All UI actions bind to either skills or A2A endpoints

⸻

9. Capability Declaration

The ## Capabilities section MUST declare all requested runtime surfaces.

Capabilities are declarations only and MUST NOT imply permission.

Each declared capability MUST have a corresponding governance evaluation path.

⸻

10. Skills Requirements

Skills MUST declare:
	•	id
	•	entrypoint
	•	required_tools

Compilation MUST verify:
	•	Referenced tools exist
	•	Entrypoints are valid (compile-time or runtime resolvable)

⸻

11. Tools Requirements

Tools MUST explicitly declare:
	•	id
	•	type

Additional constraints (models, domains, etc.) SHOULD be declared.

Tools referencing MCP capabilities MUST match declared MCP servers.

⸻

12. MCP Server Declaration

The ## MCP Servers section MUST:
	•	Declare id
	•	Declare endpoint
	•	Declare authentication method
	•	Declare supported capabilities

Compilation MUST:
	•	Validate structural correctness
	•	Ensure tool-to-capability mapping integrity
	•	Fail if undeclared MCP capability is referenced

⸻

13. Knowledge & Memory

Knowledge references MUST be symbolic and resolved at runtime.

Memory declarations MUST define isolation scope.

Compilation MUST ensure no implicit cross-tenant access is possible.

⸻

14. A2A Contract Requirements

Each endpoint MUST declare:
	•	id
	•	input_schema
	•	output_schema

Schemas MUST be valid JSON Schema.

Compilation MUST register endpoints into the actor routing layer.

Allowed callers MUST be enforced via governance.

⸻

15. Governance Requirements

The ## Governance section MUST embed Cedar-compatible policy text.

Compilation MUST:
	1.	Parse policy
	2.	Type-check entity references
	3.	Validate referenced actions/resources
	4.	Ensure all sensitive capability surfaces have explicit policy coverage or explicit denial

Compilation MUST fail if governance is incomplete or invalid.

⸻

16. Budgets & Constraints

Budgets MUST define explicit ceilings.

Compilation MUST ensure:
	•	Budgets do not exceed deployment maximums
	•	Budget context is injected into governance evaluation

⸻

17. Execution Model

Execution configuration MUST define:
	•	model type (actor/supervised/etc.)
	•	concurrency limits
	•	timeout limits

Invalid execution models MUST fail compilation.

⸻

18. Observability

Observability configuration MUST specify audit requirements.

If audit is required, compilation MUST ensure logging pipeline is enabled.

⸻

19. Deployment Profiles

Profiles MAY override budgets and configuration.

Compilation MUST:
	•	Merge profile settings deterministically
	•	Validate resulting configuration against constraints

⸻

20. Deterministic Compilation Requirements

The compilation process MUST:
	1.	Be deterministic (identical input → identical descriptor hash)
	2.	Fail closed on validation error
	3.	Emit canonical JSON descriptor
	4.	Emit signature over descriptor and referenced fingerprints

⸻

21. Security Requirements

An implementation conforming to this RFC MUST ensure:
	•	No implicit capability grants
	•	No bypass paths around governance
	•	No implicit network or file access
	•	Capability surfaces are guarded by PEP

⸻

22. Versioning

This specification SHALL follow semantic versioning.

Breaking changes require:
	•	New schema version
	•	Explicit migration documentation

⸻

23. Conformance

A runtime implementation is compliant with this RFC if:
	•	It accepts valid UAR-AGENT-MD documents
	•	It rejects invalid documents
	•	It produces deterministic descriptors
	•	It enforces governance at declared capability surfaces

⸻

24. Future Work
	•	Formal JSON Schema publication for UAR-AGENT-MD
	•	Descriptor registry standard
	•	Remote attestation standard
	•	Policy Cards interoperability

⸻

25. Conclusion

This RFC formalizes the UAR Agent Definition Standard as a deterministic, governance-first, specification-driven artifact model.

By codifying agent definition as a strict document standard, UAR becomes a compiler-based agent operating system, not an ad-hoc framework.

The agent definition is no longer configuration — it is a contract.