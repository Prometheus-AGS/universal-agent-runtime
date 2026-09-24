# UAR team decision

## Decision
Install seven role definitions with a default three-person working group (lead, relevant specialist, independent verifier); at most four active participants when two disjoint implementation domains exist. This is a reusable roster, not seven mandatory agents. A trivial task can use one implementer followed by independent review.

## Independent alternatives
Candidate A, repository investigator: six Rust-focused roles separating runtime, providers, trust/tools, memory, skills/compiler and verifier; strong graph alignment but leaves frontend and explicit coordination uncovered. Candidate B, skill investigator: five roles combining runtime/providers and trust/persistence, with lead, console and verifier; fewer handoffs but combines distinct high-risk trust and durability work. Candidate C, native harness investigator: four general roles lead/scout/builder/verifier, or three without scout; broadly portable but generic builder ownership obscures UAR contracts. These candidates were generated independently before synthesis.

Choose a lead, runtime (including skills/compiler), providers, trust/tools, state/memory, console and verifier. Separating provider wire contracts, host execution authority and durable state follows actual source boundaries. Combine skills/compiler with runtime because both contribute to runtime capability assembly. Utilization is unmeasured; this is a provisional grouping. Split skills/compiler only if two of five completed tasks require simultaneous independent edits in that domain and runtime orchestration. The console role is justified by direct frontend inspection, not the Rust graph.

The substantive synthesis rests on three independently generated investigator reports retained beside this decision. The separate ideation service returned no branches; its receipt is retained as a limitation and contributes no evidence for this choice.

## Assumptions
The task is reusable project development tooling, not live UAR registration or an autonomous daemon. Ownership is coordination, not sandbox enforcement. Existing KBD position and versions.toml remain binding. Seven prompts need not be loaded at once. Cross-project work is a handoff request, not implicit write permission. Models and skill IDs are grounded in observed catalogs/native docs; prices and inference performance remain unverified. MiniMax executable is missing and its files can only be source-validated here.

## Falsifier
A single-domain task means all requested production edits fall within one specialist ownership set, excluding coordinator records and independent findings. It uses one implementer. If two of five consecutive completed single-domain tasks require a second implementer, replace specialist dispatch with lead/builder/verifier and reassess the ownership map. Any overlapping write assignment must be resolved to one writer before dispatch. Any rejected native role blocks runnable certification for that harness. Record role count, touched domains, elapsed review time and actionable findings in each task receipt; do not infer dollar savings from unknown prices. If the requested independent reviewer is unavailable, implementation may continue but delivery remains pending independent review; never substitute self-certification.

## Validation
Manifest and exports, all skill paths/provenance, preserved existing MCP config, and native parser/discovery where available. No Rust product code changes, so Cargo compilation does not establish acceptance of this tooling change. Independent final artifact review and sycophancy detection must be retained with limitations.

## Operational evidence and acceptance
Read repository-map.md for actual source boundaries and graph failures; team.json supplies exact owned paths. Read handoffs.md for the creator handoff format and peer request/acknowledgement paths. The lead owns coordination; console owns frontend source. Manifest validation must pass the shipped schema; every skill binding must resolve to a project SKILL.md; seven generated native files per harness must match installed bytes. Existing MCP configuration must retain its pre-team digest. Native discovery is a separate receipt and cannot be inferred from parser success.

Concurrency is capped at four including the lead. Each task starts by checking configured model availability and the operator budget; no arbitrary dollar ceiling is invented. Default one review dispatch, at most one corrective re-review; current gateway dispatch is bounded to 120 seconds per attempt. If a native harness lacks a total token budget control, record that limitation instead of promising enforcement.
