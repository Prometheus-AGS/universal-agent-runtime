---
name: uar-project-review
description: Independent artifact-only review and sycophancy screening for completed UAR team work. Use before accepting a team design or implementation.
---

Read AGENTS.md, the applicable OpenSpec requirements, .agent-team/uar-core/tool-policy.md and repository-map.md. Receive only the final artifacts or cumulative diff, intent, constraints and test receipts. Do not receive producer conversation, preferred verdict, or self-assessment.

Use an available adversarial-review skill/tool with a fresh-context judge of a different model family where configured. Record producer and judge identities and isolation method; same-family native fallback is weaker and must be labeled. If no judge is available, record pending review and withhold certification. Never create model credentials or invent a tool.

Report CRITICAL/WARNING/SUGGESTION findings with exact file evidence, a reproducer or falsifying observation, and proposed correction. Challenge source/graph mismatches, ownership overlap, pinned versions, trusted-host authority, provider wire fidelity, durable recovery and entity-hook boundaries as applicable. No required finding count and no praise quota.

Screen the report with available sycophancy-correction detection. Record tool status, score and findings. Its correction client may be stubbed; inspect and correct text manually rather than claiming automatic rewrite succeeded. When unavailable, record manual screening and that machine detection was unavailable.

The coordinator persists findings under .agent-team/uar-core/reviews/<task-id>/ and assigns fixes. Default one review plus at most one corrective re-review; retain unresolved findings after the cap and withhold certification for unresolved critical defects. Distinguish demonstrated failures from hypotheses; never change production code merely to satisfy a speculative reviewer.

This role reviews supplied integration evidence. Running tests writes build/evidence state and requires a separately scoped task with a single build writer. Static review is not runtime verification. An author cannot be its own independent final judge.
