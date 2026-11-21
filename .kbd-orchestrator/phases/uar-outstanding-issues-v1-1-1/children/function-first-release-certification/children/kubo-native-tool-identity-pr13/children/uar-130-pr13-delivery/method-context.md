# Derived method context

Method: VLBV (pack-default); phase: uar-outstanding-issues-v1-1-1::function-first-release-certification::kubo-native-tool-identity-pr13::uar-130-pr13-delivery; revision: 3436; status: BLOCKED.

This is a recovery index, not execution permission. Resolve current canonical decisions and contract before dependent action.

```json
{
  "schemaVersion": 1,
  "kind": "kbd.derived-method-context",
  "generatedAt": "2026-11-19T14:52:25.529Z",
  "authority": "derived-read-only-index; canonical signed decisions, phase binding, contract and governing host policy remain authority",
  "projectId": "c8d33ac7-e3cd-4db6-b938-1425ec19ee9a",
  "runId": "fix-runtime-settings-namespace-routes-20260825T091750Z",
  "phaseId": "uar-outstanding-issues-v1-1-1::function-first-release-certification::kubo-native-tool-identity-pr13::uar-130-pr13-delivery",
  "sourceRevision": 3436,
  "phaseDirectory": ".kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/children/function-first-release-certification/children/kubo-native-tool-identity-pr13/children/uar-130-pr13-delivery",
  "contract": {
    "path": ".kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/children/function-first-release-certification/children/kubo-native-tool-identity-pr13/children/uar-130-pr13-delivery/adopted-contract.json",
    "observation": "missing",
    "sha256": null,
    "contractDigest": null,
    "binding": null,
    "identity": null
  },
  "selectedMethod": "VLBV",
  "methodSource": "pack-default",
  "deviation": null,
  "methodDecision": null,
  "retiredMethods": [],
  "references": [],
  "includedValue": [],
  "exclusions": [],
  "stage": "execute",
  "stageAuthority": {
    "operations": [
      "author agreed production without generated-code verification"
    ],
    "implementationComplete": false,
    "verificationAuthority": "separately authorized whole-value-phase boundary required; this index grants none",
    "executableTestAuthoring": "governed by host policy; where that policy is production-first, defer until the complete agreed production work"
  },
  "governingInstructions": [
    {
      "path": "AGENTS.md",
      "observation": "observed",
      "sha256": "4f42bff65e3fb43a46f7c8e764aac6bf66a7bae47976f4f62bb277a4af9e21b0"
    },
    {
      "path": "CLAUDE.md",
      "observation": "observed",
      "sha256": "4f42bff65e3fb43a46f7c8e764aac6bf66a7bae47976f4f62bb277a4af9e21b0"
    },
    {
      "path": ".kbd-orchestrator/constraints.md",
      "observation": "missing",
      "sha256": null
    }
  ],
  "effectiveRules": {
    "harness": null,
    "profile": "not-found",
    "files": [
      "AGENTS.md",
      "CLAUDE.md",
      ".kbd-orchestrator/constraints.md"
    ],
    "shadowing": null
  },
  "policyConflicts": [
    {
      "path": "AGENTS.md",
      "sha256": "4f42bff65e3fb43a46f7c8e764aac6bf66a7bae47976f4f62bb277a4af9e21b0",
      "issue": "Host compiler or cadence exception conflicts with VLBV; do not infer permission to run generated-code checks during generation.",
      "acknowledged": false,
      "ackDecisionId": null
    },
    {
      "path": "CLAUDE.md",
      "sha256": "4f42bff65e3fb43a46f7c8e764aac6bf66a7bae47976f4f62bb277a4af9e21b0",
      "issue": "Host compiler or cadence exception conflicts with VLBV; do not infer permission to run generated-code checks during generation.",
      "acknowledged": false,
      "ackDecisionId": null
    }
  ],
  "candidateAndEvidence": {
    "freshness": "UNKNOWN",
    "paths": []
  },
  "budgets": {
    "adopted": null,
    "consumption": "UNKNOWN; recover original canonical/evidence receipts before execution"
  },
  "unresolved": {
    "boundaryObligations": {
      "change:dot-07-approval-recovery": {
        "key": "change:dot-07-approval-recovery",
        "receiptId": "c398b296f2bf07a31a64a11e045127adbbacc5945775f95bfab1df7cc5198611",
        "boundary": "change",
        "subject": "dot-07-approval-recovery",
        "sourceRevision": 2719,
        "exactSignal": "Starting change 7 out of 17: Recover pending approvals without stale authority"
      },
      "phase:dot-feature-support": {
        "key": "phase:dot-feature-support",
        "receiptId": "532bb1b384f401bf6ff969b486fe8bce4eb867749e9db27d3dc7fbd5bff8e4b4",
        "boundary": "phase",
        "subject": "dot-feature-support",
        "sourceRevision": 2588,
        "exactSignal": "Starting phase 26 out of 26: dot-feature-support"
      },
      "phase:runtime-harness-gap-closure": {
        "key": "phase:runtime-harness-gap-closure",
        "receiptId": "842d1f1a4101c2c3fe448188fa16003d87b1521375b7ac4657fc25c20a01a2da",
        "boundary": "phase",
        "subject": "runtime-harness-gap-closure",
        "sourceRevision": 2484,
        "exactSignal": "Starting phase 25 out of 25: runtime-harness-gap-closure"
      },
      "phase:skills-a2ui-library-and-runtime-observability::agui-a2ui-selection-architecture::classify-readiness-diagnostic-websocket-frame": {
        "key": "phase:skills-a2ui-library-and-runtime-observability::agui-a2ui-selection-architecture::classify-readiness-diagnostic-websocket-frame",
        "receiptId": "22e93c03361d7f0e12a1b6be4e9960d5ed0895d725a6bd7c9825dd1433242be1",
        "boundary": "phase",
        "subject": "skills-a2ui-library-and-runtime-observability::agui-a2ui-selection-architecture::classify-readiness-diagnostic-websocket-frame",
        "sourceRevision": 2411,
        "exactSignal": "Starting phase 3 out of 3: classify-readiness-diagnostic-websocket-frame"
      },
      "phase:skills-a2ui-library-and-runtime-observability::agui-a2ui-selection-architecture::codex-harness-comparative-analysis": {
        "key": "phase:skills-a2ui-library-and-runtime-observability::agui-a2ui-selection-architecture::codex-harness-comparative-analysis",
        "receiptId": "35f9069b93d48b0aa2835caa0544677f18371584afb51253b1b97f128c18b38a",
        "boundary": "phase",
        "subject": "skills-a2ui-library-and-runtime-observability::agui-a2ui-selection-architecture::codex-harness-comparative-analysis",
        "sourceRevision": 1265,
        "exactSignal": "Starting phase 1 out of 1: codex-harness-comparative-analysis"
      },
      "phase:skills-a2ui-library-and-runtime-observability::agui-a2ui-selection-architecture::diagnose-database-readiness-latency": {
        "key": "phase:skills-a2ui-library-and-runtime-observability::agui-a2ui-selection-architecture::diagnose-database-readiness-latency",
        "receiptId": "74259c64d49f1c7d23c5b306d2f638f2708c8117615d0c65ee1400ff3ddde1cb",
        "boundary": "phase",
        "subject": "skills-a2ui-library-and-runtime-observability::agui-a2ui-selection-architecture::diagnose-database-readiness-latency",
        "sourceRevision": 2363,
        "exactSignal": "Starting phase 2 out of 2: Diagnose persistent database-backed readiness latency"
      },
      "phase:uar-outstanding-issues-v1-1-1": {
        "key": "phase:uar-outstanding-issues-v1-1-1",
        "receiptId": "5b78d0fc56f4a5c38477c336178a9c20a9e71034e0bfc772e0cf41d64050ef5a",
        "boundary": "phase",
        "subject": "uar-outstanding-issues-v1-1-1",
        "sourceRevision": 2922,
        "exactSignal": "Starting phase 28 out of 28: uar-outstanding-issues-v1-1-1"
      },
      "phase:uar-outstanding-issues-v1-1-1::function-first-release-certification": {
        "key": "phase:uar-outstanding-issues-v1-1-1::function-first-release-certification",
        "receiptId": "38d1e529600f042b7ece4132af582a88a591345420f8012323d632ff494d7189",
        "boundary": "phase",
        "subject": "uar-outstanding-issues-v1-1-1::function-first-release-certification",
        "sourceRevision": 3059,
        "exactSignal": "Starting phase 1 out of 1: function-first-release-certification"
      },
      "phase:uar-outstanding-issues-v1-1-1::function-first-release-certification::kubo-native-tool-identity-pr13::uar-130-pr13-delivery": {
        "key": "phase:uar-outstanding-issues-v1-1-1::function-first-release-certification::kubo-native-tool-identity-pr13::uar-130-pr13-delivery",
        "receiptId": "bc7b856d8cccb7a6000762e4204641e3c806c180390641f3a5f01a3b65830b46",
        "boundary": "phase",
        "subject": "uar-outstanding-issues-v1-1-1::function-first-release-certification::kubo-native-tool-identity-pr13::uar-130-pr13-delivery",
        "sourceRevision": 3376,
        "exactSignal": "Starting phase 1 out of 1: uar-130-pr13-delivery"
      },
      "task:4.2 `agentnode` spawns and waits on a real child thread; remove the synthesized two-message call and the `[route]` prefix": {
        "key": "task:4.2 `agentnode` spawns and waits on a real child thread; remove the synthesized two-message call and the `[route]` prefix",
        "receiptId": "8937f584bc57c12eceb9c8c1db29fc928f40e0897016463760a2d5e8d9f9895a",
        "boundary": "task",
        "subject": "4.2 `AgentNode` spawns and waits on a real child thread; remove the synthesized two-message call and the `[route]` prefix",
        "sourceRevision": 1862,
        "exactSignal": "Starting task 18 out of 26: 4.2 `AgentNode` spawns and waits on a real child thread; remove the synthesized two-message call and the `[route]` prefix"
      },
      "task:add src/uar/runtime/prompt/assemble.rs: fixed section enum, per-section deterministic sort, render(&[promptfragment]) -> string": {
        "key": "task:add src/uar/runtime/prompt/assemble.rs: fixed section enum, per-section deterministic sort, render(&[promptfragment]) -> string",
        "receiptId": "9e6e8af0cfd9795aba17d862a51fb799e9819a92155a249bc9e27ec6db693828",
        "boundary": "task",
        "subject": "Add src/uar/runtime/prompt/assemble.rs: fixed section enum, per-section deterministic sort, render(&[PromptFragment]) -> String",
        "sourceRevision": 1640,
        "exactSignal": "Starting task 10 out of 20: Add src/uar/runtime/prompt/assemble.rs: fixed section enum, per-section deterministic sort, render(&[PromptFragment]) -> String"
      },
      "task:dot-07-approval-recovery/2": {
        "key": "task:dot-07-approval-recovery/2",
        "receiptId": "f615ca4f132120c700d1b344543a42ed3c6c8c516b8c69864d8b749f7d77a9c5",
        "boundary": "task",
        "subject": "dot-07-approval-recovery/2",
        "sourceRevision": 2724,
        "exactSignal": "Starting task 2 out of 2: 2.1 Verify recover pending approvals without stale authority through production boundaries. Restart while pending, read it through the shipped interface and decide on a second runtime. Race duplicate decisions, expiration, revoked authority and changed bindings; assert at most one approved effect and no effect for invalidated/denied decisions."
      },
      "task:make `uar::domain::context::contextstrategy` the only enum; map `truncatemiddle` to `keepfirstlast` and `hierarchical` to `progressivesummar": {
        "key": "task:make `uar::domain::context::contextstrategy` the only enum; map `truncatemiddle` to `keepfirstlast` and `hierarchical` to `progressivesummar",
        "receiptId": "cf9ea650c9a2fdd7df98e56d96b5637cd10bea4ca4659ccc69ee6bd916a91f02",
        "boundary": "task",
        "subject": "Make `uar::domain::context::ContextStrategy` the only enum; map `TruncateMiddle` to `KeepFirstLast` and `Hierarchical` to `ProgressiveSummar",
        "sourceRevision": 1512,
        "exactSignal": "Starting task 11 out of 19: Make `uar::domain::context::ContextStrategy` the only enum; map `TruncateMiddle` to `KeepFirstLast` and `Hierarchical` to `ProgressiveSummar"
      },
      "task:projected-mcp-runtime::3.1": {
        "key": "task:projected-mcp-runtime::3.1",
        "receiptId": "80630babba729db1ffa6529847d28b2d11e287bc60e96553c5dfbceac88af1ee",
        "boundary": "task",
        "subject": "projected-mcp-runtime::3.1",
        "sourceRevision": 1842,
        "exactSignal": "Starting task 13 out of 22: Lazy start for skill-contributed and child-only servers gated on cached catalog completeness; `wait_until_ready` on the call path"
      },
      "task:projected-mcp-runtime::4.1": {
        "key": "task:projected-mcp-runtime::4.1",
        "receiptId": "cfac8c22d12769f7b9e939586c5c1b012c9a691027d82ebfb771ff918362f039",
        "boundary": "task",
        "subject": "projected-mcp-runtime::4.1",
        "sourceRevision": 1856,
        "exactSignal": "Starting task 17 out of 22: Replace per-run `from_config` with catalog resolution and binding lookup"
      },
      "task:thread-native-subagents::4.1": {
        "key": "task:thread-native-subagents::4.1",
        "receiptId": "d42c9c4e61a1b92da6a8017109a9ce4d28dd1705300e2cf265a56fbc6cc0999a",
        "boundary": "task",
        "subject": "thread-native-subagents::4.1",
        "sourceRevision": 1824,
        "exactSignal": "Starting task 17 out of 25: `AgentActor` as a mailbox over the kernel; actor API requires user context"
      },
      "task:thread-native-subagents::5.1": {
        "key": "task:thread-native-subagents::5.1",
        "receiptId": "fc129807934bcb5288af5f6a7fc3b3b846354f7c2820e584ea6d456b6d329463",
        "boundary": "task",
        "subject": "thread-native-subagents::5.1",
        "sourceRevision": 1858,
        "exactSignal": "Starting task 20 out of 25: Record child usage against the root run; enforce `Exceeded` on spawn and model call"
      }
    },
    "blockers": [
      {
        "id": "bottleneck-07c3cb0843752d589e4edebe52266b1e11fc360d8ab9c24fdc89b071f7a5e7aa",
        "summary": "Task Before boundary Verify `versions.toml` contains a `jsonschema` entry at `0.49.4` (operator-edited, outside this change's scope); if absent, stop and ask the operator before any implementation task blocked: Task subject Verify `versions.toml` contains a `jsonschema` entry at `0.49.4` (operator-edited, outside this change's scope); if absent, stop and ask the operator before any implementation task is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-0eb919c0a26d4ca88edd2a5537fc07b9104f8d4692daeea81d0af9ed04505dc3",
        "summary": "Task Before boundary tests/prompt_assembly.rs: deterministic registry-order and repeated-RAG assembly blocked: Task subject tests/prompt_assembly.rs: deterministic registry-order and repeated-RAG assembly is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-1354e3ca52371401219f8f8414287778d6eea79f4616bb30297c476d606e53ea",
        "summary": "Task Before boundary Tier 1: the two new test targets blocked: Task subject Tier 1: the two new test targets is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-18164297fad3d343a2c4920ce6511239e0695aa2984ea291a021e59b45b526d6",
        "summary": "Task Before boundary A retrieved chunk renders inside `Retrieved` markers and its fragment authority is `Retrieved`; a skill body renders inside `Skill` markers blocked: Task subject A retrieved chunk renders inside `Retrieved` markers and its fragment authority is `Retrieved`; a skill body renders inside `Skill` markers is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-1909e5f79a472a034065a3d02b92a9fb7fe633b4ecff545159a5d17063d9debe",
        "summary": "Task Before boundary A stream that fails mid-turn persists the partial assistant text with a `TurnInterrupted` marker; the next turn's history contains the marker fragment blocked: Task subject A stream that fails mid-turn persists the partial assistant text with a `TurnInterrupted` marker; the next turn's history contains the marker fragment is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-1d1eb2e780726c94f1b8ede47c91a4b888315d242578dada283e37a20d1f18a2",
        "summary": "Task Before boundary Tier 2: fmt check and full test run blocked: Task subject Tier 2: fmt check and full test run is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-2139fa2e8ec1d5c6e0ac6769b9098bf76876211ec1158e09a1bfce2212c9a67d",
        "summary": "Task After boundary Verify `versions.toml` contains a `jsonschema` entry at `0.49.4` (operator-edited, outside this change's scope); if absent, stop and ask the operator before any implementation task blocked: Task subject Verify `versions.toml` contains a `jsonschema` entry at `0.49.4` (operator-edited, outside this change's scope); if absent, stop and ask the operator before any implementation task is not a unique canonical work item; boundary Verify `versions.toml` contains a `jsonschema` entry at `0.49.4` (operator-edited, outside this change's scope); if absent, stop and ask the operator before any implementation task has no matching start receipt",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-2276118dd00ec03bdd9e2a728be95d08b4d2f170708deb62b0c4a89e6b9405aa",
        "summary": "Task Before boundary deterministic-prompt-assembly:1.2 blocked: Task subject deterministic-prompt-assembly:1.2 is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-23b16273a70c70b450096b2f08e608327de5a0a5434ad2ec16c0d35686b79202",
        "summary": "Task Before boundary `tests/typed_turn_assembly.rs`: a `RunExecutionRequest` built from the legacy positional arguments resolves to a `ResolvedTurn` whose fragments, policy, and credentials equal the legacy path blocked: Task subject `tests/typed_turn_assembly.rs`: a `RunExecutionRequest` built from the legacy positional arguments resolves to a `ResolvedTurn` whose fragments, policy, and credentials equal the legacy path is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-28128c8116302e16798b6bd0b64159c751dc4a1a53a58acc1a25784f8b262542",
        "summary": "Task Before boundary A retrieved chunk renders inside `Retrieved` markers and its fragment authority is `Retrieved`; a skill body renders inside `Skill` markers blocked: Task subject A retrieved chunk renders inside `Retrieved` markers and its fragment authority is `Retrieved`; a skill body renders inside `Skill` markers is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-2c2a34b7cf5d12cc1ebda78ff2410de37274c8892db67587671c6076297701fa",
        "summary": "Task Before boundary typed-turn-assembly:1.1 blocked: Task subject typed-turn-assembly:1.1 is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-2e12b655cb393992551554c4c7394bb8d3c0c37bf8c6fd58a14dd858b2c36a75",
        "summary": "Task Before boundary fail-closed-tool-arguments:5.2 blocked: Task subject fail-closed-tool-arguments:5.2 is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-3accd20667944346e5c1954977611097dbfece73ea9b70f004f42ec6152882ca",
        "summary": "Task Before boundary A skill cannot replace a global server definition or loosen its sandbox policy; delegation transfers frozen bindings locally and no connection recipes to remote UAR peers blocked: Task subject A skill cannot replace a global server definition or loosen its sandbox policy; delegation transfers frozen bindings locally and no connection recipes to remote UAR peers is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-505de30dd631def4776214df6848dee1b6d971ae14e89e0db5613a7b94b2735a",
        "summary": "Task After boundary A skill-contributed server with a complete cached catalog is not started until its first tool call; the call waits for readiness and succeeds blocked: Task subject A skill-contributed server with a complete cached catalog is not started until its first tool call; the call waits for readiness and succeeds is not a unique canonical work item; boundary A skill-contributed server with a complete cached catalog is not started until its first tool call; the call waits for readiness and succeeds has no matching start receipt",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-619853752b6c61d80fce694630d9efc51dc00647a42d9558ce590beaced7adbd",
        "summary": "Task Before boundary `send_agent_message` delivers a typed message whose sender identity is metadata; the child user turn contains no prepended identity text blocked: Task subject `send_agent_message` delivers a typed message whose sender identity is metadata; the child user turn contains no prepended identity text is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-6faa3bb6453bb6b0191cb6e2a73502623e53ce1858eeb4beff4a469d2688f5a9",
        "summary": "Task After boundary Record child usage against the root run; enforce Exceeded on spawn and model call blocked: Task subject Record child usage against the root run; enforce Exceeded on spawn and model call is not a unique canonical work item; boundary Record child usage against the root run; enforce Exceeded on spawn and model call has no matching start receipt",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-800647e7fa0c8b5fe7994d0b7e7ce93ac7fc86c1655e01911df928b4a8949a17",
        "summary": "Task Before boundary A child scope cannot replace a global server definition or loosen its sandbox policy blocked: boundary A child scope cannot replace a global server definition or loosen its sandbox policy already has an outstanding start receipt",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-82901028ebd5977ebef852e36e141aa757167fd6a50548739e99b49954213b62",
        "summary": "Task Before boundary  blocked: Task subject  is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-84011cc0a55fd467ada68a2a47769f87afcd264e3b2eb807e9a295f447ecc1d9",
        "summary": "Task Before boundary send_agent_message delivers a typed message whose sender identity is metadata; the child user turn contains no prepended identity text blocked: Task subject send_agent_message delivers a typed message whose sender identity is metadata; the child user turn contains no prepended identity text is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-99bfbdd6d42f46ae5e2aba845d790f2418a4e6fc77c61507ce7334a54c5810d1",
        "summary": "Task Before boundary deterministic-prompt-assembly:1.2 blocked: Task subject deterministic-prompt-assembly:1.2 is not a unique canonical work item; 36 compatibility projection(s) differ from canonical revision 1913",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-9fdc6faf1c92d1cf83752de47d94bb4fcf4829a29ae107f865b98759da3a52a8",
        "summary": "Task Before boundary 1.2 blocked: Task subject 1.2 is not a unique canonical work item; 36 compatibility projection(s) differ from canonical revision 1914",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-ac9ff0cc5089ce7980671195c0870669d3a6b2d327ae33c3ce8d191a6849979d",
        "summary": "Task Before boundary A skill-contributed server with a complete cached catalog is not started until its first tool call; the call waits for readiness and succeeds blocked: Task subject A skill-contributed server with a complete cached catalog is not started until its first tool call; the call waits for readiness and succeeds is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-b2280a669c2d7f73708cad7aa36433224cb5a80b2f0e53a19b2092973447b2f2",
        "summary": "Task Before boundary typed-turn-assembly:5.1 blocked: Task subject typed-turn-assembly:5.1 is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-bb29f60fcc9835aac256d5ad1774843d04a02a8f84bfc47a414e3f3ca665ce28",
        "summary": "Task Before boundary 1.1 `tests/prompt_assembly.rs`: assembling the same artifact, three matched skills inserted in different registry orders, and the same RAG hits twice yields byte-identical system prompts and identical manifest hashes blocked: Task subject 1.1 `tests/prompt_assembly.rs`: assembling the same artifact, three matched skills inserted in different registry orders, and the same RAG hits twice yields byte-identical system prompts and identical manifest hashes is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-bce4137ba015a6f44a975ba500a15c0cab0cf593875337905b044a72764cc54a",
        "summary": "Task After boundary thread-native-subagents::4.1 blocked: Task subject thread-native-subagents::4.1 is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-c2ed5767a1a51d865f0faef6b897410866094fd12720970c3a86f9c395711237",
        "summary": "Task Before boundary AgentNode spawns and waits on a real child thread; remove the synthesized two-message call and the [route] prefix blocked: Task subject AgentNode spawns and waits on a real child thread; remove the synthesized two-message call and the [route] prefix is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-d910345b27a28bb2540c2c4ebbd48f38c1c9fb29d26b2600606c81171f1e5c7a",
        "summary": "Task Before boundary Tier 1: extend the existing reconnect tests, then run `cargo test --locked --no-default-features --features server-full --test mcp_projection` and the focused registry reconnect target blocked: Task subject Tier 1: extend the existing reconnect tests, then run `cargo test --locked --no-default-features --features server-full --test mcp_projection` and the focused registry reconnect target is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-ee3b817d9d3be05d2ee2d989ad78b5521cb18b8e5230743578d2334e387e1df2",
        "summary": "Task After boundary DEVIATION, recorded during execution. The task as written (\"make `uar::domain::context::ContextStrategy` the only enum\") is wrong in the opp blocked: Task subject DEVIATION, recorded during execution. The task as written (\"make `uar::domain::context::ContextStrategy` the only enum\") is wrong in the opp is not a unique canonical work item; boundary DEVIATION, recorded during execution. The task as written (\"make `uar::domain::context::ContextStrategy` the only enum\") is wrong in the opp has no matching start receipt",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-f5c01d4d83845ab24d61979c41b492a8a065b3f9b792d1fb03e97e5a386d8f57",
        "summary": "Task After boundary AgentActor as a mailbox over the kernel; actor API requires user context blocked: Task subject AgentActor as a mailbox over the kernel; actor API requires user context is not a unique canonical work item; boundary AgentActor as a mailbox over the kernel; actor API requires user context has no matching start receipt",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-f9223e04afe8a8ce4b076689a44da90ff1649f7cb5a0456d51a29a2adde1f1b4",
        "summary": "Task Before boundary 1.2 A retrieved chunk renders inside `Retrieved` markers and its fragment authority is `Retrieved`; a skill body renders inside `Skill` markers blocked: Task subject 1.2 A retrieved chunk renders inside `Retrieved` markers and its fragment authority is `Retrieved`; a skill body renders inside `Skill` markers is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-faffdcc31eee3f49a351e4b3b5b7b1296b3c861379d69a791b9c51ad80153042",
        "summary": "Task Before boundary tests/context_history_integrity.rs: an assistant message with two tool calls followed by one tool result is normalized to two results, the missing one typed cancelled blocked: Task subject tests/context_history_integrity.rs: an assistant message with two tool calls followed by one tool result is normalized to two results, the missing one typed cancelled is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "bottleneck-fc86c7cc9c92ad3dc2998c189c3bb47fb0810f9f68c781f195774aef8f02cad2",
        "summary": "Task Before boundary The manifest is stored in `Run.context` and emitted as a `turn_manifest` artifact; the existing `effective_run_policy` artifact is still emitted blocked: Task subject The manifest is stored in `Run.context` and emitted as a `turn_manifest` artifact; the existing `effective_run_policy` artifact is still emitted is not a unique canonical work item",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "kubo-native-identity-baseline-fixtures",
        "summary": "Approved gate1 reported two E0063 errors in pre-existing v1.3.0 test fixtures: RunStreamState caller_handoff_confirmation and AnthropicMessagesRequest max_tokens/tool_choice/parallel_tool_calls. No test result; Cargo finishing remaining compilation jobs. Gates2/3 not started. All verification held for complete two-file fixture correction and refreshed inventory/operator instruction.",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "kubo-native-identity-r2-compilation-prerequisites",
        "summary": "Approved gate1 r2 exited101 before tests: E0382 twice in seed-minimax-verification retained record_id after query bindings; E0308 twice in tool_call_protocol VectorMatcher fixture helper type. Source drift none, stdout0bytes, gates2/3 never started. Correct all four together, preserve all targets/assertions, verification CLOSED.",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "kubo-native-identity-r3-library-failures",
        "summary": "Approved r3 gate1 compiled, began827 lib tests, reported four FAILED cases (SSE mapping, context conformance, memory session cascade, child cost budget), then SIGABRT stack overflow in remote persisted launch-refusal lifecycle test before complete failure/aggregate diagnostics. Cargo101, no source drift; protocol/admission targets and commands2/3 not run. Hold all verification; complete source assessment across all five observations before corrections or separately authorized diagnostics.",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "runtime-harness-checkpoint-build-contention",
        "summary": "Tier 2 checkpoint blocked after cargo fmt found three formatting-only spans in routing 2.3 and four subsequent UAR cargo check attempts were interrupted by recurring Compass cargo/clippy writers under 28.9 GiB swap use. Uncommitted repair files: src/llm/orchestrator.rs, tests/model_path_resiliency.rs. No Tier 0 or Tier 2 pass is claimed.",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "uar-outstanding-issues-v1-1-1::function-first-release-certification::embedding-primary-backup-lifecycle",
        "summary": "NOT READY (frozen exit contract): no second honored embedding space/provider reachable within limits (no new credentials, no paygo, no personal config change). Query-time space-guard leg UNVERIFIED; live provider failover NOT MET. Proven instead: conditions 1/2/4/6 PASS; restart resilience with truthful v5 provenance (boot fallback, backup query+ingestion continuity, recovery; manifest f6dcb84ec0c04198 identical across instances). Memory-subsystem mismatch recorded (OpenAI/1536 named; Mistral space UNKNOWN; #420 history). Evidence: children/embedding-primary-backup-lifecycle/{INVENTORY,RESOLUTION,ASSESS-MIRROR,RECONCILIATION,PROOF,reflection}.md",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "uar111-genuine-supervisor-setup",
        "summary": "UAR1.1.1 required native supervisor proof lacks a live isolated guest: old dot17-linux-qualification was removed; only shared Lima config remains. Concrete 4CPU/8GiB/24GiB ARM64 setup and guest-only reboot proposal documented in docs/operations/uar-1.1.1-supervisor-setup.md; operator authorization pending. No know-me mutation, paid resources or Dot matrix. R9 source work continues; no retest/publication.",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "uar130-pr13-producer-qualification",
        "summary": "R3 full T2 exited101 compiling two stale fixtures before tests; scoped four-file source repair now integrated but current producer qualification remains BLOCKED pending r4 source freeze and fresh exact approval. Retain35/35 driver and historical formatting; no later backend/customer gate started. Owned cleanup complete.",
        "resolved": false,
        "resolution": null
      }
    ],
    "instruction": "Unknown side effects require reconciliation before retry, not a fresh attempt assumption."
  },
  "preserve": [
    "latest non-superseded method decision and retiredMethods; old commands/memory are context only",
    "original contract snapshots, evidence and source/build/configuration/fixture/assertion identity",
    "all budget consumption/high-water marks, including UNKNOWN prior usage; never reset on recovery",
    "unresolved effects and cleanup/receipt obligations; reconcile uncertain effects before retry"
  ],
  "hostCapabilities": {
    "platform": "darwin",
    "nodeVersion": "v26.5.0",
    "observedInterfaces": [
      "canonical status",
      "typed phase disposition",
      "project-owned file identity"
    ],
    "executionInterception": "not provided by this helper",
    "commandInventoryIsPermission": false
  },
  "control": {
    "mode": "instruction-only",
    "observed": [
      "canonical references and file identities"
    ],
    "enforced": [
      "only named existing signed runtime mutations; no generic shell/tool interception claimed"
    ],
    "unsupported": [
      "universal model obedience or arbitrary tool interception"
    ]
  },
  "status": "BLOCKED",
  "blockers": [
    "adopted contract unavailable"
  ]
}
```
