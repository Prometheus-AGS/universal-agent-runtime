# MCP reconnect shared-service verification summary

Scope: local `server-full` on macOS arm64 plus the local Linux arm64 candidate
container. Results transfer to no other runtime profile, platform, deployment,
or GA decision.

- Shared replacement state: PASS. Focused tests show an independently filtered
  view and a pre-existing merged view both use the replacement slot. Upsert also
  propagates its authoritative reconnect configuration through an existing
  filtered view: after A→B, a crash and subsequent reconnect remain on B rather
  than restoring A.
- Crash non-replay: PASS. The installed stream contains one explicit failed
  `resilience__echo` result. The process trace contains one crash in PID 58390,
  followed by a successful echo in replacement PID 58463.
- Timeout non-replay: PASS. The installed stream contains one explicit failed
  result after 30 seconds against the configured 30-second timeout. The trace
  contains one timeout in PID 58463, followed by a successful echo in replacement
  PID 58743.
- Authorization: PASS. Focused assertions retain excluded server/tool views;
  shared transport slots do not share policy maps.
- Negative controls: PASS as fail-closed controls. Success substitution,
  duplicate failed event, duplicate fixture execution, missing transition,
  stale post-crash reuse, and stale post-timeout reuse are all rejected.
- Tier 0 and focused Tier 1: PASS. Cargo check and package-scoped Clippy exit 0;
  registry tests pass 2/0; focused operational tests pass 5/0. Three pre-existing
  warnings remain outside the child edit.
- Installed preflight: PASS. Source
  `f0298d76ea3c39853020c8a33e13f136c07a1806`, candidate
  `operational-resilience-f0298d76ea3c`, duration 60 seconds, outcome passed.
- OpenSpec and workflow policy: PASS. Child and parent validate strictly. The
  local GitHub Actions validator accepts only deployment/documentation deployment
  workflows and no routine product testing.
- Independent review: PASS. A fresh history-free critic and independent judge
  found no reachable implementation, evidence, scope, or artifact-integrity
  blocker after the generation-guarded A-to-B correction.

Source and tooling SHA-256:

```text
7222a8826be0a99640dcc8570bd34b8c0fab0e8b16d5245999f0fa2c8bcf78d8  src/mcp/registry.rs
99cc348efd56f3062da878699bbff0f3fe58d66ce9e25efcf338661446035fb9  scripts/certify-release-candidate.sh
4390b36500e4a671f538a27196f219e46c666975fb98898d8fb77d4e8467d6f0  scripts/validate-mcp-process-boundary-evidence.mjs
d3fafec3dc8d4da8fc4f83ae875fb29b192dd232cb920d31d29151bcd54ed214  scripts/validate-candidate-certification.mjs
e974be9e8d010e0e102a8ed4c330f38cdd0e5bd0cc2d4b30df1034f51613de82  scripts/validate-candidate-certification-workflow.mjs
7b10da070e0558c29d12f8d3789a54a65b02c5a4b032f0df19deab0ff3e594fd  openspec/changes/archive/2026-08-21-fix-mcp-reconnect-shared-service-state/verification.md
```

Uncomfortable result retained: the original installed candidate recorded only
`echo, crash`; later calls never reached a replacement process, and its intended
timeout returned in about 0.2 seconds. This artifact accepts the correction only
because the new raw stream and process trace prove the inverse for the installed
candidate. The parent three-hour soak remains unexecuted and mandatory.
