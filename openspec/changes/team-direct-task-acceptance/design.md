## Context

The observed packaged operation used Boss 48cfc445 and native UAR e2fd557f. Preserved diagnostic `c14-52aef1b3-3c4c-49f5-91e1-6effd67bc5d9/diagnostic-alert-v2.json` records `POST /api/v1/collaboration/packages:preflight` HTTP 422 (`collaboration_invalid`). Boss `uarCodingTeamPackage.ts:83` emits coordinator-within-binding with `allowedWorkflows: []`; draft.2 `team-definition.schema.json:160` requires `minItems: 1`. The diagnostic does not expose a native field pointer; the incompatible authored field is identified from exact source.

## Decisions

Set the array's base minimum to zero, then express the mode-specific condition with two mutually exclusive `oneOf` branches: `coordinator-within-binding` retains minimum zero and `operator` requires minimum one. Use single-value `enum` constraints in each branch. The lead reports that full/mini profile validators support `oneOf` but silently ignore `if`/`then`/`allOf`; the canonical condition must use the supported construct without changing those validators. Keep the common enum, required keys, immutable-reference item schema and closed object unchanged. Do not add a dummy WorkflowDefinition or relax operator mode. Leave the immutable draft.1 predecessor unchanged.

Document the distinction in draft.2's normative TeamDefinition section. Schema acceptance remains distinct from activation; the repaired package has no newly accepted workflow references.

## Existing authority and readers

`validation/schema.rs:267` requires an array without requiring an entry. `validation/graph.rs:195` validates each supplied workflow reference and `:278` collects each dependency; both already handle an empty list. `team_planning.rs:239` checks materialized roles, limits and same-board dependencies for direct tasks. `team_execution/peer_commands.rs:163` admits coordinator delegation only for the matching coordinator role and coordinator-within-binding mode. `workflow_execution/start.rs:92` still requires exact immutable allowlist membership, so an empty list accepts no workflow. None of these runtime files changes.

## Risks / Trade-offs

An older runtime still rejects this previously unrepresentable direct-only definition; deploy the exact rebuilt native source before retrying it. Existing valid definitions and digests remain unchanged. The uncomfortable boundary is that this source mismatch and its repair do not prove this was the only rejection or that subsequent binding/model/tool execution succeeds. The lead owns the serialized native rebuild, Boss pin/package and real failed-operation rerun. Worker checks, builds, tests and review are explicitly deferred; commit/push hooks are suppressed by invocation-local `core.hooksPath=/dev/null` under the approved boundary instruction, not treated as passing evidence.
