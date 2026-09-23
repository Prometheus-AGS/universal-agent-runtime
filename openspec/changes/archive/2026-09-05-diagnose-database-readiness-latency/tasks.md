## 1. Identity and collector preparation

- [x] 1.1 Establish diagnostic identity: deliver evidence/identity.json and execution.md with timestamped installed/source identity, safe effective target/timeout provenance and explicit unresolved fields; verify identity agreement before allowing probes, or record why active probing is skipped.
- [x] 1.2 Prepare the dependency-free collector inside the KBD child: verify local capabilities and protocol, review the fixed allowlist, privacy/size/deadline limits and stop conditions, and pass a syntax-only check; if safe collection is unsupported, record the reason without installing dependencies or changing UAR.

## 2. Observation and attribution

- [x] 2.1 Collect bounded observations only after Execute authorization and identity/collector gates: deliver evidence/observations.json and exact sanitized command/outcome records matching the plan's request, time and size ceilings; verify attempted/skipped counts, clock scopes, stop conditions and absence of secret/business payloads. If a prerequisite is unavailable, explicitly record skipped comparisons rather than running unsafe probes.
- [x] 2.2 Produce diagnosis.md: verify all five hypothesis families have supporting/disconfirming evidence, supported/contradicted-in-sample/unresolved dispositions and limitations; deliver the smallest justified remediation proposal or next required evidence/authority without implementing it or claiming unobserved recovery.

## 3. Independent acceptance

- [x] 3.1 Independently review diagnosis and evidence in fresh artifact-only context; resolve critical findings or retain a blocked disposition, and deliver verification.md plus handoff-out.md mapping the diagnostic spec to observed evidence. Verify no product/source/service/database mutation or product test suite occurred, run kbd-status, and enter Reflect only after Execute acceptance; unresolved causal goals must remain unmet.

All deliverable paths above are relative to the active diagnostic KBD child.
Task order is strictly 1.1 → 1.2 → 2.1 → 2.2 → 3.1. The reviewed plan.md is the
bounded observation contract; a prerequisite failure permits documented skipped
work and an inconclusive report, never manufactured runtime evidence.

Run kbd-status after each completed task/change/phase. No tool workflow code is
changed: these ordinary Markdown/JSON artifacts are shared inputs for Codex,
Claude Code, Cursor and OpenCode; no harness-specific integration validation is
claimed. No product tests run at intermediate tasks or in GitHub Actions.
